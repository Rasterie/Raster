use std::sync::Arc;
use winit::window::Window;

/// Why the GPU could not be brought up.
#[derive(Debug)]
pub enum GpuError {
    /// No adapter met the requirements — no compatible GPU, or a driver too old.
    NoAdapter,
    /// An adapter was found but refused to hand out a device.
    NoDevice(wgpu::RequestDeviceError),
    /// The window could not provide a drawable surface.
    NoSurface(wgpu::CreateSurfaceError),
}

impl std::fmt::Display for GpuError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoAdapter => write!(
                f,
                "no compatible GPU found; the driver may be too old or missing"
            ),
            Self::NoDevice(e) => write!(f, "the GPU refused to create a device: {e}"),
            Self::NoSurface(e) => write!(f, "could not draw to this window: {e}"),
        }
    }
}

impl std::error::Error for GpuError {}

/// The GPU device and the surface it draws to.
///
/// Detient un `Arc<Window>` : la surface ne doit pas survivre a sa fenetre.
pub struct Gpu {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    surface: wgpu::Surface<'static>,
    config: wgpu::SurfaceConfiguration,
    window: Arc<Window>,
}

impl Gpu {
    /// Brings up a device and configures a surface for `window`.
    pub async fn new(window: Arc<Window>) -> Result<Self, GpuError> {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            flags: wgpu::InstanceFlags::default(),
            memory_budget_thresholds: wgpu::MemoryBudgetThresholds::default(),
            backend_options: wgpu::BackendOptions::default(),
            display: None,
        });

        let surface = instance
            .create_surface(Arc::clone(&window))
            .map_err(GpuError::NoSurface)?;

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                // Le rendu 2D ne sature pas un GPU dedie ; sur portable, le
                // circuit integre economise la batterie sans rien couter.
                power_preference: wgpu::PowerPreference::LowPower,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
                apply_limit_buckets: false,
            })
            .await
            .map_err(|_| GpuError::NoAdapter)?;

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("raster device"),
                required_features: wgpu::Features::empty(),
                // Les limites les plus basses garanties partout : un moteur 2D
                // n'a besoin de rien de plus, et exiger davantage exclurait des
                // machines sans contrepartie.
                required_limits: wgpu::Limits::downlevel_webgl2_defaults()
                    .using_resolution(adapter.limits()),
                experimental_features: wgpu::ExperimentalFeatures::disabled(),
                memory_hints: wgpu::MemoryHints::Performance,
                trace: wgpu::Trace::Off,
            })
            .await
            .map_err(GpuError::NoDevice)?;

        let capabilities = surface.get_capabilities(&adapter);
        let format = capabilities
            .formats
            .iter()
            .copied()
            .find(wgpu::TextureFormat::is_srgb)
            .unwrap_or(capabilities.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            color_space: wgpu::SurfaceColorSpace::Srgb,
            // Une dimension nulle est refusee par wgpu, et une fenetre peut
            // demarrer ou etre reduite a zero — notamment sous Windows.
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: wgpu::PresentMode::AutoVsync,
            alpha_mode: capabilities.alpha_modes[0],
            view_formats: Vec::new(),
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        Ok(Self {
            device,
            queue,
            surface,
            config,
            window,
        })
    }

    /// Reconfigure la surface apres un redimensionnement. Une dimension nulle
    /// est ignoree : reduire une fenetre sous Windows rapporte (0, 0).
    pub fn resize(&mut self, width: u32, height: u32) {
        if width == 0 || height == 0 || (width == self.config.width && height == self.config.height)
        {
            return;
        }
        self.config.width = width;
        self.config.height = height;
        self.surface.configure(&self.device, &self.config);
    }

    /// La frame suivante, ou `None` s'il faut la sauter — ce qui est normal
    /// pendant un redimensionnement ou un changement d'ecran.
    pub fn begin_frame(&mut self) -> Option<Frame> {
        use wgpu::CurrentSurfaceTexture as Current;

        let texture = match self.surface.get_current_texture() {
            Current::Success(texture) => texture,

            // Utilisable, mais la surface a change : on dessine cette frame et
            // on reconfigure pour la suivante.
            Current::Suboptimal(texture) => {
                self.surface.configure(&self.device, &self.config);
                texture
            }

            // La surface doit etre reconfiguree avant de pouvoir redessiner.
            Current::Outdated | Current::Lost => {
                self.surface.configure(&self.device, &self.config);
                return None;
            }

            // Rien a faire : la fenetre est masquee, ou le GPU a mis trop de
            // temps. La frame suivante repartira normalement.
            Current::Timeout | Current::Occluded => return None,

            // Une erreur de validation signale un bug dans le moteur, pas une
            // condition transitoire. Sauter la frame evite d'y ajouter un
            // plantage, et l'erreur remonte par le scope d'erreur de wgpu.
            Current::Validation => return None,
        };

        let view = texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("raster frame"),
            });

        Some(Frame {
            texture,
            view,
            encoder,
        })
    }

    /// Submits a frame and presents it.
    pub fn end_frame(&mut self, frame: Frame) {
        self.queue.submit(std::iter::once(frame.encoder.finish()));
        self.window.pre_present_notify();
        self.queue.present(frame.texture);
    }

    #[must_use]
    pub fn size(&self) -> (u32, u32) {
        (self.config.width, self.config.height)
    }

    #[must_use]
    pub fn format(&self) -> wgpu::TextureFormat {
        self.config.format
    }

    /// The present mode actually in use, which may differ from the one asked
    /// for if the platform does not support it.
    #[must_use]
    pub fn present_mode(&self) -> wgpu::PresentMode {
        self.config.present_mode
    }

    #[must_use]
    pub fn window(&self) -> &Window {
        &self.window
    }
}

/// One frame being built.
pub struct Frame {
    texture: wgpu::SurfaceTexture,
    pub view: wgpu::TextureView,
    pub encoder: wgpu::CommandEncoder,
}

impl Frame {
    /// Efface la frame a une couleur.
    pub fn clear(&mut self, colour: wgpu::Color) {
        self.encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("clear"),
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
}
