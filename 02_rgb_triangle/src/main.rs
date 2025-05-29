use app::App;
use winit::event_loop::{ControlFlow, EventLoop};

mod app;
mod render;
mod state;

fn main() {
    let event_loop = EventLoop::new().unwrap();

    event_loop.set_control_flow(ControlFlow::Poll);

    event_loop.run_app(&mut App::default()).unwrap();
}
