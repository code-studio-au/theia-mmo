//! Generation-bound publication of committed inventory/journal transactions.
//! Channels can be sampled on separate frames; each projection is monotonic.
use super::*;
use theia_core::inventory::Bag;
use theia_network::renet_runtime::progression::{JournalObservation, JournalReply, Observation};
use theia_persistence::QuestJournalApplied;

pub(super) fn merge_bag(current: &mut Option<Bag>, incoming: Bag) -> Result<()> {
    if let Some(before) = current.as_ref() {
        if before.revision() > incoming.revision() {
            return Ok(());
        }
        if before.revision() == incoming.revision() && before != &incoming {
            return Err("Conflicting committed backpack projection".into());
        }
    }
    *current = Some(incoming);
    Ok(())
}
pub(super) fn merge_journal(
    state: &mut Observation,
    catalog: [u8; 32],
    applied: QuestJournalApplied,
    reply: Option<JournalReply>,
) -> Result<()> {
    if let Some(before) = &state.journal {
        if before.catalog != catalog {
            return Err("Committed journal profile changed".into());
        }
        if before.revision == applied.revision && before.journal != applied.journal {
            return Err("Conflicting committed journal projection".into());
        }
    }
    let prior_reply = state.journal.as_ref().and_then(|j| j.reply.clone());
    if state
        .journal
        .as_ref()
        .is_none_or(|j| j.revision <= applied.revision)
    {
        state.journal = Some(JournalObservation {
            catalog,
            revision: applied.revision,
            journal: applied.journal,
            reply: prior_reply,
        });
    }
    if let Some(reply) = reply {
        let journal = state.journal.as_mut().unwrap();
        if journal
            .reply
            .as_ref()
            .is_none_or(|r| r.sequence < reply.sequence)
        {
            journal.reply = Some(reply);
        }
    }
    if let Some(xp) = applied.experience {
        if let Some(before) = &state.progress
            && before.revision() == xp.revision()
            && before != &xp
        {
            return Err("Conflicting committed XP projection".into());
        }
        if state
            .progress
            .as_ref()
            .is_none_or(|before| before.revision() <= xp.revision())
        {
            state.progress = Some(xp);
        }
    }
    Ok(())
}
impl GameAdmissions {
    pub(super) fn publish_quest_items(
        &self,
        slot: usize,
        generation: u64,
        binding: Binding,
        catalog: [u8; 32],
        applied: QuestJournalApplied,
        reply: Option<JournalReply>,
    ) -> Result<()> {
        let state = self.lock()?;
        let entry = state.slots[slot]
            .as_ref()
            .filter(|e| {
                e.generation == generation
                    && e.binding == binding
                    && matches!(e.phase, Phase::Consumed | Phase::Active | Phase::Retiring)
            })
            .ok_or("Committed item journal lost character generation")?;
        let inventory = entry
            .inventory
            .as_ref()
            .ok_or("Item journal has no inventory owner")?;
        let progression = entry
            .progression
            .as_ref()
            .ok_or("Item journal has no progression owner")?;
        if let Some(bag) = applied.inventory.clone() {
            let mut result = Ok(());
            inventory
                .publish
                .send_modify(|s| result = merge_bag(&mut s.bag, bag));
            result?;
        }
        let mut result = Ok(());
        progression
            .publish
            .send_modify(|s| result = merge_journal(s, catalog, applied, reply));
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use theia_core::{
        content::ContentId,
        inventory::{Catalog, Command, Operation, Stack},
        quests::{catalog::Catalog as Quests, journal::Journal},
    };
    fn id(s: &str) -> ContentId {
        ContentId::new(s).unwrap()
    }
    fn journal() -> Journal {
        let q = Quests::from_json(include_bytes!(
            "../../../../content/quests/development-journal.json"
        ))
        .unwrap();
        Journal::default()
            .propose_accept(
                &q,
                &id("theia.dev.quest-introduction"),
                &id("theia.dev.guide"),
            )
            .unwrap()
            .journal
    }
    fn applied(journal: Journal, revision: u64) -> QuestJournalApplied {
        QuestJournalApplied {
            inventory: None,
            journal,
            revision,
            changed: Vec::new(),
            replayed: false,
            experience: None,
            experience_granted: 0,
        }
    }
    #[test]
    fn delayed_bag_before_destruction_cannot_restore_removed_item_and_equal_revision_conflict_refuses()
     {
        let c = Catalog::from_json(include_bytes!(
            "../../../../content/gameplay/development-equipment-items.json"
        ))
        .unwrap();
        let mut slots = std::array::from_fn(|_| None);
        slots[0] = Stack::new(&c, id("theia.dev.training-stone"), 3);
        let before = Bag::from_slots(&c, 0, slots).unwrap();
        let after = before
            .propose(
                &c,
                Command {
                    expected_revision: 0,
                    operation: Operation::Destroy { from: 0, amount: 1 },
                },
            )
            .unwrap();
        let mut published = None;
        merge_bag(&mut published, after.clone()).unwrap();
        merge_bag(&mut published, before).unwrap();
        assert_eq!(published, Some(after));
        let other = Bag::from_slots(&c, 1, std::array::from_fn(|_| None)).unwrap();
        assert!(merge_bag(&mut published, other).is_err());
        assert_eq!(
            published.as_ref().unwrap().slots()[0]
                .as_ref()
                .unwrap()
                .count(),
            2
        );
    }
    #[test]
    fn late_command_reply_attaches_to_latest_committed_journal_without_regressing_it_or_newer_reply()
     {
        let mut state = Observation::default();
        let j = journal();
        merge_journal(
            &mut state,
            [1; 32],
            applied(j.clone(), 4),
            Some(JournalReply {
                sequence: 8,
                rejection: None,
            }),
        )
        .unwrap();
        merge_journal(
            &mut state,
            [1; 32],
            applied(Journal::default(), 1),
            Some(JournalReply {
                sequence: 7,
                rejection: None,
            }),
        )
        .unwrap();
        assert_eq!(state.journal.as_ref().unwrap().journal, j);
        assert_eq!(state.journal.as_ref().unwrap().revision, 4);
        assert_eq!(
            state
                .journal
                .as_ref()
                .unwrap()
                .reply
                .as_ref()
                .unwrap()
                .sequence,
            8
        );
        merge_journal(
            &mut state,
            [1; 32],
            applied(Journal::default(), 1),
            Some(JournalReply {
                sequence: 9,
                rejection: None,
            }),
        )
        .unwrap();
        assert_eq!(state.journal.as_ref().unwrap().revision, 4);
        assert_eq!(
            state
                .journal
                .as_ref()
                .unwrap()
                .reply
                .as_ref()
                .unwrap()
                .sequence,
            9
        );
    }
    #[test]
    fn equal_head_conflict_or_profile_switch_refuses_and_background_update_preserves_reply() {
        let mut state = Observation::default();
        let j = journal();
        merge_journal(
            &mut state,
            [1; 32],
            applied(j.clone(), 1),
            Some(JournalReply {
                sequence: 1,
                rejection: None,
            }),
        )
        .unwrap();
        assert!(merge_journal(&mut state, [1; 32], applied(Journal::default(), 1), None).is_err());
        assert!(merge_journal(&mut state, [2; 32], applied(j.clone(), 2), None).is_err());
        merge_journal(&mut state, [1; 32], applied(j, 2), None).unwrap();
        assert_eq!(
            state
                .journal
                .as_ref()
                .unwrap()
                .reply
                .as_ref()
                .unwrap()
                .sequence,
            1
        );
    }
    #[test]
    fn delayed_xp_projection_cannot_erase_earned_reward_and_equal_revision_conflict_refuses() {
        use theia_core::progression::{Profile, State};
        let profile = Profile::from_json(include_bytes!(
            "../../../../content/gameplay/development-progression.json"
        ))
        .unwrap();
        let before = State::new(&profile, 0, 0).unwrap();
        let earned = profile.propose_grant(&before, 25).unwrap().state;
        let mut state = Observation::default();
        let mut latest = applied(journal(), 2);
        latest.experience = Some(earned.clone());
        merge_journal(&mut state, [1; 32], latest, None).unwrap();
        let mut delayed = applied(Journal::default(), 0);
        delayed.experience = Some(before);
        merge_journal(&mut state, [1; 32], delayed, None).unwrap();
        assert_eq!(state.progress, Some(earned.clone()));
        let mut conflict = applied(journal(), 2);
        conflict.experience = Some(State::new(&profile, earned.revision(), 24).unwrap());
        assert!(merge_journal(&mut state, [1; 32], conflict, None).is_err());
        assert_eq!(state.progress, Some(earned));
    }
}
