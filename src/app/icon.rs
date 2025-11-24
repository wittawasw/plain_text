use fltk::image::SvgImage;

pub fn load_app_icon() -> SvgImage {
    SvgImage::load("assets/icon.svg").expect("Cannot load SVG")
}
