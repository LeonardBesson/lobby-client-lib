use crate::renderer::Renderer;
use crate::time::{FrameLimit, FrameLimitStrategy, Time};
use crate::ui::screens::chat_screen::ChatScreen;
use crate::ui::screens::events_screen::EventScreen;
use crate::ui::screens::friend_list_screen::FriendListScreen;
use crate::ui::screens::home_screen::HomeScreen;
use crate::ui::screens::login_screen::LoginScreen;
use crate::ui::screens::root_screen::RootScreen;
use crate::ui::Ui;
use crossbeam_channel::{unbounded, Receiver, Sender};
use lobby_lib::net::packets;
use lobby_lib::net::packets::*;
use lobby_lib::net::structs::FriendRequestActionChoice;
use lobby_lib::{net, LobbyClient, LobbyClientBuilder, LobbyEvent};
use std::collections::VecDeque;
use std::time::{Duration, Instant};
use winit::dpi::PhysicalSize;
use winit::event::{Event, WindowEvent};
use winit::event_loop::ControlFlow;
use winit::event_loop::EventLoop;
use winit::window::{Window, WindowBuilder};

pub enum State {
    Boot,
    Initialize,
    Run,
    Shutdown,
}

pub enum Action {
    Exit,
    Login {
        email: String,
        password: String,
    },
    AddFriend {
        user_tag: String,
    },
    FriendRequestAction {
        request_id: String,
        action: FriendRequestActionChoice,
    },
    RemoveFriend {
        user_tag: String,
    },
    SendPrivateMessage {
        user_tag: String,
        content: String,
    },
    InviteUser {
        user_tag: String,
    },
}

// Wrapper struct to reduce boiler plate
pub struct Lobby {
    pub client: LobbyClient,
    pub events: Vec<LobbyEvent>,
}

pub struct Application {
    pub state: State,
    pub time: Time,
    pub max_fps: u32,
    pub frame_limit: FrameLimit,
    pub ui: Ui,
    pub lobby: Lobby,
    pub action_receiver: Receiver<Action>,
}

impl Application {
    pub fn new() -> Self {
        let lobby_client = match LobbyClientBuilder::new("127.0.0.1:9000")
            .with_reconnect_interval(Duration::from_secs(10))
            .build()
        {
            Ok(client) => client,
            Err(err) => {
                panic!("Couldn't create lobby client:  {:?}", err);
            }
        };
        let (action_sender, action_receiver) = unbounded();
        let max_fps = 60;
        Self {
            state: State::Boot,
            time: Time::new(),
            max_fps,
            frame_limit: FrameLimit::new(
                FrameLimitStrategy::SleepAndYield(Duration::from_millis(2)),
                max_fps,
            ),
            ui: Ui::new(action_sender),
            lobby: Lobby {
                client: lobby_client,
                events: Vec::with_capacity(256),
            },
            action_receiver,
        }
    }

    fn initialize(&mut self, window: &Window, renderer: &mut Renderer) {
        packets::init();
        self.lobby.client.connect();
        self.ui.init(window, renderer);
        self.ui.push_screen("RootScreen", Box::new(RootScreen));
        self.ui
            .push_screen("EventScreen", Box::new(EventScreen::new()));
        self.ui
            .push_screen("LoginScreen", Box::new(LoginScreen::new()));
    }

    fn tick(&mut self, renderer: &mut Renderer, window: &Window) {
        self.update(renderer, window);
        self.render(renderer, window);

        let now = Instant::now();
        let timeout = if now > self.time.next_wanted_tick {
            crate::time::ZERO
        } else {
            self.time.next_wanted_tick - now
        };
        self.lobby.client.tick(timeout);
        self.lobby.client.poll_events(&mut self.lobby.events);

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
        while let Ok(action) = self.action_receiver.try_recv() {
            match action {
                Action::Login { email, password } => {
                    self.lobby.client.authenticate(email, password);
                }
                Action::Exit => {
                    self.state = State::Shutdown;
                }
                Action::AddFriend { user_tag } => {
                    self.lobby.client.add_friend(user_tag);
                }
                Action::FriendRequestAction { request_id, action } => {
                    self.lobby.client.friend_request_action(request_id, action);
                }
                Action::RemoveFriend { user_tag } => {
                    self.lobby.client.remove_friend(user_tag);
                }
                Action::SendPrivateMessage { user_tag, content } => {
                    self.lobby.client.send_private_message(user_tag, content);
                }
                Action::InviteUser { user_tag } => {
                    self.lobby.client.invite_user(user_tag);
                }
            }
        }
        for event in &self.lobby.events {
            match event {
                LobbyEvent::AuthSuccess { user_profile, .. } => {
                    self.ui.replace_screen(
                        "LoginScreen",
                        "HomeScreen",
                        Box::new(HomeScreen::new()),
                    );
                    self.ui
                        .push_screen("Friends", Box::new(FriendListScreen::new()));
                    self.ui.push_screen(
                        "ChatScreen",
                        Box::new(ChatScreen::new(user_profile.clone())),
                    );

                    self.lobby.client.refresh_friend_requests();
                    self.lobby.client.refresh_friend_list();
                }
                _ => {}
            }
        }
        renderer.update(self.time.delta);
    }

    fn render(&mut self, renderer: &mut Renderer, window: &Window) {
        renderer.render(&mut self.ui, &self.lobby.events, window, &self.time);
    }

    fn shutdown(&mut self) {
        self.lobby.client.disconnect(true);
    }

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
