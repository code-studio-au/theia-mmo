//! Explicit conditional storage operations; legacy and runtime admission stay closed.
use super::*;
fn receipt(
    row: &Row,
    offset: usize,
    world: [u8; 32],
    profile: &RewardCatalog,
    claim: &RewardClaim,
) -> Result<()> {
    let definition = profile
        .catalog()
        .definition(&claim.reward)
        .ok_or(Error::RewardData)?;
    if definition.quest_item().is_none() || definition.ground().is_some() {
        return Ok(());
    }
    let source = claim.corpse.ok_or(Error::RewardData)?;
    let saved_world: Vec<u8> = row
        .get::<_, Option<Vec<u8>>>(offset)
        .ok_or(Error::RewardData)?;
    let saved_profile: Vec<u8> = row
        .get::<_, Option<Vec<u8>>>(offset + 1)
        .ok_or(Error::RewardData)?;
    if saved_world != world {
        return Err(Error::WorldMismatch);
    }
    if saved_profile != profile.hash() {
        return Err(Error::RewardCatalogMismatch);
    }
    let ticket = u16::try_from(
        row.get::<_, Option<i32>>(offset + 2)
            .ok_or(Error::RewardData)?,
    )
    .map_err(|_| Error::RewardData)?;
    let xyz = [
        row.get::<_, Option<f64>>(offset + 3),
        row.get(offset + 4),
        row.get(offset + 5),
    ];
    if xyz != source.position.metres().map(Some)
        || row.get::<_, Option<bool>>(offset + 6) != Some(true)
        || row.get::<_, Option<i64>>(offset + 7).is_none_or(|r| r < 0)
        || row.get::<_, Option<i64>>(offset + 8).is_none_or(|r| r < 0)
        || !definition.wins_roll(profile.catalog(), ticket)
    {
        return Err(Error::RewardData);
    }
    Ok(())
}
pub(super) async fn check_generation(
    tx: &Transaction<'_>,
    realm: RealmId,
    binding: Binding,
    world: [u8; 32],
    profile: &RewardCatalog,
    claim: &RewardClaim,
) -> Result<()> {
    let definition = profile
        .catalog()
        .definition(&claim.reward)
        .ok_or(Error::RewardData)?;
    if definition.quest_item().is_none() || definition.ground().is_some() {
        return Ok(());
    }
    let source = claim.corpse.ok_or(Error::RewardData)?;
    let key = format!("entry:{}", claim.reward.as_str());
    let row=bounded(tx.query_opt("SELECT world_id,profile_hash,ticket,corpse_x,corpse_y,corpse_z,generation_eligible,generation_bag_revision,generation_journal_revision FROM theia.loot_rolls WHERE realm_id=$1 AND character_id=$2 AND roll_key=$3 AND victim_life=$4 FOR SHARE",&[&realm.value(),&binding.character.value(),&key,&source.life.bytes().as_slice()])).await?.ok_or(Error::RewardData)?;
    receipt(&row, 0, world, profile, claim)
}
impl Store {
    /// Eligibility is applied before the bounded nearest page, using one locked
    /// committed bag/journal pair. Hidden quest claims cannot starve ordinary loot.
    pub async fn list_quest_reward_claims_nearby(
        &mut self,
        binding: Binding,
        world: [u8; 32],
        profile: &RewardCatalog,
        position: GlobalPosition,
    ) -> Result<Vec<RewardClaim>> {
        let journal = profile
            .quest_journal()
            .filter(|_| profile.catalog().has_quest_required())
            .ok_or(Error::Invalid("Quest loot requires a bound journal"))?;
        journal.check_runtime_admission()?;
        let tx = bounded(self.client.as_mut().unwrap().transaction()).await?;
        authorize(&tx, self.realm, binding, world).await?;
        let snapshot =
            super::super::quest_journal::generation_needs(&tx, self.realm, binding, world, journal)
                .await?;
        let known: Vec<String> = profile
            .catalog()
            .definitions()
            .map(|d| d.id().as_str().to_owned())
            .collect();
        let allowed: Vec<String> = profile
            .catalog()
            .definitions()
            .filter(|d| {
                d.quest_item()
                    .is_none_or(|item| snapshot.needed.contains(item))
            })
            .map(|d| d.id().as_str().to_owned())
            .collect();
        let limit = (MAX_DEFINITIONS + 1) as i64;
        let mut rows=bounded(tx.query("SELECT claim_id,reward_id,world_id,profile_hash,consumed_revision,consumed_command,claim_scope,victim_life,corpse_x,corpse_y,corpse_z FROM theia.reward_claims WHERE realm_id=$1 AND character_id=$2 AND claim_scope=0 ORDER BY reward_id,claim_id LIMIT $3",&[&self.realm.value(),&binding.character.value(),&limit])).await?;
        if rows.len() > MAX_DEFINITIONS {
            return Err(Error::RewardData);
        }
        let [x, y, z] = position.metres();
        let radius = theia_core::rewards::LOOT_INTEREST_METRES;
        let page = LIFE_CLAIM_PAGE_SIZE as i64;
        // Unknown IDs deliberately reach checked(), rather than disappearing
        // behind the eligibility filter. LEFT JOIN bounds receipt reads to one page.
        rows.extend(bounded(tx.query("SELECT c.claim_id,c.reward_id,c.world_id,c.profile_hash,c.consumed_revision,c.consumed_command,c.claim_scope,c.victim_life,c.corpse_x,c.corpse_y,c.corpse_z,g.world_id,g.profile_hash,g.ticket,g.corpse_x,g.corpse_y,g.corpse_z,g.generation_eligible,g.generation_bag_revision,g.generation_journal_revision FROM theia.reward_claims c LEFT JOIN theia.loot_rolls g ON g.realm_id=c.realm_id AND g.character_id=c.character_id AND g.roll_key=('entry:'||c.reward_id) AND g.victim_life=c.victim_life WHERE c.realm_id=$1 AND c.character_id=$2 AND c.claim_scope=1 AND c.consumed_revision IS NULL AND c.corpse_x BETWEEN $4::double precision-$7::double precision AND $4::double precision+$7::double precision AND c.corpse_y BETWEEN $5::double precision-$7::double precision AND $5::double precision+$7::double precision AND c.corpse_z BETWEEN $6::double precision-$7::double precision AND $6::double precision+$7::double precision AND ((c.corpse_x-$4::double precision)^2+(c.corpse_y-$5::double precision)^2+(c.corpse_z-$6::double precision)^2) <= $7::double precision*$7::double precision AND (NOT(c.reward_id=ANY($8::text[])) OR c.reward_id=ANY($9::text[])) ORDER BY ((c.corpse_x-$4::double precision)^2+(c.corpse_y-$5::double precision)^2+(c.corpse_z-$6::double precision)^2),c.created_at,c.claim_id LIMIT $3",&[&self.realm.value(),&binding.character.value(),&page,&x,&y,&z,&radius,&known,&allowed])).await?);
        let mut claims = Vec::with_capacity(rows.len());
        for row in &rows {
            let (claim, _) = checked(row, world, profile)?;
            receipt(row, 11, world, profile, &claim)?;
            if profile
                .catalog()
                .definition(&claim.reward)
                .is_some_and(|d| {
                    d.ground().is_some()
                        && d.quest_item().is_some_and(|i| !snapshot.needed.contains(i))
                })
                && claim.consumed_revision.is_none()
            {
                continue;
            }
            claims.push(claim);
        }
        claims.sort_by(|a, b| {
            a.reward
                .cmp(&b.reward)
                .then_with(|| a.id.bytes().cmp(&b.id.bytes()))
        });
        bounded(tx.commit()).await?;
        Ok(claims)
    }
}
