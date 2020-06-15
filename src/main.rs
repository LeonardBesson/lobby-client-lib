use crate::application::Application;
use futures::executor::block_on;
use lobby_lib;

mod application;
mod renderer;
mod time;
mod ui;

fn main() {
    env_logger::init();

    let mut app = Application::new();
    block_on(app.run());
}
