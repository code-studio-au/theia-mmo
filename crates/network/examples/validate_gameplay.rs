//! Offline checks against the actual gameplay schemas. This manifest is not
//! adopted by the game runtime and does not establish a world identity.
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
}
impl Manifest {
    fn roles(&self) -> [(&'static str, &str, usize); 10] {
        [
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
        ]
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
    Ok(())
}
fn validate(path: &Path) -> Result<Report> {
    let path = path.canonicalize().map_err(|_| "Manifest unavailable")?;
    let raw = bounded(&path, MAX_MANIFEST_BYTES)?;
    let manifest: Manifest =
        serde_json::from_slice(&raw).map_err(|_| "Invalid gameplay manifest")?;
    if manifest.schema != 1 || manifest.revision == 0 || ContentId::new(&manifest.id).is_none() {
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
}
