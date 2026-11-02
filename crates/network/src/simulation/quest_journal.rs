//! Explicit expanded-quest World adapters. Installed-reference checks and
//! normalized facts establish no durable journal or normal-client adoption.
use super::World;
use std::{num::NonZeroU128, sync::Arc};
use theia_core::quests::catalog::{Catalog, EventKey};
/// Immutable admission precursor, tied to one World incarnation and its content.
/// This is not a saved journal or a transport/profile compatibility assertion.
pub struct Prepared {
    catalog: Arc<Catalog>,
    epoch: u64,
    world: [u8; 32],
    kill: Option<theia_core::content::ContentId>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Occurrence {
    pub actor: theia_core::identity::EntityKey,
    pub id: NonZeroU128,
    pub event: EventKey,
}

impl World {
    pub fn prepare_quest_journal(&self, catalog: Arc<Catalog>) -> crate::Result<Prepared> {
        self.validate_quest_journal_references(&catalog)?;
        let kill = catalog
            .quests()
            .flat_map(|q| q.stages())
            .flat_map(|s| s.objectives())
            .find_map(|o| match o.event() {
                EventKey::Kill(id) => Some(id.clone()),
                _ => None,
            });
        Ok(Prepared {
            kill,
            catalog,
            epoch: self.epoch(),
            world: self.identity(),
        })
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
    /// policy. Trusted input is the bounded authoritative combat event batch.
    pub fn quest_kill_occurrences(
        &self,
        prepared: &Prepared,
        events: &[super::combat::Event],
    ) -> crate::Result<Vec<Occurrence>> {
        if prepared.epoch != self.epoch() || prepared.world != self.identity() {
            return Err("Quest occurrence World binding changed".into());
        }
        if events.len() > super::combat::MAX_PROJECTILE_STEP_EVENTS {
            return Err("Quest combat source batch exceeds event budget".into());
        }
        let Some(creature) = &prepared.kill else {
            return Ok(Vec::new());
        };
        let patrol = self
            .patrol
            .as_ref()
            .ok_or("Quest creature authority missing")?;
        let event = EventKey::Kill(creature.clone());
        if prepared.catalog.matching(&event).is_empty() {
            return Ok(Vec::new());
        }
        let mut occurrences = Vec::new();
        for source in events {
            let Some((actor, victim)) = self.kill_recipient(source)? else {
                continue;
            };
            if victim != patrol.key || self.entity_key(actor.entity.value()) != Some(actor) {
                continue;
            }
            let mut bytes = b"theia-journal-kill-occurrence-v1\0".to_vec();
            bytes.extend(victim.epoch.value().to_le_bytes());
            bytes.extend(victim.entity.value().to_le_bytes());
            let hash = crate::terrain::sha256(&bytes);
            let id = NonZeroU128::new(u128::from_le_bytes(hash[..16].try_into().unwrap()).max(1))
                .unwrap();
            let occurrence = Occurrence {
                actor,
                id,
                event: event.clone(),
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
        if self.tick != 0
            || !self.players.is_empty()
            || self.quest.is_some()
            || self.quest_lifecycle.is_some()
        {
            return Err("Expanded quest reference checks require inactive exclusive World".into());
        }
        let guide = self
            .dialogue
            .as_ref()
            .filter(|d| d.available())
            .ok_or("Quest contacts require an installed available guide")?;
        let dialogue = guide.profile.profile();
        for quest in catalog.quests() {
            if quest.giver() != dialogue.npc() || quest.turn_in() != dialogue.npc() {
                return Err("Quest contact is not installed in this World".into());
            }
            for stage in quest.stages() {
                for objective in stage.objectives() {
                    match objective.event() {
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
    fn setup_at(hz: u32) -> World {
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
        let port = Port {
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
}
