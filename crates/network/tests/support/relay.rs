//! Bounded deterministic UDP impairment relay for executable-runtime tests.
use std::{
    collections::VecDeque,
    net::{SocketAddr, UdpSocket},
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use tokio::sync::watch;

pub const QUEUE_LIMIT: usize = 512;
const CAPTURE_LIMIT: usize = 128;
#[derive(Clone, Copy, Debug, serde::Serialize)]
pub struct Profile {
    pub name: &'static str,
    pub delay_ms: u64,
    pub jitter_ms: u64,
    pub loss_every: u64,
    pub duplicate_every: u64,
}
impl Profile {
    pub const BASELINE: Self = Self {
        name: "baseline",
        delay_ms: 0,
        jitter_ms: 0,
        loss_every: 0,
        duplicate_every: 0,
    };
}
#[derive(Clone, Debug, Default, serde::Serialize)]
pub struct Stats {
    pub received_packets: [u64; 2],
    pub received_bytes: [u64; 2],
    pub forwarded_packets: [u64; 2],
    pub forwarded_bytes: [u64; 2],
    pub configured_drops: u64,
    pub outage_drops: u64,
    pub duplicates: u64,
    pub reordered: u64,
    pub queue_high_water: usize,
    pub overflow: u64,
    pub injected_old_packets: u64,
}
#[derive(Default)]
pub struct Control {
    pub blocked: bool,
    pub stats: Stats,
    pub captures: VecDeque<Vec<u8>>,
    pub inject: Vec<Vec<u8>>,
}
pub struct Relay {
    front: UdpSocket,
    back: UdpSocket,
    pub address: SocketAddr,
    pub control: Arc<Mutex<Control>>,
    profile: Profile,
}
struct Packet {
    due: Instant,
    direction: usize,
    serial: u64,
    bytes: Vec<u8>,
}
impl Relay {
    pub fn bind(profile: Profile) -> Self {
        let front = UdpSocket::bind("127.0.0.1:0").unwrap();
        let back = UdpSocket::bind("127.0.0.1:0").unwrap();
        front.set_nonblocking(true).unwrap();
        back.set_nonblocking(true).unwrap();
        Self {
            address: front.local_addr().unwrap(),
            front,
            back,
            profile,
            control: Arc::default(),
        }
    }
    pub async fn run(self, backend: SocketAddr, mut shutdown: watch::Receiver<bool>) {
        let mut interval = tokio::time::interval(Duration::from_millis(2));
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        let mut client = None;
        let mut pending = VecDeque::<Packet>::new();
        let mut serials = [0u64; 2];
        let mut last_delivered = [0u64; 2];
        let mut buffer = [0u8; 2048];
        loop {
            tokio::select! {
                biased;
                _ = shutdown.changed() => break,
                _ = interval.tick() => {
                    let mut control = self.control.lock().unwrap();
                    for bytes in control.inject.drain(..) {
                        self.back.send_to(&bytes, backend).unwrap();
                        // These are deliberately raw captures, never logged or saved.
                    }
                    for (direction, serial_counter) in serials.iter_mut().enumerate() {
                        let socket = if direction == 0 { &self.front } else { &self.back };
                        for _ in 0..128 {
                            let (len, source) = match socket.recv_from(&mut buffer) {
                                Ok(value) => value,
                                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => break,
                                Err(e) => panic!("Relay receive: {e}"),
                            };
                            if direction == 0 { client = Some(source); }
                            else { assert_eq!(source, backend); }
                            *serial_counter += 1;
                            let serial = *serial_counter;
                            control.stats.received_packets[direction] += 1;
                            control.stats.received_bytes[direction] += len as u64;
                            if direction == 0 {
                                if control.captures.len() == CAPTURE_LIMIT { control.captures.pop_front(); }
                                control.captures.push_back(buffer[..len].to_vec());
                            }
                            if control.blocked { control.stats.outage_drops += 1; continue; }
                            if self.profile.loss_every != 0 && serial.is_multiple_of(self.profile.loss_every) { control.stats.configured_drops += 1; continue; }
                            let copies = if self.profile.duplicate_every != 0 && serial.is_multiple_of(self.profile.duplicate_every) {
                                control.stats.duplicates += 1; 2
                            } else { 1 };
                            for copy in 0..copies {
                                if pending.len() == QUEUE_LIMIT { control.stats.overflow += 1; continue; }
                                let jitter = serial % 4 * self.profile.jitter_ms / 3;
                                let due = Instant::now() + Duration::from_millis(self.profile.delay_ms + jitter + copy);
                                let index = pending.iter().position(|p|p.due > due).unwrap_or(pending.len());
                                pending.insert(index, Packet { due, direction, serial, bytes: buffer[..len].to_vec() });
                                control.stats.queue_high_water = control.stats.queue_high_water.max(pending.len());
                            }
                        }
                    }
                    while pending.front().is_some_and(|p|p.due <= Instant::now()) {
                        let packet = pending.pop_front().unwrap();
                        if control.blocked { control.stats.outage_drops += 1; continue; }
                        let (socket, target) = if packet.direction == 0 { (&self.back, backend) } else { (&self.front, client.expect("Response requires client")) };
                        socket.send_to(&packet.bytes, target).unwrap();
                        let stats = &mut control.stats;
                        stats.forwarded_packets[packet.direction] += 1;
                        stats.forwarded_bytes[packet.direction] += packet.bytes.len() as u64;
                        if packet.serial < last_delivered[packet.direction] { stats.reordered += 1; }
                        last_delivered[packet.direction] = last_delivered[packet.direction].max(packet.serial);
                    }
                }
            }
        }
    }
}
