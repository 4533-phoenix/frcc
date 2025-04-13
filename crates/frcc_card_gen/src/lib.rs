//! Card generation library for FRCC

use std::io::{Error, ErrorKind};

// External crate imports
use resvg::{
    tiny_skia::Pixmap,
    usvg::{self, Options, Transform, Tree},
};
use rxing::{BarcodeFormat, EncodeHints, Writer};

/// Generates an Aztec code fiducial mark as SVG
///
/// # Arguments
///
/// * `id` - The identifier string to encode in the Aztec code
///
/// # Returns
///
/// An SVG string representation of the Aztec code
pub fn gen_fiducial(id: String) -> String {
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
    let fiducial_svg: svg::Document = (&bit_matrix).into();
    fiducial_svg.to_string()
}

/// Common setup for card rendering
/// 
/// # Arguments
/// 
/// * `svg_content` - SVG content for the card template
/// 
/// # Returns
/// 
/// A tuple containing the initialized pixmap and card size
fn setup_card(svg_content: &str) -> (Pixmap, usvg::Size) {
    let mut options = Options::default();
    options.fontdb_mut().load_system_fonts();

    let card_tree = Tree::from_str(svg_content, &options).unwrap();
    let card_size = card_tree.size();

    let width = card_size.width() as u32;
    let height = card_size.height() as u32;
    let mut pixmap = Pixmap::new(width, height).unwrap();

    // Render the card background
    resvg::render(&card_tree, Transform::identity(), &mut pixmap.as_mut());

    (pixmap, card_size)
}

/// Renders the back of a card with a fiducial mark
/// 
/// # Arguments
/// 
/// * `card_template` - SVG string for the card template
/// * `id` - The identifier to encode in the fiducial mark
/// * `output_path` - Path where to save the PNG file (optional)
/// 
/// # Returns
/// 
/// The rendered card pixmap
pub fn render_back_card(card_template: &str, id: &str, output_path: Option<&str>) -> Pixmap {
    let (mut pixmap, card_size) = setup_card(card_template);
    
    // Render the fiducial mark
    let fiducial_svg = gen_fiducial(id.to_owned());
    let fiducial_options = Options::default();
    let fiducial_tree = Tree::from_str(&fiducial_svg, &fiducial_options).unwrap();

    resvg::render(
        &fiducial_tree,
        Transform::identity().pre_translate(
            (card_size.width() / 2.0) - 100.0,
            (card_size.height() / 2.0) - 100.0,
        ),
        &mut pixmap.as_mut(),
    );
    
    // Save to file if output path is provided
    if let Some(path) = output_path {
        save_card(&pixmap, path).unwrap();
    }
    
    pixmap
}

/// Renders the front of a card with team and robot details
/// 
/// # Arguments
/// 
/// * `card_template` - SVG string for the card template
/// * `name` - The robot's name
/// * `number` - The team number
/// * `image_path` - Path to the robot image
/// * `abilities` - Vector of robot abilities
/// * `output_path` - Path where to save the PNG file (optional)
/// 
/// # Returns
/// 
/// The rendered card pixmap
pub fn render_front_card(
    card_template: &str,
    name: &str,
    number: &str,
    image_path: &str,
    abilities: &[Ability],
    output_path: Option<&str>,
) -> Pixmap {
    let (mut pixmap, _) = setup_card(card_template);
    
    // TODO: Implement front card rendering logic:
    // 1. Add robot name and team number
    // 2. Add robot image
    // 3. Render abilities with their levels and descriptions
    
    // Save to file if output path is provided
    if let Some(path) = output_path {
        save_card(&pixmap, path).unwrap();
    }
    
    pixmap
}

/// Saves a rendered card to a PNG file
///
/// # Arguments
///
/// * `pixmap` - The rendered card image
/// * `output_path` - Path where to save the PNG file
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
    pub amount: String
}

/// Test function demonstrating both card renderings
pub fn test() {
    // Test back card generation
    let back_template = include_str!("../../../cards/back/default.svg");
    render_back_card(back_template, "FRCC-12345678910", Some("back.png"));

    // Test front card generation
    let front_template = include_str!("../../../cards/front/default.svg");
    let name = "Ferris";
    let number = "4533";
    let image = "../../../temp/rustacean.png";
    let abilities = vec![
        Ability {
            name: "Ability 1".to_string(),
            description: "Description of ability 1".to_string(),
            level: 1,
            amount: "L1".to_string()
        },
        Ability {
            name: "Ability 2".to_string(),
            description: "Description of ability 2".to_string(),
            level: 2,
            amount: "L2".to_string()
        },
    ];

    render_front_card(front_template, name, number, image, &abilities, Some("front.png"));
}
