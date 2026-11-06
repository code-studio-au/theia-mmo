//! Static quest search geography, separate from live actor observation and credit.
use super::projection::{PIXELS, RADIUS, View};
use std::collections::BTreeSet;
use theia_core::quests::{areas::Area, catalog::EventKey, journal::Status};
use theia_network::{client::State, quest_journal::Profile};

/// Only current owner-bound, validated progress can select authored regions.
/// Reach volumes and explicit objective links provide authored search geography;
/// current creature positions are not spawn territories or loot guarantees.
pub(super) fn regions<'a>(state: &State, profile: &'a Profile) -> Vec<&'a Area> {
    let (Some(welcome), Some(view), Some(areas)) =
        (&state.welcome, &state.quest_journal, profile.area_catalog())
    else {
        return Vec::new();
    };
    if state.error.is_some()
        || state.id != Some(welcome.id)
        || view.actor.entity.value() != welcome.id
        || view.actor.epoch.value() != welcome.epoch
        || view.profile_hash != profile.hash()
        || view.validate_successor(profile, view.actor, None).is_err()
    {
        return Vec::new();
    }
    let mut ids = BTreeSet::new();
    for quest in profile.catalog().quests() {
        let Some(progress) = view
            .journal
            .progress(quest.id())
            .filter(|p| p.status() == Status::Active)
        else {
            continue;
        };
        let Some(stage) = quest.stages().find(|stage| stage.id() == progress.stage()) else {
            continue;
        };
        for objective in stage.objectives() {
            if progress.count(objective.id()).unwrap_or(0) >= objective.required() {
                continue;
            }
            if let EventKey::ReachArea(area) = objective.event() {
                ids.insert(area);
            }
            for area in areas.areas() {
                if area.map_objectives().contains(objective.id()) {
                    ids.insert(area.id());
                }
            }
        }
    }
    // The validated bound catalogue contains at most MAX_AREAS distinct regions.
    ids.into_iter().filter_map(|id| areas.area(id)).collect()
}

/// Tint the fresh existing 64-square terrain raster. This adds no image, camera,
/// worker, actor location, tile request or quest mutation. X/Z projection ignores
/// the volume's vertical occupancy rules, which remain authoritative gameplay.
pub(super) fn shade(data: &mut [u8], view: View, regions: &[&Area], circular: bool) -> usize {
    if data.len() != PIXELS * PIXELS * 4 {
        return 0;
    }
    let centre = view.centre.metres();
    let boxes: Vec<_> = regions
        .iter()
        .map(|area| {
            let min = area.min().metres();
            let max = area.max().metres();
            [
                min[0] - centre[0],
                max[0] - centre[0],
                min[2] - centre[2],
                max[2] - centre[2],
            ]
        })
        .filter(|b| b[0] <= RADIUS && b[1] >= -RADIUS && b[2] <= RADIUS && b[3] >= -RADIUS)
        .collect();
    if boxes.is_empty() {
        return 0;
    }
    let offset = |v: usize| ((v as f64 + 0.5) / PIXELS as f64 * 2. - 1.) * RADIUS;
    let mut tinted = 0;
    for z in 0..PIXELS {
        let dz = offset(z);
        for x in 0..PIXELS {
            let dx = offset(x);
            if circular && dx * dx + dz * dz > RADIUS * RADIUS {
                continue;
            }
            if boxes
                .iter()
                .any(|b| b[0] <= dx && dx <= b[1] && b[2] <= dz && dz <= b[3])
            {
                let pixel = &mut data[(z * PIXELS + x) * 4..][..4];
                // A restrained blue search-area wash; this is Theia presentation,
                // not evidence of a creature's presence or guaranteed loot.
                for (channel, tint) in pixel[..3].iter_mut().zip([40u16, 110, 210]) {
                    *channel = ((u16::from(*channel) * 3 + tint) / 4) as u8;
                }
                tinted += 1;
            }
        }
    }
    tinted
}

#[cfg(test)]
mod tests {
    use super::*;
    use theia_core::{
        content::ContentId,
        coordinates::GlobalPosition,
        quests::{areas::Catalog, journal::Journal},
    };
    fn id(s: &str) -> ContentId {
        ContentId::new(s).unwrap()
    }
    fn fixture() -> (State, Profile) {
        let xp = theia_network::progression_wire::Profile::from_json(include_bytes!(
            "../../../../content/gameplay/development-progression.json"
        ))
        .unwrap();
        let profile = Profile::from_json_with_areas(
            include_bytes!("../../../../content/quests/development-journal-areas.json"),
            include_bytes!("../../../../content/quests/development-quest-areas.json"),
            &xp,
        )
        .unwrap();
        let (mut state, _) = crate::quest_journal::tests::fixture();
        let view = state.quest_journal.as_mut().unwrap();
        view.profile_hash = profile.hash();
        view.revision = 1;
        view.journal = Journal::default()
            .propose_accept(
                profile.catalog(),
                &id("theia.dev.quest-overlook"),
                &id("theia.dev.guide"),
            )
            .unwrap()
            .journal;
        (state, profile)
    }
    #[test]
    fn regions_follow_active_goals_and_refuse_foreign_or_invalid_context() {
        let (state, profile) = fixture();
        assert_eq!(regions(&state, &profile).len(), 1);
        let mut without_guide = state.clone();
        without_guide.guide = None;
        assert_eq!(
            regions(&without_guide, &profile).len(),
            1,
            "Authored search geography does not reveal live NPCs"
        );
        for case in 0..7 {
            let mut bad = state.clone();
            match case {
                0 => bad.error = Some("disconnected".into()),
                1 => bad.welcome = None,
                2 => bad.id = None,
                3 => bad.welcome.as_mut().unwrap().epoch += 1,
                4 => bad.welcome.as_mut().unwrap().id += 1,
                5 => bad.quest_journal.as_mut().unwrap().profile_hash[0] ^= 1,
                _ => bad.quest_journal.as_mut().unwrap().revision = 0,
            }
            assert!(regions(&bad, &profile).is_empty(), "case {case}");
        }
        let mut empty = state.clone();
        empty.quest_journal.as_mut().unwrap().journal = Journal::default();
        assert!(regions(&empty, &profile).is_empty());
        let mut ready = state.clone();
        let view = ready.quest_journal.as_mut().unwrap();
        let observation = view
            .journal
            .prepare_reached_areas(
                profile.catalog(),
                &BTreeSet::from([id("theia.dev.area.western-overlook")]),
            )
            .unwrap();
        view.journal = view
            .journal
            .propose_reached_areas(profile.catalog(), &observation)
            .unwrap()
            .journal;
        view.revision = 2;
        assert!(regions(&ready, &profile).is_empty());
        let view = ready.quest_journal.as_mut().unwrap();
        view.journal = view
            .journal
            .propose_turn_in(
                profile.catalog(),
                &id("theia.dev.quest-overlook"),
                &id("theia.dev.guide"),
                2,
            )
            .unwrap()
            .journal;
        view.revision = 3;
        assert!(regions(&ready, &profile).is_empty());
    }
    #[test]
    fn collection_search_regions_reappear_after_confirmed_item_loss() {
        use theia_core::inventory::{Bag, Stack};
        let xp = theia_network::progression_wire::Profile::from_json(include_bytes!(
            "../../../../content/gameplay/development-progression.json"
        ))
        .unwrap();
        let inventory = theia_network::renet_runtime::inventory::Profile::from_json(
            include_bytes!("../../../../content/gameplay/development-document-items.json"),
        )
        .unwrap();
        let mut q: serde_json::Value = serde_json::from_slice(include_bytes!(
            "../../../../content/quests/development-journal-items.json"
        ))
        .unwrap();
        let mut quest = q["quests"].as_array_mut().unwrap().pop().unwrap();
        quest["prerequisites"] = serde_json::json!([]);
        q["quests"] = serde_json::json!([quest]);
        let a = serde_json::json!({"schema":1,"id":"theia.test.collection-map","revision":1,"areas":[
            {"id":"theia.test.collection-region","name":"Collection search","min":[7900.,-10.,7900.],"max":[7950.,10.,7950.],
             "map_objectives":["theia.dev.quest-wolf-pelts.objective.pelts"]},
            {"id":"theia.test.collection-region-two","name":"Second search","min":[7960.,-10.,7900.],"max":[7980.,10.,7950.],
             "map_objectives":["theia.dev.quest-wolf-pelts.objective.pelts"]}
        ]});
        let profile = Profile::from_json_with_areas(
            &serde_json::to_vec(&q).unwrap(),
            &serde_json::to_vec(&a).unwrap(),
            &xp,
        )
        .unwrap()
        .with_inventory(&inventory)
        .unwrap();
        let (mut state, _) = crate::quest_journal::tests::fixture();
        let view = state.quest_journal.as_mut().unwrap();
        view.profile_hash = profile.hash();
        view.journal = Journal::default()
            .propose_accept(
                profile.catalog(),
                &id("theia.dev.quest-wolf-pelts"),
                &id("theia.dev.guide"),
            )
            .unwrap()
            .journal;
        view.revision = 1;
        assert_eq!(regions(&state, &profile).len(), 2);
        for (count, expected) in [(3, 0), (2, 2), (3, 0)] {
            let mut slots = std::array::from_fn(|_| None);
            slots[0] = Stack::new(inventory.catalog(), id("theia.dev.wolf-pelt"), count);
            let bag = Bag::from_slots(inventory.catalog(), 0, slots).unwrap();
            let view = state.quest_journal.as_mut().unwrap();
            view.journal = view
                .journal
                .propose_inventory(profile.catalog(), inventory.catalog(), &bag)
                .unwrap()
                .journal;
            view.revision = view
                .journal
                .progress(&id("theia.dev.quest-wolf-pelts"))
                .unwrap()
                .revision();
            assert_eq!(
                regions(&state, &profile).len(),
                expected,
                "confirmed holdings {count}"
            );
        }
    }
    #[test]
    fn shade_is_bounded_circular_and_independent_of_global_origin() {
        let catalogue = |origin: f64| {
            Catalog::from_json(&serde_json::to_vec(&serde_json::json!({
            "schema":1,"id":"theia.test.map-areas","revision":1,"areas":[
                {"id":"theia.test.region","name":"Search region","min":[origin-128.,-10.,origin-128.],"max":[origin+128.,10.,origin+128.]}
            ]})).unwrap()).unwrap()
        };
        let mut outputs = Vec::new();
        for origin in [0., 1_000_000_000.] {
            let areas = catalogue(origin);
            let region = areas.areas().next().unwrap();
            let mut data = [44, 62, 42, 255].repeat(PIXELS * PIXELS);
            let view = View {
                centre: GlobalPosition::new([origin, 0., origin]).unwrap(),
            };
            let count = shade(&mut data, view, &[region], true);
            assert!(count > 3000 && count < PIXELS * PIXELS);
            assert_eq!(
                &data[..4],
                &[44, 62, 42, 255],
                "Outside round map remains unchanged"
            );
            assert_ne!(&data[(32 * PIXELS + 32) * 4..][..3], &[44, 62, 42]);
            assert!(data.as_chunks::<4>().0.iter().all(|pixel| pixel[3] == 255));
            let mut fresh = [44, 62, 42, 255].repeat(PIXELS * PIXELS);
            assert_eq!(shade(&mut fresh, view, &[], true), 0);
            assert!(
                fresh
                    .as_chunks::<4>()
                    .0
                    .iter()
                    .all(|pixel| *pixel == [44, 62, 42, 255])
            );
            assert_eq!(shade(&mut [], view, &[region], true), 0);
            outputs.push(data);
        }
        assert_eq!(outputs[0], outputs[1]);
    }
    #[test]
    fn projected_area_clips_at_map_edge_and_does_not_tint_distant_regions() {
        let area = Catalog::from_json(
            &serde_json::to_vec(&serde_json::json!({
            "schema":1,"id":"theia.test.clip-areas","revision":1,"areas":[
                {"id":"theia.test.edge","name":"Edge","min":[126.,-1.,-8.],"max":[142.,1.,8.]}
            ]}))
            .unwrap(),
        )
        .unwrap();
        let region = area.areas().next().unwrap();
        let mut data = [44, 62, 42, 255].repeat(PIXELS * PIXELS);
        let view = View {
            centre: GlobalPosition::new([0., 0., 0.]).unwrap(),
        };
        assert_eq!(shade(&mut data, view, &[region], false), 4);
        let mut distant = [44, 62, 42, 255].repeat(PIXELS * PIXELS);
        assert_eq!(
            shade(
                &mut distant,
                View {
                    centre: GlobalPosition::new([1000., 0., 1000.]).unwrap()
                },
                &[region],
                true
            ),
            0
        );
        assert!(
            distant
                .as_chunks::<4>()
                .0
                .iter()
                .all(|p| p == &[44, 62, 42, 255])
        );
    }
    #[test]
    fn bounded_32_region_raster_cost_sample() {
        let raw: Vec<_> = (0..32).map(|i| {
            let x = f64::from(i % 8)*24.-96.; let z = f64::from(i / 8)*32.-64.;
            serde_json::json!({"id":format!("theia.test.search-{i}"),"name":"Search area","min":[x,-1.,z],"max":[x+8.,1.,z+8.]})
        }).collect();
        let areas = Catalog::from_json(&serde_json::to_vec(&serde_json::json!({"schema":1,"id":"theia.test.budget-areas","revision":1,"areas":raw})).unwrap()).unwrap();
        let regions: Vec<_> = areas.areas().collect();
        assert_eq!(regions.len(), theia_core::quests::areas::MAX_AREAS);
        let base = [44, 62, 42, 255].repeat(PIXELS * PIXELS);
        let view = View {
            centre: GlobalPosition::new([0., 0., 0.]).unwrap(),
        };
        let mut timings = Vec::new();
        for _ in 0..200 {
            let mut pixels = base.clone();
            let started = std::time::Instant::now();
            assert_eq!(shade(&mut pixels, view, &regions, true), 128);
            timings.push(started.elapsed().as_micros());
        }
        timings.sort_unstable();
        println!(
            "objective_overlay_cost: regions=32 samples=200 p95_us={} max_us={} (synthetic shader only, no FPS/capacity claim)",
            timings[189], timings[199]
        );
    }
}
