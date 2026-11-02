//! Bounded deterministic explicit target cycling. No camera, attacks or I/O.
//! Ordering is Theia policy, not a claimed reconstruction of Classic internals.
use crate::{
    coordinates::GlobalPosition,
    facing::Facing,
    identity::{EntityKey, WorldEpoch},
};
use serde::Deserialize;
pub const MAX_CANDIDATES: usize = 256;
#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Settings {
    pub prefer_front: bool,
    pub maximum_distance_metres: Option<f64>,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            prefer_front: true,
            maximum_distance_metres: None,
        }
    }
}
impl Settings {
    pub fn valid(self) -> bool {
        self.maximum_distance_metres
            .is_none_or(|r| r.is_finite() && (0. ..=4096.).contains(&r) && r > 0.)
    }
}
#[derive(Clone, Copy)]
pub struct Observer {
    pub epoch: WorldEpoch,
    pub position: GlobalPosition,
    pub facing: Facing,
}
#[derive(Clone, Copy)]
pub struct Candidate {
    pub key: EntityKey,
    pub position: GlobalPosition,
    pub eligible: bool,
}
#[derive(Clone, Copy)]
pub enum Direction {
    Forward,
    Reverse,
}
pub fn cycle(
    settings: Settings,
    observer: Observer,
    selected: Option<EntityKey>,
    candidates: &[Candidate],
    direction: Direction,
) -> Result<Option<EntityKey>, &'static str> {
    if !settings.valid() || candidates.len() > MAX_CANDIDATES {
        return Err("Invalid target-search settings or candidate budget");
    }
    let mut seen = std::collections::BTreeSet::new();
    let mut ranked = Vec::with_capacity(candidates.len());
    let from = observer.position.metres();
    let [fx, fz] = observer.facing.forward();
    for candidate in candidates {
        if candidate.key.epoch != observer.epoch || !seen.insert(candidate.key.entity) {
            return Err("Foreign or duplicate target candidate");
        }
        if !candidate.eligible {
            continue;
        }
        let to = candidate.position.metres();
        let delta = std::array::from_fn::<_, 3, _>(|i| to[i] - from[i]);
        let distance = delta.iter().map(|v| v * v).sum::<f64>();
        if settings
            .maximum_distance_metres
            .is_some_and(|r| distance > r * r)
        {
            continue;
        }
        let behind = settings.prefer_front && fx * delta[0] + fz * delta[2] < -1e-12;
        ranked.push((behind, distance, candidate.key));
    }
    ranked.sort_by(|a, b| {
        a.0.cmp(&b.0)
            .then(a.1.total_cmp(&b.1))
            .then(a.2.entity.cmp(&b.2.entity))
    });
    if ranked.is_empty() {
        return Ok(None);
    }
    let next = selected
        .and_then(|key| ranked.iter().position(|v| v.2 == key))
        .map_or(0, |i| match direction {
            Direction::Forward => (i + 1) % ranked.len(),
            Direction::Reverse => (i + ranked.len() - 1) % ranked.len(),
        });
    Ok(Some(ranked[next].2))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identity::EntityId;
    fn p(x: f64, z: f64) -> GlobalPosition {
        GlobalPosition::new([x, 0., z]).unwrap()
    }
    fn key(n: u64) -> EntityKey {
        EntityKey {
            epoch: WorldEpoch::new(1).unwrap(),
            entity: EntityId::new(n).unwrap(),
        }
    }
    fn c(n: u64, x: f64, z: f64) -> Candidate {
        Candidate {
            key: key(n),
            position: p(x, z),
            eligible: true,
        }
    }
    fn observer() -> Observer {
        Observer {
            epoch: WorldEpoch::new(1).unwrap(),
            position: p(0., 0.),
            facing: Facing::default(),
        }
    }
    #[test]
    fn front_preference_uses_character_heading_and_cycle_is_reversible() {
        let list = [c(1, 0., 2.), c(3, 0., -8.), c(2, 0., -3.)];
        let o = observer();
        let s = Settings::default();
        assert_eq!(
            cycle(s, o, None, &list, Direction::Forward).unwrap(),
            Some(key(2))
        );
        assert_eq!(
            cycle(s, o, Some(key(2)), &list, Direction::Forward).unwrap(),
            Some(key(3))
        );
        assert_eq!(
            cycle(s, o, Some(key(3)), &list, Direction::Reverse).unwrap(),
            Some(key(2))
        );
        assert_eq!(
            cycle(s, o, Some(key(2)), &list, Direction::Reverse).unwrap(),
            Some(key(1))
        );
        let o = Observer {
            facing: Facing::from_units(32768),
            ..o
        };
        assert_eq!(
            cycle(s, o, None, &list, Direction::Forward).unwrap(),
            Some(key(1))
        );
        assert_eq!(
            cycle(s, o, None, &list, Direction::Reverse).unwrap(),
            Some(key(1))
        );
    }
    #[test]
    fn ties_and_missing_selection_are_stable_and_no_candidate_does_not_choose_a_target() {
        let a = [c(3, 1., -1.), c(2, -1., -1.)];
        let b = [a[1], a[0]];
        let o = observer();
        let s = Settings::default();
        assert_eq!(
            cycle(s, o, None, &a, Direction::Forward).unwrap(),
            Some(key(2))
        );
        assert_eq!(
            cycle(s, o, Some(key(99)), &b, Direction::Reverse).unwrap(),
            Some(key(2))
        );
        assert!(
            cycle(s, o, Some(key(2)), &[], Direction::Forward)
                .unwrap()
                .is_none()
        );
        assert!(
            cycle(
                s,
                o,
                None,
                &[Candidate {
                    eligible: false,
                    ..a[0]
                }],
                Direction::Forward
            )
            .unwrap()
            .is_none()
        );
    }
    #[test]
    fn metre_range_and_front_preference_are_explicit_settings_not_attack_requirements() {
        let list = [c(1, 0., 1.), c(2, 0., -4.)];
        let o = observer();
        let s = Settings {
            maximum_distance_metres: Some(2.),
            ..Settings::default()
        };
        assert_eq!(
            cycle(s, o, None, &list, Direction::Forward).unwrap(),
            Some(key(1))
        );
        let s = Settings {
            prefer_front: false,
            maximum_distance_metres: None,
        };
        assert_eq!(
            cycle(s, o, None, &list, Direction::Forward).unwrap(),
            Some(key(1))
        );
        for r in [-1., 0., 4097., f64::INFINITY, f64::NAN] {
            assert!(
                !Settings {
                    maximum_distance_metres: Some(r),
                    ..s
                }
                .valid()
            );
        }
    }
    #[test]
    fn duplicates_foreign_epochs_and_excess_work_are_refused_before_selection() {
        let o = observer();
        let s = Settings::default();
        let a = c(1, 0., -1.);
        assert!(cycle(s, o, None, &[a, a], Direction::Forward).is_err());
        let mut foreign = a;
        foreign.key.epoch = WorldEpoch::new(2).unwrap();
        assert!(cycle(s, o, None, &[foreign], Direction::Forward).is_err());
        assert!(cycle(s, o, None, &vec![a; MAX_CANDIDATES + 1], Direction::Forward).is_err());
    }
}
