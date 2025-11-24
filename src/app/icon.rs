use fltk::image::SvgImage;

pub fn load_app_icon() -> SvgImage {
    let bytes = include_bytes!("../../assets/icon.svg");
    let svg_str = std::str::from_utf8(bytes).expect("Invalid SVG");

    SvgImage::from_data(svg_str).expect("Invalid SVG")
}
