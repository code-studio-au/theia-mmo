//! Real checked population setup for the owned SQL first-tag qualification.
use super::*;

pub(super) async fn prepare(
    package: PathBuf,
    rate: theia_core::time::TickRate,
    threat: bool,
) -> theia_network::Result<(World, Vec<u8>, [f64; 2], u32)> {
    use theia_network::{authored_world, simulation::population};
    let store = Arc::new(authored_world::Store::open(&package.join("world"))?);
    let saved = store.spawn(0).ok_or("Missing authored spawn")?.metres();
    let source = Source::Authored(store);
    let loaded =
        population::configuration::Loaded::open(&package.join("population.json"), &source)?;
    assert_eq!(loaded.count(), 5);
    let creature = loaded.creature_bytes().to_vec();
    let worker = if threat {
        Some(
            loaded
                .pursuit_worker()?
                .ok_or("Threat receipt fixture requires pursuing creatures")?,
        )
    } else {
        None
    };
    let respawn_ms = loaded.respawn_ms();
    let (prepared, binding) = loaded.into_parts();
    let feet = prepared.points()[0].metres();
    let placement = [feet[0], feet[2]];
    let mut world = World::with_terrain_source([saved[0], saved[2]], 1., source)?;
    assert!(world.set_tick_rate(rate));
    world.enable_prediction()?;
    world.enable_classic()?;
    if let Some(worker) = worker {
        world.enable_navigating_population(prepared, binding, worker)?;
    } else {
        world.enable_defined_population(prepared, binding)?;
    }
    theia_server::development::prepare_guide(
        &mut world,
        Arc::new(theia_network::dialogue::Profile::from_json(
            include_bytes!("../../../../content/gameplay/development-dialogue.json"),
        )?),
        placement,
    )
    .await?;
    Ok((world, creature, placement, respawn_ms))
}

#[derive(Default, serde::Serialize)]
pub(super) struct ThreatProof {
    pub samples: u64,
    pub transferred: bool,
    pub public_finisher_windup: bool,
    pub private_finisher_windup: bool,
    pub incoming_finisher_strike: bool,
}
impl ThreatProof {
    pub fn observe(
        &mut self,
        world: &World,
        npc: theia_core::identity::EntityKey,
        owner: u64,
        finisher: u64,
    ) -> theia_network::Result<()> {
        assert!(world.creature_threat_error().is_none());
        assert!(world.player_life_error().is_none());
        let target = world
            .combat_snapshot(owner)
            .unwrap()
            .targets
            .into_iter()
            .find(|v| v.key == npc)
            .unwrap();
        if target.current > 0 {
            let entries = world.creature_threat_entries_for(npc).unwrap();
            assert_eq!(
                entries.iter().map(|e| e.damage_threat).sum::<u64>(),
                u64::from(target.maximum - target.current)
            );
            if target.current < target.maximum {
                assert_eq!(world.creature_tag_owner_for(npc), world.entity_key(owner));
            }
            self.samples += 1;
        }
        let finisher_life = world.player_life_snapshot(finisher).unwrap();
        let own_windup = world
            .player_life_snapshot(owner)
            .unwrap()
            .windup
            .filter(|w| w.creature == npc);
        let other_windup = finisher_life.windup.filter(|w| w.creature == npc);
        if world.creature_pursuit_directive_for(npc).unwrap().target == Some(finisher_life.key) {
            self.transferred = true;
            assert!(
                own_windup.is_none(),
                "Transferred NPC retained its old victim windup"
            );
        }
        self.private_finisher_windup |= other_windup.is_some();
        let frame = world.public_population_attack_snapshot()?.unwrap();
        assert_eq!(frame.actors.len(), 5);
        if let Some(windup) = frame
            .actors
            .iter()
            .find(|v| v.creature == npc)
            .unwrap()
            .windup
            && windup.target == finisher_life.key.actor
        {
            assert_eq!(other_windup.unwrap().completes, windup.completes);
            self.public_finisher_windup = true;
        }
        self.incoming_finisher_strike |= world.combat_events().iter().any(|event| {
            matches!(event, combat::Event::WhiteMelee(fact) if fact.actor == npc && fact.target == finisher_life.key.actor)
        });
        Ok(())
    }
    pub fn check(&self, world: &World, npc: theia_core::identity::EntityKey) {
        assert!(
            self.samples > 0
                && self.transferred
                && self.public_finisher_windup
                && self.private_finisher_windup
                && self.incoming_finisher_strike
        );
        let metrics = world.creature_threat_metrics().unwrap();
        assert!(metrics.transfers > 0 && metrics.clears > 0);
        assert_eq!(metrics.health_lost, 100);
        assert!(world.creature_threat_entries_for(npc).unwrap().is_empty());
    }
}
