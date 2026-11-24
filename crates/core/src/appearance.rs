//! Versioned cosmetic choices for the curated modular hero. No gameplay attributes.
use serde::{Deserialize, Serialize};

pub const CHOICE_COUNTS: [u8; 7] = [2, 4, 5, 4, 4, 4, 2];
pub const LABELS: [&str; 7] = [
    "Body",
    "Face",
    "Hair",
    "Facial hair",
    "Skin tone",
    "Hair colour",
    "Outfit",
];

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "Raw", into = "Raw")]
pub struct Appearance([u8; 7]);
impl Default for Appearance {
    fn default() -> Self {
        Self([0, 0, 1, 0, 0, 0, 0])
    }
}
#[derive(Clone, Copy, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Raw {
    schema: u8,
    body: u8,
    face: u8,
    hair: u8,
    facial_hair: u8,
    skin: u8,
    hair_colour: u8,
    outfit: u8,
}
impl TryFrom<Raw> for Appearance {
    type Error = &'static str;
    fn try_from(v: Raw) -> Result<Self, Self::Error> {
        if v.schema != 1 {
            return Err("Unsupported appearance version");
        }
        Self::new([
            v.body,
            v.face,
            v.hair,
            v.facial_hair,
            v.skin,
            v.hair_colour,
            v.outfit,
        ])
    }
}
impl From<Appearance> for Raw {
    fn from(v: Appearance) -> Self {
        let [body, face, hair, facial_hair, skin, hair_colour, outfit] = v.0;
        Self {
            schema: 1,
            body,
            face,
            hair,
            facial_hair,
            skin,
            hair_colour,
            outfit,
        }
    }
}
impl Appearance {
    pub fn new(values: [u8; 7]) -> Result<Self, &'static str> {
        if values
            .into_iter()
            .zip(CHOICE_COUNTS)
            .any(|(v, count)| v >= count)
        {
            return Err("Appearance option is outside the supported catalogue");
        }
        if values[0] == 1 && values[3] != 0 {
            return Err("This body has no facial-hair meshes");
        }
        Ok(Self(values))
    }
    pub fn values(self) -> [u8; 7] {
        self.0
    }
    pub fn cycle(self, field: usize, forward: bool) -> Self {
        let Some(&count) = CHOICE_COUNTS.get(field) else {
            return self;
        };
        let mut values = self.0;
        values[field] = (values[field] + if forward { 1 } else { count - 1 }) % count;
        if values[0] == 1 {
            values[3] = 0;
        }
        Self(values)
    }
    pub fn palette(self) -> String {
        format!("palette-{}-{}.png", self.0[4], self.0[5])
    }
    pub fn parts(self) -> Vec<String> {
        let gender = ["Male", "Female"][self.0[0] as usize];
        let outfit = [1, 3][self.0[6] as usize];
        let mut parts: Vec<_> = [
            "Torso",
            "Hips",
            "ArmUpperLeft",
            "ArmUpperRight",
            "ArmLowerLeft",
            "ArmLowerRight",
            "HandLeft",
            "HandRight",
            "LegLeft",
            "LegRight",
        ]
        .into_iter()
        .map(|p| format!("Chr_{p}_{gender}_{outfit:02}"))
        .collect();
        parts.push(format!("Chr_Head_{gender}_{:02}", self.0[1]));
        parts.push(format!("Chr_Eyebrow_{gender}_01"));
        if self.0[2] != 0 {
            parts.push(format!("Chr_Hair_{:02}", self.0[2]));
        }
        if self.0[3] != 0 {
            parts.push(format!("Chr_FacialHair_Male_{:02}", self.0[3]));
        }
        parts
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn appearance_roundtrips_and_validates_every_boundary() {
        let a = Appearance::default();
        let json = serde_json::to_value(a).unwrap();
        assert_eq!(
            serde_json::from_value::<Appearance>(json.clone()).unwrap(),
            a
        );
        for field in [
            "body",
            "face",
            "hair",
            "facial_hair",
            "skin",
            "hair_colour",
            "outfit",
            "schema",
        ] {
            let mut bad = json.clone();
            bad[field] = 255.into();
            assert!(serde_json::from_value::<Appearance>(bad).is_err());
        }
        let mut bad = json;
        bad["arbitrary_asset_path"] = "../other.glb".into();
        assert!(serde_json::from_value::<Appearance>(bad).is_err());
    }
    #[test]
    fn selections_are_complete_and_body_changes_keep_valid_combinations() {
        for body in 0..2 {
            for face in 0..4 {
                for hair in 0..5 {
                    for outfit in 0..2 {
                        let a = Appearance::new([body, face, hair, 0, 0, 0, outfit]).unwrap();
                        let parts = a.parts();
                        assert_eq!(parts.len(), if hair == 0 { 12 } else { 13 });
                        assert_eq!(
                            parts.iter().filter(|p| p.starts_with("Chr_Head_")).count(),
                            1
                        );
                        assert!(parts.iter().all(|p| !p.contains(if body == 0 {
                            "Female"
                        } else {
                            "Male"
                        })));
                    }
                }
            }
        }
        let a = Appearance::new([0, 0, 1, 3, 0, 0, 0])
            .unwrap()
            .cycle(0, true);
        assert_eq!(a.values()[3], 0);
        assert_eq!(a.cycle(3, true), a);
    }
}
