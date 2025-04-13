//! Card generation library for FRCC

pub mod printout;

static OPTIONS: Lazy<Options> = Lazy::new(|| {
    let mut options = Options::default();

    options.shape_rendering = ShapeRendering::CrispEdges;
    options.text_rendering = TextRendering::GeometricPrecision;
    options.image_rendering = ImageRendering::CrispEdges;

    options.fontdb_mut().load_system_fonts();

    options
});

use std::io::{Error, ErrorKind};

// External crate imports
use image::{GenericImage, ImageBuffer, Rgba, RgbaImage};
use once_cell::sync::Lazy;
use resvg::{
    tiny_skia::{self, BlendMode, IntSize, Pixmap, Transform},
    usvg::{self, ImageRendering, Options, ShapeRendering, TextRendering, Tree},
};
use rxing::{BarcodeFormat, EncodeHints, Writer};

pub fn gen_fiducial(id: String) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    let wr = rxing::aztec::AztecWriter;
    let bit_matrix = wr
        .encode_with_hints(
            &id,
            &BarcodeFormat::AZTEC,
            200,
            200,
            &EncodeHints {
                ErrorCorrection: Some("45".to_owned()),
                CharacterSet: Some("UTF-8".to_owned()),
                ..Default::default()
            },
        )
        .unwrap();

    // Fix: Properly convert to svg::Document without generic parameter
    let fiducial_img: image::DynamicImage = (&bit_matrix).into();

    fiducial_img.to_rgba8()
}

/// Common setup for card rendering
fn setup_card(svg_content: &str) -> (Pixmap, usvg::Size) {
    let tree = Tree::from_str(svg_content, &OPTIONS).unwrap();
    let size = tree.size();
    let mut pixmap = Pixmap::new(size.width() as u32, size.height() as u32).unwrap();

    resvg::render(&tree, Transform::identity(), &mut pixmap.as_mut());

    (pixmap, size)
}

/// Load and process an image for rendering
fn load_image(path: &str) -> (Pixmap, u32, u32) {
    // Load image or panic with useful error message
    let img_data = std::fs::read(path).expect(&format!("Failed to read image: {}", path));
    let img = image::load_from_memory(&img_data).expect(&format!("Failed to decode: {}", path));

    // Process image to fix border artifacts
    let mut rgba = img.to_rgba8();
    let (width, height) = (rgba.width(), rgba.height());

    if width == 0 || height == 0 {
        panic!("Invalid image dimensions for: {}", path);
    }

    // Premultiply alpha to fix edge artifacts
    for pixel in rgba.pixels_mut() {
        let alpha = pixel[3] as f32 / 255.0;
        if alpha > 0.0 {
            for i in 0..3 {
                pixel[i] = (pixel[i] as f32 * alpha).round() as u8;
            }
        }
    }

    // Create pixmap
    let size = IntSize::from_wh(width, height).expect("Invalid image dimensions");
    let pixmap =
        Pixmap::from_vec(rgba.into_raw(), size).expect("Failed to create pixmap from image");

    (pixmap, width, height)
}

/// Renders the back of a card with a fiducial mark
pub fn render_back_card(card_template: &str, id: &str, output_path: Option<&str>) -> RgbaImage {
    let (pixmap, card_size) = setup_card(card_template);

    // Render fiducial mark
    let fiducial_img = gen_fiducial(id.to_owned());
    let x_pos = (card_size.width() / 2.0) - 100.0;
    let y_pos = (card_size.height() / 2.0) - 100.0;

    let mut img = RgbaImage::from_vec(card_size.width() as u32, card_size.height() as u32, pixmap.data().to_vec()).unwrap();

    img.sub_image(x_pos as u32, y_pos as u32, 200, 200).copy_from(&fiducial_img, 0, 0).unwrap();

    if let Some(path) = output_path {
        img.save_with_format(path, image::ImageFormat::Png).unwrap();
    }

    img
}

/// Renders the front of a card with team and robot details
pub fn render_front_card(
    card_template: &str,
    name: &str,
    number: &str,
    image_path: &str,
    abilities: &[Ability],
    output_path: Option<&str>,
) -> Pixmap {
    // Prepare template with text replacements
    let mut template = String::from(card_template);

    // Replace robot name and team number
    template = template.replace("ROBOT_NAME", name);
    template = template.replace("NUMBER", number); // Just the number itself now

    // Replace ability placeholders
    for (i, ability) in abilities.iter().enumerate().take(3) {
        let idx = i + 1;

        // Replace ability name and description
        template = template
            .replace(&format!("ABILITY {}", idx), &ability.name)
            .replace(&format!("DESCRIPTION {}", idx), &ability.description);

        // Replace amount in the dedicated amount field
        template = template.replace(&format!("AMOUNT_{}", idx), &ability.amount);
    }

    // Hide unused ability placeholders
    for i in abilities.len() + 1..=3 {
        template = template.replace(
            &format!("<g id=\"ability{}\">", i),
            &format!("<g id=\"ability{}\" visibility=\"hidden\">", i),
        );
    }

    // Setup card and load image
    let (mut pixmap, _) = setup_card(&template);
    let (img_pixmap, width, height) = load_image(image_path);

    // Calculate layout for image placement
    let target = (44.444859, 104.444859, 541.11053, 350.0); // x, y, width, height
    let src_width = width as f32;
    let src_height = height as f32;

    // Scale to fit while maintaining aspect ratio
    let scale = (target.2 / src_width).min(target.3 / src_height);
    let scaled_size = (src_width * scale, src_height * scale);

    // Center the image
    let centered_x = target.0 + (target.2 - scaled_size.0) / 2.0;
    let centered_y = target.1 + (target.3 - scaled_size.1) / 2.0;

    // Draw the image with proper blending
    let mut paint = tiny_skia::PixmapPaint::default();
    paint.blend_mode = BlendMode::SourceOver;

    pixmap.draw_pixmap(
        0,
        0,
        img_pixmap.as_ref(),
        &paint,
        Transform::from_scale(scale, scale).pre_translate(centered_x / scale, centered_y / scale),
        None,
    );

    if let Some(path) = output_path {
        save_card(&pixmap, path).unwrap();
    }

    pixmap
}

/// Saves a rendered card to a PNG file
pub fn save_card(pixmap: &Pixmap, output_path: &str) -> Result<(), Error> {
    pixmap
        .save_png(output_path)
        .map_err(|e| Error::new(ErrorKind::Other, e.to_string()))
}

#[derive(Debug, Clone)]
pub struct Ability {
    pub name: String,
    pub description: String,
    pub level: i8,
    pub amount: String,
}

/// Test function demonstrating both card renderings
pub fn test() {
    // Back card
    let back_template = include_str!("../../../cards/back/default.svg");
    render_back_card(back_template, "FRCC-12345678910", Some("back.png"));

    // Front card
    let front_template = include_str!("../../../cards/front/default.svg");
    render_front_card(
        front_template,
        "Ferris",
        "4533",
        "E:/Programs/rust_programs/fcc/temp/rustacean.png",
        &[
            Ability {
                name: "Ability 1".to_string(),
                description: "Description of ability 1".to_string(),
                level: 1,
                amount: "L1".to_string(),
            },
            Ability {
                name: "Ability 2".to_string(),
                description: "Description of ability 2".to_string(),
                level: 2,
                amount: "L2".to_string(),
            },
        ],
        Some("front.png"),
    );
}
