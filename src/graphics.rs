pub struct Loader<'a> {
    pub device: &'a wgpu::Device,
    pub encoder: &'a mut wgpu::CommandEncoder,
}

fn load_png(bytes: &'static [u8], loader: &mut Loader) -> Sprite {
    let image = image::load_from_memory_with_format(bytes, image::ImageFormat::Png).unwrap()
        .into_rgba();

    let temp_buf =
        loader.device.create_buffer_with_data( &*image, wgpu::BufferUsage::COPY_SRC);

    let texture_extent = wgpu::Extent3d {
        width: image.width(),
        height: image.height(),
        depth: 1,
    };

    let texture = loader.device.create_texture(&wgpu::TextureDescriptor {
        size: texture_extent,
        array_layer_count: 1,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsage::SAMPLED | wgpu::TextureUsage::COPY_DST,
    });


    loader.encoder.copy_buffer_to_texture(
        wgpu::BufferCopyView {
            buffer: &temp_buf,
            offset: 0,
            bytes_per_row: 4 * image.width(),
            rows_per_image: image.height(),
        },
        wgpu::TextureCopyView {
            texture: &texture,
            mip_level: 0,
            array_layer: 0,
            origin: wgpu::Origin3d::ZERO,
        },
        texture_extent,
    );

    let texture_view = texture.create_default_view();

    Sprite {
        texture_view
    }
}

pub struct Sprite {
    pub texture_view: wgpu::TextureView,
}

pub struct Resources {
    pub sprites: Sprite
}

include!(concat!(env!("OUT_DIR"), "/image.rs"));

impl Resources {
    pub fn load(device: &wgpu::Device, encoder: &mut wgpu::CommandEncoder) -> Self {
        let loader = &mut Loader {device, encoder};

        Self {
            sprites: load_png(include_bytes!(concat!(env!("OUT_DIR"), "/packed.png")), loader),
        }
    }
}

impl Default for Resources {
    fn default() -> Self {
        panic!()
    }
}
