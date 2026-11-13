//! Validated creature behavior independent of an individual patrol assignment.
//! Schema 3 stores a standalone type; legacy patrol profiles remain readable.
use crate::Result;
use serde::Deserialize;
use theia_core::{content::ContentId, pursuit::Settings};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Document {
    schema: u32,
    id: String,
    speed_metres_per_second: f32,
    wait_ms: u32,
    aggro: Option<Settings>,
    can_deflect: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Definition {
    pub(super) id: ContentId,
    pub(super) speed: f32,
    pub(super) wait_ms: u32,
    pub(super) aggro: Option<Settings>,
    pub(super) can_deflect: bool,
}
impl Definition {
    /// Startup-only type loading. Schema 3 deliberately has no route field;
    /// schema 1/2 inputs retain all existing patrol validation before extraction.
    pub fn from_json(bytes: &[u8]) -> Result<Self> {
        if bytes.len() > super::MAX_PROFILE_BYTES {
            return Err("Creature definition exceeds byte budget".into());
        }
        #[derive(Deserialize)]
        struct Version {
            schema: u32,
        }
        let version: Version = serde_json::from_slice(bytes)?;
        if matches!(version.schema, 1 | 2) {
            return Ok(super::Profile::from_json(bytes)?.definition);
        }
        let raw: Document = serde_json::from_slice(bytes)?;
        if raw.schema != 3 {
            return Err("Unsupported standalone creature definition schema".into());
        }
        Self::checked(
            &raw.id,
            raw.speed_metres_per_second,
            raw.wait_ms,
            raw.aggro,
            raw.can_deflect,
        )
    }
    pub(super) fn checked(
        id: &str,
        speed: f32,
        wait_ms: u32,
        aggro: Option<Settings>,
        can_deflect: bool,
    ) -> Result<Self> {
        if !speed.is_finite()
            || !(0.1..=5.).contains(&speed)
            || !(100..=60000).contains(&wait_ms)
            || aggro.is_some_and(|settings| !settings.valid())
        {
            return Err("Creature profile exceeds initial patrol bounds".into());
        }
        Ok(Self {
            id: ContentId::new(id).ok_or("Invalid creature ID")?,
            speed,
            wait_ms,
            aggro,
            can_deflect,
        })
    }
    pub fn id(&self) -> &ContentId {
        &self.id
    }
    pub fn speed_metres_per_second(&self) -> f32 {
        self.speed
    }
    pub fn wait_ms(&self) -> u32 {
        self.wait_ms
    }
    pub fn aggro(&self) -> Option<Settings> {
        self.aggro
    }
    pub fn can_deflect(&self) -> bool {
        self.can_deflect
    }
    pub fn profile_identity(&self, base: [u8; 32]) -> [u8; 32] {
        let mut bytes = b"theia-population-definition-world-v1\0".to_vec();
        bytes.extend(base);
        bytes.extend(self.identity());
        crate::terrain::sha256(&bytes)
    }
    /// Canonical behavior identity. Route selection and JSON formatting do not
    /// change a creature type; existing patrol profile hashes remain exact-byte.
    pub fn identity(&self) -> [u8; 32] {
        let mut bytes = b"theia-creature-definition-v1\0".to_vec();
        bytes.extend((self.id.as_str().len() as u64).to_le_bytes());
        bytes.extend(self.id.as_str().as_bytes());
        bytes.extend(self.speed.to_le_bytes());
        bytes.extend(self.wait_ms.to_le_bytes());
        bytes.push(u8::from(self.can_deflect));
        bytes.push(u8::from(self.aggro.is_some()));
        if let Some(aggro) = self.aggro {
            bytes.extend(aggro.perception_metres.to_le_bytes());
            bytes.extend(aggro.leash_metres.to_le_bytes());
            bytes.extend(aggro.chase_speed.to_le_bytes());
            bytes.extend(aggro.return_speed.to_le_bytes());
            bytes.extend(aggro.los_grace_ms.to_le_bytes());
            bytes.extend(aggro.engagement_ms.to_le_bytes());
            bytes.extend(aggro.reacquire_ms.to_le_bytes());
        }
        crate::terrain::sha256(&bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::super::Profile;
    use super::Definition;
    const SOURCE: &[u8] =
        include_bytes!("../../../../../content/gameplay/development-creature-deflect-pursuit.json");
    #[test]
    fn standalone_types_match_legacy_behavior_without_inventing_a_patrol() {
        let legacy = Profile::from_json(SOURCE).unwrap();
        let mut raw: serde_json::Value = serde_json::from_slice(SOURCE).unwrap();
        raw["schema"] = 3.into();
        raw.as_object_mut().unwrap().remove("route");
        let bytes = serde_json::to_vec(&raw).unwrap();
        let standalone = Definition::from_json(&bytes).unwrap();
        assert_eq!(&standalone, legacy.definition());
        assert_eq!(standalone.identity(), legacy.definition().identity());
        assert!(Profile::from_json(&bytes).is_err());
        assert_eq!(Definition::from_json(SOURCE).unwrap(), standalone);
        let passive = Definition::from_json(include_bytes!(
            "../../../../../content/gameplay/development-creature-passive.json"
        ))
        .unwrap();
        let legacy_passive = Definition::from_json(include_bytes!(
            "../../../../../content/gameplay/development-creature-patrol.json"
        ))
        .unwrap();
        assert_eq!(passive, legacy_passive);
    }
    #[test]
    fn standalone_schema_refuses_routes_unknown_fields_and_invalid_behavior() {
        let source: serde_json::Value = serde_json::from_slice(include_bytes!(
            "../../../../../content/gameplay/development-creature-passive.json"
        ))
        .unwrap();
        for (field, value) in [
            ("schema", serde_json::json!(4)),
            ("schema", serde_json::json!(2)),
            ("route", serde_json::json!("theia.dev.training-patrol")),
            ("route", serde_json::Value::Null),
            ("unknown", serde_json::json!(true)),
            ("id", serde_json::json!("invalid id")),
            ("speed_metres_per_second", serde_json::json!(0)),
            ("speed_metres_per_second", serde_json::json!(5.1)),
            ("wait_ms", serde_json::json!(99)),
            ("wait_ms", serde_json::json!(60001)),
            ("can_deflect", serde_json::Value::Null),
            ("aggro", serde_json::json!({})),
        ] {
            let mut raw = source.clone();
            raw[field] = value;
            assert!(
                Definition::from_json(&serde_json::to_vec(&raw).unwrap()).is_err(),
                "{field}"
            );
        }
        for missing in [
            "schema",
            "id",
            "speed_metres_per_second",
            "wait_ms",
            "can_deflect",
        ] {
            let mut raw = source.clone();
            raw.as_object_mut().unwrap().remove(missing);
            assert!(
                Definition::from_json(&serde_json::to_vec(&raw).unwrap()).is_err(),
                "{missing}"
            );
        }
        let mut bytes = serde_json::to_vec(&source).unwrap();
        bytes.resize(super::super::MAX_PROFILE_BYTES + 1, b' ');
        assert!(Definition::from_json(&bytes).is_err());
    }
    #[test]
    fn definitions_ignore_route_and_format_but_patrol_profiles_keep_exact_identity() {
        let original = Profile::from_json(SOURCE).unwrap();
        let mut document: serde_json::Value = serde_json::from_slice(SOURCE).unwrap();
        document["route"] = "theia.zone.patrol.1002".into();
        let rebound = Profile::from_json(&serde_json::to_vec(&document).unwrap()).unwrap();
        assert_eq!(original.definition(), rebound.definition());
        assert_eq!(
            original.definition().identity(),
            rebound.definition().identity()
        );
        assert_ne!(
            super::super::profile_identity([0; 32], &original, [0; 32]),
            super::super::profile_identity([0; 32], &rebound, [0; 32])
        );
        let compact = Profile::from_json(
            &serde_json::to_vec(&serde_json::from_slice::<serde_json::Value>(SOURCE).unwrap())
                .unwrap(),
        )
        .unwrap();
        assert_eq!(
            original.definition().identity(),
            compact.definition().identity()
        );
    }
    #[test]
    fn every_gameplay_field_changes_the_definition_identity() {
        let original = Profile::from_json(SOURCE).unwrap();
        let source: serde_json::Value = serde_json::from_slice(SOURCE).unwrap();
        for (path, value) in [
            ("/id", serde_json::json!("theia.creature.other")),
            ("/speed_metres_per_second", serde_json::json!(1.5)),
            ("/wait_ms", serde_json::json!(1250)),
            ("/can_deflect", serde_json::json!(false)),
            ("/aggro/perception_metres", serde_json::json!(9.)),
            ("/aggro/leash_metres", serde_json::json!(70.)),
            ("/aggro/chase_speed", serde_json::json!(3.5)),
            ("/aggro/return_speed", serde_json::json!(3.5)),
            ("/aggro/los_grace_ms", serde_json::json!(500)),
            ("/aggro/engagement_ms", serde_json::json!(15000)),
            ("/aggro/reacquire_ms", serde_json::json!(1500)),
        ] {
            let mut changed = source.clone();
            assert_ne!(changed.pointer(path).unwrap(), &value, "{path}");
            *changed.pointer_mut(path).unwrap() = value;
            let profile = Profile::from_json(&serde_json::to_vec(&changed).unwrap()).unwrap();
            assert_ne!(
                profile.definition().identity(),
                original.definition().identity(),
                "{path}"
            );
        }
        let mut changed = source;
        changed.as_object_mut().unwrap().remove("aggro");
        let profile = Profile::from_json(&serde_json::to_vec(&changed).unwrap()).unwrap();
        assert!(profile.definition().aggro().is_none());
        assert_ne!(
            profile.definition().identity(),
            original.definition().identity()
        );
    }
}
