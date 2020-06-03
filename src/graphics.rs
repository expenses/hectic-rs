include!(concat!(env!("OUT_DIR"), "/image.rs"));

pub fn load_packed(device: &wgpu::Device, queue: &wgpu::Queue) -> wgpu::TextureView {
    let bytes = include_bytes!(concat!(env!("OUT_DIR"), "/packed.png"));

    let image = image::load_from_memory_with_format(bytes, image::ImageFormat::Png).unwrap()
        .into_rgba();

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
        label: Some("Hectic Texture"),
    });

    queue.write_texture(
        wgpu::TextureCopyView {
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
        },
        &*image,
        wgpu::TextureDataLayout {
            offset: 0,
            bytes_per_row: 4 * image.width(),
            rows_per_image: 0,
        },
        texture_extent,
    );

    texture.create_default_view()
}
