use crate::{
    Binding, Character, CharacterId, CharacterName, CharacterSummary, Error,
    MAX_CHARACTERS_PER_OWNER, OwnerId, RealmId, Result, Saved, SavedCharacter,
};
use ring::digest::{SHA256, digest};
use std::{collections::BTreeSet, future::Future, net::IpAddr, str::FromStr, time::Duration};
use theia_core::coordinates::GlobalPosition;
use tokio::{task::JoinHandle, time::timeout};
use tokio_postgres::{Client, Config, NoTls, config::Host};
use uuid::Uuid;
mod inventory;
mod progression;
mod quests;
mod rewards;
mod vendors;

pub const MAX_BATCH: usize = 8;
const TIMEOUT: Duration = Duration::from_secs(3);
const MIGRATION_LOCK: i64 = -0x5448_4549_4100_0001;
const MIGRATIONS: [&str; 8] = [
    include_str!("../migrations/0001_characters.sql"),
    include_str!("../migrations/0002_development_bindings.sql"),
    include_str!("../migrations/0003_accounts.sql"),
    include_str!("../migrations/0004_character_names.sql"),
    include_str!("../migrations/0005_backpacks.sql"),
    include_str!("../migrations/0006_reward_claims.sql"),
    include_str!("../migrations/0007_progression.sql"),
    include_str!("../migrations/0008_quest_objectives.sql"),
];
async fn bounded<T>(
    future: impl Future<Output = std::result::Result<T, tokio_postgres::Error>>,
) -> Result<T> {
    timeout(TIMEOUT, future)
        .await
        .map_err(|_| Error::Timeout)?
        .map_err(Error::Database)
}
/// One private connection owns one realm advisory lock. No client clone or
/// automatic reconnection can silently resume without obtaining ownership again.
pub struct Store {
    client: Option<Client>,
    driver: Option<JoinHandle<std::result::Result<(), tokio_postgres::Error>>>,
    realm: RealmId,
    session: Uuid,
}
impl Drop for Store {
    fn drop(&mut self) {
        self.client.take();
        if let Some(driver) = self.driver.take() {
            driver.abort();
        }
    }
}
impl Store {
    /// Initial development boundary: NoTls is accepted only for one numeric
    /// loopback endpoint. Credentials come from caller-owned private environment.
    /// This method never formats the connection string or Config for diagnostics.
    pub async fn connect_loopback(connection: &str, realm: RealmId) -> Result<Self> {
        let mut config = Config::from_str(connection)?;
        if config.get_hosts().len() != 1
            || !matches!(&config.get_hosts()[0], Host::Tcp(host) if host.parse::<IpAddr>().is_ok_and(|ip| ip.is_loopback()))
            || config.get_hostaddrs().len() > 1
            || config.get_hostaddrs().iter().any(|ip| !ip.is_loopback())
        {
            return Err(Error::Invalid(
                "Development persistence requires one numeric loopback PostgreSQL host",
            ));
        }
        config.connect_timeout(TIMEOUT);
        let (client, connection) = bounded(config.connect(NoTls)).await?;
        let mut store = Self {
            client: Some(client),
            driver: Some(tokio::spawn(connection)),
            realm,
            session: Uuid::new_v4(),
        };
        bounded(store.client().batch_execute("SET application_name='theia-development-persistence'; SET statement_timeout='3000ms'; SET lock_timeout='500ms'; SET idle_in_transaction_session_timeout='5000ms'; SET synchronous_commit=on;")).await?;
        let flags = bounded(store.client().query_one("SELECT current_setting('fsync'),current_setting('full_page_writes'),current_setting('synchronous_commit')", &[])).await?;
        if (0..3).any(|i| flags.get::<_, String>(i) != "on") {
            return Err(Error::Durability);
        }
        store.migrate().await?;
        let hash = digest(&SHA256, realm.value().as_bytes());
        let key = i64::from_be_bytes(hash.as_ref()[..8].try_into().unwrap());
        let locked: bool = bounded(
            store
                .client()
                .query_one("SELECT pg_try_advisory_lock($1)", &[&key]),
        )
        .await?
        .get(0);
        if !locked {
            return Err(Error::RealmBusy);
        }
        Ok(store)
    }
    fn client(&self) -> &Client {
        self.client.as_ref().expect("Open store")
    }
    async fn migrate(&mut self) -> Result<()> {
        let transaction = bounded(self.client.as_mut().unwrap().transaction()).await?;
        bounded(transaction.query_one("SELECT pg_advisory_xact_lock($1)", &[&MIGRATION_LOCK]))
            .await?;
        bounded(transaction.batch_execute("CREATE SCHEMA IF NOT EXISTS theia; CREATE TABLE IF NOT EXISTS theia.schema_migrations(version INTEGER PRIMARY KEY CHECK(version>0), checksum BYTEA NOT NULL CHECK(octet_length(checksum)=32));")).await?;
        let limit = (MIGRATIONS.len() + 1) as i64;
        let rows = bounded(transaction.query(
            "SELECT version,checksum FROM theia.schema_migrations ORDER BY version LIMIT $1",
            &[&limit],
        ))
        .await?;
        if rows.len() > MIGRATIONS.len() {
            return Err(Error::Migration);
        }
        for (i, row) in rows.iter().enumerate() {
            let hash = digest(&SHA256, MIGRATIONS[i].as_bytes());
            if row.get::<_, i32>(0) != i as i32 + 1
                || row.get::<_, Vec<u8>>(1).as_slice() != hash.as_ref()
            {
                return Err(Error::Migration);
            }
        }
        for (i, sql) in MIGRATIONS.iter().enumerate().skip(rows.len()) {
            bounded(transaction.batch_execute(sql)).await?;
            let hash = digest(&SHA256, sql.as_bytes());
            let version = i as i32 + 1;
            bounded(transaction.execute(
                "INSERT INTO theia.schema_migrations(version,checksum) VALUES($1,$2)",
                &[&version, &hash.as_ref()],
            ))
            .await?;
        }
        bounded(transaction.commit()).await?;
        Ok(())
    }
    /// Trusted server use only: caller must supply an already verified OIDC
    /// issuer/subject. These are exact, case-sensitive identifiers, never email
    /// or client-supplied ownership. A returned owner mapping is committed.
    pub async fn resolve_owner(&mut self, issuer: &str, subject: &str) -> Result<OwnerId> {
        validate_identity(issuer, subject)?;
        let transaction = bounded(self.client.as_mut().unwrap().transaction()).await?;
        let proposed = OwnerId::random();
        bounded(transaction.execute("INSERT INTO theia.accounts(owner_id,issuer,subject) VALUES($1,$2,$3) ON CONFLICT(issuer,subject) DO NOTHING", &[&proposed.value(), &issuer, &subject])).await?;
        let row = bounded(transaction.query_one(
            "SELECT owner_id FROM theia.accounts WHERE issuer=$1 AND subject=$2",
            &[&issuer, &subject],
        ))
        .await?;
        let owner =
            OwnerId::new(row.get(0)).ok_or(Error::Invalid("Invalid persistent account owner"))?;
        bounded(transaction.commit()).await?;
        Ok(owner)
    }
    /// Account owner and supported spawn must be selected by the trusted server,
    /// never from client ownership/coordinate claims. Character + name are atomic.
    pub async fn create_character(
        &mut self,
        owner: OwnerId,
        world: [u8; 32],
        name: CharacterName,
        position: GlobalPosition,
    ) -> Result<Character> {
        if world == [0; 32] {
            return Err(Error::Invalid("World content identity must be nonzero"));
        }
        let realm = self.realm.value();
        let character = CharacterId::random();
        let writer = self.session;
        let transaction = bounded(self.client.as_mut().unwrap().transaction()).await?;
        if bounded(transaction.query_opt(
            "SELECT owner_id FROM theia.accounts WHERE owner_id=$1 FOR KEY SHARE",
            &[&owner.value()],
        ))
        .await?
        .is_none()
        {
            return Err(Error::AccountMissing);
        }
        let count: i64 = bounded(transaction.query_one(
            "SELECT count(*) FROM theia.characters WHERE realm_id=$1 AND owner_id=$2",
            &[&realm, &owner.value()],
        ))
        .await?
        .get(0);
        // One guarded realm owner/serialized worker owns initial creation. Future
        // multiple realm writers must introduce reviewed transaction-level quota
        // ownership before relaxing that guard.
        if count >= MAX_CHARACTERS_PER_OWNER as i64 {
            return Err(Error::CharacterLimit);
        }
        let [x, y, z] = position.metres();
        bounded(transaction.execute("INSERT INTO theia.characters(realm_id,character_id,owner_id,world_id,x,y,z,writer_session,checkpoint_sequence) VALUES($1,$2,$3,$4,$5,$6,$7,$8,0)", &[&realm, &character.value(), &owner.value(), &world.as_slice(), &x, &y, &z, &writer])).await?;
        let named=bounded(transaction.execute("INSERT INTO theia.character_names(realm_id,character_id,display_name) VALUES($1,$2,$3)", &[&realm, &character.value(), &name.as_str()])).await;
        if let Err(error) = named {
            if matches!(&error, Error::Database(db) if db.as_db_error().is_some_and(|db| db.code()==&tokio_postgres::error::SqlState::UNIQUE_VIOLATION && db.constraint()==Some("character_names_realm_name_unique")))
            {
                return Err(Error::NameUnavailable);
            }
            return Err(error);
        }
        bounded(transaction.commit()).await?;
        Ok(Character {
            saved: SavedCharacter {
                binding: Binding { character, owner },
                world,
                position,
                sequence: 0,
            },
            name,
        })
    }
    pub async fn list_characters(
        &self,
        owner: OwnerId,
        world: [u8; 32],
    ) -> Result<Vec<CharacterSummary>> {
        if world == [0; 32] {
            return Err(Error::Invalid("World content identity must be nonzero"));
        }
        if bounded(self.client().query_opt(
            "SELECT owner_id FROM theia.accounts WHERE owner_id=$1",
            &[&owner.value()],
        ))
        .await?
        .is_none()
        {
            return Err(Error::AccountMissing);
        }
        let limit = (MAX_CHARACTERS_PER_OWNER + 1) as i64;
        let rows=bounded(self.client().query(r#"SELECT c.character_id,c.world_id,n.display_name FROM theia.characters c LEFT JOIN theia.character_names n ON c.realm_id=n.realm_id AND c.character_id=n.character_id WHERE c.realm_id=$1 AND c.owner_id=$2 ORDER BY n.name_key COLLATE "C",c.character_id LIMIT $3"#, &[&self.realm.value(), &owner.value(), &limit])).await?;
        if rows.len() > MAX_CHARACTERS_PER_OWNER {
            return Err(Error::Invalid(
                "Character list exceeds bounded owner capacity",
            ));
        }
        let mut result = Vec::with_capacity(rows.len());
        for row in rows {
            if row.get::<_, Vec<u8>>(1).as_slice() != world {
                return Err(Error::WorldMismatch);
            }
            let name: String = row
                .get::<_, Option<String>>(2)
                .ok_or(Error::CharacterMissing)?;
            result.push(CharacterSummary {
                id: CharacterId::new(row.get(0))
                    .ok_or(Error::Invalid("Invalid persistent character identity"))?,
                name: CharacterName::parse(&name)?,
            });
        }
        Ok(result)
    }
    /// Binding owner must come from a valid account session. UUID lookup alone
    /// is never authorization. Selection neither issues a ticket nor claims an
    /// active character session; native collision/admission remain separate.
    pub async fn select_character(&self, binding: Binding, world: [u8; 32]) -> Result<Character> {
        if world == [0; 32] {
            return Err(Error::Invalid("World content identity must be nonzero"));
        }
        if bounded(self.client().query_opt(
            "SELECT owner_id FROM theia.accounts WHERE owner_id=$1",
            &[&binding.owner.value()],
        ))
        .await?
        .is_none()
        {
            return Err(Error::AccountMissing);
        }
        let saved = self
            .load(binding, world)
            .await?
            .ok_or(Error::CharacterMissing)?;
        let row = bounded(self.client().query_opt(
            "SELECT display_name FROM theia.character_names WHERE realm_id=$1 AND character_id=$2",
            &[&self.realm.value(), &binding.character.value()],
        ))
        .await?
        .ok_or(Error::CharacterMissing)?;
        let name: String = row.get(0);
        Ok(Character {
            saved,
            name: CharacterName::parse(&name)?,
        })
    }
    /// Bind a trusted local ticket slot to stable development identities. Never
    /// expose this as client-selected account/character authorization.
    pub async fn development_binding(&self, slot: u8) -> Result<Binding> {
        if !(1..=8).contains(&slot) {
            return Err(Error::Invalid("Development slot outside issuer capacity"));
        }
        let slot = i16::from(slot);
        let character = Uuid::new_v4();
        let owner = Uuid::new_v4();
        bounded(self.client().execute("INSERT INTO theia.development_bindings(realm_id,slot,character_id,owner_id) VALUES($1,$2,$3,$4) ON CONFLICT(realm_id,slot) DO NOTHING",&[&self.realm.value(),&slot,&character,&owner])).await?;
        let row=bounded(self.client().query_one("SELECT character_id,owner_id FROM theia.development_bindings WHERE realm_id=$1 AND slot=$2",&[&self.realm.value(),&slot])).await?;
        Ok(Binding {
            character: CharacterId::new(row.get(0))
                .ok_or(Error::Invalid("Invalid development character identity"))?,
            owner: OwnerId::new(row.get(1))
                .ok_or(Error::Invalid("Invalid development owner identity"))?,
        })
    }
    /// Create a trusted server-defined development binding, never a client claim.
    /// Existing owner/world content is checked and is never replaced by seeding.
    pub async fn seed(
        &mut self,
        binding: Binding,
        world: [u8; 32],
        position: GlobalPosition,
    ) -> Result<SavedCharacter> {
        if world == [0; 32] {
            return Err(Error::Invalid("World content identity must be nonzero"));
        }
        let [x, y, z] = position.metres();
        bounded(self.client().execute("INSERT INTO theia.characters(realm_id,character_id,owner_id,world_id,x,y,z,writer_session,checkpoint_sequence) VALUES($1,$2,$3,$4,$5,$6,$7,$8,0) ON CONFLICT(realm_id,character_id) DO NOTHING", &[&self.realm.value(),&binding.character.value(),&binding.owner.value(),&world.as_slice(),&x,&y,&z,&self.session])).await?;
        self.load(binding, world)
            .await?
            .ok_or(Error::Invalid("Seeded character missing"))
    }
    pub async fn load(&self, binding: Binding, world: [u8; 32]) -> Result<Option<SavedCharacter>> {
        let row = bounded(self.client().query_opt("SELECT owner_id,world_id,x,y,z,checkpoint_sequence FROM theia.characters WHERE realm_id=$1 AND character_id=$2", &[&self.realm.value(),&binding.character.value()])).await?;
        let Some(row) = row else {
            return Ok(None);
        };
        if row.get::<_, Uuid>(0) != binding.owner.value() {
            return Err(Error::Ownership);
        }
        if row.get::<_, Vec<u8>>(1).as_slice() != world {
            return Err(Error::WorldMismatch);
        }
        let position = GlobalPosition::new([row.get(2), row.get(3), row.get(4)])
            .ok_or(Error::Invalid("Invalid persisted global position"))?;
        let sequence = u64::try_from(row.get::<_, i64>(5))
            .map_err(|_| Error::Invalid("Invalid persisted checkpoint sequence"))?;
        Ok(Some(SavedCharacter {
            binding,
            world,
            position,
            sequence,
        }))
    }
    /// Atomic batch, bounded to current eight-peer authority. Character checkpoint
    /// sequences remain monotonic across restarts and are distinct from simulation ticks.
    /// A saved result means COMMIT completed, never merely that a job was queued.
    pub async fn checkpoint(&mut self, records: &[SavedCharacter]) -> Result<Vec<Saved>> {
        if records.is_empty() || records.len() > MAX_BATCH {
            return Err(Error::Invalid("Checkpoint batch outside bounded capacity"));
        }
        let mut ids = BTreeSet::new();
        for record in records {
            if !ids.insert(record.binding.character)
                || record.sequence == 0
                || record.sequence > i64::MAX as u64
                || record.world == [0; 32]
            {
                return Err(Error::Invalid("Invalid or duplicate checkpoint record"));
            }
        }
        let realm = self.realm.value();
        let session = self.session;
        let transaction = bounded(self.client.as_mut().unwrap().transaction()).await?;
        let mut outcomes = Vec::with_capacity(records.len());
        for record in records {
            let character = record.binding.character.value();
            let row = bounded(transaction.query_opt("SELECT owner_id,world_id FROM theia.characters WHERE realm_id=$1 AND character_id=$2 FOR UPDATE", &[&realm,&character])).await?.ok_or(Error::Invalid("Unknown persistent character"))?;
            if row.get::<_, Uuid>(0) != record.binding.owner.value() {
                return Err(Error::Ownership);
            }
            if row.get::<_, Vec<u8>>(1).as_slice() != record.world {
                return Err(Error::WorldMismatch);
            }
            let [x, y, z] = record.position.metres();
            let sequence = record.sequence as i64;
            let rows = bounded(transaction.execute("UPDATE theia.characters SET x=$1,y=$2,z=$3,writer_session=$4,checkpoint_sequence=$5,updated_at=clock_timestamp() WHERE realm_id=$6 AND character_id=$7 AND checkpoint_sequence<$5", &[&x,&y,&z,&session,&sequence,&realm,&character])).await?;
            outcomes.push(if rows == 1 {
                Saved::Committed
            } else {
                Saved::Stale
            });
        }
        bounded(transaction.commit()).await?;
        Ok(outcomes)
    }
    pub(crate) async fn connection_stopped(&mut self) -> Error {
        match self.driver.as_mut().unwrap().await {
            Ok(Err(error)) => Error::Database(error),
            Ok(Ok(())) => Error::Invalid("PostgreSQL connection closed unexpectedly"),
            Err(_) => Error::Invalid("PostgreSQL driver stopped unexpectedly"),
        }
    }
    pub async fn close(mut self) -> Result<()> {
        self.client.take();
        if let Some(mut driver) = self.driver.take() {
            match timeout(TIMEOUT, &mut driver).await {
                Ok(Ok(result)) => result.map_err(Error::Database)?,
                Ok(Err(_)) => return Err(Error::Invalid("PostgreSQL driver stopped unexpectedly")),
                Err(_) => {
                    driver.abort();
                    let _ = driver.await;
                    return Err(Error::Timeout);
                }
            }
        }
        Ok(())
    }
}
pub(crate) fn validate_identity(issuer: &str, subject: &str) -> Result<()> {
    if issuer.is_empty()
        || issuer.len() > 2048
        || subject.is_empty()
        || subject.len() > 255
        || issuer.chars().any(char::is_control)
        || subject.chars().any(char::is_control)
    {
        return Err(Error::Invalid("Invalid bounded external account identity"));
    }
    Ok(())
}
