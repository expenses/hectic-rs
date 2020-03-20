use std::path::Path;
use std::env;

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let gen_code_path = Path::new(&out_dir).join("image.rs");
    let packed_path = Path::new(&out_dir).join("packed.png");

    yaap::pack(
        std::fs::read_dir("src/images").unwrap().map(|x| x.unwrap().path()),
        packed_path,
        gen_code_path,
    ).unwrap();
}
