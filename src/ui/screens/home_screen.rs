use crate::application::Action;
use crate::ui::screens::Screen;
use crossbeam_channel::Sender;
use imgui::{im_str, Condition, ImString, Ui};
use lobby_lib::LobbyEvent;
use std::time::{Duration, Instant};
use winit::dpi::PhysicalSize;

struct Notification {
    id: String,
    expire_at: Instant,
    content: String,
}

pub struct HomeScreen {
    notifications: Vec<Notification>,
}

impl HomeScreen {
    pub fn new() -> Self {
        Self {
            notifications: Vec::new(),
        }
    }

    fn add_notifications(&mut self, events: &[LobbyEvent]) {
        for event in events {
            match event {
                LobbyEvent::SystemNotification { content } => {
                    let id = format!("notification-{}", self.notifications.len());
                    self.notifications.push(Notification {
                        id,
                        expire_at: Instant::now() + Duration::from_secs(3),
                        content: content.clone(),
                    })
                }
                _ => {}
            }
        }
    }

    fn remove_expired_notifications(&mut self) {
        let now = Instant::now();
        self.notifications.retain(|n| n.expire_at > now);
    }

    fn update(&mut self, events: &[LobbyEvent]) {
        self.add_notifications(events);
        self.remove_expired_notifications();
    }
}

const NOTIF_WIDTH: f32 = 200.0;
const NOTIF_HEIGHT: f32 = 50.0;
const MARGIN: f32 = 5.0;

impl Screen for HomeScreen {
    fn draw(
        &mut self,
        ui: &Ui,
        size: PhysicalSize<u32>,
        events: &[LobbyEvent],
        action_sender: &Sender<Action>,
    ) {
        self.update(events);
        let mut begin_pos = [
            size.width as f32 - NOTIF_WIDTH - MARGIN,
            size.height as f32 - NOTIF_HEIGHT - MARGIN,
        ];
        for notification in &self.notifications {
            let id = ImString::new(&notification.id);
            imgui::Window::new(&id)
                .position(begin_pos, Condition::FirstUseEver)
                .size([NOTIF_WIDTH, NOTIF_HEIGHT], Condition::FirstUseEver)
                .movable(false)
                .no_inputs()
                .no_decoration()
                .build(&ui, || {
                    ui.text_wrapped(&ImString::new(&notification.content));
                });
            begin_pos[0] -= NOTIF_WIDTH - MARGIN;
            begin_pos[1] -= NOTIF_HEIGHT - MARGIN;
        }
    }
}
