use crate::components::{Position, Image, FrozenUntil};
use cgmath::Vector2;
use serde::{Serialize, Deserialize};
use std::net::SocketAddr;
use std::collections::HashSet;
use std::sync::Mutex;
use specs::prelude::*;

#[derive(Serialize, Deserialize, Debug)]
pub enum Key {
    Left,
    Right,
    Up,
    Down,
    Fire,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ClientPacket {
    key: Key,
    pressed: bool,
}

#[derive(Serialize, Deserialize)]
struct ServerPacket {
    positions: Vec<(Image, Vector2<f32>)>,
}

pub struct Connection {
    sender: crossbeam_channel::Sender<laminar::Packet>,
    receiver: crossbeam_channel::Receiver<laminar::SocketEvent>,
    polling_thread: std::thread::JoinHandle<()>,
}

impl Connection {
    pub fn new() -> Self {
        let mut socket = laminar::Socket::bind_any_with_config(laminar::Config {
            idle_connection_timeout: std::time::Duration::from_secs(100),
            receive_buffer_max_size: 8 * 1024,
            ..laminar::Config::default()
        }).unwrap();

        let mut lc = laminar::LinkConditioner::new();
        lc.set_packet_loss(0.5);

        socket.set_link_conditioner(Some(lc));

        println!("{}", socket.local_addr().unwrap());

        Self {
            receiver: socket.get_event_receiver(),
            sender: socket.get_packet_sender(),
            polling_thread: std::thread::spawn(move || socket.start_polling()),
        }
    }
}

pub struct Server<'a, 'b> {
    connection: Connection,
    clients: HashSet<SocketAddr>,
    world: specs::World,
    dispatcher: specs::Dispatcher<'a, 'b>,
}

impl<'a, 'b> Server<'a, 'b> {
    pub fn new(world: specs::World, dispatcher: specs::Dispatcher<'a, 'b>) -> Self {
        Self {
            connection: Connection::new(),
            clients: HashSet::new(),
            world,
            dispatcher
        }
    }

    pub fn step(&mut self) {
        loop {
            match self.connection.receiver.try_recv() {
                Ok(laminar::SocketEvent::Packet(packet)) => {
                    let addr = packet.addr();
                    let packet: ClientPacket = bincode::deserialize(packet.payload()).unwrap();
                    println!("Got {:?} from {}", packet, addr);

                    let code = match packet.key {
                        Key::Up => winit::event::VirtualKeyCode::Up,
                        Key::Down => winit::event::VirtualKeyCode::Down,
                        Key::Left => winit::event::VirtualKeyCode::Left,
                        Key::Right => winit::event::VirtualKeyCode::Right,
                        Key::Fire => winit::event::VirtualKeyCode::Z,
                    };

                    self.world.fetch_mut::<crate::resources::KeyPresses>().0.push((code, packet.pressed));


                },
                Ok(laminar::SocketEvent::Connect(addr)) => {
                    println!("Connection from {}", addr);
                    self.clients.insert(addr);
                },
                Ok(laminar::SocketEvent::Timeout(addr)) => {
                    println!("Timeout from {}", addr);
                    self.clients.remove(&addr);
                },
                Err(crossbeam_channel::TryRecvError::Empty) => break,
                Err(crossbeam_channel::TryRecvError::Disconnected) => panic!("Socket DCd"),
            }
        }

        self.dispatcher.dispatch(&self.world);
        self.world.maintain();

        let (pos, image, frozen): (specs::ReadStorage<Position>, specs::ReadStorage<Image>, specs::ReadStorage<FrozenUntil>) = self.world.system_data();

        let server_packet = ServerPacket {
            positions: (&pos, &image, !&frozen).join().map(|(pos, image, _)| (*image, pos.0)).collect()
        };

        let bytes = bincode::serialize(&server_packet).unwrap();

        //println!("Sending {}", bytes.len());

        for address in &self.clients {
            self.connection.sender.send(
                laminar::Packet::unreliable(
                    *address, bytes.clone()
                )
            ).unwrap();
        }
    }
}

pub struct Client {
    server_addr: SocketAddr,
    connection: Connection,
    state: Mutex<Vec<(Image, Vector2<f32>)>>,
}

impl Client {
    pub fn new(server_addr: SocketAddr) -> Self {
        let mut client = Self {
            server_addr,
            connection: Connection::new(),
            state: Mutex::new(Vec::new()),
        };
        client.send(Key::Fire, false);
        client
    }

    pub fn send(&self, key: Key, pressed: bool) {
        let bytes = bincode::serialize(&ClientPacket { key, pressed }).unwrap();
        self.connection.sender.send(
            laminar::Packet::reliable_sequenced(
                self.server_addr, bytes, None
            )
        ).unwrap();
    }

    pub fn update_loop(&self) {
        for event in self.connection.receiver.iter() {
            match event {
                laminar::SocketEvent::Packet(packet) => {
                    let addr = packet.addr();
                    let packet: ServerPacket = bincode::deserialize(packet.payload()).unwrap();
                    *self.state.lock().unwrap() = packet.positions;
                },
                laminar::SocketEvent::Connect(addr) => {
                    println!("Connection from {}", addr);
                },
                laminar::SocketEvent::Timeout(addr) => {
                    println!("Timeout from {}", addr);
                },
            }
        }
    }

    pub fn state(&self) -> std::sync::MutexGuard<Vec<(Image, Vector2<f32>)>> {
        self.state.lock().unwrap()
    }
}

struct PacketIterator {
    rev: crossbeam_channel::Receiver<laminar::SocketEvent>
}

impl Iterator for PacketIterator {
    type Item = laminar::Packet;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.rev.try_recv() {
                Ok(laminar::SocketEvent::Packet(packet)) => return Some(packet),
                Ok(laminar::SocketEvent::Connect(addr)) => println!("Connection from {}", addr),
                Ok(laminar::SocketEvent::Timeout(addr)) => println!("Timeout from {}", addr),
                Err(crossbeam_channel::TryRecvError::Empty) => return None,
                Err(crossbeam_channel::TryRecvError::Disconnected) => println!("Socket DCd"),
            }
        }
    }
}
