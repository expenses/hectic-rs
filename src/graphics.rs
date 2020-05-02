fn load_png(bytes: &'static [u8], device: &wgpu::Device, encoder: &mut wgpu::CommandEncoder) -> wgpu::TextureView {
    let image = image::load_from_memory_with_format(bytes, image::ImageFormat::Png).unwrap()
        .into_rgba();

    let temp_buf =
        device.create_buffer_with_data(&*image, wgpu::BufferUsage::COPY_SRC);

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

    encoder.copy_buffer_to_texture(
        wgpu::BufferCopyView {
            buffer: &temp_buf,
            offset: 0,
            bytes_per_row: 4 * image.width(),
            rows_per_image: 0,
        },
        wgpu::TextureCopyView {
            texture: &texture,
            mip_level: 0,
            array_layer: 0,
            origin: wgpu::Origin3d::ZERO,
        },
        texture_extent,
    );

    texture.create_view(&wgpu::TextureViewDescriptor {
        format: wgpu::TextureFormat::Rgba8Unorm,
        dimension: wgpu::TextureViewDimension::D2,
        aspect: wgpu::TextureAspect::All,
        base_mip_level: 0,
        level_count: 1,
        base_array_layer: 0,
        array_layer_count: 0,
        label: Some("Hectic TextureView")
    })
}

include!(concat!(env!("OUT_DIR"), "/image.rs"));

pub fn load_packed(device: &wgpu::Device, encoder: &mut wgpu::CommandEncoder) -> wgpu::TextureView {
    load_png(include_bytes!(concat!(env!("OUT_DIR"), "/packed.png")), device, encoder)
}
