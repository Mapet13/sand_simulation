use super::vec::Vec2;
use lazy_static::lazy_static;
use rand::Rng;
use sdl2::{
    event::Event, keyboard::Keycode, mouse::MouseButton, pixels::Color, rect::Rect, render::Canvas,
    video::Window, EventPump,
};

type UpdateParticleFn = dyn FnMut(
    &mut Simulator,
    usize,
    usize,
    Vec2<usize>,
) -> ParticleUpdateState;

const SCALE: usize = 1;
const GRAVITY: f64 = 0.9;
const BACKGROUND_COLOR: Color = Color::RGB(96, 96, 96);
const BRUSH_RADIUS: usize = 40;
lazy_static! {
    static ref WINDOW_SIZE: Vec2<u32> = Vec2::<u32>::new(600, 600);
    static ref GRID_SIZE: Vec2<usize> = Vec2::<usize>::new(
        WINDOW_SIZE.x as usize / SCALE,
        WINDOW_SIZE.y as usize / SCALE,
    );
}
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum FieldState {
    Sand(usize),
    Water(usize),
    Wood(usize),
    Empty,
}

impl FieldState {
    fn is_empty(&self) -> bool {
        matches!(self, FieldState::Empty)
    }
}

enum ParticleUpdateState {
    Moving,
    Stopped,
    Stable,
}

pub struct Particle {
    pos: Vec2<usize>,
    updated: bool,
    kind: FieldState,
    color: Color,
    velocity: f64,
}

impl Particle {
    pub fn new(pos: Vec2<usize>, kind: FieldState) -> Self {
        Self {
            pos,
            updated: false,
            kind,
            color: set_color(kind),
            velocity: 1.0,
        }
    }
}

fn set_color(kind: FieldState) -> Color {
    match kind {
        FieldState::Sand(_) => Color::RGB(237, 201, 175),
        FieldState::Water(_) => Color::RGB(84, 206, 246),
        FieldState::Wood(_) => Color::RGB(85, 60, 42),
        _ => BACKGROUND_COLOR,
    }
}

fn get_id_from_pos(pos: Vec2<usize>) -> usize {
    pos.x + GRID_SIZE.x * pos.y
}

fn distance_between(a: Vec2<usize>, b: Vec2<usize>) -> f32 {
    ((a.x as f32 - b.x as f32).powi(2) + (a.y as f32 - b.y as f32).powi(2)).sqrt()
}

struct Simulator {
    particles: Vec<Particle>,
    current_spawning_field: Option<FieldState>,
    grid: Vec<FieldState>,
}

impl Simulator {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            particles: Vec::new(),
            current_spawning_field: None,
            grid: vec![FieldState::Empty; GRID_SIZE.x * GRID_SIZE.y],
        }
    }

    pub fn render(&self, canvas: &mut Canvas<Window>) {
        self.particles.iter().for_each(|p| {
            canvas.set_draw_color(p.color);
            let _ = canvas.fill_rect::<_>(Rect::new(
                (p.pos.x * SCALE) as i32,
                (p.pos.y * SCALE) as i32,
                SCALE as u32,
                SCALE as u32,
            ));
        });
    }

    pub fn update(&mut self, mouse_pos: Vec2<usize>) {
        self.update_particle_pos();
        self.create_particles(mouse_pos);
        self.reset_updated_status();
    }

    fn swap_fields_on_grid(&mut self, id_a: usize, id_b: usize) {
        self.grid.swap(id_a, id_b);
    }

    fn swap_particles_positions(&mut self, id_a: usize, id_b: usize) {
        let temp_b_pos = self.particles[id_b].pos;
        self.particles[id_b].pos = self.particles[id_a].pos;
        self.particles[id_a].pos = temp_b_pos;
    }

    fn sand_case(&mut self, grid_id: usize, id: usize, check_pos: Vec2<usize>) -> bool {
        if self.standard_case(grid_id, id, check_pos) {
            return true;
        }
        if let FieldState::Water(id_b) = self.grid[get_id_from_pos(check_pos)] {
            self.swap_fields_on_grid(grid_id, get_id_from_pos(check_pos));
            self.swap_particles_positions(id, id_b);
            self.update_water(id_b, grid_id, self.particles[id_b].pos);
            self.particles[id_b].updated = false;
            return true;
        }
        false
    }

    fn standard_case(&mut self, grid_id: usize, id: usize, check_pos: Vec2<usize>) -> bool {
        if self.grid[get_id_from_pos(check_pos)].is_empty() {
            self.swap_fields_on_grid(grid_id, get_id_from_pos(check_pos));
            self.particles[id].pos = check_pos;
            return true;
        }
        false
    }

    fn update_sand(&mut self, id: usize, grid_id: usize, pos: Vec2<usize>) -> ParticleUpdateState {
        let [x, y]: [usize; 2] = pos.into();
        if y >= GRID_SIZE.y - 1 {
            return ParticleUpdateState::Stable;
        }
        if self.sand_case(grid_id, id, [x, y + 1].into()) {
            return ParticleUpdateState::Moving;
        }
        if (x > 0 && self.sand_case(grid_id, id, [x - 1, y + 1].into()))
            || (x < GRID_SIZE.y - 1 && self.sand_case(grid_id, id, [x + 1, y + 1].into()))
        {
            return ParticleUpdateState::Stopped;
        }
        ParticleUpdateState::Stable
    }

    fn update_water(&mut self, id: usize, grid_id: usize, pos: Vec2<usize>) -> ParticleUpdateState {
        let [x, y]: [usize; 2] = pos.into();
        if y >= GRID_SIZE.y - 1 {
            return ParticleUpdateState::Stable;
        }
        if self.standard_case(grid_id, id, [x, y + 1].into()) {
            return ParticleUpdateState::Moving;
        }
        if (x > 0 && self.standard_case(grid_id, id, [x - 1, y + 1].into()))
            || (x < GRID_SIZE.y - 1 && self.standard_case(grid_id, id, [x + 1, y + 1].into()))
            || (x < GRID_SIZE.y - 1 && self.standard_case(grid_id, id, [x + 1, y].into()))
            || (x > 0 && self.standard_case(grid_id, id, [x - 1, y].into()))
        {
            return ParticleUpdateState::Stopped;
        }
        ParticleUpdateState::Stable
    }

    fn update_movable_particle(
        &mut self,
        id: usize,
        update_particle_fn: &mut UpdateParticleFn,
    ) {
        self.particles[id].velocity += GRAVITY;
        for _ in 0..(self.particles[id].velocity as usize) {
            let grid_id = get_id_from_pos(self.particles[id].pos);
            match update_particle_fn(self, id, grid_id, self.particles[id].pos) {
                ParticleUpdateState::Stopped => return,
                ParticleUpdateState::Stable => {
                    self.particles[id].velocity = 1.0;
                    return;
                }
                ParticleUpdateState::Moving => {}
            };
        }
        self.particles[id].updated = true;
    }

    fn update_particle_pos(&mut self) {
        for id in (0..self.particles.len()).rev() {
            if !self.particles[id].updated {
                match self.particles[id].kind {
                    FieldState::Sand(_) => {
                        self.update_movable_particle(id, &mut Simulator::update_sand)
                    }
                    FieldState::Water(_) => {
                        self.update_movable_particle(id, &mut Simulator::update_water)
                    }
                    _ => {}
                };
            }
        }
    }

    pub fn set_creation(&mut self, con: bool, state: FieldState) {
        match (con, self.current_spawning_field) {
            (false, Some(s)) if s == state => self.current_spawning_field = None,
            (true, None) => self.current_spawning_field = Some(state),
            _ => {}
        }
    }

    fn create_particles(&mut self, pos: Vec2<usize>) {
        match self.current_spawning_field {
            Some(FieldState::Wood(_)) => {
                self.create_particle_with_rules(pos, Simulator::solid_particle_creation_rules);
            }
            Some(FieldState::Water(_)) | Some(FieldState::Sand(_)) => {
                self.create_particle_with_rules(pos, Simulator::falling_particle_creation_rules);
            }
            _ => {}
        }
    }

    fn create_particle_with_rules(
        &mut self,
        pos: Vec2<usize>,
        creation_rules_fn: fn(&mut Simulator, Vec2<usize>),
    ) {
        let get_corrected_pos = |pos: usize, border: usize| -> usize {
            match pos {
                x if x < BRUSH_RADIUS => BRUSH_RADIUS,
                x if x > border - BRUSH_RADIUS => border - BRUSH_RADIUS,
                _ => pos,
            }
        };

        let pos_x = get_corrected_pos(pos.x, GRID_SIZE.x);
        let pos_y = get_corrected_pos(pos.y, GRID_SIZE.y);

        creation_rules_fn(self, [pos_x, pos_y].into());
    }

    fn solid_particle_creation_rules(&mut self, pos: Vec2<usize>) {
        for x in (pos.x - BRUSH_RADIUS)..(pos.x + BRUSH_RADIUS) {
            for y in (pos.y - BRUSH_RADIUS)..(pos.y + BRUSH_RADIUS) {
                if distance_between(pos, [x, y].into()) < BRUSH_RADIUS as f32 {
                    let id = get_id_from_pos([x, y].into());
                    if self.grid[id].is_empty() {
                        self.create_particle(
                            id,
                            [x, y].into(),
                            FieldState::Wood(self.particles.len()),
                        );
                    }
                }
            }
        }
    }

    fn falling_particle_creation_rules(&mut self, pos: Vec2<usize>) {
        let mut rng = rand::thread_rng();

        for _ in 0..BRUSH_RADIUS {
            let x = rng.gen_range(pos.x - BRUSH_RADIUS .. pos.x + BRUSH_RADIUS);
            let y = rng.gen_range(pos.y - BRUSH_RADIUS .. pos.y + BRUSH_RADIUS);

            if distance_between(pos, [x, y].into()) < BRUSH_RADIUS as f32 {
                let id = get_id_from_pos([x, y].into());
                if self.grid[id].is_empty() {
                    let state = match self.current_spawning_field {
                        Some(FieldState::Water(_)) => FieldState::Water(self.particles.len()),
                        Some(FieldState::Sand(_)) => FieldState::Sand(self.particles.len()),
                        _ => return,
                    };
                    self.create_particle(id, [x, y].into(), state);
                }
            }
        }
    }

    fn create_particle(&mut self, id: usize, pos: Vec2<usize>, state: FieldState) {
        self.grid[id] = state;
        self.particles.push(Particle::new(pos, self.grid[id]));
    }

    fn reset_updated_status(&mut self) {
        for p in &mut self.particles {
            p.updated = false;
        }
    }
}

fn get_sdl_window(sdl_context: &sdl2::Sdl, title: &str, size: Vec2<u32>) -> Window {
    let video_subsystem = sdl_context.video().unwrap();
    video_subsystem
        .window(title, size.x, size.y)
        .position_centered()
        .build()
        .unwrap()
}

pub struct App {
    running: bool,
    simulator: Simulator,
    canvas: Canvas<Window>,
    event_pump: EventPump,
    mouse_pos: Vec2<usize>,
}

impl App {
    pub fn new() -> Self {
        let sdl_context = sdl2::init().unwrap();
        let window = get_sdl_window(&sdl_context, "sand", *WINDOW_SIZE);

        let mut app = App {
            simulator: Simulator::new(),
            running: true,
            event_pump: sdl_context.event_pump().unwrap(),
            canvas: window.into_canvas().build().unwrap(),
            mouse_pos: [0, 0].into(),
        };

        app.render();

        app
    }

    pub fn is_running(&self) -> bool {
        self.running
    }

    pub fn update(&mut self) {
        self.simulator
            .update([self.mouse_pos.x / SCALE, self.mouse_pos.y / SCALE].into());
    }
    pub fn input(&mut self) {
        for event in self.event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => self.running = false,
                Event::MouseButtonDown {
                    mouse_btn, x, y, ..
                } => {
                    self.mouse_pos = [x as usize, y as usize].into();
                    match mouse_btn {
                        //todo: refactor here
                        MouseButton::Left => self.simulator.set_creation(true, FieldState::Sand(0)),
                        MouseButton::Right => {
                            self.simulator.set_creation(true, FieldState::Water(0))
                        }
                        MouseButton::Middle => {
                            self.simulator.set_creation(true, FieldState::Wood(0))
                        }
                        _ => {}
                    }
                }
                Event::MouseButtonUp { mouse_btn, .. } => match mouse_btn {
                    MouseButton::Left => self.simulator.set_creation(false, FieldState::Sand(0)),
                    MouseButton::Right => self.simulator.set_creation(false, FieldState::Water(0)),
                    MouseButton::Middle => self.simulator.set_creation(false, FieldState::Wood(0)),
                    _ => {}
                },
                Event::MouseMotion { x, y, .. } => {
                    self.mouse_pos = [x as usize, y as usize].into();
                }
                _ => {}
            }
        }
    }

    pub fn render(&mut self) {
        clear_canvas_with_color(&mut self.canvas, BACKGROUND_COLOR);
        self.simulator.render(&mut self.canvas);
        self.canvas.present();
    }
}

fn clear_canvas_with_color(canvas: &mut Canvas<Window>, color: Color) {
    canvas.set_draw_color(color);
    canvas.clear();
}
