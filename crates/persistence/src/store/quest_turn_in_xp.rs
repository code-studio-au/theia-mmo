//! Atomic completion/XP intent receipts. NPC entitlement and retained ownership
//! belong to the runtime; this method never runs in the fixed simulation step.
use super::{Store, bounded, inventory::authorize, progression, quest_lifecycle};
use crate::{
    Binding, EarnedQuestLifecycle, Error, ProgressionApplied, ProgressionCatalog,
    QuestLifecycleApplied, QuestLifecycleCatalog, Result,
};
use theia_core::{
    progression::State as Experience,
    quests::{
        lifecycle::{self as life, Action, Command, Status},
        turn_in_xp,
    },
};

fn checked_experience_receipt(
    row: &tokio_postgres::Row,
    xp: &ProgressionCatalog,
    state: &Experience,
    recipe: &turn_in_xp::Recipe,
    action: Action,
) -> Result<()> {
    let (Some(hash), Some(granted), Some(revision), Some(total)) = (
        row.get::<_, Option<Vec<u8>>>(2),
        row.get::<_, Option<i32>>(3),
        row.get::<_, Option<i64>>(4),
        row.get::<_, Option<i32>>(5),
    ) else {
        return Err(Error::QuestData);
    };
    if hash != xp.hash() {
        return Err(Error::ProgressionCatalogMismatch);
    }
    if granted < 0
        || granted as u32 > recipe.experience()
        || revision < 0
        || revision as u64 > state.revision()
        || total < granted
        || total as u32 > state.total()
        || (revision as u64 == state.revision() && total as u32 != state.total())
        || (granted > 0 && revision == 0)
    {
        return Err(Error::ProgressionData);
    }
    match action {
        Action::Accept if granted != 0 => return Err(Error::ProgressionData),
        Action::TurnIn => {
            let before = (total - granted) as u32;
            let expected = recipe
                .experience()
                .min(xp.profile().maximum_experience() - before);
            if expected != granted as u32 {
                return Err(Error::ProgressionData);
            }
        }
        _ => {}
    }
    Ok(())
}
impl Store {
    /// XP first, then quest. State, actual award and receipt COMMIT together.
    /// Retry recovery returns current state and newly granted zero, including
    /// acceptance retries after later kills/completion. Old mode is separate.
    pub async fn command_quest_lifecycle_with_experience(
        &mut self,
        binding: Binding,
        world: [u8; 32],
        xp: &ProgressionCatalog,
        catalog: &QuestLifecycleCatalog,
        command: Command,
    ) -> Result<EarnedQuestLifecycle> {
        if command.expected_revision > 101 {
            return Err(Error::QuestLifecycleCommand(life::Error::StaleRevision));
        }
        catalog.check_progression(xp)?;
        let recipe = catalog
            .turn_in_xp()
            .ok_or(Error::Invalid("Completion XP recipe is required"))?;
        let tx = bounded(self.client.as_mut().unwrap().transaction()).await?;
        authorize(&tx, self.realm, binding, world).await?;
        let experience = progression::lock_state(&tx, self.realm, binding, world, xp).await?;
        let state = quest_lifecycle::lock_state(&tx, self.realm, binding, world, catalog).await?;
        let hash = catalog.command_hash(command);
        if let Some(row)=bounded(tx.query_opt("SELECT command_hash,action,xp_profile_hash,xp_granted,xp_revision,xp_total FROM theia.quest_action_receipts WHERE realm_id=$1 AND character_id=$2 AND expected_revision=$3 FOR SHARE",&[&self.realm.value(),&binding.character.value(),&(command.expected_revision as i64)])).await? {
            if row.get::<_,Vec<u8>>(0)!=hash || row.get::<_,i16>(1)!=i16::from(command.canonical_bytes()[8]) {return Err(Error::QuestConflict);}
            if state.revision()<=command.expected_revision || (command.action==Action::TurnIn && state.status()!=Status::TurnedIn) {return Err(Error::QuestData);}
            checked_experience_receipt(&row,xp,&experience,recipe,command.action)?;
            bounded(tx.commit()).await?;
            return Ok(EarnedQuestLifecycle {
                experience:ProgressionApplied {state:experience,granted:0,replayed:true},
                quest:quest_lifecycle::unchanged(state,true),
            });
        }
        let (proposal, next, granted) = match command.action {
            Action::Accept => (
                life::propose_action(catalog.profile(), &state, command)
                    .map_err(Error::QuestLifecycleCommand)?,
                experience.clone(),
                0,
            ),
            Action::TurnIn => {
                let proposal = recipe
                    .propose(
                        catalog.profile(),
                        &state,
                        xp.profile(),
                        &experience,
                        command,
                    )
                    .map_err(|error| match error {
                        turn_in_xp::Error::Quest(e) => Error::QuestLifecycleCommand(e),
                        turn_in_xp::Error::Progression(e) => Error::ProgressionCommand(e),
                        turn_in_xp::Error::RecipeMismatch => Error::QuestCatalogMismatch,
                        turn_in_xp::Error::NotTurnIn => {
                            Error::Invalid("Unexpected completion action")
                        }
                    })?;
                (
                    proposal.quest,
                    proposal.experience.state,
                    proposal.experience.granted,
                )
            }
        };
        bounded(tx.execute("INSERT INTO theia.quest_action_receipts(realm_id,character_id,expected_revision,action,command_hash,xp_profile_hash,xp_granted,xp_revision,xp_total) VALUES($1,$2,$3,$4,$5,$6,$7,$8,$9)",&[&self.realm.value(),&binding.character.value(),&(command.expected_revision as i64),&i16::from(command.canonical_bytes()[8]),&hash.as_slice(),&xp.hash().as_slice(),&(granted as i32),&(next.revision()as i64),&(next.total()as i32)])).await?;
        quest_lifecycle::update(&tx, self.realm, binding, &state, &proposal.state).await?;
        if granted > 0 {
            progression::update_state(&tx, self.realm, binding, &experience, &next).await?;
        }
        bounded(tx.commit()).await?;
        Ok(EarnedQuestLifecycle {
            experience: ProgressionApplied {
                state: next,
                granted,
                replayed: false,
            },
            quest: QuestLifecycleApplied {
                state: proposal.state,
                changed: proposal.changed,
                became_ready: false,
                turned_in_now: command.action == Action::TurnIn,
                replayed: false,
            },
        })
    }
}
