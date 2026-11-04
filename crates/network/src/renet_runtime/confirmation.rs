//! Do not start game traffic until the encrypted peer can receive it.
//! renetcode 2.0.0's initial KeepAlive can be lost; continuous server payloads
//! then suppress further KeepAlives while the client is still connecting.
use super::*;

#[derive(Clone, Debug, Default, serde::Serialize)]
pub struct Stats {
    pub confirmed: u64,
    pub rejected: u64,
    pub expired: u64,
    pub high_water: usize,
}

#[derive(Default)]
pub(super) struct Gate {
    waiting: BTreeMap<u64, Instant>,
    pub stats: Stats,
}

impl Gate {
    pub fn connected(&mut self, id: u64, current: Instant, server: &mut RenetServer) {
        if !(1..=MAX_PLAYERS as u64).contains(&id)
            || self.waiting.contains_key(&id)
            || self.waiting.len() >= MAX_PLAYERS
        {
            self.stats.rejected += 1;
            server.disconnect(id);
            return;
        }
        self.waiting.insert(id, current + Duration::from_secs(5));
        self.stats.high_water = self.stats.high_water.max(self.waiting.len());
    }

    pub fn disconnected(&mut self, id: u64) {
        self.waiting.remove(&id);
    }

    pub fn poll(&mut self, server: &mut RenetServer, current: Instant) -> Vec<u64> {
        let mut confirmed = Vec::with_capacity(MAX_PLAYERS);
        self.waiting.retain(|&id, deadline| {
            if !server.is_connected(id) {
                return false;
            }
            if current >= *deadline {
                self.stats.expired += 1;
                server.disconnect(id);
                return false;
            }
            let Some(bytes) = server.receive_message(id, EVENT_CHANNEL) else {
                return true;
            };
            if bytes.as_ref() != CONNECTION_CONFIRMATION {
                self.stats.rejected += 1;
                server.disconnect(id);
            } else {
                self.stats.confirmed += 1;
                confirmed.push(id);
            }
            false
        });
        confirmed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn message(server: &mut RenetServer, client: &mut RenetClient, id: u64, bytes: &[u8]) {
        client.send_message(EVENT_CHANNEL, bytes.to_vec());
        for packet in client.get_packets_to_send() {
            server.process_packet_from(&packet, id).unwrap();
        }
    }

    #[test]
    fn exact_confirmation_is_required_and_waits_are_bounded() {
        let now = Instant::now();
        let mut server = RenetServer::new(connection_config());
        let mut first = server.new_local_client(1);
        let mut forged = server.new_local_client(2);
        server.add_connection(3);
        let mut gate = Gate::default();
        for id in 1..=3 {
            gate.connected(id, now, &mut server);
        }
        assert!(gate.poll(&mut server, now).is_empty());
        message(&mut server, &mut first, 1, CONNECTION_CONFIRMATION);
        message(&mut server, &mut forged, 2, b"NC00");
        assert_eq!(gate.poll(&mut server, now), vec![1]);
        assert!(!server.is_connected(2));
        assert!(
            gate.poll(&mut server, now + Duration::from_secs(5))
                .is_empty()
        );
        assert!(!server.is_connected(3));
        assert!(gate.waiting.is_empty());
        assert_eq!(gate.stats.confirmed, 1);
        assert_eq!(gate.stats.rejected, 1);
        assert_eq!(gate.stats.expired, 1);
        assert_eq!(gate.stats.high_water, 3);
    }

    #[test]
    fn invalid_duplicate_and_disconnected_waiters_do_not_accumulate() {
        let now = Instant::now();
        let mut gate = Gate::default();
        let mut server = RenetServer::new(connection_config());
        for id in [0, 1, 9] {
            server.add_connection(id);
            gate.connected(id, now, &mut server);
        }
        assert_eq!(gate.waiting.len(), 1);
        gate.connected(1, now, &mut server);
        assert!(gate.poll(&mut server, now).is_empty());
        assert!(gate.waiting.is_empty());
        assert_eq!(gate.stats.rejected, 3);
        server.add_connection(2);
        gate.connected(2, now, &mut server);
        gate.disconnected(2);
        assert!(gate.waiting.is_empty());
    }

    #[test]
    fn client_confirms_only_after_connection_and_refuses_full_queue() {
        let mut client = RenetClient::new(connection_config());
        let mut sent = false;
        confirm_connection(&mut client, &mut sent).unwrap();
        assert!(!sent);
        client.set_connected();
        for _ in 0..8 {
            client.send_message(EVENT_CHANNEL, vec![0; 1024]);
        }
        assert!(confirm_connection(&mut client, &mut sent).is_err());
        assert!(!sent);
        let mut client = RenetClient::new(connection_config());
        client.set_connected();
        confirm_connection(&mut client, &mut sent).unwrap();
        let memory = client.channel_available_memory(EVENT_CHANNEL);
        confirm_connection(&mut client, &mut sent).unwrap();
        assert!(sent);
        assert_eq!(memory, client.channel_available_memory(EVENT_CHANNEL));
    }
}
