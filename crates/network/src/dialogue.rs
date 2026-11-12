//! Owner-side informational interaction contracts. Transport encoding and Bevy
//! adoption are separate; none of these structs authorize a client actor ID.
use std::sync::Arc;
use theia_core::{
    content::ContentId, coordinates::GlobalPosition, dialogue::Profile as CoreProfile,
    identity::EntityKey,
};
pub const INTERACTION_METRES: f64 = 3.;
pub const REQUESTS_PER_SECOND: u8 = 8;

#[derive(Clone, Debug)]
pub struct Profile {
    profile: Arc<CoreProfile>,
    hash: [u8; 32],
}
impl Profile {
    pub fn from_json(bytes: &[u8]) -> crate::Result<Self> {
        let profile = Arc::new(CoreProfile::from_json(bytes)?);
        let mut bound = theia_core::dialogue::PROFILE_DOMAIN.to_vec();
        bound.extend(bytes);
        Ok(Self {
            profile,
            hash: crate::terrain::sha256(&bound),
        })
    }
    pub fn profile(&self) -> &CoreProfile {
        &self.profile
    }
    pub fn hash(&self) -> [u8; 32] {
        self.hash
    }
}
pub fn profile_identity(base: [u8; 32], profile: [u8; 32], placement: [f64; 2]) -> [u8; 32] {
    let mut bytes = b"theia-informational-guide-world-v1\0".to_vec();
    bytes.extend(base);
    bytes.extend(profile);
    for value in placement {
        bytes.extend(value.to_le_bytes());
    }
    crate::terrain::sha256(&bytes)
}
/// Trusted startup placement. Horizontal coordinates retain the legacy terrain
/// fixture contract; authored feet select an explicit vertical support layer.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Placement {
    Horizontal([f64; 2]),
    Feet(GlobalPosition),
}
impl Placement {
    pub fn parse(value: &str) -> crate::Result<Self> {
        let values: Vec<f64> = value.split(',').map(str::parse).collect::<Result<_, _>>()?;
        let placement = match values.as_slice() {
            [x, z] => Self::Horizontal([*x, *z]),
            [x, y, z] => Self::Feet(GlobalPosition::new([*x, *y, *z]).ok_or("Invalid guide feet")?),
            _ => return Err("Expected guide x,z or x,y,z feet in metres".into()),
        };
        placement.focus()?;
        Ok(placement)
    }
    pub fn focus(self) -> crate::Result<GlobalPosition> {
        let focus = match self {
            Self::Horizontal([x, z]) => {
                GlobalPosition::new([x, 0., z]).ok_or("Invalid guide position")?
            }
            Self::Feet(position) => position,
        };
        if crate::procedural::owner(focus).is_none() {
            return Err("Guide outside supported world".into());
        }
        Ok(focus)
    }
    pub fn identity(self, base: [u8; 32], profile: [u8; 32]) -> [u8; 32] {
        match self {
            Self::Horizontal(xz) => profile_identity(base, profile, xz),
            Self::Feet(position) => {
                let mut bytes = b"theia-informational-guide-xyz-world-v1\0".to_vec();
                bytes.extend(base);
                bytes.extend(profile);
                for value in position.metres() {
                    bytes.extend(value.to_le_bytes());
                }
                crate::terrain::sha256(&bytes)
            }
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Guide {
    pub key: EntityKey,
    pub position: GlobalPosition,
}
pub fn in_range(actor: GlobalPosition, guide: GlobalPosition) -> bool {
    let a = actor.metres();
    let b = guide.metres();
    a.into_iter()
        .zip(b)
        .map(|(a, b)| (a - b).powi(2))
        .sum::<f64>()
        <= INTERACTION_METRES * INTERACTION_METRES
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Action {
    Open(EntityKey),
    Choose(ContentId),
    Close,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Request {
    pub sequence: u64,
    pub expected_revision: u64,
    pub action: Action,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Open {
    pub guide: EntityKey,
    pub node: ContentId,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct View {
    pub actor: EntityKey,
    pub profile_hash: [u8; 32],
    /// Includes closure, preventing a delayed prior node from reopening a panel.
    pub revision: u64,
    pub open: Option<Open>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rejection {
    Unavailable,
    RateLimited,
    StaleSequence,
    StaleRevision,
    InvalidAction,
    OutOfRange,
    Exhausted,
}

#[cfg(test)]
mod placement_tests {
    use super::*;
    #[test]
    fn explicit_height_is_checked_and_bound_without_changing_legacy_identity() {
        let horizontal = Placement::parse("8192,8192").unwrap();
        assert_eq!(
            horizontal.identity([1; 32], [2; 32]),
            profile_identity([1; 32], [2; 32], [8192.; 2])
        );
        let floor = Placement::parse("8192,0.1,8192").unwrap();
        let roof = Placement::parse("8192,4.6,8192").unwrap();
        assert_eq!(roof.focus().unwrap().metres(), [8192., 4.6, 8192.]);
        assert_ne!(
            floor.identity([1; 32], [2; 32]),
            roof.identity([1; 32], [2; 32])
        );
        assert_ne!(
            floor.identity([1; 32], [2; 32]),
            horizontal.identity([1; 32], [2; 32])
        );
        for text in [
            "",
            "1",
            "1,2,3,4",
            "NaN,1",
            "1,NaN,2",
            "1,inf,2",
            "-1,0,1",
            "16000,1",
            "1,0,16000",
        ] {
            assert!(Placement::parse(text).is_err(), "{text}");
        }
    }
}
