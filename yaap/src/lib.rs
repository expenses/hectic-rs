struct Packed {
    name: String,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
}

use std::path::Path;
use texture_packer::{*, importer::*, exporter::*, texture::*};
use codegen::*;

pub fn pack(
    filenames: impl Iterator<Item=impl AsRef<Path>>,
    output_image: impl AsRef<Path>,
    output_bindings: impl AsRef<Path>,
    max_width: u32,
    max_height: u32,
    derive_line: Option<&str>
) -> Result<(), std::io::Error> {
    let mut packer = TexturePacker::new_skyline(TexturePackerConfig {
        trim: false,
        max_width,
        max_height,
        texture_padding: 1,
        allow_rotation: false,
        .. Default::default()
    });
   
    for filename in filenames {
        let filename = filename.as_ref();
        let base = filename.file_stem().unwrap();
        
        packer.pack_own(base.to_str().unwrap().to_string(), ImageImporter::import_from_file(filename).unwrap()).unwrap();
    }

    let width = packer.width();

    if width % 64 != 0 {
        let needed = 64 - (width % 64);
        packer.set_row_padding(needed);
    }

    create_file(packer.get_frames().iter().map(|(k, frame)| {
        let name = case_style::CaseStyle::guess(k.as_str()).unwrap().to_pascalcase();

        Packed {
            name,
            x: frame.frame.x,
            y: frame.frame.y,
            width: frame.frame.w,
            height: frame.frame.h,
        }
    }), output_bindings, packer.width(), packer.height(), derive_line)?;


    let exporter = ImageExporter::export(&packer).unwrap();

    exporter.save(output_image).unwrap();

    Ok(())
}

fn create_file<'a>(assets: impl Iterator<Item=Packed> + Clone, filename: impl AsRef<Path>, width: u32, height: u32, derive_line: Option<&str>) -> Result<(), std::io::Error> {
    let mut scope = Scope::new();
    
    let image_enum = scope.new_enum("Image")
        .vis("pub");

    if let Some(derive_line) = derive_line {
        image_enum.derive(derive_line);
    }

    for asset in assets.clone() {
        image_enum.push_variant(Variant::new(&asset.name));
    }
    
    let impl_block = scope.new_impl("Image");

    impl_block.new_fn("coordinates")
        .arg_self()
        .vis("pub")
        .ret("(u32, u32, u32, u32)")
        .push_block({
            let mut block = Block::new("match self");

            for asset in assets.clone() {
                block.line(&format!(
                    "Image::{} => ({}, {}, {}, {}),",
                    asset.name, asset.x, asset.y, asset.width, asset.height
                ));
            }

            block
        });

    impl_block.new_fn("image_dimensions")
        .arg_self()
        .vis("pub")
        .ret("(u32, u32)")
        .line(&format!("({}, {})", width, height));

    std::fs::write(filename, &scope.to_string())
}
