//! Read-only admission check, outside the fixed simulation and active owners.
use super::{Store, bounded};
use crate::{Binding, Error, GameplayProfiles, ProfileCompatibility, Result};
use tokio_postgres::IsolationLevel;

impl Store {
    pub async fn check_gameplay_profiles(
        &mut self,
        binding: Binding,
        world: [u8; 32],
        profiles: GameplayProfiles,
    ) -> Result<ProfileCompatibility> {
        if world == [0; 32] {
            return Err(Error::Invalid("World content identity must be nonzero"));
        }
        let tx = bounded(
            self.client
                .as_mut()
                .unwrap()
                .build_transaction()
                .isolation_level(IsolationLevel::RepeatableRead)
                .read_only(true)
                .start(),
        )
        .await?;
        let row = bounded(tx.query_opt(
            "SELECT owner_id,world_id FROM theia.characters WHERE realm_id=$1 AND character_id=$2",
            &[&self.realm.value(), &binding.character.value()],
        ))
        .await?
        .ok_or(Error::CharacterMissing)?;
        if row.get::<_, uuid::Uuid>(0) != binding.owner.value() {
            return Err(Error::Ownership);
        }
        if row.get::<_, Vec<u8>>(1) != world {
            return Err(Error::WorldMismatch);
        }
        // Fixed tables/roles, at most four rows, no dynamic SQL or client hashes.
        let rows = bounded(tx.query(
            "SELECT 0::int AS role,catalog_hash AS hash,NULL::bytea AS world FROM theia.backpacks WHERE realm_id=$1 AND character_id=$2
             UNION ALL SELECT 1,profile_hash,world_id FROM theia.progression WHERE realm_id=$1 AND character_id=$2
             UNION ALL SELECT 2,profile_hash,world_id FROM theia.quest_objectives WHERE realm_id=$1 AND character_id=$2
             UNION ALL SELECT 3,profile_hash,world_id FROM theia.quest_lifecycles WHERE realm_id=$1 AND character_id=$2",
            &[&self.realm.value(), &binding.character.value()],
        )).await?;
        let mut result = ProfileCompatibility::Compatible;
        for row in rows {
            let role = row.get::<_, i32>(0) as usize;
            let Some(expected) = profiles.hashes[role] else {
                continue;
            };
            let hash = row.get::<_, Vec<u8>>(1);
            let saved_world = row.get::<_, Option<Vec<u8>>>(2);
            if hash.len() != 32 || hash.iter().all(|v| *v == 0) {
                return Err(Error::Invalid("Invalid saved gameplay profile identity"));
            }
            if saved_world
                .as_ref()
                .is_some_and(|value| value.as_slice() != world)
            {
                return Err(Error::WorldMismatch);
            }
            if hash != expected {
                result = ProfileCompatibility::Incompatible;
            }
        }
        bounded(tx.commit()).await?;
        Ok(result)
    }
}
