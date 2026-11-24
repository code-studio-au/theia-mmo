//! Low-priority bounded public labels for the same player view as movement.
use super::*;

pub(super) fn publish(
    server: &mut RenetServer,
    peers: &mut BTreeMap<u64, Peer>,
    world: &World,
    stats: &mut ServerStats,
) -> Result<()> {
    if world.completed_tick() == 0 || world.identity() == [0; 32] {
        return Ok(());
    }
    let names: BTreeMap<_, _> = peers
        .values()
        .map(|peer| (peer.actor, peer.display_name.clone()))
        .collect();
    let appearances: BTreeMap<_, _> = peers
        .values()
        .map(|peer| (peer.actor, peer.appearance))
        .collect();
    for (&id, peer) in peers.iter_mut() {
        let actors = world
            .snapshot_for(peer.actor)
            .players
            .into_iter()
            .map(|player| {
                Ok(crate::player_identity_wire::Actor {
                    key: world
                        .entity_key(player.id)
                        .ok_or("Identity actor missing")?,
                    name: names.get(&player.id).cloned().flatten(),
                    appearance: appearances.get(&player.id).copied().unwrap_or_default(),
                })
            })
            .collect::<Result<Vec<_>>>()?;
        let view = crate::player_identity_wire::State {
            epoch: world.epoch(),
            tick: world.completed_tick(),
            world: world.identity(),
            actors,
        };
        if peer
            .identity_sent
            .as_ref()
            .is_some_and(|old| old.same_view(&view))
        {
            continue;
        }
        // Membership can change between transport polls without a simulation
        // step. Publish at most one roster per completed tick so reliable peers
        // never receive two conflicting views with the same authority tick.
        if peer
            .identity_sent
            .as_ref()
            .is_some_and(|old| old.tick >= view.tick)
        {
            stats.player_identity_deferred += 1;
            continue;
        }
        let bytes = view.encode()?;
        if !server.can_send_message(id, EVENT_CHANNEL, bytes.len()) {
            stats.player_identity_deferred += 1;
            continue;
        }
        server.send_message(id, EVENT_CHANNEL, bytes);
        peer.identity_sent = Some(view);
        stats.player_identity_states_sent += 1;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn roster_changes_wait_for_a_new_tick_and_queue_pressure_preserves_pending_metadata() {
        let mut world = World::with_procedural([8000., 8000.], 1.).unwrap();
        let mut server = RenetServer::new(connection_config());
        server.add_connection(1);
        let peer = admit_peer(&mut world, &mut server, 1, None, None, Instant::now())
            .unwrap()
            .unwrap();
        let mut peers = BTreeMap::from([(1, peer)]);
        let mut stats = ServerStats::default();
        world.step();
        publish(&mut server, &mut peers, &world, &mut stats).unwrap();
        assert_eq!(stats.player_identity_states_sent, 1);
        peers.get_mut(&1).unwrap().display_name =
            Some(theia_core::character_name::CharacterName::parse("Arden").unwrap());
        publish(&mut server, &mut peers, &world, &mut stats).unwrap();
        assert_eq!(stats.player_identity_states_sent, 1);
        assert!(
            peers[&1].identity_sent.as_ref().unwrap().actors[0]
                .name
                .is_none()
        );
        world.step();
        while server.can_send_message(1, EVENT_CHANNEL, 64) {
            server.send_message(1, EVENT_CHANNEL, vec![0; 64]);
        }
        publish(&mut server, &mut peers, &world, &mut stats).unwrap();
        assert_eq!(peers.len(), 1);
        assert_eq!(stats.player_identity_states_sent, 1);
        assert_eq!(stats.player_identity_deferred, 2);
        assert!(
            peers[&1].identity_sent.as_ref().unwrap().actors[0]
                .name
                .is_none()
        );
        // Synthetic drained-queue replacement; no authority or pose is reset.
        let mut recovered = RenetServer::new(connection_config());
        recovered.add_connection(1);
        publish(&mut recovered, &mut peers, &world, &mut stats).unwrap();
        assert_eq!(stats.player_identity_states_sent, 2);
        assert_eq!(
            peers[&1].identity_sent.as_ref().unwrap().actors[0]
                .name
                .as_ref()
                .unwrap()
                .as_str(),
            "Arden"
        );
        world.step();
        publish(&mut recovered, &mut peers, &world, &mut stats).unwrap();
        assert_eq!(stats.player_identity_states_sent, 2);
    }
}
