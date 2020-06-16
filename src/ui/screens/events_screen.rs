use crate::application::Action;
use crate::ui::screens::Screen;
use crossbeam_channel::Sender;
use imgui::{im_str, Condition, Ui};
use lobby_lib::LobbyEvent;
use winit::dpi::PhysicalSize;

const MAX_HISTORY_SIZE: usize = 500;

pub struct EventScreen {
    event_history: Vec<LobbyEvent>,
}

impl EventScreen {
    pub fn new() -> Self {
        Self {
            event_history: Vec::with_capacity(MAX_HISTORY_SIZE),
        }
    }

    fn update_history(&mut self, new_events: &[LobbyEvent]) {
        if self.event_history.len() + new_events.len() > MAX_HISTORY_SIZE {
            let drain_count = self.event_history.len() + new_events.len() - MAX_HISTORY_SIZE;
            self.event_history.drain(0..drain_count);
        }
        self.event_history.extend_from_slice(new_events);
    }
}

impl Screen for EventScreen {
    fn draw(
        &mut self,
        ui: &Ui,
        size: PhysicalSize<u32>,
        events: &[LobbyEvent],
        action_sender: &Sender<Action>,
    ) {
        self.update_history(events);
        let window = imgui::Window::new(im_str!("Events"));
        window
            .size([400.0, 435.0], Condition::FirstUseEver)
            .position([5.0, 5.0], Condition::FirstUseEver)
            .build(&ui, || {
                for event in &self.event_history {
                    ui.text(format!("{:?}", event));
                }
            });
    }
}
