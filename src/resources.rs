use winit::event::VirtualKeyCode;
use cgmath::Vector2;
use rand::Rng;
use crate::components::Player;

use serde::{Serialize, Deserialize};

#[derive(Clone, Copy, Debug)]
pub enum Mode {
    Playing,
    Paused(usize),
    MainMenu(usize),
    Controls(usize),
    Quit,
    Stages(usize),
    StageOne,
    StageTwo,
}

impl Default for Mode {
    fn default() -> Self {
        Mode::MainMenu(0)
    }
}

impl Mode {
    pub fn as_menu(&mut self) -> Option<Menu> {
        match self {
            Mode::Paused(selected) => Some(Menu {
                title: "Paused".into(),
                items: vec!["Resume".into(), "Main Menu".into()],
                selected,
            }),
            Mode::MainMenu(selected) => Some(Menu {
                title: "Hectic".into(),
                items: vec!["Play".into(), "Controls".into(), "Quit".into()],
                selected,
            }),
            Mode::Stages(selected) => Some(Menu {
                title: "Stages".into(),
                items: vec!["Stage One".into(), "Stage Two".into(), "Back".into()],
                selected,
            }),
            _ => None,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct ControlsState {
    pub pause: KeyState,
    pub debug: KeyState,
    single_player: PlayerControlsState,
    player_1: PlayerControlsState,
    player_2: PlayerControlsState,
}

impl ControlsState {
    pub fn press(&mut self, key: VirtualKeyCode, pressed: bool) {
        self.single_player.press(key, pressed);
        self.player_1.press(key, pressed);
        self.player_2.press(key, pressed);
        self.pause.toggle(key, pressed);
        self.debug.toggle(key, pressed);
    }

    pub fn get(&self, player: Player) -> &PlayerControlsState {
        match player {
            Player::Single => &self.single_player,
            Player::One => &self.player_1,
            Player::Two => &self.player_2,
        }
    }

    pub fn get_mut(&mut self, player: Player) -> &mut PlayerControlsState {
        match player {
            Player::Single => &mut self.single_player,
            Player::One => &mut self.player_1,
            Player::Two => &mut self.player_2,
        }
    }

    pub fn load() -> Self {
        match std::fs::read("controls.toml") {
            Ok(vec) => match toml::from_slice(&vec) {
                Ok(controls) => controls,
                Err(err) => panic!("{}", err)
            },
            Err(err) => {
                if !matches!(err.kind(), std::io::ErrorKind::NotFound) {
                    log::warn!("Failed to read `controls.toml` with: {}. Switching to default controls.", err);
                }
                Self::default()
            }
        }
    }

    pub fn save(&self) {
        let vec = toml::to_vec(self).unwrap();
        std::fs::write("controls.toml", vec).unwrap();
    }
}

impl Default for ControlsState {
    fn default() -> Self {
        Self {
            single_player: PlayerControlsState::single_player(),
            player_1: PlayerControlsState::player_one(),
            player_2: PlayerControlsState::player_two(),
            pause: KeyState::new(VirtualKeyCode::P),
            debug: KeyState::new(VirtualKeyCode::Semicolon),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PlayerControlsState {
    pub up: KeyState,
    pub left: KeyState,
    pub right: KeyState,
    pub down: KeyState,
    pub fire: KeyState,
}

impl PlayerControlsState {
    fn single_player() -> Self {
        Self {
            up: KeyState::new(VirtualKeyCode::Up),
            left: KeyState::new(VirtualKeyCode::Left),
            right: KeyState::new(VirtualKeyCode::Right),
            down: KeyState::new(VirtualKeyCode::Down),
            fire: KeyState::new(VirtualKeyCode::Z),
        }
    }

    fn player_one() -> Self {
        let mut controls = Self::single_player();
        controls.fire = KeyState::new(VirtualKeyCode::Slash);
        controls
    }

    fn player_two() -> Self {
        Self {
            up: KeyState::new(VirtualKeyCode::W),
            left: KeyState::new(VirtualKeyCode::A),
            right: KeyState::new(VirtualKeyCode::D),
            down: KeyState::new(VirtualKeyCode::S),
            fire: KeyState::new(VirtualKeyCode::V),
        }
    }

    fn press(&mut self, key: VirtualKeyCode, pressed: bool) {
        self.up.press(key, pressed);
        self.left.press(key, pressed);
        self.right.press(key, pressed);
        self.down.press(key, pressed);
        self.fire.press(key, pressed);
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(transparent)]
pub struct KeyState {
    key: VirtualKeyCode,
    #[serde(skip)] 
    pub pressed: bool
}

impl KeyState {
    fn new(key: VirtualKeyCode) -> Self {
        Self {
            key,
            pressed: false
        }
    }

    fn toggle(&mut self, key: VirtualKeyCode, pressed: bool) {
        if self.key == key && pressed {
            self.pressed = !self.pressed;
        }
    }

    fn press(&mut self, key: VirtualKeyCode, pressed: bool) {
        if self.key == key {
            self.pressed = pressed;
        }
    }
}

pub struct GameTime {
    pub total_time: f32,
}

impl Default for GameTime {
    fn default() -> Self {
        Self {
            total_time: 0.0,
        }
    }
}

#[derive(Default)]
pub struct BulletSpawner(pub Vec<BulletToBeSpawned>);

pub struct BulletToBeSpawned {
    pub pos: Vector2<f32>,
    pub image: crate::components::Image,
    pub velocity: Vector2<f32>,
    pub enemy: bool,
}

#[derive(Default)]
pub struct DamageTracker(pub Vec<Damage>);

pub struct Damage {
    pub friendly: specs::Entity,
    pub enemy: specs::Entity,
    pub position: Vector2<f32>,
}

#[derive(Default)]
pub struct PlayerPositions(pub Vec<Vector2<f32>>);

impl PlayerPositions {
    pub fn random(&self, rng: &mut rand::rngs::ThreadRng) -> Vector2<f32> {
        // If there aren't any players, just aim wherever
        if self.0.is_empty() {
            return Vector2::new(
                rng.gen_range(0.0, crate::WIDTH),
                rng.gen_range(0.0, crate::HEIGHT)
            );
        }
        let index = rng.gen_range(0, self.0.len());
        self.0[index]
    }
}

pub struct Menu<'a> {
    pub title: String,
    pub items: Vec<String>,
    pub selected: &'a mut usize,
}

impl<'a> Menu<'a> {
    pub fn rotate_down(&mut self) {
        *self.selected = (*self.selected + 1) % self.items.len();
    }

    pub fn rotate_up(&mut self) {
        *self.selected = match self.selected.checked_sub(1) {
            None => self.items.len() - 1,
            Some(selected) => selected
        }
    }
}
