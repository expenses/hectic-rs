use std::path::Path;
use std::env;

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();

    yaap::pack(
        std::fs::read_dir("src/images").unwrap().map(|x| x.unwrap().path()),
        Path::new(&out_dir).join("packed.png"),
        Path::new(&out_dir).join("image.rs"),
        2000, 2500,
        Some("Clone, Copy")
    ).unwrap();
}
