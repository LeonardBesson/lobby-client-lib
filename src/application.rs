use crate::renderer::Renderer;
use crate::time::{FrameLimit, FrameLimitStrategy, Time};
use imgui::Key;
use lobby_lib::net::packets::*;
use lobby_lib::net::{packets, Net};
use std::time::{Duration, Instant};
use winit::dpi::PhysicalSize;
use winit::event::{Event, WindowEvent};
use winit::event_loop::ControlFlow;
use winit::event_loop::EventLoop;
use winit::window::{Window, WindowBuilder};
use crate::ui::Ui;
use crate::ui::screens::login_screen::LoginScreen;

pub enum State {
    Boot,
    Initialize,
    Run,
    Shutdown,
}

pub struct Application {
    pub state: State,
    pub time: Time,
    pub max_fps: u32,
    pub frame_limit: FrameLimit,
    pub ui: Ui,
}

impl Application {
    pub fn new() -> Self {
        let max_fps = 60;
        Self {
            state: State::Boot,
            time: Time::new(),
            max_fps,
            frame_limit: FrameLimit::new(
                FrameLimitStrategy::SleepAndYield(Duration::from_millis(2)),
                max_fps,
            ),
            ui: Ui::new(),
        }
    }

    fn initialize(&mut self, window: &Window, renderer: &mut Renderer) {
        packets::init();
        self.ui.init(window, renderer);
        self.ui.push_screen(Box::new(LoginScreen::new()));
    }

    fn tick(&mut self, renderer: &mut Renderer, window: &Window) {
        self.update(renderer, window);
        self.render(renderer, window);

        self.frame_limit.run();

        self.time.tick(&self.frame_limit);
    }

    fn handle_window_event(&mut self, event: &WindowEvent, renderer: &mut Renderer) {
        match event {
            WindowEvent::CloseRequested => self.state = State::Shutdown,
            WindowEvent::Resized(physical_size) => {
                self.resize(renderer, *physical_size);
            }
            WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                self.resize(renderer, **new_inner_size);
            }
            _ => self.input(event),
        }
    }

    fn input(&mut self, event: &WindowEvent) {}

    fn resize(&mut self, renderer: &mut Renderer, new_size: winit::dpi::PhysicalSize<u32>) {
        renderer.resize(new_size);
    }

    fn update(&mut self, renderer: &mut Renderer, window: &Window) {
        renderer.update(self.time.delta);
    }

    fn render(&mut self, renderer: &mut Renderer, window: &Window) {
        renderer.render(&mut self.ui, window, &self.time);
    }

    fn shutdown(&mut self) {}

    pub async fn run(mut self) {
        let event_loop = EventLoop::new();
        let window = WindowBuilder::new()
            .with_inner_size(PhysicalSize::new(1024, 768))
            .with_min_inner_size(PhysicalSize::new(640, 480))
            .with_title("Lobby client example")
            .build(&event_loop)
            .expect("Could not create window");

        let mut renderer = Renderer::new(&window).await;
        window.set_cursor_grab(false);
        window.set_cursor_visible(true);

        event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Poll;

            match self.state {
                State::Boot => {
                    self.state = State::Initialize;
                }
                State::Initialize => {
                    self.initialize(&window, &mut renderer);
                    self.state = State::Run;
                }
                State::Run => {
                    match event {
                        Event::MainEventsCleared => window.request_redraw(),
                        Event::WindowEvent {
                            ref event,
                            window_id,
                        } if window_id == window.id() => {
                            self.handle_window_event(event, &mut renderer);
                        }
                        Event::RedrawRequested(_) => {
                            self.tick(&mut renderer, &window);
                        }
                        _ => {}
                    }
                    self.ui.handle_event(&window, &event);
                }
                State::Shutdown => {
                    self.shutdown();
                    *control_flow = ControlFlow::Exit;
                }
            }
        });
    }
}
