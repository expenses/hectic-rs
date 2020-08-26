use wgpu::util::DeviceExt;

fn load_png(bytes: &'static [u8], device: &wgpu::Device, encoder: &mut wgpu::CommandEncoder) -> wgpu::TextureView {
    let image = image::load_from_memory_with_format(bytes, image::ImageFormat::Png).unwrap()
        .into_rgba();

    let temp_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Hectic texture buffer"),
        contents: &*image,
        usage: wgpu::BufferUsage::COPY_SRC,
    });

    let texture_extent = wgpu::Extent3d {
        width: image.width(),
        height: image.height(),
        depth: 1,
    };

    let texture = device.create_texture(&wgpu::TextureDescriptor {
        size: texture_extent,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsage::SAMPLED | wgpu::TextureUsage::COPY_DST,
        label: Some("Hectic Texture".into()),
    });

    encoder.copy_buffer_to_texture(
        wgpu::BufferCopyView {
            buffer: &temp_buf,
            layout: wgpu::TextureDataLayout {
                offset: 0,
                bytes_per_row: 4 * image.width(),
                rows_per_image: 0,
            },
        },
        wgpu::TextureCopyView {
            texture: &texture,
            mip_level: 0,
            //array_layer: 0,
            origin: wgpu::Origin3d::ZERO,
        },
        texture_extent,
    );

    texture.create_view(&wgpu::TextureViewDescriptor::default())
}

include!(concat!(env!("OUT_DIR"), "/image.rs"));

pub fn load_packed(device: &wgpu::Device, encoder: &mut wgpu::CommandEncoder) -> wgpu::TextureView {
    load_png(include_bytes!(concat!(env!("OUT_DIR"), "/packed.png")), device, encoder)
}
