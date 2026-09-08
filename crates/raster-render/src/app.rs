use crate::{Gpu, GpuError};
use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowId};

/// How the window should be set up.
#[derive(Debug, Clone)]
pub struct WindowConfig {
    pub title: String,
    /// The window size in logical pixels, before display scaling.
    pub width: u32,
    pub height: u32,
    pub resizable: bool,
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            title: "Raster".to_owned(),
            width: 1280,
            height: 720,
            resizable: true,
        }
    }
}

/// What a game implements to be driven by the engine's frame loop.
///
/// The engine owns the loop and calls in; the game does not own `main`. That is
/// what lets the same game run inside the editor's play-in-editor panel later.
pub trait App {
    /// Called once, after the GPU is ready.
    fn init(&mut self, _gpu: &mut Gpu) {}

    /// Called once per frame, before drawing.
    fn update(&mut self, _dt: f32) {}

    /// Called once per frame to draw.
    fn render(&mut self, gpu: &mut Gpu);

    /// Called when the window is resized, after the surface is reconfigured.
    fn resized(&mut self, _width: u32, _height: u32) {}

    /// Called when the window is asked to close. Returning `false` keeps it
    /// open — a game can use this to show a "save first?" prompt.
    fn close_requested(&mut self) -> bool {
        true
    }
}

/// Runs `app` until its window closes.
///
/// # Errors
///
/// Fails if the event loop cannot start, or if the GPU cannot be brought up.
pub fn run<A: App + 'static>(config: WindowConfig, app: A) -> Result<(), RunError> {
    let event_loop = EventLoop::new().map_err(RunError::EventLoop)?;

    // Poll plutot que Wait : un jeu redessine en continu, il n'attend pas une
    // interaction. Wait conviendrait a un editeur, pas a une boucle de jeu.
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut runner = Runner {
        config,
        app,
        state: None,
        last_frame: std::time::Instant::now(),
        error: None,
        window_error: None,
    };

    event_loop
        .run_app(&mut runner)
        .map_err(RunError::EventLoop)?;

    if let Some(e) = runner.window_error {
        return Err(RunError::Window(e));
    }
    match runner.error {
        Some(e) => Err(RunError::Gpu(e)),
        None => Ok(()),
    }
}

/// Why the engine could not run.
#[derive(Debug)]
pub enum RunError {
    EventLoop(winit::error::EventLoopError),
    Window(winit::error::OsError),
    Gpu(GpuError),
}

impl std::fmt::Display for RunError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EventLoop(e) => write!(f, "the window system failed: {e}"),
            Self::Window(e) => write!(f, "could not create a window: {e}"),
            Self::Gpu(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for RunError {}

/// What exists only once the event loop has started.
///
/// `winit` cannot create a window before `resumed` fires, so the window and the
/// GPU cannot be built in `run` — hence this being an `Option` rather than a
/// plain field.
struct State {
    gpu: Gpu,
    window: Arc<Window>,
}

struct Runner<A: App> {
    config: WindowConfig,
    app: A,
    state: Option<State>,
    last_frame: std::time::Instant,
    /// A startup failure, kept so `run` can report it once the loop exits:
    /// `resumed` has no way to return an error.
    error: Option<GpuError>,
    window_error: Option<winit::error::OsError>,
}

impl<A: App> ApplicationHandler for Runner<A> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // Sur mobile, `resumed` se declenche a chaque retour au premier plan.
        // Sur bureau il n'arrive qu'une fois, mais le garde evite de recreer
        // la fenetre le jour ou une plateforme mobile sera visee.
        if self.state.is_some() {
            return;
        }

        let attributes = Window::default_attributes()
            .with_title(&self.config.title)
            .with_inner_size(LogicalSize::new(self.config.width, self.config.height))
            .with_resizable(self.config.resizable);

        let window = match event_loop.create_window(attributes) {
            Ok(window) => Arc::new(window),
            Err(e) => {
                self.window_error = Some(e);
                event_loop.exit();
                return;
            }
        };

        let gpu = match pollster::block_on(Gpu::new(Arc::clone(&window))) {
            Ok(gpu) => gpu,
            Err(e) => {
                self.error = Some(e);
                event_loop.exit();
                return;
            }
        };

        let mut state = State { gpu, window };
        self.app.init(&mut state.gpu);
        self.last_frame = std::time::Instant::now();
        self.state = Some(state);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let Some(state) = self.state.as_mut() else {
            return;
        };

        match event {
            WindowEvent::CloseRequested => {
                if self.app.close_requested() {
                    event_loop.exit();
                }
            }

            WindowEvent::Resized(size) => {
                state.gpu.resize(size.width, size.height);
                let (width, height) = state.gpu.size();
                self.app.resized(width, height);
            }

            WindowEvent::RedrawRequested => {
                let now = std::time::Instant::now();
                let dt = now.duration_since(self.last_frame).as_secs_f32();
                self.last_frame = now;

                self.app.update(dt);
                self.app.render(&mut state.gpu);
            }

            _ => {}
        }
    }

    /// Redemande une frame des que la file d'evenements est vide.
    ///
    /// C'est ce qui fait tourner la boucle en continu : un jeu redessine sans
    /// attendre d'interaction. Le faire ici plutot que dans `RedrawRequested`
    /// evite de programmer deux redraws pour une seule frame.
    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(state) = self.state.as_ref() {
            state.window.request_redraw();
        }
    }
}
