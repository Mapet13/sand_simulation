use rand::Rng;
use sdl2::{
    event::Event, keyboard::Keycode, mouse::MouseButton, pixels::Color, rect::Rect, render::Canvas,
    video::Window,
};

const SCALE: usize = 1;
const GRID_SIZE: [usize; 2] = [600 / SCALE, 600 / SCALE];

const GRAVITY: f64 = 0.9;

const BACKGROUND_COLOR: Color = Color::RGB(96, 96, 96);

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum FieldState {
    Sand(usize),
    Water(usize),
    Wood(usize),
    Empty,
}

impl FieldState {
    fn is_empty(&self) -> bool {
        match *self {
            FieldState::Empty => true,
            _ => false,
        }
    }
}

enum ParticleUpdateState {
    Moving,
    Stoped,
    Stable,
}

pub struct Particle {
    pos: [usize; 2],
    updated: bool,
    kind: FieldState,
    color: Color,
    velocity: f64,
}

impl Particle {
    pub fn new(pos: [usize; 2], kind: FieldState) -> Self {
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

fn get_id_from_pos(x: usize, y: usize) -> usize {
    x + GRID_SIZE[0] * y
}

pub struct App {
    particles: Vec<Particle>,
    current_adding: FieldState,
    grid: Vec<FieldState>,
}

impl App {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            particles: Vec::new(),
            current_adding: FieldState::Empty,
            grid: vec![FieldState::Empty; GRID_SIZE[0] * GRID_SIZE[1]],
        }
    }

    fn render(&self, canvas: &mut Canvas<Window>) {
        self.particles.iter().for_each(|p| {
            canvas.set_draw_color(p.color);
            canvas
                .fill_rect::<_>(Rect::new(
                    (p.pos[0] * SCALE) as i32,
                    (p.pos[1] * SCALE) as i32,
                    SCALE as u32,
                    SCALE as u32,
                ))
                .unwrap();
        });
    }

    fn set_adding(&mut self, con: bool, state: FieldState) {
        if !con && state == self.current_adding {
            self.current_adding = FieldState::Empty;
        }
        if con && FieldState::Empty == self.current_adding {
            self.current_adding = state;
        }
    }

    fn update(&mut self, mouse_pos: [usize; 2]) {
        self.update_particle_pos();
        self.add_particles(mouse_pos);
        self.reset_updated_status();
    }

    fn swap_grid_states(&mut self, id_a: usize, id_b: usize) {
        self.grid.swap(id_a, id_b);
    }

    fn swap_particles_pos(&mut self, id_a: usize, id_b: usize) {
        let temp_b_pos = self.particles[id_b].pos;
        self.particles[id_b].pos = self.particles[id_a].pos;
        self.particles[id_a].pos = temp_b_pos;
    }

    fn sand_case(&mut self, grid_id: usize, id: usize, check_x: usize, check_y: usize) -> bool {
        if self.standard_case(grid_id, id, check_x, check_y) {
            return true;
        }
        if let FieldState::Water(id_b) = self.grid[get_id_from_pos(check_x, check_y)] {
            self.swap_grid_states(grid_id, get_id_from_pos(check_x, check_y));
            self.swap_particles_pos(id, id_b);
            self.update_water(id_b, grid_id, self.particles[id_b].pos);
            self.particles[id_b].updated = false;
            return true;
        }
        false
    }
    
    fn standard_case(&mut self, grid_id: usize, id: usize, check_x: usize, check_y: usize) -> bool {
        if self.grid[get_id_from_pos(check_x, check_y)].is_empty() {
            self.swap_grid_states(grid_id, get_id_from_pos(check_x, check_y));
            self.particles[id].pos = [check_x, check_y];
            return true;
        }
        false
    }

    fn update_sand(&mut self, id: usize, grid_id: usize, [x, y]: [usize; 2]) -> ParticleUpdateState {
        if y >= GRID_SIZE[1] - 1 {
            return ParticleUpdateState::Stable;
        }
        if self.sand_case(grid_id, id, x, y + 1) {
            return ParticleUpdateState::Moving;
        }
        if (x > 0                && self.sand_case(grid_id, id, x - 1, y + 1))
        || (x < GRID_SIZE[1] - 1 && self.sand_case(grid_id, id, x + 1, y + 1))
        {
            return ParticleUpdateState::Stoped;
        }
        ParticleUpdateState::Stable
    }

    fn update_water(&mut self, id: usize, grid_id: usize, [x, y]: [usize; 2]) -> ParticleUpdateState {
        if y >= GRID_SIZE[1] - 1 {
            return ParticleUpdateState::Stable;
        }
        if self.standard_case(grid_id, id, x, y + 1) {
            return ParticleUpdateState::Moving;
        }
        if (x > 0                && self.standard_case(grid_id, id, x - 1, y + 1))
        || (x < GRID_SIZE[1] - 1 && self.standard_case(grid_id, id, x + 1, y + 1))
        || (x < GRID_SIZE[1] - 1 && self.standard_case(grid_id, id, x + 1, y))
        || (x > 0                && self.standard_case(grid_id, id, x - 1, y))
        {
            return ParticleUpdateState::Stoped;
        }
        ParticleUpdateState::Stable
    }
    
    fn update_movable_particle(
        &mut self,
        id: usize,
        update_particle_fn: &mut dyn std::ops::FnMut(
            &mut App,
            usize,
            usize,
            [usize; 2],
        ) -> ParticleUpdateState,
    ) {
        for _ in 0..(self.particles[id].velocity as usize) {
            let [x, y] = self.particles[id].pos;
            let grid_id = get_id_from_pos(x, y);
            match update_particle_fn(self, id, grid_id, [x, y]) {
                ParticleUpdateState::Moving => {
                    continue;
                }
                ParticleUpdateState::Stoped => {
                    return;
                }
                ParticleUpdateState::Stable => {
                    self.particles[id].velocity = 1.0;
                    return;
                }
            };
        }
    }

    fn update_particle_pos(&mut self) {
        for i in 0..self.particles.len() {
            let i = self.particles.len() - 1 - i;
            match self.particles[i].kind {
                FieldState::Sand(_) if !self.particles[i].updated => {
                    self.particles[i].velocity += GRAVITY;
                    self.update_movable_particle(i, &mut App::update_sand);
                    self.particles[i].updated = true;
                }
                FieldState::Water(_) if !self.particles[i].updated => {
                    self.particles[i].velocity += GRAVITY;
                    self.update_movable_particle(i, &mut App::update_water);
                    self.particles[i].updated = true;
                }
                _ => {}
            };
        }
    }

    fn add_particles(&mut self, pos: [usize; 2]) {
        if let FieldState::Empty = self.current_adding {
            return;
        }

        let dist = |ax: usize, ay: usize, bx: usize, by: usize| -> f32 {
            ((ax as f32 - bx as f32).powi(2) + (ay as f32 - by as f32).powi(2)).sqrt()
        };

        let radius = 12;

        let pos_x = match pos[0] {
            x if x < radius => radius,
            x if x > GRID_SIZE[0] - radius => GRID_SIZE[0] - radius,
            _ => pos[0],
        };

        let pos_y = match pos[1] {
            x if x < radius => radius,
            x if x > GRID_SIZE[1] - radius => GRID_SIZE[1] - radius,
            _ => pos[1],
        };

        if let FieldState::Wood(_) = self.current_adding {
            for x in (pos_x - radius)..(pos_x + radius) {
                for y in (pos_y - radius)..(pos_y + radius) {
                    if dist(pos_x, pos_y, x, y) < radius as f32 {
                        let id = get_id_from_pos(x, y);
                        if self.grid[id].is_empty() {
                            self.add_particle(id, [x, y], FieldState::Wood(self.particles.len()));
                        }
                    }
                }
            }
            return;
        }

        let mut rng = rand::thread_rng();

        for _ in 0..10 {
            let x = rng.gen_range(pos_x - radius, pos_x + radius);
            let y = rng.gen_range(pos_y - radius, pos_y + radius);

            if dist(pos_x, pos_y, x, y) < radius as f32 {
                let id = get_id_from_pos(x, y);
                if self.grid[id].is_empty() {
                    match self.current_adding {
                        FieldState::Sand(_) => {
                            self.add_particle(id, [x, y], FieldState::Sand(self.particles.len()))
                        }
                        FieldState::Water(_) => {
                            self.add_particle(id, [x, y], FieldState::Water(self.particles.len()))
                        }
                        _ => {}
                    };
                }
            }
        }
    }

    fn add_particle(&mut self, id: usize, pos: [usize; 2], state: FieldState) {
        self.grid[id] = state;
        self.particles.push(Particle::new(pos, self.grid[id]));
    }

    fn reset_updated_status(&mut self) {
        for p in &mut self.particles {
            p.updated = false;
        }
    }
}

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("sand", 600, 600)
        .position_centered()
        .build()
        .unwrap();

    let mut app = App::new();
    let mut canvas = window.into_canvas().build().unwrap();

    let mut mouse_pos = [0, 0];

    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();
    canvas.present();
    let mut event_pump = sdl_context.event_pump().unwrap();
    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                Event::MouseButtonDown {
                    mouse_btn, x, y, ..
                } => {
                    mouse_pos = [x, y];
                    match mouse_btn {
                        MouseButton::Left => app.set_adding(true, FieldState::Sand(0)),
                        MouseButton::Right => app.set_adding(true, FieldState::Water(0)),
                        MouseButton::Middle => app.set_adding(true, FieldState::Wood(0)),
                        _ => {}
                    }
                }
                Event::MouseButtonUp { mouse_btn, .. } => match mouse_btn {
                    MouseButton::Left => app.set_adding(false, FieldState::Sand(0)),
                    MouseButton::Right => app.set_adding(false, FieldState::Water(0)),
                    MouseButton::Middle => app.set_adding(false, FieldState::Wood(0)),
                    _ => {}
                },
                Event::MouseMotion { x, y, .. } => {
                    mouse_pos = [x, y];
                }
                _ => {}
            }
        }
        app.update([mouse_pos[0] as usize / SCALE, mouse_pos[1] as usize / SCALE]);

        canvas.set_draw_color(BACKGROUND_COLOR);
        canvas.clear();
        app.render(&mut canvas);
        canvas.present();
    }
}