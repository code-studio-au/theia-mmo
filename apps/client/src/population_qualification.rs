//! Opt-in native population check through ordinary equipment/target/cast requests.
//! Never sets poses, facing, camera, inventory observations or server health.
use crate::network::Network;
use bevy::prelude::*;
use std::{
    path::PathBuf,
    time::{Duration, Instant},
};
use theia_core::{
    content::ContentId, facing::Arc as FacingArc, identity::EntityKey,
    inventory::EquipmentOperation, player_life::LifeKey,
};

#[derive(Resource)]
struct Run {
    output: PathBuf,
    started: Option<Instant>,
    done: bool,
    life: Option<LifeKey>,
    target: Option<EntityKey>,
    equipped: bool,
    casts: u32,
    next: Instant,
    entered: bool,
    spent_mana: bool,
}
impl Run {
    fn finish(&mut self, result: serde_json::Value) {
        std::fs::write(
            self.output.join("typed-combat.json"),
            serde_json::to_vec_pretty(&result).unwrap(),
        )
        .expect("Population qualification report");
        self.done = true;
    }
    fn fail(&mut self, reason: &str) {
        self.finish(serde_json::json!({"passed":false,"error":reason,"casts":self.casts}));
    }
}
pub(crate) fn install(app: &mut App, context: bool) -> Result<(), Box<dyn std::error::Error>> {
    let Some(output) = std::env::var_os("THEIA_POPULATION_TYPED_QUALIFY") else {
        return Ok(());
    };
    if !context {
        return Err(
            "Population qualification requires connected typed population membership".into(),
        );
    }
    let output = PathBuf::from(output);
    if !output.is_dir() {
        return Err("Population qualification output must exist".into());
    }
    app.insert_resource(Run {
        output,
        started: None,
        done: false,
        life: None,
        target: None,
        equipped: false,
        casts: 0,
        next: Instant::now(),
        entered: false,
        spent_mana: false,
    })
    .add_systems(Update, drive);
    Ok(())
}
fn drive(network: Res<Network>, mut run: ResMut<Run>) {
    if run.done || !run.output.join("combat.start").is_file() {
        return;
    }
    let now = Instant::now();
    let started = *run.started.get_or_insert(now);
    if now.duration_since(started) > Duration::from_secs(30) {
        run.fail("Bounded population combat deadline exceeded");
        return;
    }
    let state = network.state();
    if state.error.is_some() {
        run.fail("Connection failed");
        return;
    }
    if !network.connected() {
        return;
    }
    let (
        Some(life),
        Some(inventory),
        Some(combat),
        Some(targeting),
        Some(correction),
        Some(position),
    ) = (
        state.player_life.as_ref().and_then(|p| p.current()),
        state.inventory.as_ref(),
        state.combat.as_ref(),
        state.targeting.as_ref(),
        state.correction,
        state.authority_focus(),
    )
    else {
        return;
    };
    if run.life.is_some_and(|key| key != life.key) || life.health == 0 {
        run.fail("Player life changed during qualification");
        return;
    }
    if !network.can_act() {
        return;
    }
    run.life = Some(life.key);
    run.entered |= life.combat.is_some_and(|v| v.in_combat());
    run.spent_mana |= combat.vitals.is_some_and(|v| v.mana() < v.maximum_mana());
    if combat.targets.len() != 5 {
        run.fail("Expected five creature lives");
        return;
    }
    if run.target.is_none() {
        let Some(facing) = correction.state.pose.facing else {
            return;
        };
        let arc = FacingArc::from_degrees(160.).unwrap();
        let a = position.metres();
        run.target = targeting
            .targets
            .iter()
            .filter(|target| {
                let b = target.position.metres();
                let distance = (a[0] - b[0]).hypot(a[2] - b[2]);
                distance < 25.
                    && arc.contains(facing, position, target.position)
                    && combat
                        .targets
                        .iter()
                        .any(|h| h.entity == target.key.entity && h.current == 100)
            })
            .min_by_key(|target| target.key.entity.value())
            .map(|target| target.key);
        let Some(target) = run.target else {
            run.fail("No healthy population NPC already within frontal spell range");
            return;
        };
        network.request_target(Some(target.entity));
    }
    let target = run.target.unwrap();
    let Some(health) = combat.targets.iter().find(|h| h.entity == target.entity) else {
        run.fail("Selected creature life retired");
        return;
    };
    if targeting.epoch != target.epoch.value()
        || combat
            .targets
            .iter()
            .any(|h| h.entity != target.entity && h.current != h.maximum)
    {
        run.fail("World changed or a bystander was damaged");
        return;
    }
    if health.current < 100 {
        if run.entered && run.spent_mana && life.combat.is_some_and(|v| !v.in_combat()) {
            let casts = run.casts;
            run.finish(serde_json::json!({"passed":true,"casts":casts,"target":target.entity.value(),
                "epoch":target.epoch.value(),"remaining":health.current,"inventory_revision":inventory.bag.revision(),
                "entered_and_exited_combat":true,"mana_spent":true,"bystanders_unchanged":true,
                "player_health":life.health,"player_generation":life.key.generation.get()}));
        }
        return;
    }
    let wand = "theia.dev.training-wand";
    if inventory
        .bag
        .main_hand()
        .is_none_or(|s| s.item().as_str() != wand)
    {
        if !run.equipped && !state.inventory_pending {
            let Some(slot) = inventory
                .bag
                .slots()
                .iter()
                .position(|s| s.as_ref().is_some_and(|s| s.item().as_str() == wand))
            else {
                run.fail("Committed inventory has no training wand");
                return;
            };
            run.equipped =
                network.request_equipment(EquipmentOperation::Equip { from: slot as u8 });
        }
        return;
    }
    if targeting.selected != Some(target.entity) {
        return;
    }
    if now >= run.next
        && run.casts < 3
        && combat.cast.is_none()
        && network.request_cast(
            ContentId::new("theia.dev.arcane-bolt").unwrap(),
            target.entity,
        )
    {
        run.casts += 1;
        run.next = now + Duration::from_secs(3);
    }
}
