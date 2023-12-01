pub use bytemuck::Zeroable;
// pub use rodio::{Sink, Source};
// pub use std::io::{BufReader, Cursor};
pub use frenderer::{
    input::{Input, Key},
    wgpu, BitFont, Camera2D as Camera, Frenderer, SheetRegion, Transform,
};
pub trait Game: Sized + 'static {
    fn new(engine: &mut Engine) -> Self;
    fn update(&mut self, engine: &mut Engine, acc: f32);
    fn is_game_over(&self) -> bool;
    fn render(&mut self, engine: &mut Engine);
    // fn play_sound(&self, engine: &mut Engine, sound_data: &[u8]);
}

pub struct Engine {
    pub renderer: Frenderer,
    pub input: Input,
    // pub audio_sink: Sink,
    event_loop: Option<winit::event_loop::EventLoop<()>>,
    window: winit::window::Window,
}

impl Engine {
    pub fn new(builder: winit::window::WindowBuilder) -> Self {
        let event_loop = winit::event_loop::EventLoop::new();
        let window_width = 768.0;
        let window_height  = 1280.0;
        let window = builder.with_inner_size(winit::dpi::LogicalSize::new(window_width, window_height)).build(&event_loop).unwrap();
        let renderer = frenderer::with_default_runtime(&window);
        // let audio_device = rodio::default_output_device().unwrap();
        // let audio_sink = Sink::new(&audio_device);
        let input = Input::default();
        Self {
            renderer,
            input,
            // audio_sink,
            window,
            event_loop: Some(event_loop),
        }
    }
    pub fn run<G: Game>(mut self) {
        let mut game = G::new(&mut self);
        const DT: f32 = 1.0 / 60.0;
        const DT_FUDGE_AMOUNT: f32 = 0.0002;
        const DT_MAX: f32 = DT * 5.0;
        const TIME_SNAPS: [f32; 5] = [15.0, 30.0, 60.0, 120.0, 144.0];
        let mut acc = 0.0;
        let mut now = std::time::Instant::now();
        self.event_loop
            .take()
            .unwrap()
            .run(move |event, _, control_flow| {
                use winit::event::{Event, WindowEvent};
                control_flow.set_poll();
                match event {
                    Event::WindowEvent {
                        event: WindowEvent::CloseRequested,
                        ..
                    } => {
                        *control_flow = winit::event_loop::ControlFlow::Exit;
                    }
                    Event::MainEventsCleared => {
                        // end game if there is a collision
                        if G::is_game_over(&game) == true {
                            *control_flow = winit::event_loop::ControlFlow::Exit;
                        }
                        // compute elapsed time since last frame
                        let mut elapsed = now.elapsed().as_secs_f32();
                        // println!("{elapsed}");
                        // snap time to nearby vsync framerate
                        TIME_SNAPS.iter().for_each(|s| {
                            if (elapsed - 1.0 / s).abs() < DT_FUDGE_AMOUNT {
                                elapsed = 1.0 / s;
                            }
                        });
                        // Death spiral prevention
                        if elapsed > DT_MAX {
                            acc = 0.0;
                            elapsed = DT;
                        }
                        acc += elapsed;
                        now = std::time::Instant::now();
                        // While we have time to spend
                        while acc >= DT {
                            // simulate a frame
                            acc -= DT;
                            game.update(&mut self, acc);
                            self.input.next_frame();
                        }
                        game.render(&mut self);
                        // Render prep
                        //self.renderer.sprites.set_camera_all(&frend.gpu, camera);
                        // update sprite positions and sheet regions
                        self.renderer.render();
                        self.window.request_redraw();
                    }
                    event => {
                        if self.renderer.process_window_event(&event) {
                            self.window.request_redraw();
                        }
                        self.input.process_input_event(&event);
                    }
                }
            });
    }
}
pub mod geom;
