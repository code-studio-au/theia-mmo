use super::*;
use crate::identity::{EntityId, WorldEpoch};
fn key(epoch: u64, id: u64) -> EntityKey {
    EntityKey {
        epoch: WorldEpoch::new(epoch).unwrap(),
        entity: EntityId::new(id).unwrap(),
    }
}
fn speaker() -> Speaker {
    Speaker {
        key: key(7, 1),
        name: CharacterName::parse("Arden").unwrap(),
        position: GlobalPosition::new([1e9, 3., -1e9]).unwrap(),
        area: Area::new(1),
    }
}
fn request(sequence: u64, channel: Channel) -> Request {
    Request {
        sequence,
        channel,
        text: Text::parse("Hello nearby travellers 🙂").unwrap(),
    }
}
fn listener(offset: [f64; 3]) -> Listener {
    let origin = speaker().position.metres();
    Listener {
        key: key(7, 2),
        position: GlobalPosition::new(std::array::from_fn(|i| origin[i] + offset[i])).unwrap(),
        area: Area::new(1),
        blocks_sender: false,
    }
}
#[test]
fn unicode_text_bounds_preserve_plain_spelling_and_reject_control_line_spoofing() {
    for value in [
        "Hello",
        "  Hello  ",
        "你好 🐉",
        "👨‍👩‍👧‍👦",
        "<b>plain</b> |cffff0000text",
    ] {
        assert_eq!(Text::parse(value).unwrap().as_str(), value);
    }
    assert!(Text::parse(&"a".repeat(MAX_TEXT_BYTES)).is_ok());
    assert!(Text::parse(&"é".repeat(MAX_TEXT_BYTES / 2)).is_ok());
    assert_eq!(
        Text::parse(&"é".repeat(MAX_TEXT_BYTES / 2 + 1)),
        Err(TextError::Length)
    );
    for value in [
        "",
        "   ",
        "\u{200b}\u{200b}",
        "\u{3000}",
        "a\nb",
        "a\rb",
        "a\tb",
        "\0bad",
        "a\u{0085}b",
        "a\u{2028}b",
        "a\u{202e}b",
        "a\u{2066}b",
        "a\u{061c}b",
        "a\u{feff}b",
    ] {
        assert!(Text::parse(value).is_err(), "{value:?}");
    }
    assert!(!format!("{:?}", Text::parse("private message").unwrap()).contains("private"));
    assert!(Text::parse(&"x".repeat(100_000)).is_err());
}
#[test]
fn local_range_uses_global_metric_three_dimensional_positions_at_far_origins() {
    let mut gate = Gate::new(speaker().key, TickRate::Hz30);
    let message = gate
        .submit(speaker(), request(1, Channel::Say), 1, false)
        .unwrap();
    for offset in [
        [25., 0., 0.],
        [-25., 0., 0.],
        [0., 25., 0.],
        [0., 0., -25.],
        [15., 20., 0.],
    ] {
        assert!(message.reaches(listener(offset)));
    }
    for offset in [
        [25.001, 0., 0.],
        [0., 25.001, 0.],
        [25., 0., 0.001],
        [18., 18., 0.],
    ] {
        assert!(!message.reaches(listener(offset)), "{offset:?}");
    }
    let mut other = listener([0.; 3]);
    other.area = Area::new(99);
    assert!(message.reaches(other));
    other.key = key(8, 2);
    assert!(!message.reaches(other));
    other.key = key(7, 2);
    other.blocks_sender = true;
    assert!(!message.reaches(other));
}
#[test]
fn area_delivery_requires_matching_authority_membership_and_epoch() {
    assert!(Area::new(0).is_none());
    let mut gate = Gate::new(speaker().key, TickRate::Hz30);
    let message = gate
        .submit(speaker(), request(1, Channel::Area), 1, false)
        .unwrap();
    assert!(message.reaches(listener([1000., 500., -1000.])));
    let mut other = listener([0.; 3]);
    other.area = Area::new(2);
    assert!(!message.reaches(other));
    other.area = None;
    assert!(!message.reaches(other));
    let mut missing = speaker();
    missing.area = None;
    assert!(matches!(
        gate.submit(missing, request(2, Channel::Area), 2, false),
        Err(Refusal::NoArea)
    ));
    assert!(matches!(
        gate.submit(speaker(), request(2, Channel::Area), 3, false),
        Err(Refusal::Sequence)
    ));
}
#[test]
fn shared_channel_flood_budget_is_three_per_five_seconds_at_all_supported_rates() {
    for rate in [TickRate::Hz20, TickRate::Hz30, TickRate::Hz60] {
        let mut gate = Gate::new(speaker().key, rate);
        for seq in 1..=3 {
            assert!(
                gate.submit(
                    speaker(),
                    request(
                        seq,
                        if seq % 2 == 0 {
                            Channel::Area
                        } else {
                            Channel::Say
                        }
                    ),
                    1,
                    false
                )
                .is_ok()
            );
        }
        let expiry = 1 + u64::from(rate.hz()) * WINDOW_SECONDS;
        assert!(matches!(
            gate.submit(speaker(), request(4, Channel::Say), expiry - 1, false),
            Err(Refusal::RateLimited)
        ));
        assert!(
            gate.submit(speaker(), request(5, Channel::Say), expiry, false)
                .is_ok()
        );
        assert!(matches!(
            gate.submit(speaker(), request(4, Channel::Say), expiry, false),
            Err(Refusal::Sequence)
        ));
        assert!(
            gate.submit(speaker(), request(6, Channel::Say), expiry, false)
                .is_ok()
        );
        assert!(
            gate.submit(speaker(), request(7, Channel::Say), expiry, false)
                .is_ok()
        );
        assert!(matches!(
            gate.submit(speaker(), request(8, Channel::Say), expiry, false),
            Err(Refusal::RateLimited)
        ));
    }
}
#[test]
fn authority_context_time_replay_mute_and_exhaustion_fail_without_spending_accepted_capacity() {
    let mut gate = Gate::new(speaker().key, TickRate::Hz30);
    let mut forged = speaker();
    forged.key = key(7, 2);
    assert!(matches!(
        gate.submit(forged, request(1, Channel::Say), 1, false),
        Err(Refusal::Context)
    ));
    assert!(matches!(
        gate.submit(speaker(), request(1, Channel::Say), 0, false),
        Err(Refusal::Time)
    ));
    assert!(matches!(
        gate.submit(speaker(), request(129, Channel::Say), 1, false),
        Err(Refusal::Sequence)
    ));
    assert!(matches!(
        gate.submit(speaker(), request(0, Channel::Say), 1, false),
        Err(Refusal::Sequence)
    ));
    assert!(matches!(
        gate.submit(speaker(), request(1, Channel::Say), 1, true),
        Err(Refusal::Muted)
    ));
    assert!(matches!(
        gate.submit(speaker(), request(1, Channel::Say), 2, false),
        Err(Refusal::Sequence)
    ));
    assert!(
        gate.submit(speaker(), request(2, Channel::Say), 2, false)
            .is_ok()
    );
    assert!(matches!(
        gate.submit(speaker(), request(3, Channel::Say), 1, false),
        Err(Refusal::Time)
    ));
    assert!(
        gate.submit(speaker(), request(3, Channel::Say), 2, false)
            .is_ok()
    );
    assert!(
        gate.submit(speaker(), request(4, Channel::Say), 2, false)
            .is_ok()
    );
    gate.sequence = u64::MAX - 1;
    let message = gate
        .submit(speaker(), request(u64::MAX, Channel::Say), u64::MAX, false)
        .unwrap();
    assert_eq!(message.sequence, u64::MAX);
    assert_eq!(message.speaker.name.as_str(), "Arden");
    assert!(matches!(
        gate.submit(speaker(), request(0, Channel::Say), u64::MAX, false),
        Err(Refusal::Sequence)
    ));
}
