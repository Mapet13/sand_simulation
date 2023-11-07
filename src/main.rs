mod sand_simulator;
mod vec;

fn main() {
    let mut app = sand_simulator::App::new();
    while app.is_running() {
        app.input();
        app.update();
        app.render();
    }
}
