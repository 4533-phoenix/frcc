use resvg::{tiny_skia::Pixmap, usvg::{Options, Transform, Tree}};
use rxing::{BarcodeFormat, EncodeHints, Writer};

pub fn test() {
    let mut pixmap = Pixmap::new(630, 880).unwrap();
    resvg::render(&Tree::from_str(include_str!("../../../cards/back/default.svg"), &Options::default()).unwrap(), Transform::identity(), &mut pixmap.as_mut());
    resvg::render(&Tree::from_str(&gen_fiducial("FRCC-12345678910".to_owned()), &Options::default()).unwrap(), Transform::identity().pre_translate((630.0 / 2.0) - 100.0, (880.0 / 2.0) - 100.0), &mut pixmap.as_mut());
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
    fiducial_svg.to_string()
}
