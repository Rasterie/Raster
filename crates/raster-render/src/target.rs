use crate::{Frame, Gpu};
use raster_math::{Rect, Vec2};

/// An offscreen image at the game's resolution, scaled up to the window by a
/// whole number.
///
/// Without this the renderer draws straight to the window, so a 1000-pixel-wide
/// window showing a 320-pixel-wide game scales by 3.125 — and every eighth
/// column of pixels ends up wider than its neighbours. Rendering small and
/// scaling by an integer is the only way to keep every pixel the same size.
pub struct RenderTarget {
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    bind_group: wgpu::BindGroup,
    pipeline: wgpu::RenderPipeline,
    width: u32,
    height: u32,
}

impl RenderTarget {
    /// Creates a target at the game's internal resolution.
    ///
    /// # Panics
    ///
    /// If either dimension is zero.
    pub fn new(gpu: &Gpu, width: u32, height: u32) -> Self {
        assert!(
            width > 0 && height > 0,
            "a render target cannot be {width}x{height}"
        );

        let texture = gpu.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("render target"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            // Le meme format que la surface, sinon le pipeline de dessin
            // devrait etre compile deux fois.
            format: gpu.format(),
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let sampler = gpu.device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("blit sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            // Nearest : l'agrandissement doit dupliquer les pixels, pas les
            // melanger. C'est tout l'interet de passer par cette cible.
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });

        let layout = gpu
            .device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("blit"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });

        let bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("blit"),
            layout: &layout,
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

        let shader = gpu
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("blit"),
                source: wgpu::ShaderSource::Wgsl(include_str!("shaders/blit.wgsl").into()),
            });

        let pipeline_layout = gpu
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("blit"),
                bind_group_layouts: &[Some(&layout)],
                immediate_size: 0,
            });

        let pipeline = gpu
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("blit"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some("vs_main"),
                    buffers: &[],
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some("fs_main"),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: gpu.format(),
                        blend: None,
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: None,
                    polygon_mode: wgpu::PolygonMode::Fill,
                    unclipped_depth: false,
                    conservative: false,
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                multiview_mask: None,
                cache: None,
            });

        Self {
            texture,
            view,
            bind_group,
            pipeline,
            width,
            height,
        }
    }

    /// The view sprites draw into.
    #[must_use]
    pub fn view(&self) -> &wgpu::TextureView {
        &self.view
    }

    #[must_use]
    pub fn size(&self) -> Vec2 {
        Vec2::new(self.width as f32, self.height as f32)
    }

    /// Clears the target to a colour, ready for a frame's sprites.
    pub fn clear(&self, frame: &mut Frame, colour: wgpu::Color) {
        frame
            .encoder
            .begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("clear target"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(colour),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
    }

    /// Scales the target onto the window, centred, with black bars where the
    /// aspect ratios differ.
    pub fn present(&self, frame: &mut Frame, window: Vec2) {
        let viewport = self.viewport(window);

        let mut pass = frame
            .encoder
            .begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("blit"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &frame.view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        // Efface en noir : ce qui reste autour de l'image
                        // agrandie forme les bandes.
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });

        pass.set_viewport(
            viewport.position.x,
            viewport.position.y,
            viewport.size.x,
            viewport.size.y,
            0.0,
            1.0,
        );
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.bind_group, &[]);
        // Trois sommets : le triangle vient de l'index, pas d'un tampon.
        pass.draw(0..3, 0..1);
    }

    /// The integer factor this target scales by in a window of that size.
    #[must_use]
    pub fn scale(&self, window: Vec2) -> u32 {
        let x = (window.x / self.width as f32).floor() as u32;
        let y = (window.y / self.height as f32).floor() as u32;
        x.min(y).max(1)
    }

    /// Where the scaled image sits in the window.
    #[must_use]
    pub fn viewport(&self, window: Vec2) -> Rect {
        let scale = self.scale(window) as f32;
        let size = self.size() * scale;
        Rect::from_position_size(((window - size) * 0.5).floor(), size)
    }

    /// Whether the target still matches the resolution asked for.
    #[must_use]
    pub fn matches(&self, width: u32, height: u32) -> bool {
        self.width == width && self.height == height
    }

    /// The underlying texture, for reading pixels back in a test.
    #[must_use]
    pub fn texture(&self) -> &wgpu::Texture {
        &self.texture
    }
}
