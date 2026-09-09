use crate::input_bridge::{translate_key, translate_mouse_button};
use crate::{Gpu, GpuError};
use raster_core::{FrameLoop, Time};
use raster_input::Input;
use raster_math::Vec2;
use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::{ElementState, MouseScrollDelta, WindowEvent};
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
/// Le moteur possede la boucle : c'est ce qui permettra de faire tourner le
/// meme jeu dans le panneau de l'editeur.
pub trait App {
    /// Called once, after the GPU is ready.
    fn init(&mut self, _gpu: &mut Gpu) {}

    /// Appele a cadence fixe, zero a plusieurs fois par frame. Le mouvement et
    /// la physique vont ici, sinon ils dependent de la machine.
    fn fixed_update(&mut self, _input: &Input, _dt: f32) {}

    /// Appele une fois par frame, avant le dessin. Pour ce qui doit suivre la
    /// frequence d'images : camera, interface, effets.
    fn update(&mut self, _input: &mut Input, _time: &Time) {}

    /// Called once per frame to draw.
    fn render(&mut self, gpu: &mut Gpu);

    /// Called when the window is resized, after the surface is reconfigured.
    fn resized(&mut self, _width: u32, _height: u32) {}

    /// Called when the window is asked to close. Returning `false` keeps it
    /// open — a game can use this to show a "save first?" prompt.
    fn close_requested(&mut self) -> bool {
        true
    }

    /// Si le jeu veut s'arreter. Distinct de `close_requested`, qui repond a
    /// la fenetre : ici c'est le jeu qui decide.
    fn should_exit(&self) -> bool {
        false
    }
}

/// Runs `app` until its window closes.
///
/// # Errors
///
/// Fails if the event loop cannot start, or if the GPU cannot be brought up.
pub fn run<A: App + 'static>(config: WindowConfig, app: A) -> Result<(), RunError> {
    let event_loop = EventLoop::new().map_err(RunError::EventLoop)?;

    // Poll : un jeu redessine en continu, il n'attend pas d'interaction.
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut runner = Runner {
        config,
        app,
        state: None,
        last_frame: std::time::Instant::now(),
        input: Input::default(),
        frame_loop: FrameLoop::default(),
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

/// Ce qui n'existe qu'une fois la boucle demarree : `winit` ne cree pas de
/// fenetre avant `resumed`.
struct State {
    gpu: Gpu,
    window: Arc<Window>,
}

struct Runner<A: App> {
    config: WindowConfig,
    app: A,
    state: Option<State>,
    last_frame: std::time::Instant,
    input: Input,
    frame_loop: FrameLoop,
    /// A startup failure, kept so `run` can report it once the loop exits:
    /// `resumed` has no way to return an error.
    error: Option<GpuError>,
    window_error: Option<winit::error::OsError>,
}

impl<A: App> ApplicationHandler for Runner<A> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // `resumed` se redeclenche a chaque retour au premier plan sur mobile.
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

            WindowEvent::KeyboardInput { event, .. } => {
                // Ignore les repetitions systeme : une touche maintenue ne doit
                // pas produire des pressions repetees.
                if event.repeat {
                    return;
                }
                if let Some(key) = translate_key(event.physical_key) {
                    match event.state {
                        ElementState::Pressed => self.input.key_down(key),
                        ElementState::Released => self.input.key_up(key),
                    }
                }
            }

            WindowEvent::MouseInput {
                state: button_state,
                button,
                ..
            } => {
                if let Some(button) = translate_mouse_button(button) {
                    match button_state {
                        ElementState::Pressed => self.input.mouse_down(button),
                        ElementState::Released => self.input.mouse_up(button),
                    }
                }
            }

            WindowEvent::CursorMoved { position, .. } => {
                self.input
                    .set_mouse_position(Vec2::new(position.x as f32, position.y as f32));
            }

            WindowEvent::MouseWheel { delta, .. } => {
                let amount = match delta {
                    MouseScrollDelta::LineDelta(_, y) => y,
                    // Ramene au cran de molette : un pave tactile donnerait
                    // sinon des valeurs cent fois plus grandes.
                    MouseScrollDelta::PixelDelta(p) => p.y as f32 / 50.0,
                };
                self.input.add_scroll(amount);
            }

            WindowEvent::Focused(false) => {
                // Sans cela une touche maintenue resterait enfoncee : son
                // relachement partirait a l'autre fenetre.
                self.input.release_all();
            }

            WindowEvent::RedrawRequested => {
                let now = std::time::Instant::now();
                let dt = now.duration_since(self.last_frame).as_secs_f32();
                self.last_frame = now;

                let steps = self.frame_loop.advance(dt);
                self.input.begin_frame(dt);

                // Pas fixes, mise a jour, dessin : l'ordre de game.md.
                for fixed_dt in steps {
                    self.app.fixed_update(&self.input, fixed_dt);
                }

                self.app.update(&mut self.input, self.frame_loop.time());
                self.app.render(&mut state.gpu);

                if self.app.should_exit() {
                    event_loop.exit();
                }
            }

            _ => {}
        }
    }

    /// Redemande une frame des que la file est vide : c'est ce qui fait
    /// tourner la boucle en continu.
    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(state) = self.state.as_ref() {
            state.window.request_redraw();
        }
    }
}
