//! Card generation library for FRCC

use std::io::{Error, ErrorKind};

// External crate imports
use resvg::{
    tiny_skia::Pixmap,
    usvg::{Options, Transform, Tree},
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

/// Renders a card with a fiducial mark
///
/// # Arguments
///
/// * `card_svg_path` - Path to the SVG file for the card template
/// * `id` - The identifier to encode in the fiducial mark
///
/// # Returns
///
/// A pixmap containing the rendered card
pub fn render_card(card_svg_path: &str, id: &str) -> Pixmap {
    let mut options = Options::default();
    options.fontdb_mut().load_system_fonts();
    
    let card_tree = Tree::from_str(card_svg_path, &options).unwrap();
    let card_size = card_tree.size();

    let width = card_size.width() as u32;
    let height = card_size.height() as u32;
    let mut pixmap = Pixmap::new(width, height).unwrap();

    // Render the card background
    resvg::render(
        &card_tree,
        Transform::identity(),
        &mut pixmap.as_mut(),
    );
    
    // Render the fiducial mark
    let fiducial_svg = gen_fiducial(id.to_owned());
    let fiducial_options = Options::default();
    let fiducial_tree = Tree::from_str(&fiducial_svg, &fiducial_options).unwrap();
    
    resvg::render(
        &fiducial_tree,
        Transform::identity().pre_translate(
            (card_size.width() / 2.0) - 100.0, 
            (card_size.height() / 2.0) - 100.0
        ),
        &mut pixmap.as_mut(),
    );
    
    pixmap
}

/// Saves a rendered card to a PNG file
///
/// # Arguments
///
/// * `pixmap` - The rendered card image
/// * `output_path` - Path where to save the PNG file
pub fn save_card(pixmap: &Pixmap, output_path: &str) -> Result<(), Error> {
    pixmap.save_png(output_path)
        .map_err(|e| Error::new(ErrorKind::Other, e.to_string()))
}

/// Test function to demonstrate card generation
pub fn test() {
    let card_template = include_str!("../../../cards/back/default.svg");
    let pixmap = render_card(card_template, "FRCC-12345678910");
    save_card(&pixmap, "test.png").unwrap();
}
