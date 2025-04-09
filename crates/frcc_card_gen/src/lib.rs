use std::collections::HashMap;

use resvg::{tiny_skia::{Pixmap, PixmapMut}, usvg::{Options, Transform, Tree}};
use rxing::{BarcodeFormat, EncodeHintType, EncodeHintValue, EncodeHints, EncodingHintDictionary, Writer};

pub fn test() {
    let mut pixmap = Pixmap::new(635, 889).unwrap();
    resvg::render(&Tree::from_str(include_str!("../../../cards/back/default.svg"), &Options::default()).unwrap(), Transform::identity(), &mut pixmap.as_mut());
    resvg::render(&Tree::from_str(&gen_fiducial("12345678910".to_owned()), &Options::default()).unwrap(), Transform::identity().pre_translate((635.0 / 2.0) - 100.0, (889.0 / 2.0) - 100.0), &mut pixmap.as_mut());
    //pixmap.encode_png().unwrap();
    pixmap.save_png(format!("test.png")).unwrap();


}
pub fn gen_fiducial(id: String) -> String {
    let wr = rxing::aztec::AztecWriter;
    let bit_matrix = wr.encode_with_hints(&id, &BarcodeFormat::AZTEC, 200, 200, &EncodeHints {
        ErrorCorrection: Some("45".to_owned()),
        CharacterSet: Some("UTF-8".to_owned()),
        ..Default::default()
    }).unwrap();
    let fiducial_svg: svg::Document = (&bit_matrix).into();
    fiducial_svg.set("background-color", "white").to_string()
}
