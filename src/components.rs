use cgmath::Vector2;
use specs::*;
use cgmath::MetricSpace;
use rand::{Rng, rngs::ThreadRng};
use palette::IntoColor;

use crate::{WIDTH, HEIGHT};
use crate::graphics::Image as GraphicsImage;

#[derive(Component, Clone, Copy)]
pub struct Image(u16);

impl Image {
    pub fn coordinates(self) -> (f32, f32, f32, f32) {
        let (x, y, w, h) = GraphicsImage::from_u16(self.0).coordinates();
        let size = GraphicsImage::from_u16(self.0).image_size() as f32;
        (x as f32 / size, y as f32 / size, w as f32 / size, h as f32 / size)
    }

    pub fn size(self) -> Vector2<f32> {
        let (_, _, w, h) = GraphicsImage::from_u16(self.0).coordinates();
        Vector2::new(w as f32, h as f32)
    }

    pub fn from(image: GraphicsImage) -> Self {
        Self(image.to_u16())
    }
}

#[derive(Component, Copy, Clone)]
pub enum Player {
    Single,
    One,
    Two,
}

#[derive(Component)]
pub struct BackgroundLayer { pub depth: u32 }

#[derive(Component)]
pub struct Position(pub Vector2<f32>);

#[derive(Component, Clone)]
pub enum Movement {
    Linear(Vector2<f32>),
    Falling { speed: f32, down: bool },
    FollowCurve(Curve),
    FiringMove { speed: f32, return_time: f32, stop_time: f32 }
}

#[derive(Component)]
pub struct DieOffscreen;

#[derive(Component)]
pub struct BeenOnscreen;

#[derive(Component)]
pub struct FrozenUntil(pub f32);

#[derive(Component)]
pub struct Circle { pub radius: f32 } 

#[derive(Component)]
pub struct CollidesWithBomb;

#[derive(Component)]
pub struct Text {
    pub text: String,
    pub font: usize,
    pub layout: wgpu_glyph::Layout<wgpu_glyph::BuiltInLineBreaker>,
}

impl Text {
    pub fn title(text: &str) -> Self {
        Self {
            text: text.into(), font: 0,
            layout: wgpu_glyph::Layout::default().h_align(wgpu_glyph::HorizontalAlign::Center)
        }
    }
}

#[derive(Component)]
pub struct Boss {
    pub move_timer: f32,
    pub current_move: usize,
    pub moves: Vec<BossMove>,
}

impl Boss {
    pub fn current_move(&self) -> &BossMove {
        &self.moves[self.current_move]
    }
}

pub struct BossMove {
    pub position: Vector2<f32>,
    pub duration: f32,
    pub fires: FiresBullets,
}

#[derive(Clone, Copy)]
pub struct BulletSetup {
    pub image: Image,
    pub speed: f32,
    pub colour: Option<ColourBullets>
}

#[derive(Component)]
pub struct ColourOverlay(pub [f32; 4]);

#[derive(Clone, Copy)]
pub enum ColourBullets {
    Purple,
    Orange
}

impl ColourBullets {
    pub fn overlay(&self, rng: &mut ThreadRng) -> [f32; 4] {
        match *self {
            Self::Purple => {
                let hsv = palette::Hsv::<_, f32>::new(270.0, 0.8, rng.gen_range(0.5, 1.0));
                let rgb: palette::LinSrgb = hsv.into_rgb();
                [rgb.red, rgb.green, rgb.blue, 0.75]
            },
            Self::Orange => {
                let hsv = palette::Hsv::<_, f32>::new(rng.gen_range(15.0, 45.0), 1.0, 1.0);
                let rgb: palette::LinSrgb = hsv.into_rgb();
                [rgb.red, rgb.green, rgb.blue, 0.75]
            }
        }
    }
}

#[derive(Component)]
pub struct TargetPlayer(pub f32);

#[derive(Component)]
pub struct MoveTowards { pub position: Vector2<f32>, pub speed: f32 }

#[derive(Component, Clone)]
pub struct Cooldown {
    cooldown_time: f32,
    last_fired: f32,
}

impl Cooldown {
    pub fn new(cooldown_time: f32) -> Self {
        Self {
            cooldown_time,
            last_fired: std::f32::MIN,
        }
    }

    pub fn ready_at(cooldown_time: f32, ready_at: f32) -> Self {
        Self {
            cooldown_time,
            last_fired: ready_at - cooldown_time
        }
    }

    pub fn is_ready(&mut self, time: f32) -> bool {
        let is_ready = self.last_fired + self.cooldown_time <= time;
        if is_ready {
            self.last_fired = time;
        }

        is_ready
    }
}

#[derive(Clone, Component)]
pub enum FiresBullets {
    AtPlayer { num_bullets: u16, spread: f32, cooldown: Cooldown, setup: BulletSetup },
    Circle { sides: u16, rotation_per_fire: f32, rotation: f32, cooldown: Cooldown, setup: BulletSetup },
    Arc { initial_rotation: f32, spread: f32, fired_at_once: u16, number_to_fire: u16, fired_so_far: u16, cooldown: Cooldown, setup: BulletSetup },
    Multiple(Vec<FiresBullets>),
}


const S: f32 = 0.0;

const CURVE_BASIS_MATRIX: [[f32; 4]; 4] = [
    [(S-1.0)/2.0, (S+3.0)/2.0,  (-3.0-S)/2.0, (1.-S)/2.0],
    [(1.-S), (-5.-S)/2., (S+2.), (S-1.)/2.],
    [(S-1.)/2., 0., (1.-S)/2., 0.],
    [0., 1., 0., 0.]
];

fn curve_point_scalar(a: f32, b: f32, c: f32, d: f32, t: f32) -> f32 {
    let tt = t * t;
    let ttt = t * tt;
    let cb = CURVE_BASIS_MATRIX;

    a * (ttt*cb[0][0] + tt*cb[1][0] + t*cb[2][0] + cb[3][0]) +
    b * (ttt*cb[0][1] + tt*cb[1][1] + t*cb[2][1] + cb[3][1]) +
    c * (ttt*cb[0][2] + tt*cb[1][2] + t*cb[2][2] + cb[3][2]) +
    d * (ttt*cb[0][3] + tt*cb[1][3] + t*cb[2][3] + cb[3][3])
}

#[derive(Component)]
pub struct Friendly;

#[derive(Component)]
pub struct Enemy;

#[derive(Component)]
pub struct Health(pub u32);

#[derive(Component)]
pub struct Hitbox(pub Vector2<f32>);

#[derive(Component)]
pub struct Explosion(pub f32);

#[derive(Component)]
pub struct Invulnerability(f32);

#[derive(Component)]
pub struct PowerOrb(pub u32);

#[derive(Component)]
pub struct PowerBar(pub u32);

impl PowerBar {
    pub const FULL: u32 = 35;

    pub fn add(&mut self, value: u32) {
        self.0 = (self.0 + value).min(Self::FULL)
    }

    pub fn perc(&self) -> f32 {
        self.0 as f32 / Self::FULL as f32
    }

    pub fn empty(&mut self) -> bool {
        if self.0 == Self::FULL {
            self.0 = 0;
            true
        } else {
            false
        }
    }
}

impl Invulnerability {
    pub fn new() -> Self {
        Self(std::f32::MIN)
    }

    pub fn can_damage(&mut self, time: f32) -> bool {
        if !self.is_invul(time) {
            self.0 = time;
            true
        } else {
            false
        }
    }

    pub fn is_invul(&self, time: f32) -> bool {
        self.0 + 5.0 >= time
    }
}

#[derive(Clone)]
pub struct Curve {
    pub a: Vector2<f32>,
    pub b: Vector2<f32>,
    pub c: Vector2<f32>,
    pub d: Vector2<f32>,
    pub time: f32,
    pub speed: f32,
}

impl Curve {
    fn point(&self, time: f32) -> Vector2<f32> {
        Vector2::new(
            curve_point_scalar(self.a.x, self.b.x, self.c.x, self.d.x, time),
            curve_point_scalar(self.a.y, self.b.y, self.c.y, self.d.y, time)
        )
    }

    pub fn step(&mut self, previous_point: Vector2<f32>) -> Vector2<f32> {
        let mut min_time = self.time;
        let mut max_time = self.time + 1.0;

        loop {
            let mid_time = (min_time + max_time) / 2.0;
            let mid_point = self.point(mid_time);
            let mid_dist = mid_point.distance(previous_point);

            // If it's precise enough, set it and return
            if (mid_dist - self.speed).abs() < 0.1 {
                self.time = mid_time;
                return mid_point;
            // Else change the min/max values
            } else if mid_dist < self.speed {
                min_time = mid_time;
            } else {
                max_time = mid_time;
            }
        }
    }


    pub fn horizontal(start_y: f32, end_y: f32, left_to_right: bool, speed: f32) -> Self {
        const FORCE: f32 = 1500.0;
        const OFFSET: f32 = 20.0;

        if left_to_right {
            Self {
                a: Vector2::new(-FORCE - OFFSET, start_y),
                b: Vector2::new(-OFFSET, start_y),
                c: Vector2::new(WIDTH + OFFSET, end_y),
                d: Vector2::new(WIDTH + FORCE + OFFSET, end_y),
                time: 0.0,
                speed,
            }
        } else {
            Self {
                a: Vector2::new(WIDTH + FORCE + OFFSET, start_y),
                b: Vector2::new(WIDTH + OFFSET, start_y),
                c: Vector2::new(-OFFSET, end_y),
                d: Vector2::new(-FORCE - OFFSET, end_y),
                time: 0.0,
                speed,
            }
        }
    }

    pub fn vertical(mut start_x: f32, mut end_x: f32, speed: f32) -> Self {
        start_x *= WIDTH;
        end_x *= WIDTH;
        let force = 2000.0;

        Self {
            a: Vector2::new(start_x, -20.0 -force),
            b: Vector2::new(start_x, -20.0),
            c: Vector2::new(end_x, HEIGHT),
            d: Vector2::new(end_x, HEIGHT + force),
            time: 0.0,
            speed,
        }
    }

    pub fn circular(start_y: f32, force: f32, speed: f32) -> Self {
        let offset = 20.0;

        Self {
            a: Vector2::new(-offset, start_y - force),
            b: Vector2::new(-offset, start_y),
            c: Vector2::new(WIDTH + offset, start_y),
            d: Vector2::new(WIDTH + offset, start_y - force),
            time: 0.0,
            speed
        }
    }
}
