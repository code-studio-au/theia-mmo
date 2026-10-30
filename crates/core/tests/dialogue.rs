//! Pure informational branches; no interaction authority or session state.
use serde_json::{Value, json};
use theia_core::{
    content::ContentId,
    dialogue::{self, Error, Profile},
};
const DATA: &[u8] = include_bytes!("../../../content/gameplay/development-dialogue.json");
fn data() -> Value {
    serde_json::from_slice(DATA).unwrap()
}
fn load(value: &Value) -> Result<Profile, &'static str> {
    Profile::from_json(&serde_json::to_vec(value).unwrap())
}
fn id(value: &str) -> ContentId {
    ContentId::new(value).unwrap()
}

#[test]
fn branches_resolve_only_offered_choices_and_leave_content_immutable() {
    let profile = Profile::from_json(DATA).unwrap();
    let original = profile.clone();
    assert_eq!(profile.id().as_str(), "theia.dev.guide-dialogue");
    assert_eq!(profile.npc().as_str(), "theia.dev.guide");
    assert_eq!(profile.name(), "Development guide");
    assert!(!profile.start().text().is_empty());
    assert_eq!(profile.start().choices().len(), 2);
    for choice in profile.start().choices() {
        assert!(!choice.label().is_empty());
        let next = profile.choose(profile.start().id(), choice.id()).unwrap();
        assert_eq!(next.id(), choice.next());
        assert_eq!(profile.node(next.id()), Some(next));
        let terminal = profile.choose(next.id(), next.choices()[0].id()).unwrap();
        assert_eq!(terminal.id().as_str(), "theia.dev.guide.goodbye");
        assert!(terminal.choices().is_empty());
        assert_eq!(
            profile.choose(terminal.id(), choice.id()),
            Err(Error::ChoiceNotOffered)
        );
        assert_eq!(
            profile.choose(profile.start().id(), next.choices()[0].id()),
            Err(Error::ChoiceNotOffered)
        );
    }
    assert_eq!(profile.node(&id("missing")), None);
    assert_eq!(
        profile.choose(&id("missing"), &id("choice")),
        Err(Error::UnknownNode)
    );
    assert_eq!(profile, original);
}

#[test]
fn malformed_profiles_and_unsupported_effects_fail_before_navigation() {
    for bytes in [b"".as_slice(), b"{}", &DATA[..DATA.len() - 2]] {
        assert!(Profile::from_json(bytes).is_err());
    }
    for (key, value) in [
        ("schema", json!(2)),
        ("schema", json!(1.5)),
        ("id", json!("../dialogue")),
        ("npc", json!("Npc")),
        ("start", json!("../node")),
        ("nodes", json!([])),
        ("grant_xp", json!(100)),
    ] {
        let mut raw = data();
        raw[key] = value;
        assert!(load(&raw).is_err(), "{key}");
    }
    for (path, key) in [(0, "reward"), (1, "command")] {
        let mut raw = data();
        raw["nodes"][path][key] = json!("complete-quest");
        assert!(load(&raw).is_err());
    }
    let mut raw = data();
    raw["nodes"][0]["choices"][0]["effect"] = json!("purchase");
    assert!(load(&raw).is_err());
    let raw = String::from_utf8(DATA.to_vec()).unwrap();
    for duplicate in [
        raw.replacen("\"schema\": 1", "\"schema\": 1, \"schema\": 1", 1),
        raw.replacen(
            "\"label\": \"How do I move?\"",
            "\"label\": \"How do I move?\", \"label\": \"Other\"",
            1,
        ),
    ] {
        assert!(Profile::from_json(duplicate.as_bytes()).is_err());
    }
}

#[test]
fn duplicate_definitions_and_invalid_references_cannot_alias_content() {
    for (path, source) in [("npc", "theia.dev.guide-dialogue"), ("start", "missing")] {
        let mut raw = data();
        raw[path] = json!(source);
        assert!(load(&raw).is_err());
    }
    for alias in [
        "theia.dev.guide",
        "theia.dev.guide-dialogue",
        "theia.dev.guide.welcome",
    ] {
        let mut raw = data();
        raw["nodes"][1]["id"] = json!(alias);
        assert!(load(&raw).is_err());
        let mut raw = data();
        raw["nodes"][0]["choices"][0]["id"] = json!(alias);
        assert!(load(&raw).is_err());
    }
    let mut raw = data();
    raw["nodes"][1]["choices"][0]["id"] = raw["nodes"][0]["choices"][0]["id"].clone();
    assert!(load(&raw).is_err());
    for destination in ["missing", "theia.dev.guide", "../node"] {
        let mut raw = data();
        raw["nodes"][0]["choices"][0]["next"] = json!(destination);
        assert!(load(&raw).is_err());
    }
}

#[test]
fn unreachable_and_cyclic_graphs_reject_but_shared_terminal_branches_work() {
    assert!(load(&data()).is_ok());
    let mut raw = data();
    raw["nodes"]
        .as_array_mut()
        .unwrap()
        .push(json!({"id":"unreachable","text":"Unused.","choices":[]}));
    assert!(load(&raw).is_err());
    for node in [0, 1, 3] {
        let mut raw = data();
        raw["nodes"][node]["choices"] =
            json!([{"id":"loop-choice","label":"Loop","next":"theia.dev.guide.welcome"}]);
        assert!(load(&raw).is_err(), "cycle from node {node}");
    }
}

#[test]
fn text_and_total_bytes_are_bounded_without_truncating_utf8() {
    for (kind, maximum) in [
        (0, dialogue::MAX_NAME_BYTES),
        (1, dialogue::MAX_TEXT_BYTES),
        (2, dialogue::MAX_CHOICE_BYTES),
    ] {
        for value in [
            "".to_owned(),
            " leading".to_owned(),
            "trailing ".to_owned(),
            "line\nbreak".to_owned(),
            "tab\there".to_owned(),
            "nul\0byte".to_owned(),
            "é".repeat(maximum / 2 + 1),
        ] {
            let mut raw = data();
            match kind {
                0 => raw["name"] = json!(value),
                1 => raw["nodes"][0]["text"] = json!(value),
                _ => raw["nodes"][0]["choices"][0]["label"] = json!(value),
            }
            assert!(load(&raw).is_err(), "field {kind}");
        }
        let mut raw = data();
        let value = json!("é".repeat(maximum / 2));
        match kind {
            0 => raw["name"] = value,
            1 => raw["nodes"][0]["text"] = value,
            _ => raw["nodes"][0]["choices"][0]["label"] = value,
        }
        assert!(load(&raw).is_ok(), "exact UTF-8 byte limit {kind}");
    }
    let mut padded = DATA.to_vec();
    padded.resize(dialogue::MAX_PROFILE_BYTES, b' ');
    assert!(Profile::from_json(&padded).is_ok());
    padded.push(b' ');
    assert!(Profile::from_json(&padded).is_err());
}

#[test]
fn maximum_graph_and_choice_counts_have_finite_navigation() {
    let nodes: Vec<_> = (0..dialogue::MAX_NODES).map(|n| json!({
        "id":format!("node-{n}"),"text":"Bounded node.",
        "choices": if n + 1 == dialogue::MAX_NODES {vec![]} else {
            (0..dialogue::MAX_CHOICES).map(|c| json!({"id":format!("choice-{n}-{c}"),"label":"Continue","next":format!("node-{}",n+1)})).collect::<Vec<_>>()
        }
    })).collect();
    let raw = json!({"schema":1,"id":"dialogue","npc":"guide","name":"Guide","start":"node-0","nodes":nodes});
    let profile = load(&raw).unwrap();
    let mut node = profile.start();
    let mut steps = 1;
    while let Some(choice) = node.choices().first() {
        node = profile.choose(node.id(), choice.id()).unwrap();
        steps += 1;
    }
    assert_eq!(steps, dialogue::MAX_NODES);
    let mut excessive = raw.clone();
    excessive["nodes"]
        .as_array_mut()
        .unwrap()
        .push(json!({"id":"excess","text":"Excess.","choices":[]}));
    assert!(load(&excessive).is_err());
    let mut excessive = raw;
    excessive["nodes"][0]["choices"]
        .as_array_mut()
        .unwrap()
        .push(json!({"id":"extra-choice","label":"Excess","next":"node-1"}));
    assert!(load(&excessive).is_err());
    let terminal = json!({"schema":1,"id":"dialogue","npc":"guide","name":"Guide","start":"only","nodes":[{"id":"only","text":"Goodbye.","choices":[]}]});
    assert!(load(&terminal).unwrap().start().choices().is_empty());
}
