use serde_json::{Value, json};
use theia_core::{
    content::ContentId,
    coordinates::{GlobalPosition, LocalFrame},
    quests::areas::{self, Catalog},
};
fn raw() -> Value {
    json!({"schema":1,"id":"test.areas","revision":1,"areas":[
        {"id":"test.overlook","name":"Overlook","min":[128.,-8.,128.],"max":[144.,8.,144.]}
    ]})
}
fn parse(value: &Value) -> Result<Catalog, &'static str> {
    Catalog::from_json(&serde_json::to_vec(value).unwrap())
}
fn p(position: [f64; 3]) -> GlobalPosition {
    GlobalPosition::new(position).unwrap()
}
#[test]
fn exact_faces_corners_and_height_use_checked_global_metres() {
    let c = parse(&raw()).unwrap();
    let area = c.areas().next().unwrap();
    for point in [
        [128., -8., 128.],
        [144., 8., 144.],
        [136., 0., 136.],
        [128., 0., 136.],
        [144., 0., 136.],
        [136., -8., 136.],
        [136., 8., 136.],
        [136., 0., 128.],
        [136., 0., 144.],
    ] {
        assert!(area.contains(p(point)), "{point:?}");
    }
    for point in [
        [127.999, 0., 136.],
        [144.001, 0., 136.],
        [136., -8.001, 136.],
        [136., 8.001, 136.],
        [136., 0., 127.999],
        [136., 0., 144.001],
    ] {
        assert!(!area.contains(p(point)), "{point:?}");
    }
    assert_eq!(area.min(), p([128., -8., 128.]));
    assert_eq!(area.max(), p([144., 8., 144.]));
    assert_eq!(area.name(), "Overlook");
    assert_eq!(c.id().as_str(), "test.areas");
    assert_eq!(c.revision(), 1);
}
#[test]
fn submetre_global_boundaries_survive_different_render_origins_without_f32_roundtrip() {
    let mut value = raw();
    let origin = 1e12 - 1024.;
    value["areas"][0]["min"] = json!([origin + 0.125, -2., origin + 0.125]);
    value["areas"][0]["max"] = json!([origin + 4.125, 2., origin + 4.125]);
    let c = parse(&value).unwrap();
    let area = c.areas().next().unwrap();
    let inside = p([origin + 0.125, 0., origin + 0.125]);
    let outside = p([origin + 0.124, 0., origin + 0.125]);
    assert!(area.contains(inside));
    assert!(!area.contains(outside));
    for offset in [0., 128., 512.] {
        let frame = LocalFrame::new(p([origin - offset, 0., origin - offset]));
        let restored = frame.to_global(frame.to_local(inside).unwrap()).unwrap();
        assert_eq!(restored, inside);
        assert!(area.contains(restored));
    }
}
#[test]
fn invalid_schema_names_ids_fields_and_coordinate_extents_refuse() {
    for (field, value) in [
        ("schema", json!(2)),
        ("revision", json!(0)),
        ("id", json!("BAD ID")),
        ("unexpected", json!(true)),
    ] {
        let mut data = raw();
        data[field] = value;
        assert!(parse(&data).is_err());
    }
    for (field, value) in [
        ("name", json!("")),
        ("name", json!(" padded ")),
        ("name", json!("control\n")),
        ("name", json!("x".repeat(97))),
        ("id", json!("test.areas")),
        ("id", json!("Bad")),
        ("min", json!([128., 8., 128.])),
        ("min", json!([144., -8., 128.])),
        ("max", json!([127., 8., 144.])),
        ("max", json!([128.5, 8., 144.])),
        ("max", json!([128. + 1024.001, 8., 144.])),
        ("max", json!([1e12 + 1., 8., 144.])),
        ("max", json!([144., 8.])),
        ("min", Value::Null),
        ("extra", json!(1)),
    ] {
        let mut data = raw();
        data["areas"][0][field] = value;
        assert!(parse(&data).is_err(), "{field}={data}");
    }
    let duplicate = serde_json::to_string(&raw()).unwrap().replacen(
        "\"schema\":1",
        "\"schema\":1,\"schema\":1",
        1,
    );
    assert!(Catalog::from_json(duplicate.as_bytes()).is_err());
}
#[test]
fn area_and_byte_limits_are_exact_development_bounds() {
    let mut data = raw();
    let original = data["areas"][0].clone();
    data["areas"] = json!(
        (0..areas::MAX_AREAS)
            .map(|n| {
                let mut area = original.clone();
                area["id"] = json!(format!("test.area-{n}"));
                area
            })
            .collect::<Vec<_>>()
    );
    assert_eq!(parse(&data).unwrap().areas().count(), areas::MAX_AREAS);
    data["areas"]
        .as_array_mut()
        .unwrap()
        .push(json!({"id":"test.extra","name":"Extra","min":[0.,0.,0.],"max":[1.,1.,1.]}));
    assert!(parse(&data).is_err());
    data["areas"] = json!([]);
    assert!(parse(&data).is_err());
    let mut bytes = serde_json::to_vec(&raw()).unwrap();
    bytes.resize(areas::MAX_CATALOG_BYTES, b' ');
    assert!(Catalog::from_json(&bytes).is_ok());
    bytes.push(b' ');
    assert!(Catalog::from_json(&bytes).is_err());
    let mut extremes = raw();
    extremes["areas"][0]["min"] = json!([0., 0., 0.]);
    extremes["areas"][0]["max"] = json!([1., 1024., 1.]);
    assert!(parse(&extremes).is_ok());
}
#[test]
fn overlapping_areas_are_independent_and_source_order_is_not_membership_order() {
    let mut data = raw();
    let mut second = data["areas"][0].clone();
    second["id"] = json!("test.another");
    data["areas"].as_array_mut().unwrap().push(second);
    let c = parse(&data).unwrap();
    let names = c
        .containing(p([136., 0., 136.]))
        .map(|a| a.id().as_str())
        .collect::<Vec<_>>();
    assert_eq!(names, ["test.another", "test.overlook"]);
    data["areas"].as_array_mut().unwrap().reverse();
    assert_eq!(parse(&data).unwrap(), c);
    assert!(c.area(&ContentId::new("test.missing").unwrap()).is_none());
    data["areas"][1] = data["areas"][0].clone();
    assert!(parse(&data).is_err());
}
#[test]
fn development_fixture_is_geometry_only_and_not_loaded_by_normal_quest_catalogues() {
    let catalog = Catalog::from_json(include_bytes!(
        "../../../content/quests/development-quest-areas.json"
    ))
    .unwrap();
    let area = catalog
        .area(&ContentId::new("theia.dev.area.western-overlook").unwrap())
        .unwrap();
    assert!(area.contains(p([7864., 0., 8000.])));
    assert!(!area.contains(p([8002., 0., 8002.])));
    assert!(
        theia_core::quests::catalog::Catalog::from_json(include_bytes!(
            "../../../content/quests/development-quest-areas.json"
        ))
        .is_err()
    );
}
