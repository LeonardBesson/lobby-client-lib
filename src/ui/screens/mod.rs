pub mod login_screen;

pub trait Screen {
    fn draw(&mut self, ui: &imgui::Ui, size: winit::dpi::PhysicalSize<u32>);
}
