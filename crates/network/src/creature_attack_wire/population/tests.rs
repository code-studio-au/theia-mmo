use super::*;
fn key(id: u64) -> EntityKey {
    EntityKey {
        epoch: WorldEpoch::new(7).unwrap(),
        entity: EntityId::new(id).unwrap(),
    }
}
fn profile() -> Arc<PopulationProfile> {
    let life = crate::player_life_wire::Profile::from_json(
        include_bytes!("../../../../../content/gameplay/development-character-stats.json"),
        include_bytes!("../../../../../content/gameplay/development-creature-pursuit.json"),
        include_bytes!("../../../../../content/gameplay/development-creature-retaliation.json"),
        include_bytes!("../../../../../content/gameplay/development-player-respawn.json"),
    )
    .unwrap();
    Arc::new(PopulationProfile::new(Arc::new(Profile::new(Arc::new(
        life,
    )))))
}
fn frame(profile: &PopulationProfile, rate: TickRate, ids: &[u64], tick: u64) -> Frame {
    Frame {
        epoch: key(1).epoch,
        tick,
        actors: ids
            .iter()
            .map(|id| Observation {
                profile: profile.source().hash(),
                creature: key(*id),
                tick,
                windup: Some(Windup {
                    target: key(100 + id % 2),
                    starts: 10,
                    completes: 10 + profile.source().windup_ticks(rate).unwrap(),
                }),
            })
            .collect(),
    }
}
#[test]
fn full_population_round_trip_is_bounded_and_distinct_from_singleton_at_all_rates() {
    let profile = profile();
    assert_ne!(
        profile.identity([1; 32]),
        profile.source().identity([1; 32])
    );
    for rate in [TickRate::Hz20, TickRate::Hz30, TickRate::Hz60] {
        for count in [0, 1, MAX_ACTORS] {
            let ids: Vec<_> = (1..=count as u64).collect();
            let value = frame(&profile, rate, &ids, 10);
            let bytes = encode(&profile, rate, &value).unwrap();
            assert_eq!(bytes.len(), HEADER_BYTES + count * 33);
            assert!(!super::super::is_state(&bytes));
            assert_eq!(decode(&profile, key(1).epoch, rate, &bytes).unwrap(), value);
        }
    }
}
#[test]
fn malformed_or_foreign_frames_never_change_current_presentation() {
    let profile = profile();
    let rate = TickRate::Hz30;
    let initial = frame(&profile, rate, &[1, 3], 10);
    let bytes = encode(&profile, rate, &initial).unwrap();
    let mut state = PopulationState::new(profile.clone(), key(1).epoch, rate, [1; 32]).unwrap();
    assert_eq!(state.apply(&bytes), Ok(Applied::Changed));
    for end in 0..bytes.len() {
        assert!(state.apply(&bytes[..end]).is_err());
        assert_eq!(state.current(), Some(&initial));
    }
    for (offset, value) in [
        (0, b'X'),
        (4, 60),
        (5, bytes[5] ^ 128),
        (37, 8),
        (53, 17),
        (54, 0),
        (62, 2),
        (63, 1),
        (87, 1),
    ] {
        let mut bad = bytes.clone();
        bad[offset] = value;
        assert!(state.apply(&bad).is_err(), "offset {offset}");
        assert_eq!(state.current(), Some(&initial));
    }
    let mut extra = bytes.clone();
    extra.push(0);
    assert!(state.apply(&extra).is_err());
    let mut unsorted = initial.clone();
    unsorted.actors.reverse();
    assert!(encode(&profile, rate, &unsorted).is_err());
    let mut oversized = frame(&profile, rate, &(1..=17).collect::<Vec<_>>(), 10);
    assert!(encode(&profile, rate, &oversized).is_err());
    oversized.actors.clear();
    oversized.epoch = WorldEpoch::new(8).unwrap();
    assert!(
        state
            .apply(&encode(&profile, rate, &oversized).unwrap())
            .is_err()
    );
    assert_eq!(state.current(), Some(&initial));
}
#[test]
fn replacement_and_empty_frames_retire_windups_without_resurrecting_old_entities() {
    let profile = profile();
    for rate in [TickRate::Hz20, TickRate::Hz30, TickRate::Hz60] {
        let mut state = PopulationState::new(profile.clone(), key(1).epoch, rate, [1; 32]).unwrap();
        let original = frame(&profile, rate, &[1, 3], 10);
        let old_bytes = encode(&profile, rate, &original).unwrap();
        assert_eq!(state.apply(&old_bytes), Ok(Applied::Changed));
        assert_eq!(state.apply(&old_bytes), Ok(Applied::Duplicate));
        let replacement = frame(&profile, rate, &[3, 4], 11);
        let bytes = encode(&profile, rate, &replacement).unwrap();
        assert_eq!(state.apply(&bytes), Ok(Applied::Changed));
        assert!(state.active_at(key(1), 11).is_none());
        assert!(state.active_at(key(3), 11).is_some());
        assert_eq!(state.apply(&old_bytes), Ok(Applied::Stale));
        let revived = frame(&profile, rate, &[1, 3, 4], 12);
        assert!(
            state
                .apply(&encode(&profile, rate, &revived).unwrap())
                .is_err()
        );
        assert_eq!(state.current(), Some(&replacement));
        let mut conflict = replacement.clone();
        conflict.actors[0].windup = None;
        assert!(
            state
                .apply(&encode(&profile, rate, &conflict).unwrap())
                .is_err()
        );
        let expires = replacement.actors[0].windup.unwrap().completes;
        assert!(state.active_at(key(3), expires).is_none());
        let empty = frame(&profile, rate, &[], 12);
        assert_eq!(
            state.apply(&encode(&profile, rate, &empty).unwrap()),
            Ok(Applied::Changed)
        );
        assert!(state.active_at(key(4), 12).is_none());
        assert!(
            state
                .apply(&encode(&profile, rate, &frame(&profile, rate, &[3], 13)).unwrap())
                .is_err()
        );
        assert_eq!(
            state.apply(&encode(&profile, rate, &frame(&profile, rate, &[5], 13)).unwrap()),
            Ok(Applied::Changed)
        );
        assert_eq!(state.world(), [1; 32]);
    }
    assert!(PopulationState::new(profile, key(1).epoch, TickRate::Hz30, [0; 32]).is_err());
}
