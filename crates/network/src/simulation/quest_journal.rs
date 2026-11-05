//! Explicit expanded-quest World adapters. Installed-reference checks and
//! normalized facts establish no durable journal or normal-client adoption.
use super::World;
use std::{num::NonZeroU128, sync::Arc};
use theia_core::quests::catalog::{Catalog, EventKey};
/// Immutable admission precursor, tied to one World incarnation and its content.
/// This is not a saved journal or a transport/profile compatibility assertion.
pub struct Prepared {
    catalog: Arc<Catalog>,
    areas: Option<Arc<theia_core::quests::areas::Catalog>>,
    epoch: u64,
    world: [u8; 32],
}
impl Prepared {
    pub fn matches_world(&self, world: &World) -> bool {
        self.epoch == world.epoch() && self.world == world.identity()
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Occurrence {
    pub actor: theia_core::identity::EntityKey,
    pub id: NonZeroU128,
    pub event: EventKey,
}

impl World {
    /// Explicit runtime mode; install after guide/creature content,
    /// before admission. Existing legacy quest modes remain unchanged.
    pub fn enable_quest_journal(
        &mut self,
        profile: Arc<crate::quest_journal::Profile>,
        xp: Arc<crate::progression_wire::Profile>,
    ) -> crate::Result<()> {
        self.validate_quest_journal_references_with_areas(
            profile.catalog(),
            profile.shared_areas().as_deref(),
        )?;
        profile.check_progression(&xp)?;
        if self
            .progression
            .as_ref()
            .is_some_and(|old| old.hash() != xp.hash())
        {
            return Err("Journal cannot replace installed progression policy".into());
        }
        // Journal mode also supports conversation-only/quest-only creatures.
        // XP recipes can belong to other content; only matching local kills
        // grant XP. Legacy enable_progression keeps its installed-creature rule.
        self.progression = Some(xp);
        self.quest_journal = Some(profile);
        Ok(())
    }
    pub fn quest_journal_profile(&self) -> Option<&Arc<crate::quest_journal::Profile>> {
        self.quest_journal.as_ref()
    }
    /// Bind an installed and previously validated catalogue to the current
    /// World/actor incarnation. No async work or journal state is created here.
    pub fn quest_journal_router(
        &self,
        actor: theia_core::identity::EntityKey,
    ) -> crate::Result<crate::quest_journal::Router> {
        let profile = self
            .quest_journal
            .as_ref()
            .ok_or("Journal mode is not installed")?;
        if self.entity_key(actor.entity.value()) != Some(actor) {
            return Err("Journal router actor is not admitted".into());
        }
        Ok(profile.bind_authority(
            Prepared {
                catalog: profile.shared_catalog(),
                areas: profile.shared_areas(),
                epoch: self.epoch(),
                world: self.identity(),
            },
            actor,
        ))
    }
    pub fn prepare_quest_journal(&self, catalog: Arc<Catalog>) -> crate::Result<Prepared> {
        self.validate_quest_journal_references(&catalog)?;
        Ok(Prepared {
            catalog,
            areas: None,
            epoch: self.epoch(),
            world: self.identity(),
        })
    }
    pub(crate) fn prepare_quest_journal_profile(
        &self,
        profile: &crate::quest_journal::Profile,
    ) -> crate::Result<Prepared> {
        let areas = profile.shared_areas();
        self.validate_quest_journal_references_with_areas(profile.catalog(), areas.as_deref())?;
        Ok(Prepared {
            catalog: profile.shared_catalog(),
            areas,
            epoch: self.epoch(),
            world: self.identity(),
        })
    }
    /// Trusted sampled occupancy only. No caller-supplied coordinates, disk I/O,
    /// collision generation, durable credit or client acknowledgement.
    pub fn observe_quest_areas(
        &self,
        prepared: &Prepared,
        actor: theia_core::identity::EntityKey,
        journal: &theia_core::quests::journal::Journal,
    ) -> crate::Result<theia_core::quests::journal::ReachedAreas> {
        let areas = prepared
            .areas
            .as_ref()
            .ok_or("Quest area profile is not installed")?;
        let feet = self
            .quest_area_feet(prepared, actor)
            .ok_or("Quest area observation lacks current alive prepared authority")?;
        let inside = areas.containing(feet).map(|a| a.id().clone()).collect();
        journal
            .prepare_reached_areas(&prepared.catalog, &inside)
            .map_err(|_| "Invalid journal area observation".into())
    }
    pub(crate) fn quest_area_feet(
        &self,
        prepared: &Prepared,
        actor: theia_core::identity::EntityKey,
    ) -> Option<theia_core::coordinates::GlobalPosition> {
        if !prepared.matches_world(self)
            || self.entity_key(actor.entity.value()) != Some(actor)
            || !self.player_alive(actor.entity.value())
        {
            return None;
        }
        let player = self.players.get(&actor.entity.value())?;
        let mut position = player.position.metres();
        position[1] -= f64::from(crate::world::PAWN_Y);
        let feet = theia_core::coordinates::GlobalPosition::new(position)?;
        let runtime = self.procedural.as_ref()?;
        (runtime.error.is_none()
            && runtime
                .patch(feet)
                .is_some_and(|p| p.covers(feet, player.ready.as_ref())))
        .then_some(feet)
    }
    /// Explicit server adapter: only a successfully selected offered choice can
    /// yield semantic credit. Opening/closing a panel yields no occurrence.
    /// Caller must reserve bounded owner capacity before invoking this and
    /// retain any returned occurrence until its durable receipt resolves. This
    /// API itself makes no durable acknowledgement and is not wired into Renet.
    pub fn interact_quest_journal(
        &mut self,
        prepared: &Prepared,
        actor: u64,
        request: &crate::dialogue::Request,
    ) -> Result<Option<Occurrence>, crate::dialogue::Rejection> {
        use crate::dialogue::{Action, Rejection};
        if prepared.epoch != self.epoch() || prepared.world != self.identity() {
            return Err(Rejection::Unavailable);
        }
        let event = match &request.action {
            Action::Choose(interaction) => {
                let npc = self
                    .dialogue
                    .as_ref()
                    .ok_or(Rejection::Unavailable)?
                    .profile
                    .profile()
                    .npc()
                    .clone();
                let event = EventKey::NpcInteraction {
                    npc,
                    interaction: interaction.clone(),
                };
                (!prepared.catalog.matching(&event).is_empty()).then_some(event)
            }
            _ => None,
        };
        // The existing authority checks actual actor life, guide incarnation,
        // offered current node, 3D range, revision, sequence and rate limit.
        self.interact(actor, request)?;
        let Some(event) = event else { return Ok(None) };
        let actor = self.entity_key(actor).ok_or(Rejection::Unavailable)?;
        let mut bytes = b"theia-journal-dialogue-occurrence-v1\0".to_vec();
        bytes.extend(actor.epoch.value().to_le_bytes());
        bytes.extend(actor.entity.value().to_le_bytes());
        bytes.extend(request.sequence.to_le_bytes());
        // Domain-separated deterministic 128-bit receipt. Reserve zero. This is
        // occurrence identity, not authorization or a client-provided UUID.
        let hash = crate::terrain::sha256(&bytes);
        let id =
            NonZeroU128::new(u128::from_le_bytes(hash[..16].try_into().unwrap()).max(1)).unwrap();
        Ok(Some(Occurrence { actor, id, event }))
    }
    /// Separate quest credit from XP recipes, retaining the installed solo-credit
    /// policy. Retain kills without matching objectives too: configured XP and
    /// pre-accept disposition must not depend on the current journal. Trusted
    /// input is the bounded authoritative combat event batch.
    pub fn quest_kill_occurrences(
        &self,
        prepared: &Prepared,
        events: &[super::combat::Event],
    ) -> crate::Result<Vec<super::progression::Opportunity>> {
        if prepared.epoch != self.epoch() || prepared.world != self.identity() {
            return Err("Quest occurrence World binding changed".into());
        }
        if events.len() > super::combat::MAX_PROJECTILE_STEP_EVENTS {
            return Err("Quest combat source batch exceeds event budget".into());
        }
        let Some(patrol) = self.patrol.as_ref() else {
            return Ok(Vec::new());
        };
        let mut occurrences = Vec::new();
        for source in events {
            let Some((actor, victim)) = self.kill_recipient(source)? else {
                continue;
            };
            if victim != patrol.key || self.entity_key(actor.entity.value()) != Some(actor) {
                continue;
            }
            let occurrence = super::progression::Opportunity {
                actor,
                event: theia_core::progression::KillEvent {
                    id: theia_core::progression::KillId::from_victim(victim),
                    creature: patrol.definition().clone(),
                },
            };
            if occurrences.contains(&occurrence) {
                continue;
            }
            if occurrences.len() == super::progression::FACT_CAPACITY {
                return Err("Quest occurrence capacity exhausted".into());
            }
            occurrences.push(occurrence);
        }
        Ok(occurrences)
    }
    /// Every referenced contact/action/creature must exist in this initial
    /// single-guide/single-creature World. No XP kill recipe is required here.
    /// A successful check grants no actor entitlement or persisted compatibility.
    pub fn validate_quest_journal_references(&self, catalog: &Catalog) -> crate::Result<()> {
        self.validate_quest_journal_references_with_areas(catalog, None)
    }
    fn validate_quest_journal_references_with_areas(
        &self,
        catalog: &Catalog,
        areas: Option<&theia_core::quests::areas::Catalog>,
    ) -> crate::Result<()> {
        if self.tick != 0
            || !self.players.is_empty()
            || self.quest.is_some()
            || self.quest_lifecycle.is_some()
            || self.quest_journal.is_some()
        {
            return Err("Expanded quest reference checks require inactive exclusive World".into());
        }
        if let Some(areas) = areas {
            if !self.predicted || self.procedural.is_none() {
                return Err("Quest areas require prepared continental authority".into());
            }
            for area in areas.areas() {
                if [area.min(), area.max()].iter().any(|p| {
                    let p = p.metres();
                    [p[0], p[2]]
                        .iter()
                        .any(|v| !(0. ..=crate::procedural::SIDE).contains(v))
                }) {
                    return Err("Quest area lies outside installed continental bounds".into());
                }
            }
        }
        let guide = self
            .dialogue
            .as_ref()
            .filter(|d| d.available())
            .ok_or("Quest contacts require an installed available guide")?;
        let dialogue = guide.profile.profile();
        if let Some(areas) = areas {
            let aliases_world = |id: &theia_core::content::ContentId| {
                id == dialogue.npc()
                    || self.patrol.as_ref().is_some_and(|p| p.definition() == id)
                    || dialogue
                        .nodes()
                        .any(|n| n.choices().iter().any(|c| c.id() == id))
            };
            if aliases_world(areas.id()) || areas.areas().any(|a| aliases_world(a.id())) {
                return Err("Quest area definitions alias installed World content".into());
            }
        }
        for quest in catalog.quests() {
            if quest.giver() != dialogue.npc() || quest.turn_in() != dialogue.npc() {
                return Err("Quest contact is not installed in this World".into());
            }
            for stage in quest.stages() {
                for objective in stage.objectives() {
                    match objective.event() {
                        EventKey::PossessItem(_) => {
                            return Err("Possession objectives require an explicitly installed inventory binding".into());
                        }
                        EventKey::ReachArea(id) => {
                            if areas.is_none_or(|a| a.area(id).is_none()) {
                                return Err("Reach-area objectives require an explicitly installed area profile".into());
                            }
                        }
                        EventKey::Kill(creature)
                            if self
                                .patrol
                                .as_ref()
                                .is_none_or(|p| p.definition() != creature) =>
                        {
                            return Err("Quest creature is not installed in this World".into());
                        }
                        EventKey::NpcInteraction { npc, interaction }
                            if npc != dialogue.npc()
                                || !dialogue.nodes().any(|node| {
                                    node.choices()
                                        .iter()
                                        .any(|choice| choice.id() == interaction)
                                }) =>
                        {
                            return Err(
                                "Quest semantic choice is not installed for this NPC".into()
                            );
                        }
                        _ => {}
                    }
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{dialogue::Profile, simulation::creatures};
    use serde_json::{Value, json};
    use std::sync::Arc;
    use theia_core::time::TickRate;
    const DATA: &[u8] = include_bytes!("../../../../content/quests/development-journal.json");
    const DIALOGUE: &[u8] =
        include_bytes!("../../../../content/quests/development-journal-dialogue.json");
    fn setup() -> World {
        setup_at(30)
    }
    pub(super) fn setup_at(hz: u32) -> World {
        let (mut world, _) =
            creatures::tests::fixture_at(TickRate::from_hz(hz).unwrap(), [8000., 8000.]);
        world
            .spawn_development_guide(
                Arc::new(Profile::from_json(DIALOGUE).unwrap()),
                [8000., 8000.],
            )
            .unwrap();
        world
    }
    fn catalog(value: Value) -> Catalog {
        Catalog::from_json(&serde_json::to_vec(&value).unwrap()).unwrap()
    }
    fn combat_setup() -> World {
        let mut world = setup();
        world
            .enable_combat(
                include_bytes!("../../../../content/gameplay/development-abilities.json"),
                100,
                1000,
            )
            .unwrap();
        world
    }
    #[test]
    fn installed_mode_binds_exact_catalogue_and_xp_and_refuses_legacy_or_live_replacement() {
        let mut world = combat_setup();
        let xp = Arc::new(
            crate::progression_wire::Profile::from_json(include_bytes!(
                "../../../../content/gameplay/development-progression.json"
            ))
            .unwrap(),
        );
        let profile = Arc::new(crate::quest_journal::Profile::from_json(DATA, &xp).unwrap());
        let old = world.identity();
        let mut different_xp =
            include_bytes!("../../../../content/gameplay/development-progression.json").to_vec();
        different_xp.push(b' ');
        let different_xp =
            Arc::new(crate::progression_wire::Profile::from_json(&different_xp).unwrap());
        assert!(
            world
                .enable_quest_journal(profile.clone(), different_xp)
                .is_err()
        );
        assert_eq!(world.identity(), old);
        assert!(world.progression.is_none());
        world.enable_progression(xp.clone()).unwrap();
        let before = world.identity();
        world
            .enable_quest_journal(profile.clone(), xp.clone())
            .unwrap();
        let installed = world.identity();
        assert_ne!(installed, before);
        assert_eq!(
            world.quest_journal_profile().unwrap().hash(),
            profile.hash()
        );
        assert!(
            world
                .enable_quest_journal(profile.clone(), xp.clone())
                .is_err()
        );
        assert!(
            world
                .enable_quest(Arc::new(
                    crate::quest_wire::Profile::from_json(include_bytes!(
                        "../../../../content/gameplay/development-quest.json"
                    ))
                    .unwrap()
                ))
                .is_err()
        );
        assert!(
            world
                .enable_quest_lifecycle(Arc::new(
                    crate::quest_lifecycle_wire::Profile::from_json(include_bytes!(
                        "../../../../content/gameplay/development-quest.json"
                    ))
                    .unwrap()
                ))
                .is_err()
        );
        assert_eq!(world.identity(), installed, "Refused changes are atomic");
        let actor = world.join().unwrap();
        let actor = world.entity_key(actor).unwrap();
        assert!(world.quest_journal_router(actor).is_ok());
        let mut foreign = actor;
        foreign.epoch = theia_core::identity::WorldEpoch::new(actor.epoch.value() + 1).unwrap();
        assert!(world.quest_journal_router(foreign).is_err());
        assert!(world.enable_quest_journal(profile, xp.clone()).is_err());
        let mut other = combat_setup();
        other.enable_progression(xp.clone()).unwrap();
        let mut changed = DATA.to_vec();
        changed.push(b' ');
        other
            .enable_quest_journal(
                Arc::new(crate::quest_journal::Profile::from_json(&changed, &xp).unwrap()),
                xp.clone(),
            )
            .unwrap();
        assert_ne!(
            other.identity(),
            installed,
            "Exact catalogue bytes bind the World"
        );
    }
    #[test]
    fn installed_mode_refuses_different_xp_policy_without_mutating_identity() {
        let mut world = combat_setup();
        let bytes = include_bytes!("../../../../content/gameplay/development-progression.json");
        let xp = Arc::new(crate::progression_wire::Profile::from_json(bytes).unwrap());
        world.enable_progression(xp.clone()).unwrap();
        let mut padded = bytes.to_vec();
        padded.push(b' ');
        let changed = crate::progression_wire::Profile::from_json(&padded).unwrap();
        let profile = Arc::new(crate::quest_journal::Profile::from_json(DATA, &changed).unwrap());
        let before = world.identity();
        assert!(
            world
                .enable_quest_journal(profile.clone(), xp.clone())
                .is_err()
        );
        assert!(
            world
                .enable_quest_journal(profile, Arc::new(changed))
                .is_err()
        );
        assert!(world.quest_journal_profile().is_none());
        assert_eq!(world.identity(), before);
    }
    #[test]
    fn reference_admission_resolves_contacts_actions_and_creature_without_xp_recipe() {
        let world = setup();
        assert!(world.progression.is_none());
        let identity = world.identity();
        world
            .validate_quest_journal_references(&Catalog::from_json(DATA).unwrap())
            .unwrap();
        assert_eq!(
            world.identity(),
            identity,
            "Reference validation is read only, not activation"
        );
        assert!(world.quest.is_none() && world.quest_lifecycle.is_none());
    }
    #[test]
    fn structurally_valid_but_uninstalled_references_refuse() {
        let world = setup();
        for field in ["giver", "turn_in"] {
            let mut data: Value = serde_json::from_slice(DATA).unwrap();
            data["quests"][0][field] = json!("theia.dev.absent-npc");
            assert!(
                world
                    .validate_quest_journal_references(&catalog(data))
                    .is_err()
            );
        }
        let mut data: Value = serde_json::from_slice(DATA).unwrap();
        data["quests"][1]["stages"][0]["objectives"][0]["creature"] =
            json!("theia.dev.absent-creature");
        assert!(
            world
                .validate_quest_journal_references(&catalog(data))
                .is_err()
        );
        for field in ["npc", "interaction"] {
            let mut data: Value = serde_json::from_slice(DATA).unwrap();
            data["quests"][0]["stages"][0]["objectives"][0][field] =
                json!("theia.dev.absent-reference");
            assert!(
                world
                    .validate_quest_journal_references(&catalog(data))
                    .is_err()
            );
        }
    }
    #[test]
    fn missing_retired_informational_and_live_worlds_cannot_satisfy_reference_admission() {
        let catalog = Catalog::from_json(DATA).unwrap();
        assert!(
            World::default()
                .validate_quest_journal_references(&catalog)
                .is_err()
        );
        let mut world = setup();
        world.dialogue = None;
        world
            .spawn_development_guide(
                Arc::new(
                    Profile::from_json(include_bytes!(
                        "../../../../content/gameplay/development-dialogue.json"
                    ))
                    .unwrap(),
                ),
                [8000., 8000.],
            )
            .unwrap();
        assert!(world.validate_quest_journal_references(&catalog).is_err());
        let mut world = setup();
        world.dialogue = None;
        let guide = world
            .spawn_development_guide(
                Arc::new(Profile::from_json(DIALOGUE).unwrap()),
                [8000., 8000.],
            )
            .unwrap();
        assert!(world.retire_development_guide(guide.key));
        assert!(world.validate_quest_journal_references(&catalog).is_err());
        let mut world = setup();
        world.join().unwrap();
        assert!(world.validate_quest_journal_references(&catalog).is_err());
        let mut world = setup();
        world.step();
        assert!(world.validate_quest_journal_references(&catalog).is_err());
    }

    fn request(
        sequence: u64,
        revision: u64,
        action: crate::dialogue::Action,
    ) -> crate::dialogue::Request {
        crate::dialogue::Request {
            sequence,
            expected_revision: revision,
            action,
        }
    }
    fn id(s: &str) -> theia_core::content::ContentId {
        theia_core::content::ContentId::new(s).unwrap()
    }
    #[test]
    fn only_authorized_semantic_choices_produce_original_occurrences_at_all_rates() {
        use crate::dialogue::{Action, Rejection};
        for hz in [20, 30, 60] {
            let mut world = setup_at(hz);
            let prepared = world
                .prepare_quest_journal(Arc::new(Catalog::from_json(DATA).unwrap()))
                .unwrap();
            let actor = world.join().unwrap();
            let guide = world.visible_guide(actor).unwrap();
            world.players.get_mut(&actor).unwrap().position = guide.position;
            assert_eq!(
                world.interact_quest_journal(
                    &prepared,
                    actor,
                    &request(1, 0, Action::Open(guide.key))
                ),
                Ok(None)
            );
            let road = id("theia.dev.interaction.road");
            let choice = request(2, 1, Action::Choose(road.clone()));
            let occurrence = world
                .interact_quest_journal(&prepared, actor, &choice)
                .unwrap()
                .unwrap();
            assert_eq!(occurrence.actor, world.entity_key(actor).unwrap());
            assert_eq!(
                occurrence.event,
                EventKey::NpcInteraction {
                    npc: id("theia.dev.guide"),
                    interaction: road.clone()
                }
            );
            assert_eq!(
                world.interact_quest_journal(&prepared, actor, &choice),
                Err(Rejection::StaleSequence)
            );
            assert_eq!(
                world.interact_quest_journal(
                    &prepared,
                    actor,
                    &request(3, 2, Action::Choose(road.clone()))
                ),
                Err(Rejection::InvalidAction)
            );
            assert_eq!(
                world.interact_quest_journal(&prepared, actor, &request(4, 2, Action::Close)),
                Ok(None)
            );
            assert_eq!(
                world.interact_quest_journal(
                    &prepared,
                    actor,
                    &request(5, 3, Action::Open(guide.key))
                ),
                Ok(None)
            );
            let later = world
                .interact_quest_journal(&prepared, actor, &request(6, 4, Action::Choose(road)))
                .unwrap()
                .unwrap();
            assert_ne!(
                occurrence.id, later.id,
                "Distinct legitimate visits are distinct occurrences"
            );
        }
    }
    #[test]
    fn foreign_content_incarnation_out_of_range_and_stale_revision_produce_no_occurrence() {
        use crate::dialogue::{Action, Rejection};
        let mut world = setup();
        let catalog = Arc::new(Catalog::from_json(DATA).unwrap());
        let prepared = world.prepare_quest_journal(catalog.clone()).unwrap();
        let actor = world.join().unwrap();
        let guide = world.visible_guide(actor).unwrap();
        world.players.get_mut(&actor).unwrap().position = guide.position;
        world
            .interact_quest_journal(&prepared, actor, &request(1, 0, Action::Open(guide.key)))
            .unwrap();
        let road = id("theia.dev.interaction.road");
        assert_eq!(
            world.interact_quest_journal(
                &prepared,
                actor,
                &request(2, 0, Action::Choose(road.clone()))
            ),
            Err(Rejection::StaleRevision)
        );
        let mut far = guide.position.metres();
        far[1] += 3.0001;
        world.players.get_mut(&actor).unwrap().position =
            theia_core::coordinates::GlobalPosition::new(far).unwrap();
        assert_eq!(
            world.interact_quest_journal(
                &prepared,
                actor,
                &request(3, 1, Action::Choose(road.clone()))
            ),
            Err(Rejection::OutOfRange)
        );
        let other = setup().prepare_quest_journal(catalog).unwrap();
        assert_eq!(
            world.interact_quest_journal(
                &other,
                actor,
                &request(4, 2, Action::Choose(road.clone()))
            ),
            Err(Rejection::Unavailable)
        );
        let mut changed = DIALOGUE.to_vec();
        changed.push(b' ');
        world.dialogue.as_mut().unwrap().profile = Arc::new(Profile::from_json(&changed).unwrap());
        assert_eq!(
            world.interact_quest_journal(&prepared, actor, &request(4, 2, Action::Choose(road))),
            Err(Rejection::Unavailable)
        );
    }

    #[test]
    fn conversation_only_catalogue_needs_no_creature_or_xp_authority() {
        let mut world = World::with_procedural([8000., 8000.], 1.).unwrap();
        world
            .spawn_development_guide(
                Arc::new(Profile::from_json(DIALOGUE).unwrap()),
                [8000., 8000.],
            )
            .unwrap();
        let mut data: Value = serde_json::from_slice(DATA).unwrap();
        data["quests"].as_array_mut().unwrap().truncate(1);
        let prepared = world
            .prepare_quest_journal(Arc::new(catalog(data)))
            .unwrap();
        assert!(world.patrol.is_none() && world.progression.is_none());
        assert!(
            world
                .quest_kill_occurrences(&prepared, &[])
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn router_reserves_capacity_before_choice_and_refuses_closed_or_uninitialized_owner() {
        use crate::{
            dialogue::{Action, Rejection},
            renet_runtime::progression::{
                self, Input, JournalObservation, JournalOperation, Observation, Port,
            },
        };
        use tokio::sync::{mpsc, watch};
        let mut world = setup();
        let xp = Arc::new(
            crate::progression_wire::Profile::from_json(include_bytes!(
                "../../../../content/gameplay/development-progression.json"
            ))
            .unwrap(),
        );
        let profile = Arc::new(crate::quest_journal::Profile::from_json(DATA, &xp).unwrap());
        let prepared = profile.prepare(&world).unwrap();
        let actor = world.join().unwrap();
        let guide = world.visible_guide(actor).unwrap();
        world.players.get_mut(&actor).unwrap().position = guide.position;
        let actor = world.entity_key(actor).unwrap();
        let mut router = prepared.bind(actor);
        let (input, mut requests) = mpsc::channel(progression::CAPACITY);
        let (publish, state) = watch::channel(Observation::default());
        let mut port = Port {
            journal: Some(profile.hash()),
            lifecycle: None,
            quest: None,
            profile: xp,
            input,
            state,
        };
        let open = request(1, 0, Action::Open(guide.key));
        assert_eq!(
            router.interact(&mut world, &open, &port),
            Err(Rejection::Unavailable)
        );
        assert_eq!(
            world.dialogue_view(actor.entity.value()).unwrap().revision,
            0
        );
        // Synthetic already-committed owner view: this test proves reservation
        // and original fact routing only, not SQL or transport acknowledgement.
        publish.send_modify(|s| {
            s.journal = Some(JournalObservation {
                catalog: profile.hash(),
                revision: 0,
                journal: theia_core::quests::journal::Journal::default(),
                reply: None,
            })
        });
        for journal in [None, Some([3; 32])] {
            port.journal = journal;
            assert_eq!(
                router.interact(&mut world, &open, &port),
                Err(Rejection::Unavailable)
            );
            assert_eq!(
                world.dialogue_view(actor.entity.value()).unwrap().revision,
                0
            );
        }
        port.journal = Some(profile.hash());
        assert_eq!(router.interact(&mut world, &open, &port), Ok(false));
        let filler = progression::JournalInput {
            actor,
            catalog: profile.hash(),
            sequence: 100,
            operation: JournalOperation::Event {
                id: NonZeroU128::new(1).unwrap(),
                event: EventKey::Kill(id("theia.dev.training-creature")),
            },
        };
        for _ in 0..progression::CAPACITY {
            port.input.try_send(Input::Journal(filler.clone())).unwrap();
        }
        let choose = request(2, 1, Action::Choose(id("theia.dev.interaction.road")));
        let before = world.dialogue_view(actor.entity.value()).unwrap();
        assert_eq!(
            router.interact(&mut world, &choose, &port),
            Err(Rejection::Unavailable)
        );
        assert_eq!(world.dialogue_view(actor.entity.value()).unwrap(), before);
        for _ in 0..progression::CAPACITY {
            requests.try_recv().unwrap();
        }
        assert_eq!(router.interact(&mut world, &choose, &port), Ok(true));
        let Input::Journal(queued) = requests.try_recv().unwrap() else {
            panic!("Wrong owner input")
        };
        assert_eq!(queued.actor, actor);
        assert_eq!(queued.sequence, 1);
        assert_eq!(queued.catalog, profile.hash());
        let JournalOperation::Event { event, .. } = queued.operation else {
            panic!("Wrong owner operation")
        };
        assert_eq!(
            event,
            EventKey::NpcInteraction {
                npc: id("theia.dev.guide"),
                interaction: id("theia.dev.interaction.road")
            }
        );
        drop(requests);
        let before = world.dialogue_view(actor.entity.value()).unwrap();
        assert_eq!(
            router.interact(&mut world, &request(3, 2, Action::Close), &port),
            Err(Rejection::Unavailable)
        );
        assert_eq!(world.dialogue_view(actor.entity.value()).unwrap(), before);
    }

    #[test]
    fn commands_share_event_sequence_and_only_committed_owner_reply_clears_pending() {
        use crate::{
            dialogue::Action as Talk,
            quest_journal_wire::{Rejection as Refusal, Request as CommandRequest},
            renet_runtime::progression::{
                self, Input, JournalObservation, JournalOperation, JournalRejection, JournalReply,
                Observation, Port,
            },
        };
        use theia_core::quests::journal::{Action, Command, Error as Rule, Journal};
        let mut world = setup();
        let xp = Arc::new(
            crate::progression_wire::Profile::from_json(include_bytes!(
                "../../../../content/gameplay/development-progression.json"
            ))
            .unwrap(),
        );
        let profile = Arc::new(crate::quest_journal::Profile::from_json(DATA, &xp).unwrap());
        let prepared = profile.prepare(&world).unwrap();
        let actor = world.join().unwrap();
        let guide = world.visible_guide(actor).unwrap();
        world.players.get_mut(&actor).unwrap().position = guide.position;
        let actor = world.entity_key(actor).unwrap();
        let mut router = prepared.bind(actor);
        let (input, mut receiver) = tokio::sync::mpsc::channel(progression::CAPACITY);
        let (publish, state) = tokio::sync::watch::channel(Observation {
            journal: Some(JournalObservation {
                catalog: profile.hash(),
                revision: 0,
                journal: Journal::default(),
                reply: None,
            }),
            ..Observation::default()
        });
        let port = Port {
            journal: Some(profile.hash()),
            profile: xp,
            input,
            state,
            quest: None,
            lifecycle: None,
        };
        let mut command = CommandRequest {
            actor,
            profile_hash: profile.hash(),
            sequence: 1,
            npc: guide.key,
            dialogue_revision: 1,
            command: Command {
                request_id: NonZeroU128::new(33).unwrap(),
                quest: id("theia.dev.quest-introduction"),
                npc: id("theia.dev.guide"),
                expected_revision: 0,
                action: Action::Accept,
            },
        };
        assert_eq!(
            router.command(&world, &command, &port),
            Err(Refusal::NotEntitled)
        );
        router
            .interact(&mut world, &request(1, 0, Talk::Open(guide.key)), &port)
            .unwrap();
        router.command(&world, &command, &port).unwrap();
        let Input::Journal(queued) = receiver.try_recv().unwrap() else {
            panic!("Wrong routed command")
        };
        assert_eq!(queued.sequence, 1);
        assert!(matches!(queued.operation, JournalOperation::Command(_)));
        assert!(
            router.take_reply(&world, &port).unwrap().is_none(),
            "Queue acceptance is not commit"
        );
        command.sequence = 2;
        assert_eq!(router.command(&world, &command, &port), Err(Refusal::Busy));
        router
            .interact(
                &mut world,
                &request(2, 1, Talk::Choose(id("theia.dev.interaction.road"))),
                &port,
            )
            .unwrap();
        let Input::Journal(event) = receiver.try_recv().unwrap() else {
            panic!("Wrong routed event")
        };
        assert_eq!(event.sequence, 2);
        // Synthetic committed owner response: proves mapping/ordering only.
        let accepted = Journal::default()
            .propose_accept(
                profile.catalog(),
                &command.command.quest,
                &command.command.npc,
            )
            .unwrap()
            .journal;
        let JournalOperation::Event { event, .. } = event.operation else {
            panic!("Wrong operation")
        };
        let advanced = accepted
            .propose_trusted_event(profile.catalog(), &event)
            .unwrap()
            .journal;
        publish.send_modify(|s| {
            let j = s.journal.as_mut().unwrap();
            j.journal = advanced;
            j.revision = 2;
            j.reply = Some(JournalReply {
                sequence: 1,
                rejection: None,
            });
        });
        let reply = router.take_reply(&world, &port).unwrap().unwrap();
        assert_eq!(reply.sequence, 1);
        assert_eq!(reply.quest_revision, 2);
        assert!(reply.rejection.is_none());
        assert!(router.take_reply(&world, &port).unwrap().is_none());
        command.sequence = 3;
        command.command.action = Action::TurnIn;
        command.command.expected_revision = 2;
        command.command.request_id = NonZeroU128::new(34).unwrap();
        command.dialogue_revision = 2;
        router.command(&world, &command, &port).unwrap();
        let Input::Journal(queued) = receiver.try_recv().unwrap() else {
            panic!("Wrong input")
        };
        assert_eq!(queued.sequence, 3);
        publish.send_modify(|s| {
            s.journal.as_mut().unwrap().reply = Some(JournalReply {
                sequence: 3,
                rejection: Some(JournalRejection::Rule(Rule::NotReady)),
            })
        });
        let reply = router.take_reply(&world, &port).unwrap().unwrap();
        assert_eq!(reply.sequence, 3);
        assert_eq!(reply.rejection, Some(Refusal::NotReady));
    }
}

#[cfg(test)]
mod area_tests;
