use crate::sprite::{Instance, SpriteDraw};
use crate::{Camera, Frame, Gpu, Texture};

/// Draws sprites, grouped so that as few draw calls as possible reach the GPU.
///
/// Cleared and refilled every frame: submitting a sprite states an intent for
/// this frame, not a resource the renderer keeps.
pub struct SpriteBatch {
    pipeline: wgpu::RenderPipeline,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    texture_layout: wgpu::BindGroupLayout,

    instance_buffer: wgpu::Buffer,
    instance_capacity: u64,

    /// Sprites submitted this frame, paired with the texture that draws them.
    queued: Vec<(usize, SpriteDraw)>,
    /// Built once per flush and reused, so no allocation happens per frame.
    instances: Vec<Instance>,

    stats: Stats,
}

/// What the last frame cost.
#[derive(Debug, Clone, Copy, Default)]
pub struct Stats {
    pub sprites: u32,
    pub draw_calls: u32,
    pub culled: u32,
}

impl SpriteBatch {
    /// The instance buffer starts here and grows as needed. Large enough that
    /// a typical scene never reallocates, small enough to be trivial memory.
    const INITIAL_CAPACITY: u64 = 1024;

    pub fn new(gpu: &Gpu) -> Self {
        let shader = gpu
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("sprite"),
                source: wgpu::ShaderSource::Wgsl(include_str!("shaders/sprite.wgsl").into()),
            });

        let camera_layout = gpu
            .device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("camera"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let texture_layout =
            gpu.device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("sprite texture"),
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

        // Une mat3x3 en WGSL occupe trois colonnes alignees sur 16 octets.
        let camera_buffer = gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("camera"),
            size: 48,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let camera_bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("camera"),
            layout: &camera_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
        });

        let pipeline_layout = gpu
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("sprite"),
                bind_group_layouts: &[Some(&camera_layout), Some(&texture_layout)],
                immediate_size: 0,
            });

        let pipeline = gpu
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("sprite"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some("vs_main"),
                    buffers: &[Some(Instance::layout())],
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some("fs_main"),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: gpu.format(),
                        // Alpha classique : le sprite se compose sur le fond
                        // selon son canal alpha.
                        blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleStrip,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    // Pas d'elimination des faces : un sprite retourne
                    // inverserait son sens de parcours et disparaitrait.
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

        let instance_buffer = gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("sprite instances"),
            size: Self::INITIAL_CAPACITY * size_of::<Instance>() as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            pipeline,
            camera_buffer,
            camera_bind_group,
            texture_layout,
            instance_buffer,
            instance_capacity: Self::INITIAL_CAPACITY,
            queued: Vec::new(),
            instances: Vec::new(),
            stats: Stats::default(),
        }
    }

    /// The layout textures must be built against.
    #[must_use]
    pub fn texture_layout(&self) -> &wgpu::BindGroupLayout {
        &self.texture_layout
    }

    /// Queues a sprite for this frame.
    ///
    /// `texture` indexes the slice passed to [`SpriteBatch::flush`].
    pub fn draw(&mut self, texture: usize, sprite: SpriteDraw) {
        self.queued.push((texture, sprite));
    }

    /// Draws everything queued into the frame's own surface.
    ///
    /// Convenience for drawing straight to the window. A game wanting
    /// pixel-perfect scaling draws into a [`RenderTarget`](crate::RenderTarget)
    /// with [`SpriteBatch::flush_into`] instead.
    pub fn flush(&mut self, gpu: &Gpu, frame: &mut Frame, camera: Camera, textures: &[Texture]) {
        /*
          La vue et l'encodeur sont deux champs distincts de la frame : les
          emprunter separement est correct, mais le compilateur ne le voit pas
          a travers un appel de methode. On decoupe donc l'emprunt ici.
        */
        let Frame { view, encoder, .. } = frame;
        self.flush_parts(gpu, encoder, view, camera, textures);
    }

    /// Draws everything queued into `view`, then clears the queue.
    ///
    /// Sprites outside the camera's view are dropped before reaching the GPU.
    pub fn flush_into(
        &mut self,
        gpu: &Gpu,
        frame: &mut Frame,
        view: &wgpu::TextureView,
        camera: Camera,
        textures: &[Texture],
    ) {
        self.flush_parts(gpu, &mut frame.encoder, view, camera, textures);
    }

    fn flush_parts(
        &mut self,
        gpu: &Gpu,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
        camera: Camera,
        textures: &[Texture],
    ) {
        self.stats = Stats::default();

        if self.queued.is_empty() || textures.is_empty() {
            self.queued.clear();
            return;
        }

        self.upload_camera(gpu, camera);

        /*
          Tri par (couche, texture) : la couche impose l'ordre de dessin, et
          regrouper par texture ensuite permet un seul appel de dessin par
          groupe plutot qu'un par sprite.
        */
        self.queued
            .sort_by_key(|(texture, sprite)| (sprite.layer, *texture));

        let visible = camera.visible_area();
        self.instances.clear();

        // Chaque tranche partage une texture : un appel de dessin par tranche.
        let mut runs: Vec<(usize, u32, u32)> = Vec::new();

        for (texture, sprite) in &self.queued {
            if !visible.intersects(sprite.bounds()) {
                self.stats.culled += 1;
                continue;
            }

            let Some(atlas) = textures.get(*texture) else {
                continue;
            };

            let start = u32::try_from(self.instances.len()).unwrap_or(u32::MAX);
            self.instances.push(Instance::new(sprite, atlas.size()));

            match runs.last_mut() {
                Some((last_texture, _, count)) if *last_texture == *texture => *count += 1,
                _ => runs.push((*texture, start, 1)),
            }
        }

        self.queued.clear();

        if self.instances.is_empty() {
            return;
        }

        self.upload_instances(gpu);

        self.stats.sprites = u32::try_from(self.instances.len()).unwrap_or(u32::MAX);
        self.stats.draw_calls = u32::try_from(runs.len()).unwrap_or(u32::MAX);

        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("sprites"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    // Load : le fond a deja ete efface, et repasser dessus
                    // effacerait ce que d'autres passes ont dessine.
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });

        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.camera_bind_group, &[]);
        pass.set_vertex_buffer(0, self.instance_buffer.slice(..));

        for (texture, start, count) in runs {
            let Some(atlas) = textures.get(texture) else {
                continue;
            };
            pass.set_bind_group(1, &atlas.bind_group, &[]);
            // Quatre sommets : le quad vient de l'index, pas d'un tampon.
            pass.draw(0..4, start..start + count);
        }
    }

    /// What the last flush cost.
    #[must_use]
    pub fn stats(&self) -> Stats {
        self.stats
    }

    fn upload_camera(&self, gpu: &Gpu, camera: Camera) {
        let matrix = camera.view_projection().to_cols_array_padded();
        gpu.queue
            .write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&matrix));
    }

    fn upload_instances(&mut self, gpu: &Gpu) {
        let needed = self.instances.len() as u64;

        // Croissance par doublement : une scene qui grandit progressivement ne
        // reallouerait pas a chaque frame.
        if needed > self.instance_capacity {
            self.instance_capacity = needed.next_power_of_two();
            self.instance_buffer = gpu.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("sprite instances"),
                size: self.instance_capacity * size_of::<Instance>() as u64,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
        }

        gpu.queue.write_buffer(
            &self.instance_buffer,
            0,
            bytemuck::cast_slice(&self.instances),
        );
    }
}
