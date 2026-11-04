//! Durable once-character and per-victim-life loot entitlements. Authority creates offers; clients may only ask
//! their admitted character to consume a known claim through the later adapter.
use super::{
    Store, bounded,
    inventory::{authorize, checked as checked_bag},
};
use crate::{
    Binding, ClaimId, Error, InventoryApplied, RealmId, Result, RewardCatalog, RewardClaim,
    rewards::{CorpseSource, LIFE_CLAIM_PAGE_SIZE, command_hash},
};
use theia_core::{
    content::ContentId,
    coordinates::GlobalPosition,
    inventory::MAX_REVISION,
    progression::{KillEvent, KillId},
    rewards::{MAX_DEFINITIONS, Occurrence, Scope},
};
use tokio_postgres::{Row, Transaction};
fn checked(
    row: &Row,
    world: [u8; 32],
    profile: &RewardCatalog,
) -> Result<(RewardClaim, Option<[u8; 32]>)> {
    if row.get::<_, Vec<u8>>(2) != world {
        return Err(Error::WorldMismatch);
    }
    if row.get::<_, Vec<u8>>(3) != profile.hash() {
        return Err(Error::RewardCatalogMismatch);
    }
    let id = ClaimId::new(*row.get::<_, uuid::Uuid>(0).as_bytes()).ok_or(Error::RewardData)?;
    let reward = ContentId::new(&row.get::<_, String>(1))
        .filter(|id| profile.catalog().definition(id).is_some())
        .ok_or(Error::RewardData)?;
    let definition = profile
        .catalog()
        .definition(&reward)
        .ok_or(Error::RewardData)?;
    let corpse = match (
        row.get::<_, i16>(6),
        row.get::<_, Option<Vec<u8>>>(7),
        definition.scope(),
    ) {
        (0, None, Scope::OncePerCharacter) => {
            if (8..=10).any(|i| row.get::<_, Option<f64>>(i).is_some()) {
                return Err(Error::RewardData);
            }
            None
        }
        (1, Some(bytes), Scope::PerCreatureLife) => Some(CorpseSource {
            life: KillId::from_bytes(bytes.try_into().map_err(|_| Error::RewardData)?)
                .ok_or(Error::RewardData)?,
            position: GlobalPosition::new([
                row.get::<_, Option<f64>>(8).ok_or(Error::RewardData)?,
                row.get::<_, Option<f64>>(9).ok_or(Error::RewardData)?,
                row.get::<_, Option<f64>>(10).ok_or(Error::RewardData)?,
            ])
            .ok_or(Error::RewardData)?,
        }),
        _ => return Err(Error::RewardData),
    };
    let revision: Option<i64> = row.get(4);
    let receipt: Option<Vec<u8>> = row.get(5);
    if revision.is_some() != receipt.is_some() || revision.is_some_and(|r| r <= 0) {
        return Err(Error::RewardData);
    }
    let receipt = receipt
        .map(|b| {
            let hash: [u8; 32] = b.try_into().map_err(|_| Error::RewardData)?;
            if hash == [0; 32] {
                return Err(Error::RewardData);
            }
            Ok(hash)
        })
        .transpose()?;
    Ok((
        RewardClaim {
            id,
            reward,
            consumed_revision: revision.map(|r| r as u64),
            corpse,
        },
        receipt,
    ))
}
async fn validate_items(
    transaction: &Transaction<'_>,
    realm: RealmId,
    binding: Binding,
    profile: &RewardCatalog,
) -> Result<()> {
    let row=bounded(transaction.query_opt("SELECT catalog_hash,revision,state FROM theia.backpacks WHERE realm_id=$1 AND character_id=$2 FOR SHARE",&[&realm.value(),&binding.character.value()])).await?.ok_or(Error::InventoryMissing)?;
    checked_bag(&row, profile.items())?;
    Ok(())
}
impl Store {
    /// Trusted qualifying-event adapter only, not a client grant endpoint. The
    /// caller must establish the creature kill/recipient; this boundary enforces
    /// ownership, byte identity, nonrepeatability and COMMIT before availability.
    pub async fn offer_reward(
        &mut self,
        binding: Binding,
        world: [u8; 32],
        profile: &RewardCatalog,
        reward: &ContentId,
    ) -> Result<RewardClaim> {
        let definition = profile
            .catalog()
            .definition(reward)
            .ok_or(Error::RewardMissing)?;
        if definition.scope() != Scope::OncePerCharacter {
            return Err(Error::Invalid(
                "Repeatable loot requires a victim life and corpse position",
            ));
        }
        self.offer_reward_source(binding, world, profile, reward, None)
            .await
    }
    /// Trusted authority fact only. Offer commits availability, never items or
    /// possession progress. Pickup must separately validate ownership and range.
    pub async fn offer_reward_for_kill(
        &mut self,
        binding: Binding,
        world: [u8; 32],
        profile: &RewardCatalog,
        reward: &ContentId,
        event: &KillEvent,
        position: GlobalPosition,
    ) -> Result<RewardClaim> {
        let definition = profile
            .catalog()
            .definition(reward)
            .ok_or(Error::RewardMissing)?;
        let corpse = match definition.occurrence(event).map_err(Error::RewardCommand)? {
            Occurrence::OncePerCharacter => None,
            Occurrence::CreatureLife(life) => Some(CorpseSource { life, position }),
        };
        self.offer_reward_source(binding, world, profile, reward, corpse)
            .await
    }
    async fn offer_reward_source(
        &mut self,
        binding: Binding,
        world: [u8; 32],
        profile: &RewardCatalog,
        reward: &ContentId,
        corpse: Option<CorpseSource>,
    ) -> Result<RewardClaim> {
        let transaction = bounded(self.client.as_mut().unwrap().transaction()).await?;
        authorize(&transaction, self.realm, binding, world).await?;
        validate_items(&transaction, self.realm, binding, profile).await?;
        let claim = uuid::Uuid::new_v4();
        let scope = i16::from(corpse.is_some());
        let life = corpse.map(|c| c.life.bytes().to_vec());
        let x = corpse.map(|c| c.position.metres()[0]);
        let y = corpse.map(|c| c.position.metres()[1]);
        let z = corpse.map(|c| c.position.metres()[2]);
        bounded(transaction.execute("INSERT INTO theia.reward_claims(realm_id,claim_id,character_id,reward_id,world_id,profile_hash,claim_scope,victim_life,corpse_x,corpse_y,corpse_z) VALUES($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11) ON CONFLICT DO NOTHING",&[&self.realm.value(),&claim,&binding.character.value(),&reward.as_str(),&world.as_slice(),&profile.hash().as_slice(),&scope,&life,&x,&y,&z])).await?;
        let row=bounded(transaction.query_one("SELECT claim_id,reward_id,world_id,profile_hash,consumed_revision,consumed_command,claim_scope,victim_life,corpse_x,corpse_y,corpse_z FROM theia.reward_claims WHERE realm_id=$1 AND character_id=$2 AND reward_id=$3 AND claim_scope=$4 AND victim_life IS NOT DISTINCT FROM $5 FOR UPDATE",&[&self.realm.value(),&binding.character.value(),&reward.as_str(),&scope,&life])).await?;
        let (claim, _) = checked(&row, world, profile)?;
        if claim.corpse != corpse {
            return Err(Error::RewardData);
        }
        bounded(transaction.commit()).await?;
        Ok(claim)
    }
    /// Complete bounded once-character history plus the oldest pending per-life
    /// page. Unshown pending loot and consumed life receipts remain durable;
    /// consuming an entry reveals the next page on reload.
    pub async fn list_reward_claims(
        &mut self,
        binding: Binding,
        world: [u8; 32],
        profile: &RewardCatalog,
    ) -> Result<Vec<RewardClaim>> {
        let transaction = bounded(self.client.as_mut().unwrap().transaction()).await?;
        authorize(&transaction, self.realm, binding, world).await?;
        validate_items(&transaction, self.realm, binding, profile).await?;
        let limit = (MAX_DEFINITIONS + 1) as i64;
        let mut rows=bounded(transaction.query("SELECT claim_id,reward_id,world_id,profile_hash,consumed_revision,consumed_command,claim_scope,victim_life,corpse_x,corpse_y,corpse_z FROM theia.reward_claims WHERE realm_id=$1 AND character_id=$2 AND claim_scope=0 ORDER BY reward_id,claim_id LIMIT $3",&[&self.realm.value(),&binding.character.value(),&limit])).await?;
        if rows.len() > MAX_DEFINITIONS {
            return Err(Error::RewardData);
        }
        let page_limit = LIFE_CLAIM_PAGE_SIZE as i64;
        rows.extend(bounded(transaction.query("SELECT claim_id,reward_id,world_id,profile_hash,consumed_revision,consumed_command,claim_scope,victim_life,corpse_x,corpse_y,corpse_z FROM theia.reward_claims WHERE realm_id=$1 AND character_id=$2 AND claim_scope=1 AND consumed_revision IS NULL ORDER BY created_at,claim_id LIMIT $3",&[&self.realm.value(),&binding.character.value(),&page_limit])).await?);
        let mut claims = rows
            .iter()
            .map(|row| checked(row, world, profile).map(|(claim, _)| claim))
            .collect::<Result<Vec<_>>>()?;
        claims.sort_by(|a, b| {
            a.reward
                .cmp(&b.reward)
                .then_with(|| a.id.bytes().cmp(&b.id.bytes()))
        });
        bounded(transaction.commit()).await?;
        Ok(claims)
    }
    /// Pickup is one atomic item-state/claim transaction. An exact latest retry
    /// recovers its committed result; older/superseded intents reject. No capacity
    /// leaves both item state and durable availability untouched.
    pub async fn claim_reward(
        &mut self,
        binding: Binding,
        world: [u8; 32],
        profile: &RewardCatalog,
        claim: ClaimId,
        expected_revision: u64,
    ) -> Result<InventoryApplied> {
        self.claim_reward_inner(binding, world, profile, claim, expected_revision, None)
            .await
    }
    /// Commit the item mutation and all resulting possession projections together.
    pub async fn claim_reward_with_quest_journal(
        &mut self,
        binding: Binding,
        world: [u8; 32],
        profile: &RewardCatalog,
        claim: ClaimId,
        expected_revision: u64,
        journal: &crate::QuestJournalCatalog,
    ) -> Result<InventoryApplied> {
        self.claim_reward_inner(
            binding,
            world,
            profile,
            claim,
            expected_revision,
            Some(journal),
        )
        .await
    }
    #[allow(clippy::too_many_arguments)]
    async fn claim_reward_inner(
        &mut self,
        binding: Binding,
        world: [u8; 32],
        profile: &RewardCatalog,
        claim: ClaimId,
        expected_revision: u64,
        journal: Option<&crate::QuestJournalCatalog>,
    ) -> Result<InventoryApplied> {
        if expected_revision > MAX_REVISION {
            return Err(Error::InventoryStale);
        }
        let transaction = bounded(self.client.as_mut().unwrap().transaction()).await?;
        authorize(&transaction, self.realm, binding, world).await?;
        // Every item+claim transaction takes the bag before the claim row.
        let row=bounded(transaction.query_opt("SELECT catalog_hash,revision,state,last_command FROM theia.backpacks WHERE realm_id=$1 AND character_id=$2 FOR UPDATE",&[&self.realm.value(),&binding.character.value()])).await?.ok_or(Error::InventoryMissing)?;
        let bag = checked_bag(&row, profile.items())?;
        let receipt: Option<Vec<u8>> = row.get(3);
        let row=bounded(transaction.query_opt("SELECT claim_id,reward_id,world_id,profile_hash,consumed_revision,consumed_command,claim_scope,victim_life,corpse_x,corpse_y,corpse_z FROM theia.reward_claims WHERE realm_id=$1 AND character_id=$2 AND claim_id=$3 FOR UPDATE",&[&self.realm.value(),&binding.character.value(),&uuid::Uuid::from_bytes(claim.bytes())])).await?.ok_or(Error::RewardMissing)?;
        let (saved, consumed_receipt) = checked(&row, world, profile)?;
        let hash = command_hash(claim, expected_revision);
        if expected_revision.checked_add(1) == Some(bag.revision()) {
            if receipt.as_deref() != Some(hash.as_slice()) {
                return Err(Error::InventoryConflict);
            }
            if saved.consumed_revision != Some(bag.revision()) || consumed_receipt != Some(hash) {
                return Err(Error::RewardData);
            }
            let journal = super::quest_journal::reconcile_inventory(
                &transaction,
                self.realm,
                binding,
                world,
                journal,
                profile.items(),
                &bag,
                true,
            )
            .await?;
            bounded(transaction.commit()).await?;
            return Ok(InventoryApplied {
                bag,
                journal,
                replayed: true,
            });
        }
        if expected_revision != bag.revision() {
            return Err(Error::InventoryStale);
        }
        if saved.consumed_revision.is_some() {
            return Err(Error::RewardConsumed);
        }
        let definition = profile
            .catalog()
            .definition(&saved.reward)
            .ok_or(Error::RewardData)?;
        let next = definition
            .propose(&bag, profile.items().catalog(), expected_revision)
            .map_err(Error::RewardCommand)?;
        let revision = next.revision() as i64;
        let state = next.to_json();
        let changed=bounded(transaction.execute("UPDATE theia.backpacks SET revision=$1,state=$2,last_command=$3,updated_at=clock_timestamp() WHERE realm_id=$4 AND character_id=$5 AND revision=$6",&[&revision,&state,&hash.as_slice(),&self.realm.value(),&binding.character.value(),&(bag.revision()as i64)])).await?;
        if changed != 1 {
            return Err(Error::InventoryData);
        }
        let changed=bounded(transaction.execute("UPDATE theia.reward_claims SET consumed_revision=$1,consumed_command=$2 WHERE realm_id=$3 AND character_id=$4 AND claim_id=$5 AND consumed_revision IS NULL",&[&revision,&hash.as_slice(),&self.realm.value(),&binding.character.value(),&uuid::Uuid::from_bytes(claim.bytes())])).await?;
        if changed != 1 {
            return Err(Error::RewardData);
        }
        let journal = super::quest_journal::reconcile_inventory(
            &transaction,
            self.realm,
            binding,
            world,
            journal,
            profile.items(),
            &next,
            false,
        )
        .await?;
        bounded(transaction.commit()).await?;
        Ok(InventoryApplied {
            bag: next,
            journal,
            replayed: false,
        })
    }
}
