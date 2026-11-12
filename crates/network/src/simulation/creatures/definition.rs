//! Validated creature behavior independent of an individual patrol assignment.
//! Loaded through the existing profile parser; no new source schema or defaults.
use theia_core::{content::ContentId, pursuit::Settings};

#[derive(Clone, Debug, PartialEq)]
pub struct Definition {
    pub(super) id: ContentId,
    pub(super) speed: f32,
    pub(super) wait_ms: u32,
    pub(super) aggro: Option<Settings>,
    pub(super) can_deflect: bool,
}
impl Definition {
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
    const SOURCE: &[u8] =
        include_bytes!("../../../../../content/gameplay/development-creature-deflect-pursuit.json");
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
