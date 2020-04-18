struct Packed {
    name: String,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
}

use std::io::Write;
use image::GenericImageView;
use std::path::Path;

pub fn pack(
    filenames: impl Iterator<Item=impl AsRef<Path>>,
    output_image: impl AsRef<Path>,
    output_bindings: impl AsRef<Path>,
) -> Result<(), std::io::Error> {
    let mut target = rectangle_pack::GroupedRectsToPlace::new();

    let mut images = std::collections::HashMap::new();
   
    for filename in filenames {
        let filename = filename.as_ref();
        let image = image::open(filename).unwrap();
        let base = filename.file_stem().unwrap();
        target.push_rect(
            base.to_owned(),
            None::<Vec<()>>,
            rectangle_pack::RectToInsert::new(image.width(), image.height(), 1)
        );
        images.insert(base.to_owned(), image);
    }

    let mut size = 0;
    let mut placements = None;

    while placements.is_none() {
        let mut map = std::collections::HashMap::new();
        map.insert((), rectangle_pack::TargetBin::new(size, size, 1));
    
        placements = rectangle_pack::pack_rects(
            &target,
            map,
            &rectangle_pack::volume_heuristic,
            &rectangle_pack::contains_smallest_box
        ).ok();

        size += 64;
    }

    let placements = placements.unwrap();

    create_file(placements.packed_locations().iter().map(|(k, (_, v))| {
        let name = case_style::CaseStyle::guess(k.to_str().unwrap()).unwrap().to_pascalcase();

        Packed {
            name,
            x: v.x(),
            y: v.y(),
            width: v.width(),
            height: v.height(),
        }
    }), output_bindings, size)?;

    let mut base = image::RgbaImage::new(size, size);

    for (k, (_, v)) in placements.packed_locations() {
        let image = images.get(k).unwrap().to_rgba();
        image::imageops::replace(&mut base, &image, v.x(), v.y());
    }

    base.save(output_image).unwrap();

    Ok(())
}

fn create_file<'a>(assets: impl Iterator<Item=Packed> + Clone, filename: impl AsRef<Path>, size: u32) -> Result<(), std::io::Error> {
    let mut file = std::fs::File::create(filename)?;
    //writeln!(file, "#[derive(Clone, Copy, PartialEq, PartialOrd, Debug, Hash)]")?;
    writeln!(file, "pub enum Image {{")?;
    for asset in assets.clone() {
        writeln!(file, "\t{},", asset.name)?;
    }
    writeln!(file, "}}")?;
    writeln!(file, "impl Image {{")?;

    writeln!(file, "\tpub fn coordinates(&self) -> (u32, u32, u32, u32) {{")?;
    writeln!(file, "\t\tmatch self {{")?;
    for asset in assets.clone() {
        writeln!(
            file,
            "\t\t\tImage::{} => ({}, {}, {}, {}),",
            asset.name, asset.x, asset.y, asset.width, asset.height
        )?;
    }
    writeln!(file, "\t\t}}")?;
    writeln!(file, "\t}}")?;

    writeln!(file, "\tpub fn from_u16(i: u16) -> Self {{")?;
    writeln!(file, "\t\tmatch i {{")?;
    for (i, asset) in assets.clone().enumerate() {
        writeln!(file, "\t\t\t{} => Image::{},", i, asset.name)?;
    }
    writeln!(file, "\t\t\t_ => panic!(),")?;
    writeln!(file, "\t\t}}")?;
    writeln!(file, "\t}}")?;

    writeln!(file, "\tpub fn to_u16(&self) -> u16 {{")?;
    writeln!(file, "\t\tmatch self {{")?;
    for (i, asset) in assets.clone().enumerate() {
        writeln!(file, "\t\t\tImage::{} => {},", asset.name, i)?;
    } 
    writeln!(file, "\t\t}}")?;
    writeln!(file, "\t}}")?;

    writeln!(file, "\tpub fn image_size(&self) -> u32 {{")?;
    writeln!(file, "\t\t{}", size)?;
    writeln!(file, "\t}}")?;

    writeln!(file, "}}")?;
    Ok(())
}

#[test]
fn z() {
    pack(
        std::fs::read_dir("images").unwrap().map(|x| x.unwrap().path()),
        "out.png",
        "out.rs",
    ).unwrap();
}

#[test]
fn x() {
   // let images = [Packed {name: format!("x"), x: 0, y: 0, height: 0, width: 0}];
    //create_file(images.iter(), "s");
}
