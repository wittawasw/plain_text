use encoding_rs::Encoding;
use std::fs;

pub fn load_as_utf8(path: &str) -> Option<String> {
    let bytes = fs::read(path).ok()?;

    let enc = match Encoding::for_bom(&bytes) {
        Some((encoding, _)) => encoding,
        None => encoding_rs::UTF_8,
    };

    Some(enc.decode(&bytes).0.into_owned())
}
