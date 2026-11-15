//! Private, per-character escort journeys. Plans and collision patches are prepared
//! at startup; fixed ticks use only bounded memory and the ordinary world controller.
use super::World;
use crate::{Result, chunks::Coverage, dialogue::Guide, kinematic, navigation, procedural};
use std::{collections::BTreeMap, sync::Arc};
use theia_core::{
    content::ContentId,
    coordinates::{ChunkId, GlobalPosition},
    quests::{
        catalog::{Catalog, EventKey, Follow},
        journal::{Journal, Status},
    },
};

pub(super) struct Plan {
    quest: ContentId,
    guide: Guide,
    policy: Follow,
    route: Vec<GlobalPosition>,
    patches: BTreeMap<ChunkId, (Arc<procedural::Patch>, Coverage)>,
}
pub(super) struct Journey {
    position: GlobalPosition,
    waypoint: usize,
    body: kinematic::Motion,
    status: Status,
}
#[derive(Default)]
pub(super) struct Authority {
    plans: BTreeMap<ContentId, Plan>,
    actors: BTreeMap<u64, BTreeMap<ContentId, Journey>>,
}
impl Authority {
    pub(super) fn leave(&mut self, actor: u64) {
        self.actors.remove(&actor);
    }
    pub(super) fn position(
        &self,
        actor: u64,
        npc: &ContentId,
        fallback: GlobalPosition,
    ) -> GlobalPosition {
        self.actors
            .get(&actor)
            .and_then(|a| a.get(npc))
            .map_or(fallback, |j| j.position)
    }
    pub(super) fn prepared(&self, quest: &ContentId, npc: &ContentId, policy: &Follow) -> bool {
        self.plans
            .get(npc)
            .is_some_and(|p| &p.quest == quest && &p.policy == policy)
    }
}
fn distance(a: GlobalPosition, b: GlobalPosition) -> f64 {
    a.metres()
        .into_iter()
        .zip(b.metres())
        .map(|(a, b)| (a - b).powi(2))
        .sum::<f64>()
        .sqrt()
}
impl World {
    /// Startup only, after contact placement and before journal/player admission.
    /// All authored navigation and capsule clearance checks complete before commit.
    pub fn prepare_follow_quests(
        &mut self,
        catalog: &Catalog,
        nav: &navigation::Store,
        markers: &[theia_zone::markers::Marker],
    ) -> Result<()> {
        if self.tick != 0 || !self.players.is_empty() || self.quest_journal.is_some() {
            return Err("Follow plans require inactive World".into());
        }
        if self
            .dialogue
            .as_ref()
            .is_none_or(|d| d.profile.catalog().is_none())
        {
            return Err("Follow requires a contact catalogue".into());
        }
        let runtime = self.procedural.as_ref().ok_or("Follow requires terrain")?;
        let procedural::Source::Authored(store) = &runtime.source else {
            return Err("Follow requires authored navigation and collision".into());
        };
        let mut plans = BTreeMap::new();
        for quest in catalog.quests() {
            for objective in quest.stages().flat_map(|s| s.objectives()) {
                let (EventKey::FollowNpc(npc), Some(policy)) =
                    (objective.event(), objective.follow())
                else {
                    continue;
                };
                let guide = self
                    .dialogue
                    .as_ref()
                    .and_then(|d| d.home(npc))
                    .ok_or("Follow contact is not installed")?;
                let mut home = guide.position.metres();
                home[1] -= f64::from(crate::world::PAWN_Y);
                let destination = markers
                    .iter()
                    .find(|m| {
                        m.id == policy.destination
                            && m.kind == theia_zone::markers::MarkerKind::Quest
                    })
                    .ok_or("Follow destination marker is absent")?;
                let end = GlobalPosition::new(destination.position)
                    .ok_or("Invalid follow destination")?;
                if distance(guide.position, end) > 512. {
                    return Err("Follow destination exceeds 512 metre budget".into());
                }
                // Two endpoints only: navmesh chooses all intervening corners.
                let route = navigation::authored_checks::grounded_route(
                    store,
                    nav,
                    &[home, destination.position],
                )?;
                let mut patches = BTreeMap::new();
                // Include all sampled cells, including long simplified segments.
                for pair in route.windows(2) {
                    let steps = (distance(pair[0], pair[1]) / 0.2).ceil().max(1.) as usize;
                    for step in 0..=steps {
                        let point = GlobalPosition::new(std::array::from_fn(|i| {
                            pair[0].metres()[i]
                                + (pair[1].metres()[i] - pair[0].metres()[i]) * step as f64
                                    / steps as f64
                        }))
                        .ok_or("Invalid follow sample")?;
                        let id = procedural::owner(point).ok_or("Follow outside terrain")?;
                        if let std::collections::btree_map::Entry::Vacant(entry) = patches.entry(id)
                        {
                            let patch = Arc::new(store.patch(id)?);
                            let coverage =
                                Coverage::procedural_cells(patch.frame, patch.ids.iter().copied())?;
                            entry.insert((patch, coverage));
                        }
                    }
                }
                let route = route
                    .into_iter()
                    .map(|p| {
                        let mut p = p.metres();
                        p[1] += f64::from(crate::world::PAWN_Y);
                        GlobalPosition::new(p).ok_or("Invalid follow centre".into())
                    })
                    .collect::<Result<Vec<_>>>()?;
                plans.insert(
                    npc.clone(),
                    Plan {
                        quest: quest.id().clone(),
                        guide,
                        policy: policy.clone(),
                        route,
                        patches,
                    },
                );
            }
        }
        self.follow = Authority {
            plans,
            actors: BTreeMap::new(),
        };
        Ok(())
    }
    /// Import only the async owner's committed journal. Pending acceptance never
    /// starts travel. Restored Ready/Completed quests place this private NPC at
    /// the destination; an interrupted Active journey restarts at its home.
    pub(crate) fn sync_follow_journal(&mut self, actor: u64, journal: &Journal) -> Result<()> {
        let profile = self
            .quest_journal
            .as_ref()
            .ok_or("Follow journal missing")?;
        journal
            .validate(profile.catalog())
            .map_err(|_| "Invalid follow journal")?;
        if !self.players.contains_key(&actor) {
            return Err("Follow actor absent".into());
        }
        let journeys = self.follow.actors.entry(actor).or_default();
        for (npc, plan) in &self.follow.plans {
            let Some(progress) = journal.progress(&plan.quest) else {
                journeys.remove(npc);
                continue;
            };
            let status = progress.status();
            let journey = journeys.entry(npc.clone()).or_insert_with(|| Journey {
                position: if status == Status::Active {
                    plan.guide.position
                } else {
                    *plan.route.last().unwrap()
                },
                waypoint: if status == Status::Active {
                    1
                } else {
                    plan.route.len()
                },
                body: kinematic::Motion::default(),
                status,
            });
            journey.status = status;
        }
        Ok(())
    }
    pub(crate) fn finished_follow_events(&self, actor: u64) -> Vec<EventKey> {
        self.follow
            .actors
            .get(&actor)
            .into_iter()
            .flat_map(|a| a.iter())
            .filter_map(|(npc, j)| {
                let plan = &self.follow.plans[npc];
                (j.status == Status::Active
                    && j.waypoint == plan.route.len()
                    && self.player_alive(actor)
                    && self.global_position(actor).is_some_and(|p| {
                        distance(p, j.position) <= f64::from(plan.policy.max_distance_metres)
                    }))
                .then(|| EventKey::FollowNpc(npc.clone()))
            })
            .collect()
    }
    pub(super) fn advance_follow(&mut self) {
        let mut authority = std::mem::take(&mut self.follow);
        for (&actor, journeys) in &mut authority.actors {
            if !self.player_alive(actor) {
                continue;
            }
            let Some(player) = self.global_position(actor) else {
                continue;
            };
            for (npc, journey) in journeys {
                let plan = &authority.plans[npc];
                if journey.status != Status::Active
                    || journey.waypoint == plan.route.len()
                    || self.contact_definition(plan.guide.key).is_none()
                    || distance(player, journey.position)
                        > f64::from(plan.policy.max_distance_metres)
                {
                    continue;
                }
                while journey.waypoint < plan.route.len()
                    && distance(journey.position, plan.route[journey.waypoint]) < 0.15
                {
                    journey.waypoint += 1;
                }
                if journey.waypoint == plan.route.len() {
                    continue;
                }
                let target = plan.route[journey.waypoint];
                let a = journey.position.metres();
                let b = target.metres();
                let dx = b[0] - a[0];
                let dz = b[2] - a[2];
                let length = dx.hypot(dz);
                if length < 0.000001 {
                    continue;
                }
                let speed = (f64::from(plan.policy.speed_millimetres_per_second) / 1000.)
                    .min(length / f64::from(self.rate.seconds()));
                let intent = crate::wire::Intent {
                    axes: [
                        (dx / length * speed * 125.).round() as i16,
                        (dz / length * speed * 125.).round() as i16,
                    ],
                    sprint: true,
                    ..Default::default()
                };
                let Some((patch, coverage)) =
                    procedural::owner(journey.position).and_then(|id| plan.patches.get(&id))
                else {
                    continue;
                };
                let mut body = journey.body;
                if let Ok(position) = super::creatures::motion::step(
                    patch,
                    coverage,
                    &mut body,
                    journey.position,
                    intent,
                    self.rate,
                ) && distance(player, position) <= f64::from(plan.policy.max_distance_metres)
                {
                    journey.position = position;
                    journey.body = body;
                }
            }
        }
        self.follow = authority;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use theia_core::time::TickRate;
    fn id(s: &str) -> ContentId {
        ContentId::new(s).unwrap()
    }
    fn setup(
        rate: TickRate,
    ) -> (
        World,
        Arc<crate::quest_journal::Profile>,
        ContentId,
        ContentId,
    ) {
        let npc = id("theia.dev.guide");
        let quest = id("follow");
        let value = serde_json::json!({"schema":5,"id":"quests","revision":1,"quests":[{"id":"follow","version":1,"title":"Follow","description":"Follow","prerequisites":[],"giver":npc.as_str(),"turn_in":npc.as_str(),"repeat_policy":"once","credit_scope":"character","experience":25,"start_stage":"travel","stages":[{"id":"travel","completion":{"kind":"all"},"next":null,"objectives":[{"kind":"follow_npc","id":"arrive","label":"Follow","npc":npc.as_str(),"travel":{"destination":1202,"max_distance_metres":10,"speed_millimetres_per_second":7000}}]}]}]});
        let xp = Arc::new(
            crate::progression_wire::Profile::from_json(include_bytes!(
                "../../../../content/gameplay/development-progression.json"
            ))
            .unwrap(),
        );
        let profile = Arc::new(
            crate::quest_journal::Profile::from_json(&serde_json::to_vec(&value).unwrap(), &xp)
                .unwrap(),
        );
        let mut world = World::with_procedural([8060., 8000.], 1.).unwrap();
        world.set_tick_rate(rate);
        let contacts = world
            .spawn_placed_contacts(
                Arc::new(
                    crate::dialogue::Profile::from_catalog_json(include_bytes!(
                        "../../../../content/quests/development-valley-contacts.json"
                    ))
                    .unwrap(),
                ),
                vec![
                    (
                        npc.clone(),
                        crate::dialogue::Placement::Horizontal([8062., 8000.]),
                    ),
                    (
                        id("theia.dev.steward"),
                        crate::dialogue::Placement::Horizontal([8064., 8000.]),
                    ),
                ],
            )
            .unwrap();
        let guide = contacts.iter().find(|c| c.npc == npc).unwrap().guide;
        let patch = world
            .procedural
            .as_ref()
            .unwrap()
            .patch(guide.position)
            .unwrap()
            .clone();
        let end = patch.try_spawn(8062., 8016.).unwrap();
        let mut p = end.metres();
        p[1] += f64::from(crate::world::PAWN_Y);
        let coverage = Coverage::procedural_cells(patch.frame, patch.ids.iter().copied()).unwrap();
        world.follow.plans.insert(
            npc.clone(),
            Plan {
                quest: quest.clone(),
                guide,
                policy: Follow {
                    destination: 1202,
                    max_distance_metres: 10,
                    speed_millimetres_per_second: 7000,
                },
                route: vec![guide.position, GlobalPosition::new(p).unwrap()],
                patches: BTreeMap::from([(
                    procedural::owner(guide.position).unwrap(),
                    (patch, coverage),
                )]),
            },
        );
        world.enable_quest_journal(profile.clone(), xp).unwrap();
        (world, profile, quest, npc)
    }
    #[test]
    fn committed_acceptance_wait_resume_private_positions_arrival_and_restore_all_rates() {
        for rate in [TickRate::Hz20, TickRate::Hz30, TickRate::Hz60] {
            let (mut world, profile, quest, npc) = setup(rate);
            let home = world.follow.plans[&npc].guide.position;
            let a = world.join().unwrap();
            let b = world.join().unwrap();
            world.players.get_mut(&a).unwrap().position = home;
            world.players.get_mut(&b).unwrap().position = home;
            world.sync_follow_journal(a, &Journal::default()).unwrap();
            assert!(world.follow.actors[&a].is_empty());
            let active = Journal::default()
                .propose_accept(profile.catalog(), &quest, &npc)
                .unwrap()
                .journal;
            world.sync_follow_journal(a, &active).unwrap();
            world.sync_follow_journal(b, &active).unwrap();
            let identity = world.identity();
            for _ in 0..rate.hz() {
                world.advance_follow();
            }
            let running = world.follow.actors[&a][&npc].position.metres();
            let start = home.metres();
            assert!(((running[0] - start[0]).hypot(running[2] - start[2]) - 7.).abs() < 0.02);
            for _ in 0..rate.hz() * 10 {
                world.advance_follow();
            }
            let paused = world.follow.actors[&a][&npc].position;
            assert!(
                distance(home, paused) > 9. && distance(home, paused) <= 10.,
                "paused {:?}, home {:?}",
                paused,
                home
            );
            assert!(world.finished_follow_events(a).is_empty());
            for _ in 0..rate.hz() * 2 {
                world.advance_follow();
            }
            assert_eq!(world.follow.actors[&a][&npc].position, paused);
            for _ in 0..rate.hz() * 10 {
                world.players.get_mut(&a).unwrap().position =
                    world.follow.actors[&a][&npc].position;
                world.advance_follow();
            }
            assert_eq!(
                world.finished_follow_events(a),
                vec![EventKey::FollowNpc(npc.clone())]
            );
            assert!(world.finished_follow_events(b).is_empty());
            assert_eq!(world.follow.actors[&b][&npc].position, paused);
            assert_eq!(identity, world.identity());
            let ready = active
                .propose_trusted_event(profile.catalog(), &EventKey::FollowNpc(npc.clone()))
                .unwrap()
                .journal;
            world.sync_follow_journal(a, &ready).unwrap();
            assert!(world.finished_follow_events(a).is_empty());
            let key = world.follow.plans[&npc].guide.key;
            let request = crate::dialogue::Request {
                sequence: 1,
                expected_revision: 0,
                action: crate::dialogue::Action::Open(key),
            };
            world.interact(a, &request).unwrap();
            assert!(world.guide_interaction_eligible(a, 1, key));
            world.players.get_mut(&a).unwrap().position = home;
            assert!(!world.guide_interaction_eligible(a, 1, key));
            world.leave(a);
            assert!(!world.follow.actors.contains_key(&a));
            let restored = world.join().unwrap();
            world.sync_follow_journal(restored, &ready).unwrap();
            assert_eq!(
                world.follow.actors[&restored][&npc].position,
                *world.follow.plans[&npc].route.last().unwrap()
            );
        }
    }
}
