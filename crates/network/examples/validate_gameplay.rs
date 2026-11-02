//! Offline schema checks used by the exact-byte gameplay launcher. Validation
//! does not itself establish an admitted World identity.
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    io::Read,
    path::{Path, PathBuf},
};
use theia_core::{content::ContentId, dialogue, inventory, progression, quests, rewards, vendors};
use theia_network::{
    Result,
    simulation::{combat, creatures},
    terrain::sha256,
};
const MAX_MANIFEST_BYTES: usize = 8192;
const MAX_PATH_BYTES: usize = 256;
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Manifest {
    schema: u32,
    id: String,
    revision: u32,
    items: String,
    abilities: String,
    stats: String,
    statuses: String,
    creature: String,
    dialogue: String,
    vendor: String,
    rewards: String,
    progression: String,
    quest: String,
    #[serde(default, deserialize_with = "optional_profile")]
    quest_turn_in_xp: Option<String>,
    #[serde(default, deserialize_with = "optional_profile")]
    creature_retaliation: Option<String>,
    #[serde(default, deserialize_with = "optional_profile")]
    player_respawn: Option<String>,
    #[serde(default, deserialize_with = "optional_profile")]
    dodge: Option<String>,
    #[serde(default, deserialize_with = "optional_profile")]
    melee_weapons: Option<String>,
    #[serde(default, deserialize_with = "optional_profile")]
    physical_definitions: Option<String>,
    #[serde(default, deserialize_with = "optional_profile")]
    physical_stats: Option<String>,
    #[serde(default, deserialize_with = "optional_profile")]
    white_melee: Option<String>,
    #[serde(default, deserialize_with = "optional_profile")]
    ability_starting: Option<String>,
    #[serde(default, deserialize_with = "optional_profile")]
    ability_damage: Option<String>,
    #[serde(default, deserialize_with = "optional_profile")]
    combat_membership: Option<String>,
    #[serde(default, deserialize_with = "optional_profile")]
    creature_credit: Option<String>,
    #[serde(default, deserialize_with = "optional_profile")]
    creature_reset: Option<String>,
}
fn optional_profile<'de, D: serde::Deserializer<'de>>(
    d: D,
) -> std::result::Result<Option<String>, D::Error> {
    String::deserialize(d).map(Some)
}
impl Manifest {
    fn roles(&self) -> Vec<(&'static str, &str, usize)> {
        let mut roles: Vec<(&'static str, &str, usize)> = vec![
            ("items", &self.items, inventory::MAX_CATALOG_BYTES),
            (
                "abilities",
                &self.abilities,
                theia_core::abilities::MAX_CATALOG_BYTES,
            ),
            (
                "stats",
                &self.stats,
                theia_core::resources::MAX_PROFILE_BYTES,
            ),
            (
                "statuses",
                &self.statuses,
                theia_core::statuses::MAX_CATALOG_BYTES,
            ),
            ("creature", &self.creature, creatures::MAX_PROFILE_BYTES),
            ("dialogue", &self.dialogue, dialogue::MAX_PROFILE_BYTES),
            ("vendor", &self.vendor, vendors::MAX_CATALOG_BYTES),
            ("rewards", &self.rewards, rewards::MAX_CATALOG_BYTES),
            (
                "progression",
                &self.progression,
                progression::MAX_PROFILE_BYTES,
            ),
            ("quest", &self.quest, quests::MAX_PROFILE_BYTES),
        ];
        if let Some(path) = &self.quest_turn_in_xp {
            roles.push((
                "quest_turn_in_xp",
                path,
                quests::turn_in_xp::MAX_PROFILE_BYTES,
            ));
        }
        if let Some(path) = &self.creature_retaliation {
            roles.push((
                "creature_retaliation",
                path,
                theia_core::retaliation::MAX_PROFILE_BYTES,
            ));
        }
        if let Some(path) = &self.player_respawn {
            roles.push((
                "player_respawn",
                path,
                theia_core::player_life::MAX_PROFILE_BYTES,
            ));
        }
        if let Some(path) = &self.dodge {
            roles.push(("dodge", path, theia_core::dodge::MAX_PROFILE_BYTES));
        }
        if let Some(path) = &self.melee_weapons {
            roles.push(("melee_weapons", path, theia_core::melee::MAX_PROFILE_BYTES));
        }
        if let Some(path) = &self.physical_definitions {
            roles.push((
                "physical_definitions",
                path,
                theia_core::damage::catalog::MAX_PROFILE_BYTES,
            ));
        }
        if let Some(path) = &self.physical_stats {
            roles.push((
                "physical_stats",
                path,
                theia_core::damage::stats::MAX_PROFILE_BYTES,
            ));
        }
        if let Some(path) = &self.white_melee {
            roles.push(("white_melee", path, combat::white_melee::MAX_PROFILE_BYTES));
        }
        if let Some(path) = &self.ability_starting {
            roles.push((
                "ability_starting",
                path,
                theia_core::damage::ability_starting::catalog::MAX_PROFILE_BYTES,
            ));
        }
        if let Some(path) = &self.ability_damage {
            roles.push((
                "ability_damage",
                path,
                combat::ability_damage::MAX_PROFILE_BYTES,
            ));
        }
        if let Some(path) = &self.combat_membership {
            roles.push((
                "combat_membership",
                path,
                theia_core::combat_membership::MAX_PROFILE_BYTES,
            ));
        }
        if let Some(path) = &self.creature_credit {
            roles.push((
                "creature_credit",
                path,
                theia_core::creature_credit::MAX_PROFILE_BYTES,
            ));
        }
        if let Some(path) = &self.creature_reset {
            roles.push(("creature_reset", path, combat::reset::MAX_PROFILE_BYTES));
        }
        roles
    }
}
#[derive(Serialize, Debug, PartialEq, Eq)]
struct FileRecord {
    kind: String,
    path: String,
    bytes: usize,
    sha256: String,
}
#[derive(Serialize, Debug, PartialEq, Eq)]
struct Report {
    validated: bool,
    scope: &'static str,
    schema: u32,
    manifest_schema: u32,
    id: String,
    revision: u32,
    manifest_sha256: String,
    validation_bundle_sha256: String,
    profiles: Vec<FileRecord>,
}
fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}
fn bounded(path: &Path, limit: usize) -> Result<Vec<u8>> {
    if !fs::metadata(path)
        .map_err(|_| "Content file unavailable")?
        .is_file()
    {
        return Err("Content input must be a regular file".into());
    }
    let mut bytes = Vec::new();
    fs::File::open(path)
        .map_err(|_| "Content file unavailable")?
        .take(limit as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| "Content read failed")?;
    if bytes.len() > limit {
        return Err("Content exceeds byte budget".into());
    }
    Ok(bytes)
}
fn relative(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_PATH_BYTES
        && value.ends_with(".json")
        && value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"_./-".contains(&b))
        && value
            .split('/')
            .all(|part| !part.is_empty() && !matches!(part, "." | ".."))
        && !Path::new(value).is_absolute()
}
fn check(bytes: &BTreeMap<&str, Vec<u8>>) -> Result<()> {
    let items =
        inventory::Catalog::from_json(&bytes["items"]).map_err(|_| "Invalid items profile")?;
    let abilities = theia_core::abilities::Catalog::from_json(&bytes["abilities"])?;
    abilities.validate_equipment(Some(&items))?;
    combat::status_profile_identity(
        [0; 32],
        &bytes["abilities"],
        combat::DEVELOPMENT_HEALTH,
        combat::DEVELOPMENT_RESPAWN_MS,
        &bytes["stats"],
        &bytes["statuses"],
    )
    .map_err(|_| "Invalid ability/resource/status configuration")?;
    let guide =
        dialogue::Profile::from_json(&bytes["dialogue"]).map_err(|_| "Invalid dialogue profile")?;
    let vendor = vendors::Catalog::from_json(&bytes["vendor"], &items)
        .map_err(|_| "Invalid vendor/item configuration")?;
    if vendor.npc() != guide.npc() {
        return Err("Vendor NPC differs from declared dialogue guide".into());
    }
    let creature = creatures::Profile::from_json(&bytes["creature"])
        .map_err(|_| "Invalid creature profile")?;
    if creature.aggro().is_some()
        && (!bytes.contains_key("creature_retaliation") || !bytes.contains_key("player_respawn"))
    {
        return Err("Pursuit requires player life and retaliation profiles".into());
    }
    let rewards = rewards::Catalog::from_json(&bytes["rewards"], &items)
        .map_err(|_| "Invalid reward/item configuration")?;
    if rewards.definitions().any(|r| r.creature() != creature.id()) {
        return Err("Reward references an undeclared creature".into());
    }
    let progression = progression::Profile::from_json(&bytes["progression"])
        .map_err(|_| "Invalid progression profile")?;
    if progression
        .kill_rewards()
        .any(|(id, _)| id != creature.id())
    {
        return Err("Progression references an undeclared creature".into());
    }
    let quest = quests::Profile::from_json(&bytes["quest"]).map_err(|_| "Invalid quest profile")?;
    if quest.creature() != creature.id() {
        return Err("Quest references an undeclared creature".into());
    }
    if let Some(recipe) = bytes.get("quest_turn_in_xp") {
        quests::turn_in_xp::Recipe::from_json(recipe, &quest, &progression)
            .map_err(|_| "Invalid completion XP/quest/progression configuration")?;
    }
    if let (Some(policy), Some(respawn)) = (
        bytes.get("creature_retaliation"),
        bytes.get("player_respawn"),
    ) {
        theia_network::player_life_wire::Profile::from_json(
            &bytes["stats"],
            &bytes["creature"],
            policy,
            respawn,
        )
        .map_err(|_| "Invalid player life/retaliation configuration")?;
    }
    if let Some(weapons) = bytes.get("melee_weapons") {
        theia_core::melee::Catalog::from_json(weapons)?
            .validate_items(&inventory::Catalog::from_json(&bytes["items"])?)?;
    }
    if let (Some(definitions), Some(stats)) = (
        bytes.get("physical_definitions"),
        bytes.get("physical_stats"),
    ) {
        let weapons = theia_core::melee::Catalog::from_json(
            bytes
                .get("melee_weapons")
                .ok_or("Physical damage requires melee")?,
        )?;
        let vitals = theia_core::resources::Profile::from_json(&bytes["stats"])?;
        let physical = combat::physical::Profile::new(
            definitions,
            stats,
            combat::physical::Bindings {
                weapons: &weapons,
                items: &items,
                abilities: &abilities,
                creature: &creature,
                player_maximum_health: vitals.maximum_health(),
                creature_maximum_health: combat::DEVELOPMENT_HEALTH,
            },
        )
        .map_err(|_| "Invalid physical damage/stat configuration")?;
        if let Some(white_bytes) = bytes.get("white_melee") {
            if creature.aggro().is_none() {
                return Err("Ordinary outcomes require pursuit".into());
            }
            let profile =
                combat::white_melee::Profile::new(white_bytes, std::sync::Arc::new(physical))
                    .map_err(|_| "Invalid ordinary-melee configuration")?;
            profile.check_supported_abilities(&abilities)?;
            if let (Some(starting), Some(policy)) =
                (bytes.get("ability_starting"), bytes.get("ability_damage"))
            {
                combat::ability_damage::Profile::new(
                    starting,
                    policy,
                    &bytes["abilities"],
                    std::sync::Arc::new(profile),
                )
                .map_err(|_| "Invalid ability damage/recipe configuration")?;
            }
        }
    }
    if let Some(policy) = bytes.get("combat_membership") {
        theia_network::combat_membership_wire::Profile::from_json(policy)?;
        if abilities
            .abilities()
            .iter()
            .any(|a| !matches!(a.effect(), theia_core::abilities::Effect::Damage { .. }))
        {
            return Err("Membership requires typed damage-only abilities".into());
        }
    }
    if let Some(policy) = bytes.get("creature_credit") {
        let profile = theia_core::creature_credit::Profile::from_json(policy)?;
        if profile.recipient() != Some(theia_core::creature_credit::RecipientPolicy::SoloTagOwner) {
            return Err("Normal credit requires explicit solo tag-owner recipient policy".into());
        }
    }
    if let Some(policy) = bytes.get("creature_reset") {
        let profile = combat::reset::Profile::from_json(policy)?;
        if profile.creature() != creature.id() || creature.aggro().is_none() {
            return Err("Creature reset requires the matching pursuit creature".into());
        }
    }
    if let Some(policy) = bytes.get("dodge") {
        theia_network::dodging::Profile::from_json(policy).map_err(|_| "Invalid dodge profile")?;
    }
    Ok(())
}
fn validate(path: &Path) -> Result<Report> {
    let path = path.canonicalize().map_err(|_| "Manifest unavailable")?;
    let raw = bounded(&path, MAX_MANIFEST_BYTES)?;
    let manifest: Manifest =
        serde_json::from_slice(&raw).map_err(|_| "Invalid gameplay manifest")?;
    if !matches!(manifest.schema, 1..=11)
        || manifest.revision == 0
        || ContentId::new(&manifest.id).is_none()
        || (manifest.schema >= 2) != manifest.quest_turn_in_xp.is_some()
        || (manifest.schema >= 3) != manifest.creature_retaliation.is_some()
        || (manifest.schema >= 3) != manifest.player_respawn.is_some()
        || (manifest.schema == 4) != manifest.dodge.is_some()
        || (manifest.schema >= 5) != manifest.melee_weapons.is_some()
        || (manifest.schema >= 6) != manifest.physical_definitions.is_some()
        || (manifest.schema >= 6) != manifest.physical_stats.is_some()
        || (manifest.schema >= 7) != manifest.white_melee.is_some()
        || (manifest.schema >= 8) != manifest.ability_starting.is_some()
        || (manifest.schema >= 8) != manifest.ability_damage.is_some()
        || (manifest.schema >= 9) != manifest.combat_membership.is_some()
        || (manifest.schema >= 10) != manifest.creature_credit.is_some()
        || (manifest.schema == 11) != manifest.creature_reset.is_some()
    {
        return Err("Invalid gameplay manifest identity/version".into());
    }
    let root = path
        .parent()
        .ok_or("Manifest requires a content directory")?;
    let mut seen = BTreeSet::from([path.clone()]);
    let mut bytes = BTreeMap::new();
    let mut profiles = Vec::new();
    for (kind, name, limit) in manifest.roles() {
        if !relative(name) {
            return Err(format!("Unsafe {kind} profile path").into());
        }
        let path = root
            .join(name)
            .canonicalize()
            .map_err(|_| format!("Missing {kind} profile"))?;
        if !path.starts_with(root) || !seen.insert(path.clone()) {
            return Err(format!("Outside-root or aliased {kind} profile path").into());
        }
        let data = bounded(&path, limit)
            .map_err(|_| format!("Unreadable, nonregular or oversized {kind} profile"))?;
        profiles.push(FileRecord {
            kind: kind.into(),
            path: name.into(),
            bytes: data.len(),
            sha256: hex(&sha256(&data)),
        });
        bytes.insert(kind, data);
    }
    check(&bytes)?;
    profiles.sort_by(|a, b| a.kind.cmp(&b.kind));
    let manifest_hash = sha256(&raw);
    let mut identity = b"theia-gameplay-validation-bundle-v1\0".to_vec();
    identity.extend(manifest_hash);
    for record in &profiles {
        identity.push(record.kind.len() as u8);
        identity.extend(record.kind.as_bytes());
        identity.extend(sha256(&bytes[record.kind.as_str()]));
    }
    Ok(Report {
        validated: true,
        scope: "Offline profile and declared-reference validation",
        schema: 1,
        manifest_schema: manifest.schema,
        id: manifest.id,
        revision: manifest.revision,
        manifest_sha256: hex(&manifest_hash),
        validation_bundle_sha256: hex(&sha256(&identity)),
        profiles,
    })
}
fn main() -> Result<()> {
    let mut args = std::env::args_os().skip(1);
    let path = PathBuf::from(args.next().ok_or("Expected gameplay manifest JSON path")?);
    if args.next().is_some() {
        return Err("Unexpected gameplay validator arguments".into());
    }
    println!("{}", serde_json::to_string_pretty(&validate(&path)?)?);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Value, json};
    use std::sync::atomic::{AtomicU64, Ordering};
    static NEXT: AtomicU64 = AtomicU64::new(0);
    struct Fixture(PathBuf);
    impl Fixture {
        fn new() -> Self {
            let root = std::env::temp_dir().join(format!(
                "theia-gameplay-check-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir(&root).unwrap();
            let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../content/gameplay");
            let data = include_bytes!("../../../content/gameplay/development-gameplay.json");
            let manifest: Manifest = serde_json::from_slice(data).unwrap();
            fs::write(root.join("bundle.json"), data).unwrap();
            for (_, path, _) in manifest.roles() {
                fs::copy(source.join(path), root.join(path)).unwrap();
            }
            Self(root)
        }
        fn manifest(&self) -> PathBuf {
            self.0.join("bundle.json")
        }
        fn path(&self, role: &str) -> PathBuf {
            let manifest: Value =
                serde_json::from_slice(&fs::read(self.manifest()).unwrap()).unwrap();
            self.0.join(manifest[role].as_str().unwrap())
        }
        fn edit(&self, role: &str, change: impl FnOnce(&mut Value)) {
            let path = if role == "manifest" {
                self.manifest()
            } else {
                self.path(role)
            };
            let mut data: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
            change(&mut data);
            fs::write(path, serde_json::to_vec_pretty(&data).unwrap()).unwrap();
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            fs::remove_dir_all(&self.0).unwrap();
        }
    }
    #[test]
    fn valid_development_bundle_reports_deterministically_without_changing_inputs() {
        let f = Fixture::new();
        let before: BTreeMap<_, _> = fs::read_dir(&f.0)
            .unwrap()
            .map(|entry| {
                let path = entry.unwrap().path();
                let hash = sha256(&fs::read(&path).unwrap());
                (path, hash)
            })
            .collect();
        let first = validate(&f.manifest()).unwrap();
        assert_eq!(first, validate(&f.manifest()).unwrap());
        assert_eq!(first.id, "theia.dev.gameplay");
        assert_eq!(first.profiles.len(), 10);
        assert!(
            first
                .profiles
                .windows(2)
                .all(|pair| pair[0].kind < pair[1].kind)
        );
        for (path, hash) in before {
            assert_eq!(sha256(&fs::read(path).unwrap()), hash);
        }
    }
    #[test]
    fn exact_manifest_and_profile_bytes_change_the_validation_digest() {
        let f = Fixture::new();
        let initial = validate(&f.manifest()).unwrap();
        let mut bytes = fs::read(f.manifest()).unwrap();
        bytes.push(b' ');
        fs::write(f.manifest(), bytes).unwrap();
        let changed_manifest = validate(&f.manifest()).unwrap();
        assert_ne!(initial.manifest_sha256, changed_manifest.manifest_sha256);
        assert_ne!(
            initial.validation_bundle_sha256,
            changed_manifest.validation_bundle_sha256
        );
        assert_eq!(initial.profiles, changed_manifest.profiles);
        let path = f.path("vendor");
        let mut bytes = fs::read(&path).unwrap();
        bytes.push(b' ');
        fs::write(path, bytes).unwrap();
        let changed_profile = validate(&f.manifest()).unwrap();
        assert_eq!(
            changed_profile.manifest_sha256,
            changed_manifest.manifest_sha256
        );
        assert_ne!(
            changed_profile.validation_bundle_sha256,
            changed_manifest.validation_bundle_sha256
        );
    }
    #[test]
    fn manifests_reject_unknown_duplicate_missing_and_invalid_identity_fields() {
        for (field, value) in [
            ("schema", json!(2)),
            ("revision", json!(0)),
            ("revision", json!(-1)),
            ("revision", json!(4294967296_u64)),
            ("id", json!("bad id")),
            ("unsupported", json!("DO-NOT-ECHO")),
        ] {
            let f = Fixture::new();
            f.edit("manifest", |m| m[field] = value);
            let error = validate(&f.manifest()).unwrap_err().to_string();
            assert!(!error.contains("DO-NOT-ECHO"));
        }
        let f = Fixture::new();
        let raw = fs::read_to_string(f.manifest()).unwrap();
        let duplicate = format!(
            "{},\"schema\":1}}",
            raw.trim_end().strip_suffix('}').unwrap()
        );
        fs::write(f.manifest(), duplicate).unwrap();
        assert!(validate(&f.manifest()).is_err());
        let f = Fixture::new();
        f.edit("manifest", |m| {
            m.as_object_mut().unwrap().remove("quest");
        });
        assert!(validate(&f.manifest()).is_err());
    }
    #[test]
    fn profile_paths_must_be_bounded_portable_relative_json_files() {
        for path in [
            "/tmp/outside.json".into(),
            "../outside.json".into(),
            "a/../outside.json".into(),
            "a//b.json".into(),
            "./file.json".into(),
            "a/./file.json".into(),
            "C:/file.json".into(),
            "a\\file.json".into(),
            "".into(),
            "file.txt".into(),
            "line\nfile.json".into(),
            format!("{}.json", "a".repeat(252)),
        ] {
            let f = Fixture::new();
            f.edit("manifest", |m| m["vendor"] = json!(path));
            assert!(
                validate(&f.manifest())
                    .unwrap_err()
                    .to_string()
                    .contains("Unsafe vendor")
            );
        }
    }
    #[test]
    fn missing_nonregular_malformed_and_oversized_inputs_are_refused() {
        let f = Fixture::new();
        fs::remove_file(f.path("vendor")).unwrap();
        assert!(validate(&f.manifest()).is_err());
        let f = Fixture::new();
        let path = f.path("vendor");
        fs::remove_file(&path).unwrap();
        fs::create_dir(path).unwrap();
        assert!(validate(&f.manifest()).is_err());
        let f = Fixture::new();
        fs::write(f.path("items"), b"{\"DO-NOT-ECHO\":\"private input\"}").unwrap();
        let error = validate(&f.manifest()).unwrap_err().to_string();
        assert_eq!(error, "Invalid items profile");
        let f = Fixture::new();
        fs::write(
            f.path("abilities"),
            vec![b' '; theia_core::abilities::MAX_CATALOG_BYTES + 1],
        )
        .unwrap();
        assert!(validate(&f.manifest()).is_err());
        let f = Fixture::new();
        fs::write(f.manifest(), vec![b' '; MAX_MANIFEST_BYTES + 1]).unwrap();
        assert!(validate(&f.manifest()).is_err());
    }
    #[test]
    fn cross_profile_npc_items_and_creature_references_are_checked() {
        let cases: [(&str, fn(&mut Value)); 6] = [
            ("vendor", |v| v["npc"] = json!("theia.dev.other-guide")),
            ("vendor", |v| {
                v["item_catalog"] = json!("theia.dev.other-items")
            }),
            ("rewards", |v| {
                v["item_catalog"] = json!("theia.dev.other-items")
            }),
            ("rewards", |v| {
                v["rewards"][0]["creature"] = json!("theia.dev.other-creature")
            }),
            ("progression", |v| {
                v["kill_rewards"][0]["creature"] = json!("theia.dev.other-creature")
            }),
            ("quest", |v| {
                v["objective"]["creature"] = json!("theia.dev.other-creature")
            }),
        ];
        for (role, change) in cases {
            let f = Fixture::new();
            f.edit(role, change);
            assert!(validate(&f.manifest()).is_err(), "{role}");
        }
    }
    #[test]
    fn ability_resource_and_status_checks_use_the_runtime_configuration() {
        let f = Fixture::new();
        f.edit("stats", |v| v["maximum_mana"] = json!(1));
        assert_eq!(
            validate(&f.manifest()).unwrap_err().to_string(),
            "Invalid ability/resource/status configuration"
        );
        let f = Fixture::new();
        f.edit("abilities", |v| {
            v["abilities"][0]["effect"]["status"] = json!("theia.dev.unknown-status")
        });
        assert_eq!(
            validate(&f.manifest()).unwrap_err().to_string(),
            "Invalid ability/resource/status configuration"
        );
    }
    #[cfg(unix)]
    #[test]
    fn canonical_paths_reject_outside_symlinks_and_internal_aliases() {
        use std::os::unix::fs::symlink;
        let outside = Fixture::new();
        let f = Fixture::new();
        let path = f.path("vendor");
        fs::remove_file(&path).unwrap();
        symlink(outside.path("vendor"), &path).unwrap();
        assert!(
            validate(&f.manifest())
                .unwrap_err()
                .to_string()
                .contains("Outside-root")
        );
        let f = Fixture::new();
        let path = f.path("vendor");
        fs::remove_file(&path).unwrap();
        symlink(f.path("items"), &path).unwrap();
        assert!(
            validate(&f.manifest())
                .unwrap_err()
                .to_string()
                .contains("aliased")
        );
    }
    fn rewarded_fixture() -> Fixture {
        let f = Fixture::new();
        fs::copy(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../content/gameplay/development-quest-turn-in-xp.json"),
            f.0.join("completion.json"),
        )
        .unwrap();
        f.edit("manifest", |m| {
            m["schema"] = json!(2);
            m["quest_turn_in_xp"] = json!("completion.json");
        });
        f
    }
    #[test]
    fn schema_two_requires_a_checked_completion_recipe_and_hashes_all_eleven_roles() {
        let f = rewarded_fixture();
        let first = validate(&f.manifest()).unwrap();
        assert_eq!(first.schema, 1);
        assert_eq!(first.manifest_schema, 2);
        assert_eq!(first.profiles.len(), 11);
        assert!(first.profiles.iter().any(|r| r.kind == "quest_turn_in_xp"));
        f.edit("quest_turn_in_xp", |r| r["experience"] = json!(26));
        let next = validate(&f.manifest()).unwrap();
        assert_eq!(first.manifest_sha256, next.manifest_sha256);
        assert_ne!(
            first.validation_bundle_sha256,
            next.validation_bundle_sha256
        );
        let f = Fixture::new();
        f.edit("manifest", |m| m["schema"] = json!(2));
        assert!(validate(&f.manifest()).is_err());
        let f = rewarded_fixture();
        f.edit("manifest", |m| m["schema"] = json!(1));
        assert!(validate(&f.manifest()).is_err());
        let f = rewarded_fixture();
        f.edit("manifest", |m| m["quest_turn_in_xp"] = Value::Null);
        assert!(validate(&f.manifest()).is_err());
    }
    #[test]
    fn completion_recipe_paths_budgets_and_references_use_the_same_strict_checks() {
        for (key, value) in [
            ("quest", json!("other.quest")),
            ("progression", json!("other.progression")),
            ("experience", json!(0)),
        ] {
            let f = rewarded_fixture();
            f.edit("quest_turn_in_xp", |r| r[key] = value);
            assert!(validate(&f.manifest()).is_err());
        }
        let f = rewarded_fixture();
        f.edit("manifest", |m| {
            m["quest_turn_in_xp"] = json!("../escape.json")
        });
        assert!(validate(&f.manifest()).is_err());
        let f = rewarded_fixture();
        f.edit("manifest", |m| m["quest_turn_in_xp"] = m["quest"].clone());
        assert!(validate(&f.manifest()).is_err());
        let f = rewarded_fixture();
        fs::write(
            f.path("quest_turn_in_xp"),
            vec![b' '; quests::turn_in_xp::MAX_PROFILE_BYTES + 1],
        )
        .unwrap();
        assert!(validate(&f.manifest()).is_err());
    }

    fn life_fixture() -> Fixture {
        let f = rewarded_fixture();
        for (role, name) in [
            (
                "creature_retaliation",
                "development-creature-retaliation.json",
            ),
            ("player_respawn", "development-player-respawn.json"),
        ] {
            fs::copy(
                Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("../../content/gameplay")
                    .join(name),
                f.0.join(name),
            )
            .unwrap();
            f.edit("manifest", |m| {
                m["schema"] = json!(3);
                m[role] = json!(name);
            });
        }
        f
    }
    fn melee_fixture() -> Fixture {
        let f = Fixture::new();
        let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../content/gameplay");
        let raw = fs::read(source.join("development-gameplay-classic-melee.json")).unwrap();
        let manifest: Manifest = serde_json::from_slice(&raw).unwrap();
        fs::write(f.manifest(), raw).unwrap();
        for (_, name, _) in manifest.roles() {
            fs::copy(source.join(name), f.0.join(name)).unwrap();
        }
        f
    }
    #[test]
    fn melee_bundle_binds_weapon_bytes_and_requires_exclusive_schema() {
        let f = melee_fixture();
        let first = validate(&f.manifest()).unwrap();
        assert_eq!(first.manifest_schema, 5);
        assert_eq!(first.profiles.len(), 14);
        let path = f.path("melee_weapons");
        let mut data = fs::read(&path).unwrap();
        data.push(b' ');
        fs::write(&path, data).unwrap();
        assert_ne!(
            first.validation_bundle_sha256,
            validate(&f.manifest()).unwrap().validation_bundle_sha256
        );
        for value in [
            Value::Null,
            json!("development-character-stats.json"),
            json!("../escape.json"),
        ] {
            let f = melee_fixture();
            f.edit("manifest", |m| m["melee_weapons"] = value);
            assert!(validate(&f.manifest()).is_err());
        }
        let f = melee_fixture();
        f.edit("manifest", |m| {
            m.as_object_mut().unwrap().remove("melee_weapons");
        });
        assert!(validate(&f.manifest()).is_err());
        let f = melee_fixture();
        f.edit("manifest", |m| m["schema"] = json!(3));
        assert!(validate(&f.manifest()).is_err());
        let f = melee_fixture();
        f.edit("manifest", |m| m["dodge"] = json!("development-dodge.json"));
        assert!(validate(&f.manifest()).is_err());
    }
    #[test]
    fn melee_bundle_rejects_oversized_and_foreign_equipment_rules() {
        let f = melee_fixture();
        fs::write(
            f.path("melee_weapons"),
            vec![b' '; theia_core::melee::MAX_PROFILE_BYTES + 1],
        )
        .unwrap();
        assert!(validate(&f.manifest()).is_err());
        let f = melee_fixture();
        f.edit("melee_weapons", |w| {
            w["weapons"][0]["item"] = json!("theia.dev.training-stone")
        });
        assert!(validate(&f.manifest()).is_err());
    }
    #[test]
    fn schema_four_binds_exact_dodge_bytes_and_rejects_missing_null_alias_and_oversized_policy() {
        let source = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../content/gameplay/development-gameplay-dodge.json");
        let report = validate(&source).unwrap();
        assert_eq!(report.manifest_schema, 4);
        assert_eq!(report.profiles.len(), 14);
        for value in [
            Value::Null,
            json!("development-character-stats.json"),
            json!("../escape.json"),
        ] {
            let f = life_fixture();
            f.edit("manifest", |m| {
                m["schema"] = json!(4);
                m["dodge"] = value;
            });
            assert!(validate(&f.manifest()).is_err());
        }
        let f = life_fixture();
        f.edit("manifest", |m| m["schema"] = json!(4));
        assert!(validate(&f.manifest()).is_err());
        fs::write(
            f.0.join("dodge.json"),
            include_bytes!("../../../content/gameplay/development-dodge.json"),
        )
        .unwrap();
        f.edit("manifest", |m| m["dodge"] = json!("dodge.json"));
        let first = validate(&f.manifest()).unwrap();
        let mut bytes = fs::read(f.0.join("dodge.json")).unwrap();
        bytes.push(b' ');
        fs::write(f.0.join("dodge.json"), bytes).unwrap();
        assert_ne!(
            first.validation_bundle_sha256,
            validate(&f.manifest()).unwrap().validation_bundle_sha256
        );
        f.edit("manifest", |m| m["schema"] = json!(3));
        assert!(validate(&f.manifest()).is_err());
        f.edit("manifest", |m| m["schema"] = json!(4));
        fs::write(
            f.0.join("dodge.json"),
            vec![b' '; theia_core::dodge::MAX_PROFILE_BYTES + 1],
        )
        .unwrap();
        assert!(validate(&f.manifest()).is_err());
    }
    #[test]
    fn schema_three_binds_all_thirteen_roles_and_exact_life_recipe_bytes() {
        let f = life_fixture();
        let first = validate(&f.manifest()).unwrap();
        assert_eq!(first.manifest_schema, 3);
        assert_eq!(first.profiles.len(), 13);
        let path = f.path("creature_retaliation");
        let mut bytes = fs::read(&path).unwrap();
        bytes.push(b' ');
        fs::write(path, bytes).unwrap();
        let second = validate(&f.manifest()).unwrap();
        assert_eq!(first.manifest_sha256, second.manifest_sha256);
        assert_ne!(
            first.validation_bundle_sha256,
            second.validation_bundle_sha256
        );
    }
    #[test]
    fn life_roles_are_required_only_by_schema_three_and_cannot_be_null_or_aliased() {
        for role in ["creature_retaliation", "player_respawn", "quest_turn_in_xp"] {
            let f = life_fixture();
            f.edit("manifest", |m| {
                m.as_object_mut().unwrap().remove(role);
            });
            assert!(validate(&f.manifest()).is_err());
            let f = life_fixture();
            f.edit("manifest", |m| m[role] = Value::Null);
            assert!(validate(&f.manifest()).is_err());
        }
        for schema in [1, 2] {
            let f = life_fixture();
            f.edit("manifest", |m| m["schema"] = json!(schema));
            assert!(validate(&f.manifest()).is_err());
        }
        let f = life_fixture();
        f.edit("manifest", |m| {
            m["player_respawn"] = m["creature_retaliation"].clone()
        });
        assert!(validate(&f.manifest()).is_err());
    }
    #[test]
    fn outcome_bundle_checks_probability_and_explicit_defender_schema() {
        let source = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../content/gameplay/development-gameplay-hit-outcomes.json");
        validate(&source).unwrap();
        let f = life_fixture();
        fs::write(
            f.path("abilities"),
            include_bytes!("../../../content/gameplay/development-hit-abilities.json"),
        )
        .unwrap();
        fs::write(
            f.path("creature"),
            include_bytes!("../../../content/gameplay/development-creature-deflect-pursuit.json"),
        )
        .unwrap();
        validate(&f.manifest()).unwrap();
        f.edit("abilities", |a| {
            a["abilities"][1]["hit_table"]["hit_bp"] = json!(8501)
        });
        assert!(validate(&f.manifest()).is_err());
        fs::write(
            f.path("abilities"),
            include_bytes!("../../../content/gameplay/development-hit-abilities.json"),
        )
        .unwrap();
        f.edit("creature", |c| c["schema"] = json!(1));
        assert!(validate(&f.manifest()).is_err());
    }
    #[test]
    fn projectile_bundle_checks_weapon_references_and_finite_travel_against_launch_range() {
        let source = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../content/gameplay/development-gameplay-projectiles.json");
        validate(&source).unwrap();
        let f = life_fixture();
        fs::write(
            f.path("abilities"),
            include_bytes!("../../../content/gameplay/development-projectile-abilities.json"),
        )
        .unwrap();
        validate(&f.manifest()).unwrap();
        f.edit("abilities", |a| {
            a["abilities"][0]["projectile"]["maximum_distance_metres"] = json!(29.)
        });
        assert!(validate(&f.manifest()).is_err());
        fs::write(
            f.path("abilities"),
            include_bytes!("../../../content/gameplay/development-projectile-abilities.json"),
        )
        .unwrap();
        f.edit("abilities", |a| {
            a["abilities"][0]["requires_main_hand"] = json!("theia.dev.missing")
        });
        assert!(validate(&f.manifest()).is_err());
    }
    #[test]
    fn player_melee_bundle_checks_required_weapon_references() {
        let source = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../content/gameplay/development-gameplay-player-melee.json");
        validate(&source).unwrap();
        let f = Fixture::new();
        fs::write(
            f.path("abilities"),
            include_bytes!("../../../content/gameplay/development-melee-abilities.json"),
        )
        .unwrap();
        validate(&f.manifest()).unwrap();
        f.edit("abilities", |a| {
            a["abilities"][1]["requires_main_hand"] = json!("theia.dev.missing")
        });
        assert!(
            validate(&f.manifest())
                .unwrap_err()
                .to_string()
                .contains("undeclared or unequippable")
        );
    }
    #[test]
    fn pursuit_requires_life_roles_and_accepts_the_complete_enabled_bundle() {
        let profile: Value = serde_json::from_slice(include_bytes!(
            "../../../content/gameplay/development-creature-pursuit.json"
        ))
        .unwrap();
        let f = Fixture::new();
        f.edit("creature", |c| c["aggro"] = profile["aggro"].clone());
        assert!(
            validate(&f.manifest())
                .err()
                .unwrap()
                .to_string()
                .contains("Pursuit requires player life")
        );
        let f = life_fixture();
        f.edit("creature", |c| c["aggro"] = profile["aggro"].clone());
        validate(&f.manifest()).unwrap();
    }
    #[test]
    fn life_recipes_enforce_references_and_bounded_file_reads() {
        for (role, key) in [
            ("creature_retaliation", "creature"),
            ("player_respawn", "stats"),
        ] {
            let f = life_fixture();
            f.edit(role, |v| v[key] = json!("other.profile"));
            assert!(validate(&f.manifest()).is_err());
            let f = life_fixture();
            fs::write(f.path(role), vec![b' '; 4097]).unwrap();
            assert!(validate(&f.manifest()).is_err());
        }
    }
    fn physical_fixture() -> Fixture {
        let f = Fixture::new();
        let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../content/gameplay");
        let bytes = fs::read(source.join("development-gameplay-classic-physical.json")).unwrap();
        let manifest: Manifest = serde_json::from_slice(&bytes).unwrap();
        for (_, path, _) in manifest.roles() {
            fs::copy(source.join(path), f.0.join(path)).unwrap();
        }
        fs::write(f.manifest(), bytes).unwrap();
        f
    }
    #[test]
    fn physical_manifest_binds_sixteen_roles_and_hashes_exact_stats() {
        let f = physical_fixture();
        let first = validate(&f.manifest()).unwrap();
        assert_eq!(first.manifest_schema, 6);
        assert_eq!(first.profiles.len(), 16);
        let path = f.path("physical_stats");
        let mut bytes = fs::read(&path).unwrap();
        bytes.push(b' ');
        fs::write(path, bytes).unwrap();
        let changed = validate(&f.manifest()).unwrap();
        assert_ne!(
            first.validation_bundle_sha256,
            changed.validation_bundle_sha256
        );
    }
    #[test]
    fn physical_roles_require_schema_six_complete_pair_and_distinct_files() {
        for role in ["physical_definitions", "physical_stats"] {
            for missing in [false, true] {
                let f = physical_fixture();
                f.edit("manifest", |m| {
                    if missing {
                        m.as_object_mut().unwrap().remove(role);
                    } else {
                        m[role] = Value::Null;
                    }
                });
                assert!(validate(&f.manifest()).is_err());
            }
        }
        let f = physical_fixture();
        f.edit("manifest", |m| m["schema"] = json!(5));
        assert!(validate(&f.manifest()).is_err());
        let f = physical_fixture();
        f.edit("manifest", |m| {
            m["physical_stats"] = m["physical_definitions"].clone()
        });
        assert!(validate(&f.manifest()).is_err());
    }
    #[test]
    fn physical_manifest_checks_numeric_bounds_and_known_weapon_binding() {
        let f = physical_fixture();
        f.edit("physical_definitions", |m| {
            m["attacks"][0]["item"] = json!("theia.dev.missing")
        });
        assert!(validate(&f.manifest()).is_err());
        let f = physical_fixture();
        f.edit("physical_definitions", |m| {
            m["attacks"][0]["maximum"] = json!(1000001)
        });
        assert!(validate(&f.manifest()).is_err());
        let f = physical_fixture();
        fs::write(
            f.path("physical_stats"),
            vec![b' '; theia_core::damage::stats::MAX_PROFILE_BYTES + 1],
        )
        .unwrap();
        assert!(validate(&f.manifest()).is_err());
    }
    fn white_fixture() -> Fixture {
        let f = Fixture::new();
        let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../content/gameplay");
        let bytes = fs::read(source.join("development-gameplay-classic-outcomes.json")).unwrap();
        let manifest: Manifest = serde_json::from_slice(&bytes).unwrap();
        for (_, path, _) in manifest.roles() {
            fs::copy(source.join(path), f.0.join(path)).unwrap();
        }
        fs::write(f.manifest(), bytes).unwrap();
        f
    }
    #[test]
    fn ordinary_manifest_seals_seventeen_roles_and_exact_outcome_bytes() {
        let f = white_fixture();
        let first = validate(&f.manifest()).unwrap();
        assert_eq!((first.manifest_schema, first.profiles.len()), (7, 17));
        let path = f.path("white_melee");
        let mut bytes = fs::read(&path).unwrap();
        bytes.push(b' ');
        fs::write(path, bytes).unwrap();
        assert_ne!(
            first.validation_bundle_sha256,
            validate(&f.manifest()).unwrap().validation_bundle_sha256
        );
    }
    #[test]
    fn ordinary_role_is_required_only_in_schema_seven_and_cannot_be_null_or_aliased() {
        for missing in [false, true] {
            let f = white_fixture();
            f.edit("manifest", |m| {
                if missing {
                    m.as_object_mut().unwrap().remove("white_melee");
                } else {
                    m["white_melee"] = Value::Null;
                }
            });
            assert!(validate(&f.manifest()).is_err());
        }
        let f = white_fixture();
        f.edit("manifest", |m| m["schema"] = json!(6));
        assert!(validate(&f.manifest()).is_err());
        let f = white_fixture();
        f.edit("manifest", |m| {
            m["white_melee"] = m["physical_stats"].clone()
        });
        assert!(validate(&f.manifest()).is_err());
    }
    #[test]
    fn ordinary_manifest_checks_pursuit_references_and_current_casting_scope() {
        let f = white_fixture();
        f.edit("creature", |m| {
            m.as_object_mut().unwrap().remove("aggro");
        });
        assert!(validate(&f.manifest()).is_err());
        let f = white_fixture();
        f.edit("white_melee", |m| m["stats"] = json!("theia.dev.other"));
        assert!(validate(&f.manifest()).is_err());
        let f = white_fixture();
        f.edit("abilities", |m| m["abilities"][1]["cast_ms"] = json!(500));
        assert!(validate(&f.manifest()).is_err());
    }
    #[test]
    fn ordinary_manifest_enforces_numeric_and_byte_bounds() {
        let f = white_fixture();
        f.edit("white_melee", |m| {
            m["player"]["weapons"][0]["skill"] = json!(1001)
        });
        assert!(validate(&f.manifest()).is_err());
        let f = white_fixture();
        fs::write(
            f.path("white_melee"),
            vec![b' '; combat::white_melee::MAX_PROFILE_BYTES + 1],
        )
        .unwrap();
        assert!(validate(&f.manifest()).is_err());
    }
    fn ability_fixture() -> Fixture {
        let f = Fixture::new();
        let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../content/gameplay");
        let data =
            fs::read(source.join("development-gameplay-classic-ability-damage.json")).unwrap();
        let manifest: Manifest = serde_json::from_slice(&data).unwrap();
        for (_, path, _) in manifest.roles() {
            fs::copy(source.join(path), f.0.join(path)).unwrap();
        }
        fs::write(f.manifest(), data).unwrap();
        f
    }
    #[test]
    fn ability_manifest_seals_nineteen_roles_and_both_exact_ability_inputs() {
        let f = ability_fixture();
        let first = validate(&f.manifest()).unwrap();
        assert_eq!((first.manifest_schema, first.profiles.len()), (8, 19));
        for kind in ["ability_starting", "ability_damage"] {
            let path = f.path(kind);
            let mut data = fs::read(&path).unwrap();
            data.push(b' ');
            fs::write(path, data).unwrap();
            assert_ne!(
                first.validation_bundle_sha256,
                validate(&f.manifest()).unwrap().validation_bundle_sha256
            );
        }
    }
    #[test]
    fn ability_roles_require_schema_eight_complete_pair_and_distinct_non_null_paths() {
        for kind in ["ability_starting", "ability_damage"] {
            for missing in [true, false] {
                let f = ability_fixture();
                f.edit("manifest", |m| {
                    if missing {
                        m.as_object_mut().unwrap().remove(kind);
                    } else {
                        m[kind] = Value::Null;
                    }
                });
                assert!(validate(&f.manifest()).is_err());
            }
        }
        let f = ability_fixture();
        f.edit("manifest", |m| m["schema"] = json!(7));
        assert!(validate(&f.manifest()).is_err());
        let f = ability_fixture();
        f.edit("manifest", |m| {
            m["ability_damage"] = m["ability_starting"].clone()
        });
        assert!(validate(&f.manifest()).is_err());
    }
    #[test]
    fn ability_manifest_checks_recipe_policy_refs_categories_and_zero_resistance_scope() {
        let f = ability_fixture();
        f.edit("ability_damage", |m| {
            m["ordinary"] = json!("theia.test.unknown")
        });
        assert!(validate(&f.manifest()).is_err());
        let f = ability_fixture();
        f.edit("ability_starting", |m| {
            m["recipes"][0]["weapon_attack"] = json!("theia.test.unknown")
        });
        assert!(validate(&f.manifest()).is_err());
        let f = ability_fixture();
        f.edit("ability_damage", |m| {
            m["abilities"][1]["ability"] = json!("theia.dev.sword-slash")
        });
        assert!(validate(&f.manifest()).is_err());
        let f = ability_fixture();
        f.edit("physical_stats", |m| {
            m["creatures"][0]["physical"]["level"] = json!(2)
        });
        assert!(validate(&f.manifest()).is_err());
    }
    #[test]
    fn ability_manifest_refuses_oversized_and_combined_out_of_range_damage() {
        for kind in ["ability_starting", "ability_damage"] {
            let f = ability_fixture();
            fs::write(
                f.path(kind),
                vec![b' '; combat::ability_damage::MAX_PROFILE_BYTES + 1],
            )
            .unwrap();
            assert!(validate(&f.manifest()).is_err());
        }
        let f = ability_fixture();
        f.edit("ability_starting", |m| {
            m["recipes"][0]["flat_bonus"] = json!(1_000_000)
        });
        assert!(validate(&f.manifest()).is_err());
        let f = ability_fixture();
        f.edit("ability_damage", |m| {
            m["abilities"][0]["miss_bp"] = json!(10001)
        });
        assert!(validate(&f.manifest()).is_err());
    }
    fn membership_fixture() -> Fixture {
        let f = Fixture::new();
        let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../content/gameplay");
        let data = fs::read(source.join("development-gameplay-classic-membership.json")).unwrap();
        let manifest: Manifest = serde_json::from_slice(&data).unwrap();
        for (_, path, _) in manifest.roles() {
            fs::copy(source.join(path), f.0.join(path)).unwrap();
        }
        fs::write(f.manifest(), data).unwrap();
        f
    }
    #[test]
    fn membership_manifest_seals_twenty_roles_and_exact_policy_bytes() {
        let f = membership_fixture();
        let first = validate(&f.manifest()).unwrap();
        assert_eq!((first.manifest_schema, first.profiles.len()), (9, 20));
        let path = f.path("combat_membership");
        let mut bytes = fs::read(&path).unwrap();
        bytes.push(b' ');
        fs::write(path, bytes).unwrap();
        assert_ne!(
            first.validation_bundle_sha256,
            validate(&f.manifest()).unwrap().validation_bundle_sha256
        );
    }
    #[test]
    fn membership_manifest_requires_its_schema_and_complete_distinct_context() {
        for field in [
            "combat_membership",
            "ability_starting",
            "ability_damage",
            "white_melee",
            "player_respawn",
        ] {
            for missing in [false, true] {
                let f = membership_fixture();
                f.edit("manifest", |m| {
                    if missing {
                        m.as_object_mut().unwrap().remove(field);
                    } else {
                        m[field] = Value::Null;
                    }
                });
                assert!(validate(&f.manifest()).is_err());
            }
        }
        let f = membership_fixture();
        f.edit("manifest", |m| m["schema"] = json!(8));
        assert!(validate(&f.manifest()).is_err());
        let f = membership_fixture();
        f.edit("manifest", |m| {
            m["combat_membership"] = m["ability_damage"].clone()
        });
        assert!(validate(&f.manifest()).is_err());
        let f = ability_fixture();
        f.edit("manifest", |m| m["schema"] = json!(9));
        assert!(validate(&f.manifest()).is_err());
    }
    #[test]
    fn membership_manifest_checks_policy_limits_version_and_byte_budget() {
        for (field, value) in [
            ("ruleset", json!("classic_verified")),
            ("maximum_opponents", json!(33)),
            ("hostility_grace_ms", json!(0)),
            ("unknown", json!(true)),
        ] {
            let f = membership_fixture();
            f.edit("combat_membership", |m| m[field] = value);
            assert!(validate(&f.manifest()).is_err());
        }
        let f = membership_fixture();
        fs::write(
            f.path("combat_membership"),
            vec![b' '; theia_core::combat_membership::MAX_PROFILE_BYTES + 1],
        )
        .unwrap();
        assert!(validate(&f.manifest()).is_err());
    }
    fn credit_fixture() -> Fixture {
        let f = Fixture::new();
        let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../content/gameplay");
        let data = fs::read(source.join("development-gameplay-classic-credit.json")).unwrap();
        let manifest: Manifest = serde_json::from_slice(&data).unwrap();
        for (_, path, _) in manifest.roles() {
            fs::copy(source.join(path), f.0.join(path)).unwrap();
        }
        fs::write(f.manifest(), data).unwrap();
        f
    }
    #[test]
    fn credit_manifest_seals_twenty_one_roles_and_exact_policy_bytes() {
        let f = credit_fixture();
        let first = validate(&f.manifest()).unwrap();
        assert_eq!((first.manifest_schema, first.profiles.len()), (10, 21));
        let path = f.path("creature_credit");
        let mut bytes = fs::read(&path).unwrap();
        bytes.push(b' ');
        fs::write(path, bytes).unwrap();
        assert_ne!(
            first.validation_bundle_sha256,
            validate(&f.manifest()).unwrap().validation_bundle_sha256
        );
    }
    #[test]
    fn credit_manifest_requires_exclusive_schema_and_complete_distinct_context() {
        for field in [
            "creature_credit",
            "combat_membership",
            "ability_damage",
            "white_melee",
            "player_respawn",
        ] {
            for missing in [false, true] {
                let f = credit_fixture();
                f.edit("manifest", |m| {
                    if missing {
                        m.as_object_mut().unwrap().remove(field);
                    } else {
                        m[field] = Value::Null;
                    }
                });
                assert!(validate(&f.manifest()).is_err());
            }
        }
        let f = credit_fixture();
        f.edit("manifest", |m| m["schema"] = json!(9));
        assert!(validate(&f.manifest()).is_err());
        let f = membership_fixture();
        f.edit("manifest", |m| m["schema"] = json!(10));
        assert!(validate(&f.manifest()).is_err());
        let f = credit_fixture();
        f.edit("manifest", |m| {
            m["creature_credit"] = m["combat_membership"].clone()
        });
        assert!(validate(&f.manifest()).is_err());
    }
    #[test]
    fn credit_manifest_requires_solo_recipient_and_bounded_supported_policy() {
        for (field, value) in [
            ("recipient", Value::Null),
            ("recipient", json!("last_hitter")),
            ("maximum_contributors", json!(33)),
            ("ruleset", json!("classic_verified")),
            ("unknown", json!(true)),
        ] {
            let f = credit_fixture();
            f.edit("creature_credit", |m| m[field] = value);
            assert!(validate(&f.manifest()).is_err());
        }
        let f = credit_fixture();
        fs::write(
            f.path("creature_credit"),
            vec![b' '; theia_core::creature_credit::MAX_PROFILE_BYTES + 1],
        )
        .unwrap();
        assert!(validate(&f.manifest()).is_err());
        let f = credit_fixture();
        fs::write(
            f.path("creature_credit"),
            include_bytes!("../../../content/gameplay/development-creature-credit.json"),
        )
        .unwrap();
        assert!(validate(&f.manifest()).is_err());
    }
    fn reset_fixture() -> Fixture {
        let f = Fixture::new();
        let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../content/gameplay");
        let data = fs::read(source.join("development-gameplay-classic-reset.json")).unwrap();
        let manifest: Manifest = serde_json::from_slice(&data).unwrap();
        for (_, path, _) in manifest.roles() {
            fs::copy(source.join(path), f.0.join(path)).unwrap();
        }
        fs::write(f.manifest(), data).unwrap();
        f
    }
    #[test]
    fn reset_manifest_seals_policy_bytes_and_refuses_incomplete_or_older_context() {
        let f = reset_fixture();
        let first = validate(&f.manifest()).unwrap();
        assert_eq!((first.manifest_schema, first.profiles.len()), (11, 22));
        let path = f.path("creature_reset");
        let mut data = fs::read(&path).unwrap();
        data.push(b' ');
        fs::write(path, data).unwrap();
        assert_ne!(
            first.validation_bundle_sha256,
            validate(&f.manifest()).unwrap().validation_bundle_sha256
        );
        for field in [
            "creature_reset",
            "creature_credit",
            "combat_membership",
            "ability_damage",
            "player_respawn",
        ] {
            for missing in [false, true] {
                let f = reset_fixture();
                f.edit("manifest", |m| {
                    if missing {
                        m.as_object_mut().unwrap().remove(field);
                    } else {
                        m[field] = Value::Null;
                    }
                });
                assert!(validate(&f.manifest()).is_err());
            }
        }
        let f = reset_fixture();
        f.edit("manifest", |m| m["schema"] = json!(10));
        assert!(validate(&f.manifest()).is_err());
        let f = credit_fixture();
        f.edit("manifest", |m| m["schema"] = json!(11));
        assert!(validate(&f.manifest()).is_err());
        let f = reset_fixture();
        f.edit("manifest", |m| {
            m["creature_reset"] = m["creature_credit"].clone()
        });
        assert!(validate(&f.manifest()).is_err());
    }
    #[test]
    fn reset_manifest_refuses_wrong_creature_unsupported_policy_and_byte_overflow() {
        for (field, value) in [
            ("schema", json!(2)),
            ("ruleset", json!("classic_verified")),
            ("creature", json!("theia.dev.other-creature")),
            ("unknown", json!(true)),
        ] {
            let f = reset_fixture();
            f.edit("creature_reset", |m| m[field] = value);
            assert!(validate(&f.manifest()).is_err());
        }
        let f = reset_fixture();
        fs::write(
            f.path("creature_reset"),
            vec![b' '; combat::reset::MAX_PROFILE_BYTES + 1],
        )
        .unwrap();
        assert!(validate(&f.manifest()).is_err());
        let f = reset_fixture();
        f.edit("creature", |m| {
            m.as_object_mut().unwrap().remove("aggro");
        });
        assert!(validate(&f.manifest()).is_err());
    }
}
