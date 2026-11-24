use super::*;
use crate::inventory::{Bag, Command, Operation, Stack};
use serde_json::{Value, json};
fn id(s: &str) -> ContentId {
    ContentId::new(s).unwrap()
}
fn items() -> inventory::Catalog {
    inventory::Catalog::from_json(br#"{"schema":1,"id":"items","items":[{"id":"note-item","name":"Note","maximum_stack":20},{"id":"other-item","name":"Other","maximum_stack":20}]}"#).unwrap()
}
fn source() -> Value {
    json!({"schema":1,"id":"documents","revision":1,"item_catalog":"items","documents":[
        {"id":"note","item":"note-item","title":"A roadside note","pages":["The eastern road is quiet.\nBring supplies.","Élan and the river."]}
    ]})
}
fn parse(v: &Value) -> Result<Catalog, &'static str> {
    Catalog::from_json(&serde_json::to_vec(v).unwrap(), &items())
}
fn bag(items: &inventory::Catalog) -> Bag {
    let mut slots = std::array::from_fn(|_| None);
    slots[0] = Stack::new(items, id("note-item"), 3);
    slots[1] = Stack::new(items, id("other-item"), 1);
    Bag::from_slots(items, 0, slots).unwrap()
}
#[test]
fn ordered_plain_text_maps_to_known_item_and_resolution_does_not_mutate_bag() {
    let catalog = parse(&source()).unwrap();
    assert_eq!(catalog.id(), &id("documents"));
    assert_eq!(catalog.item_catalog(), &id("items"));
    assert_eq!(catalog.revision(), 1);
    assert_eq!(catalog.documents().len(), 1);
    let document = catalog.for_item(&id("note-item")).unwrap();
    assert_eq!(document.id(), &id("note"));
    assert_eq!(document.title(), "A roadside note");
    assert_eq!(
        document.pages(),
        [
            "The eastern road is quiet.\nBring supplies.",
            "Élan and the river."
        ]
    );
    assert!(catalog.for_item(&id("other-item")).is_none());
    let bag = bag(&items());
    let before = bag.clone();
    assert_eq!(catalog.resolve_owned(&bag, 0, 0, &id("note")), Ok(document));
    assert_eq!(bag, before);
}
#[test]
fn missing_forged_moved_or_destroyed_item_cannot_resolve_as_selected_document() {
    let items = items();
    let catalog = parse(&source()).unwrap();
    let original = bag(&items);
    assert_eq!(
        catalog.resolve_owned(&original, 1, 0, &id("note")),
        Err(OpenError::StaleRevision)
    );
    assert_eq!(
        catalog.resolve_owned(&original, 0, 16, &id("note")),
        Err(OpenError::InvalidSlot)
    );
    assert_eq!(
        catalog.resolve_owned(&original, 0, 2, &id("note")),
        Err(OpenError::MissingItem)
    );
    assert_eq!(
        catalog.resolve_owned(&original, 0, 1, &id("note")),
        Err(OpenError::WrongItem)
    );
    assert_eq!(
        catalog.resolve_owned(&original, 0, 0, &id("forged")),
        Err(OpenError::UnknownDocument)
    );
    let moved = original
        .propose(
            &items,
            Command {
                expected_revision: 0,
                operation: Operation::Move { from: 0, to: 2 },
            },
        )
        .unwrap();
    assert_eq!(
        catalog.resolve_owned(&moved, 0, 0, &id("note")),
        Err(OpenError::StaleRevision)
    );
    assert_eq!(
        catalog.resolve_owned(&moved, 1, 0, &id("note")),
        Err(OpenError::MissingItem)
    );
    assert!(catalog.resolve_owned(&moved, 1, 2, &id("note")).is_ok());
    let removed = moved
        .propose(
            &items,
            Command {
                expected_revision: 1,
                operation: Operation::Destroy { from: 2, amount: 3 },
            },
        )
        .unwrap();
    assert_eq!(
        catalog.resolve_owned(&removed, 2, 2, &id("note")),
        Err(OpenError::MissingItem)
    );
    let mut foreign:Value=serde_json::from_slice(br#"{"schema":1,"id":"foreign","items":[{"id":"note-item","name":"Note","maximum_stack":20}]}"#).unwrap();
    let wrong = inventory::Catalog::from_json(&serde_json::to_vec(&foreign).unwrap()).unwrap();
    assert_eq!(
        catalog.resolve_owned(&Bag::empty(&wrong), 0, 0, &id("note")),
        Err(OpenError::InvalidBag)
    );
    // Even the same catalogue ID cannot smuggle a foreign item into the bag.
    foreign["id"] = json!("items");
    foreign["items"][0]["id"] = json!("foreign-item");
    let wrong = inventory::Catalog::from_json(&serde_json::to_vec(&foreign).unwrap()).unwrap();
    let mut slots = std::array::from_fn(|_| None);
    slots[0] = Stack::new(&wrong, id("foreign-item"), 1);
    let wrong_bag = Bag::from_slots(&wrong, 0, slots).unwrap();
    assert_eq!(
        catalog.resolve_owned(&wrong_bag, 0, 0, &id("note")),
        Err(OpenError::InvalidBag)
    );
}
#[test]
fn split_changes_request_revision_but_each_retained_stack_resolves_the_same_document() {
    let items = items();
    let catalog = parse(&source()).unwrap();
    let split = bag(&items)
        .propose(
            &items,
            Command {
                expected_revision: 0,
                operation: Operation::Split {
                    from: 0,
                    to: 15,
                    amount: 1,
                },
            },
        )
        .unwrap();
    assert_eq!(
        catalog.resolve_owned(&split, 0, 0, &id("note")),
        Err(OpenError::StaleRevision)
    );
    assert_eq!(
        catalog.resolve_owned(&split, 1, 0, &id("note")),
        catalog.resolve_owned(&split, 1, 15, &id("note"))
    );
}
#[test]
fn schemas_unknown_fields_aliases_and_ambiguous_item_mapping_refuse() {
    for (field, value) in [
        ("schema", json!(2)),
        ("revision", json!(0)),
        ("item_catalog", json!("other")),
        ("id", json!("items")),
        ("id", json!("note-item")),
        ("unknown", json!(true)),
        ("documents", json!([])),
    ] {
        let mut v = source();
        v[field] = value;
        assert!(parse(&v).is_err(), "{field}");
    }
    for (field, value) in [
        ("id", json!("documents")),
        ("id", json!("note-item")),
        ("item", json!("missing-item")),
        ("unknown", json!(true)),
    ] {
        let mut v = source();
        v["documents"][0][field] = value;
        assert!(parse(&v).is_err(), "{field}");
    }
    for same_id in [false, true] {
        let mut v = source();
        let mut second = v["documents"][0].clone();
        if !same_id {
            second["id"] = json!("other-note");
        }
        v["documents"].as_array_mut().unwrap().push(second);
        assert!(parse(&v).is_err());
    }
}
#[test]
fn text_pages_and_input_bytes_have_explicit_utf8_budgets() {
    for title in [
        "",
        " padded",
        "line\nbreak",
        &"t".repeat(MAX_TITLE_BYTES + 1),
    ] {
        let mut v = source();
        v["documents"][0]["title"] = json!(title);
        assert!(parse(&v).is_err());
    }
    for pages in [
        json!([]),
        json!([" "]),
        json!(["bad\u{0}text"]),
        json!(["bad\u{1b}text"]),
        json!(["é".repeat(MAX_PAGE_BYTES / 2 + 1)]),
        json!(vec!["page"; MAX_PAGES + 1]),
    ] {
        let mut v = source();
        v["documents"][0]["pages"] = pages;
        assert!(parse(&v).is_err());
    }
    let mut v = source();
    v["documents"][0]["title"] = json!("t".repeat(MAX_TITLE_BYTES));
    v["documents"][0]["pages"] = json!(vec!["é".repeat(MAX_PAGE_BYTES / 2); MAX_PAGES]);
    assert_eq!(
        parse(&v)
            .unwrap()
            .document(&id("note"))
            .unwrap()
            .pages()
            .len(),
        MAX_PAGES
    );
    let mut bytes = serde_json::to_vec(&source()).unwrap();
    bytes.resize(MAX_CATALOG_BYTES + 1, b' ');
    assert!(Catalog::from_json(&bytes, &items()).is_err());
    assert!(Catalog::from_json(b"{", &items()).is_err());
    let mut windows = source();
    windows["documents"][0]["pages"] = json!(["First line.\r\nSecond line.\tEnd."]);
    assert_eq!(
        parse(&windows)
            .unwrap()
            .document(&id("note"))
            .unwrap()
            .pages()[0],
        "First line.\r\nSecond line.\tEnd."
    );
}
#[test]
fn document_count_is_bounded_and_lookup_order_is_stable() {
    let raw_items = json!({"schema":1,"id":"items","items":(0..MAX_DOCUMENTS+1).map(|n|json!({"id":format!("item-{n}"),"name":"Note","maximum_stack":1})).collect::<Vec<_>>()});
    let items = inventory::Catalog::from_json(&serde_json::to_vec(&raw_items).unwrap()).unwrap();
    let mut raw = source();
    raw["documents"]=json!((0..MAX_DOCUMENTS).rev().map(|n|json!({"id":format!("note-{n:02}"),"item":format!("item-{n}"),"title":"Note","pages":["Text"]})).collect::<Vec<_>>());
    let catalog = Catalog::from_json(&serde_json::to_vec(&raw).unwrap(), &items).unwrap();
    assert_eq!(catalog.documents().len(), MAX_DOCUMENTS);
    assert_eq!(catalog.documents().next().unwrap().id(), &id("note-00"));
    raw["documents"].as_array_mut().unwrap().push(json!({"id":"extra","item":format!("item-{MAX_DOCUMENTS}"),"title":"Note","pages":["Text"]}));
    assert!(Catalog::from_json(&serde_json::to_vec(&raw).unwrap(), &items).is_err());
}
