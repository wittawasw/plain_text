use std::path::Path;

fn main() {
    #[cfg(target_os = "windows")]
    {
        let out_dir = std::env::var("OUT_DIR").unwrap();
        let _ = embed_resource::compile("build/windows.rc", Path::new(&out_dir));
    }
}
