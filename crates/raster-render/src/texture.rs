use crate::Gpu;
use raster_math::Vec2;

/// An image on the GPU, ready to be sampled.
pub struct Texture {
    pub(crate) bind_group: wgpu::BindGroup,
    width: u32,
    height: u32,
}

impl Texture {
    /// Uploads RGBA8 pixel data, `width * height * 4` octets sans remplissage.
    ///
    /// # Panics
    ///
    /// Si la longueur ou une dimension est fausse : une texture construite sur
    /// de mauvaises donnees afficherait n'importe quoi sans le dire.
    pub fn from_rgba(
        gpu: &Gpu,
        layout: &wgpu::BindGroupLayout,
        pixels: &[u8],
        width: u32,
        height: u32,
    ) -> Self {
        assert!(
            width > 0 && height > 0,
            "a texture cannot be {width}x{height}"
        );
        assert_eq!(
            pixels.len(),
            (width as usize) * (height as usize) * 4,
            "expected {} bytes for a {width}x{height} RGBA texture, got {}",
            (width as usize) * (height as usize) * 4,
            pixels.len(),
        );

        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        let texture = gpu.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("sprite texture"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            // Srgb : sans lui le GPU traiterait l'image comme lineaire et les
            // couleurs sortiraient trop claires.
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        gpu.queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            pixels,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * width),
                rows_per_image: Some(height),
            },
            size,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let sampler = gpu.device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("pixel art sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            // Nearest partout : une interpolation lineaire rendrait chaque
            // sprite flou des la premiere mise a l'echelle.
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });

        let bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("sprite texture"),
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });

        Self {
            bind_group,
            width,
            height,
        }
    }

    /// Uploads a decoded image.
    #[must_use]
    pub fn from_image(gpu: &Gpu, layout: &wgpu::BindGroupLayout, image: &crate::Image) -> Self {
        Self::from_rgba(gpu, layout, &image.pixels, image.width, image.height)
    }

    /// Reads a PNG from disk and uploads it.
    ///
    /// # Errors
    ///
    /// Fails if the file cannot be read or decoded — see [`crate::ImageError`].
    pub fn load(
        gpu: &Gpu,
        layout: &wgpu::BindGroupLayout,
        path: impl AsRef<std::path::Path>,
    ) -> Result<Self, crate::ImageError> {
        Ok(Self::from_image(gpu, layout, &crate::Image::load(path)?))
    }

    /// Le damier magenta d'une texture manquante : voyant a dessein.
    #[must_use]
    pub fn placeholder(gpu: &Gpu, layout: &wgpu::BindGroupLayout) -> Self {
        const SIZE: u32 = 16;
        const CELL: u32 = 4;

        let mut pixels = Vec::with_capacity((SIZE * SIZE * 4) as usize);
        for y in 0..SIZE {
            for x in 0..SIZE {
                let magenta = ((x / CELL) + (y / CELL)).is_multiple_of(2);
                pixels.extend_from_slice(if magenta {
                    &[255, 0, 255, 255]
                } else {
                    &[16, 16, 16, 255]
                });
            }
        }

        Self::from_rgba(gpu, layout, &pixels, SIZE, SIZE)
    }

    #[must_use]
    pub fn size(&self) -> Vec2 {
        Vec2::new(self.width as f32, self.height as f32)
    }

    #[must_use]
    pub fn width(&self) -> u32 {
        self.width
    }

    #[must_use]
    pub fn height(&self) -> u32 {
        self.height
    }
}
