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
    kill: bool,
    resources_only: bool,
    collect_reward: bool,
    reward_requested: bool,
    collect_loot: bool,
    pickup: Option<(theia_core::rewards::ClaimId, u64)>,
    pickups: u32,
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
pub(crate) fn install(
    app: &mut App,
    typed_context: bool,
    resource_context: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let typed = std::env::var_os("THEIA_POPULATION_TYPED_QUALIFY");
    let resources = std::env::var_os("THEIA_POPULATION_RESOURCE_QUALIFY");
    if typed.is_some() && resources.is_some() {
        return Err("Choose one population qualification mode".into());
    }
    let resources_only = resources.is_some();
    let Some(output) = typed.or(resources) else {
        return Ok(());
    };
    if !(if resources_only {
        resource_context
    } else {
        typed_context
    }) {
        return Err(
            "Population qualification requires matching connected combat/life profiles".into(),
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
        resources_only,
        kill: !resources_only
            && std::env::var("THEIA_POPULATION_KILL_QUALIFY").as_deref() == Ok("1"),
        collect_reward: !resources_only
            && std::env::var("THEIA_POPULATION_REWARD_QUALIFY").as_deref() == Ok("1"),
        reward_requested: false,
        collect_loot: !resources_only
            && std::env::var("THEIA_POPULATION_LOOT_QUALIFY").as_deref() == Ok("1"),
        pickup: None,
        pickups: 0,
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
    if now.duration_since(started) > Duration::from_secs(if run.kill { 90 } else { 30 }) {
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
    let (Some(life), Some(combat), Some(targeting), Some(correction), Some(position)) = (
        state.player_life.as_ref().and_then(|p| p.current()),
        state.combat.as_ref(),
        state.targeting.as_ref(),
        state.correction,
        state.authority_focus(),
    ) else {
        return;
    };
    let inventory = state.inventory.as_ref();
    if !run.resources_only && inventory.is_none() {
        return;
    }
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
                distance < if run.collect_loot { 2.8 } else { 25. }
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
    if health.current < 100 && (!run.kill || health.current == 0) {
        if run.spent_mana
            && (run.resources_only || (run.entered && life.combat.is_some_and(|v| !v.in_combat())))
        {
            if run.collect_reward {
                let inventory = inventory.expect("Typed qualifier requires inventory");
                let Some(rewards) = state.rewards.as_ref() else {
                    return;
                };
                if !run.reward_requested {
                    if let Some(reward) = rewards.available.iter().find(|r| r.corpse.is_none()) {
                        run.reward_requested = network.request_reward(reward.claim);
                    }
                    return;
                }
                let tokens: u32 = inventory
                    .bag
                    .slots()
                    .iter()
                    .flatten()
                    .filter(|s| s.item().as_str() == "theia.dev.training-token")
                    .map(|s| s.count())
                    .sum();
                if rewards.available.iter().any(|r| r.corpse.is_none())
                    || inventory.bag.revision() < 2
                    || tokens < 100
                    || (!run.collect_loot
                        && (!rewards.available.is_empty()
                            || inventory.bag.revision() != 2
                            || tokens != 100))
                {
                    return;
                }
            }
            if run.collect_loot {
                let inventory = inventory.expect("Typed qualifier requires inventory");
                let Some(rewards) = state.rewards.as_ref() else {
                    return;
                };
                if let Some((claim, revision)) = run.pickup {
                    if inventory.bag.revision() <= revision
                        || rewards.available.iter().any(|r| r.claim == claim)
                    {
                        return;
                    }
                    run.pickup = None;
                    run.pickups += 1;
                }
                if let Some(reward) = rewards.available.iter().find(|r| r.corpse.is_some()) {
                    let source = reward.corpse.unwrap();
                    if source.life.victim() != target {
                        run.fail("Loot belongs to a different creature life");
                        return;
                    }
                    if network.request_pickup(reward.claim, source.life) {
                        run.pickup = Some((reward.claim, inventory.bag.revision()));
                    }
                    return;
                }
                if run.pickups != 2 {
                    return;
                }
                let count = |item: &str| {
                    inventory
                        .bag
                        .slots()
                        .iter()
                        .flatten()
                        .filter(|stack| stack.item().as_str() == item)
                        .map(|stack| stack.count())
                        .sum::<u32>()
                };
                if inventory.bag.revision() != 4
                    || count("theia.dev.wolf-pelt") != 1
                    || count("theia.dev.training-token") != 101
                {
                    run.fail("Committed corpse inventory differs from expected grants");
                    return;
                }
            }
            let membership = !run.resources_only;
            let collected = run.collect_reward;
            let casts = run.casts;
            let pickups = run.pickups;
            run.finish(serde_json::json!({"passed":true,"corpse_pickups":pickups,"reward_collected":collected,"casts":casts,"target":target.entity.value(),
                "epoch":target.epoch.value(),"remaining":health.current,"inventory_revision":inventory.map(|i| i.bag.revision()),
                "entered_and_exited_combat":membership,"mana_spent":true,"bystanders_unchanged":true,
                "player_health":life.health,"player_generation":life.key.generation.get()}));
        }
        return;
    }
    if !run.resources_only {
        let inventory = inventory.expect("Typed qualifier requires inventory");
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
    }
    if targeting.selected != Some(target.entity) {
        return;
    }
    if now >= run.next
        && run.casts < if run.kill { 12 } else { 3 }
        && combat.vitals.is_some_and(|v| v.mana() >= 25)
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
