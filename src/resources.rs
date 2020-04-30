use winit::event::VirtualKeyCode;
use cgmath::Vector2;
use rand::Rng;
use crate::components::Player;

pub struct ControlsState {
    single_player: PlayerControlsState,
    player_1: PlayerControlsState,
    player_2: PlayerControlsState,
    pub pause: KeyState,
    pub debug: KeyState,
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

#[derive(Debug)]
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

#[derive(Debug)]
pub struct KeyState {
    key: VirtualKeyCode,
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
        let index = rng.gen_range(0, self.0.len());
        self.0[index]
    }
}
