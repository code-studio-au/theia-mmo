//! Offline standard GLB + existing schema-4 world packages. No new runtime format.
use super::*;
use bevy::mesh::VertexAttributeValues;
use serde_json::{Value, json};
use std::{
    fs,
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};
use theia_network::{
    content::Manifest,
    terrain::{Terrain, sha256},
};
const MAX_VISUAL_TRIANGLES: usize = 262_144;
const MAX_EXPORT_BYTES: usize = 64 * 1024 * 1024;
const CYLINDER_ERROR_METRES: f64 = 0.001;

struct Glb {
    bytes: Vec<u8>,
    views: Vec<Value>,
    accessors: Vec<Value>,
    meshes: Vec<Value>,
    nodes: Vec<Value>,
    triangles: usize,
}
impl Glb {
    fn new() -> Self {
        Self {
            bytes: vec![],
            views: vec![],
            accessors: vec![],
            meshes: vec![],
            nodes: vec![],
            triangles: 0,
        }
    }
    fn view(&mut self, bytes: &[u8], target: u32) -> usize {
        let index = self.views.len();
        self.views.push(json!({"buffer":0,"byteOffset":self.bytes.len(),"byteLength":bytes.len(),"target":target}));
        self.bytes.extend(bytes);
        index
    }
    fn add(
        &mut self,
        name: &str,
        mesh: Mesh,
        transform: Transform,
        material: usize,
    ) -> Result<(), String> {
        let Some(VertexAttributeValues::Float32x3(points)) =
            mesh.attribute(Mesh::ATTRIBUTE_POSITION)
        else {
            return Err("Mesh missing positions".into());
        };
        let Some(VertexAttributeValues::Float32x3(normals)) =
            mesh.attribute(Mesh::ATTRIBUTE_NORMAL)
        else {
            return Err("Mesh missing normals".into());
        };
        let positions: Vec<_> = points
            .iter()
            .map(|p| transform.transform_point(Vec3::from_array(*p)).to_array())
            .collect();
        let normals: Vec<_> = normals
            .iter()
            .map(|n| {
                (transform.rotation * (Vec3::from_array(*n) / transform.scale))
                    .normalize()
                    .to_array()
            })
            .collect();
        let indices: Vec<u32> = mesh
            .indices()
            .ok_or("Mesh missing indices")?
            .iter()
            .map(|i| i as u32)
            .collect();
        self.triangles += indices.len() / 3;
        if self.triangles > MAX_VISUAL_TRIANGLES {
            return Err("Export visual triangle budget exceeded".into());
        }
        if positions
            .iter()
            .flatten()
            .any(|v| !v.is_finite() || v.abs() > 1023.)
        {
            return Err("Export mesh outside local frame".into());
        }
        let minimum = std::array::from_fn::<_, 3, _>(|i| {
            positions.iter().map(|p| p[i]).fold(f32::INFINITY, f32::min)
        });
        let maximum = std::array::from_fn::<_, 3, _>(|i| {
            positions
                .iter()
                .map(|p| p[i])
                .fold(f32::NEG_INFINITY, f32::max)
        });
        let start = self.accessors.len();
        for (values, bounds) in [(&positions, Some((minimum, maximum))), (&normals, None)] {
            let bytes: Vec<u8> = values
                .iter()
                .flatten()
                .flat_map(|v| v.to_le_bytes())
                .collect();
            let view = self.view(&bytes, 34962);
            let mut accessor =
                json!({"bufferView":view,"componentType":5126,"count":values.len(),"type":"VEC3"});
            if let Some((minimum, maximum)) = bounds {
                accessor["min"] = json!(minimum);
                accessor["max"] = json!(maximum);
            }
            self.accessors.push(accessor);
        }
        let colour_accessor = if let Some(attribute) = mesh.attribute(Mesh::ATTRIBUTE_COLOR) {
            let VertexAttributeValues::Float32x4(colours) = attribute else {
                return Err("Export colour attribute needs float RGBA".into());
            };
            if colours.len() != positions.len()
                || colours
                    .iter()
                    .flatten()
                    .any(|v| !v.is_finite() || !(0. ..=1.).contains(v))
            {
                return Err("Invalid export vertex colours".into());
            }
            let bytes: Vec<u8> = colours
                .iter()
                .flatten()
                .flat_map(|v| v.to_le_bytes())
                .collect();
            let view = self.view(&bytes, 34962);
            let index = self.accessors.len();
            self.accessors.push(
                json!({"bufferView":view,"componentType":5126,"count":colours.len(),"type":"VEC4"}),
            );
            Some(index)
        } else {
            None
        };
        let index_accessor = self.accessors.len();
        let bytes: Vec<u8> = indices.iter().flat_map(|v| v.to_le_bytes()).collect();
        let view = self.view(&bytes, 34963);
        self.accessors.push(
            json!({"bufferView":view,"componentType":5125,"count":indices.len(),"type":"SCALAR"}),
        );
        let index = self.meshes.len();
        let mut attributes = json!({"POSITION":start,"NORMAL":start+1});
        if let Some(index) = colour_accessor {
            attributes["COLOR_0"] = json!(index);
        }
        self.meshes.push(json!({"name":name,"primitives":[{"attributes":attributes,"indices":index_accessor,"material":material}]}));
        self.nodes.push(json!({"name":name,"mesh":index}));
        Ok(())
    }
    fn finish(
        self,
        guide_kinds: [bool; 2],
        plants: bool,
        painted: bool,
    ) -> Result<Vec<u8>, String> {
        let material = |name: &str, color: [f32; 4]| json!({"name":name,"pbrMetallicRoughness":{"baseColorFactor":color,"metallicFactor":0,"roughnessFactor":0.5}});
        // Factors are linear, matching the standard Bevy srgb materials.
        let color = |r, g, b| Color::srgb(r, g, b).to_linear().to_f32_array();
        let mut materials = vec![
            material("Ground", color(0.18, 0.24, 0.20)),
            material("Greybox", color(0.52, 0.60, 0.66)),
        ];
        if guide_kinds[0] {
            materials.push(material("Road guide", color(0.48, 0.31, 0.15)));
        }
        if guide_kinds[1] {
            let mut rgba = color(0.04, 0.36, 0.75);
            rgba[3] = 0.8;
            let mut stream = material("Stream guide", rgba);
            stream["alphaMode"] = json!("BLEND");
            stream["extensions"] = json!({"KHR_materials_unlit":{}});
            materials.push(stream);
        }
        if plants {
            materials.push(material("Vegetation trunk", color(0.3, 0.16, 0.07)));
            materials.push(material("Vegetation leaves", color(0.16, 0.39, 0.13)));
        }
        if painted {
            materials.push(material("Painted terrain", [1.; 4]));
        }
        let mut document = json!({"asset":{"version":"2.0","generator":"Theia Bevy zone exporter"},
            "scene":0,"scenes":[{"nodes":(0..self.nodes.len()).collect::<Vec<_>>()}],"nodes":self.nodes,"meshes":self.meshes,
            "materials":materials,
            "buffers":[{"byteLength":self.bytes.len()}],"bufferViews":self.views,"accessors":self.accessors});
        if guide_kinds[1] {
            document["extensionsUsed"] = json!(["KHR_materials_unlit"]);
        }
        let mut document = serde_json::to_vec(&document).map_err(|e| e.to_string())?;
        while !document.len().is_multiple_of(4) {
            document.push(b' ');
        }
        let length = 12 + 8 + document.len() + 8 + self.bytes.len();
        if length > MAX_EXPORT_BYTES {
            return Err("Export GLB byte budget exceeded".into());
        }
        let mut glb = Vec::with_capacity(length);
        for value in [
            0x46546c67_u32,
            2,
            length as u32,
            document.len() as u32,
            0x4e4f534a,
        ] {
            glb.extend(value.to_le_bytes());
        }
        glb.extend(document);
        for value in [self.bytes.len() as u32, 0x004e4942] {
            glb.extend(value.to_le_bytes());
        }
        glb.extend(self.bytes);
        Ok(glb)
    }
}
/// Render vertices split UV/normal seams. Collision needs a closed indexed
/// surface; retain the first source point within 0.1 mm without averaging/moving.
fn weld(vertices: Vec<[f32; 3]>, indices: Vec<u32>) -> (Vec<[f32; 3]>, Vec<[u32; 3]>) {
    use std::collections::BTreeMap;
    let mut bins: BTreeMap<[i64; 3], Vec<u32>> = BTreeMap::new();
    let mut unique: Vec<[f32; 3]> = vec![];
    let mut remap = vec![];
    for point in vertices {
        let key = point.map(|v| (f64::from(v) / 0.0001).floor() as i64);
        let mut found = None;
        'search: for x in -1..=1 {
            for y in -1..=1 {
                for z in -1..=1 {
                    if let Some(candidates) = bins.get(&[key[0] + x, key[1] + y, key[2] + z]) {
                        for &index in candidates {
                            if Vec3::from_array(unique[index as usize])
                                .distance_squared(Vec3::from_array(point))
                                <= 0.0001_f32.powi(2)
                            {
                                found = Some(index);
                                break 'search;
                            }
                        }
                    }
                }
            }
        }
        let index = found.unwrap_or_else(|| {
            let index = unique.len() as u32;
            unique.push(point);
            bins.entry(key).or_default().push(index);
            index
        });
        remap.push(index);
    }
    let triangles = indices
        .as_chunks::<3>()
        .0
        .iter()
        .map(|triangle| triangle.map(|i| remap[i as usize]))
        .collect();
    (unique, triangles)
}
fn cylinder_segments(radius: f64) -> u32 {
    (std::f64::consts::PI / (1. - CYLINDER_ERROR_METRES / radius).clamp(-1., 1.).acos())
        .ceil()
        .max(32.) as u32
}
fn geometry(zone: &Zone) -> Result<(Vec<u8>, Value, Value), String> {
    zone.validate()?;
    let guide_kinds = [
        zone.paths
            .iter()
            .any(|p| p.kind == theia_zone::paths::PathKind::Road),
        zone.paths
            .iter()
            .any(|p| p.kind == theia_zone::paths::PathKind::Stream),
    ];
    let painted = zone.terrain.as_ref().is_some_and(|p| !p.weights.is_empty());
    let paint_material =
        2 + guide_kinds.iter().filter(|v| **v).count() + if zone.plants.is_empty() { 0 } else { 2 };
    let mut glb = Glb::new();
    let mut bodies = vec![];
    let mut pool = vec![];
    for (i, (centre, size)) in terrain::ground_boxes(zone).into_iter().enumerate() {
        glb.add(
            &format!("Ground {i}"),
            Mesh::from(Cuboid::new(size.x, size.y, size.z)),
            Transform::from_translation(centre),
            0,
        )?;
        bodies.push(json!({"kind":"box","center":centre.to_array(),"size":size.to_array(),"rotation":[0,0,0,1]}));
    }
    let mut triangles = vec![];
    if let Some(geometry) = terrain::geometry(zone) {
        for triangle in &geometry.triangles {
            triangles.push(triangle.map(|i| geometry.positions[i as usize]));
        }
        glb.add(
            "Height patch",
            terrain::surface_mesh(zone).expect("Terrain mesh"),
            Transform::default(),
            if painted { paint_material } else { 0 },
        )?;
    }
    let mut maximum_cylinder_error = 0_f64;
    for object in &zone.objects {
        let [x, y, z] = object.pose.size.map(|v| v as f32);
        let centre = zone.local(object.pose.position)?;
        let rotation = Quat::from_rotation_y(object.pose.yaw_degrees.to_radians() as f32);
        let transform =
            Transform::from_translation(Vec3::from_array(centre)).with_rotation(rotation);
        let mesh = match object.primitive {
            Primitive::Box => {
                bodies.push(json!({"kind":"box","center":centre,"size":[x,y,z],"rotation":rotation.to_array()}));
                Mesh::from(Cuboid::new(x, y, z))
            }
            Primitive::Sphere => {
                bodies.push(
                    json!({"kind":"capsule","center":centre,"radius":x/2.,"half_segment":[0,0,0]}),
                );
                Mesh::from(Sphere::new(x / 2.))
            }
            Primitive::Cylinder => {
                let segments = cylinder_segments(f64::from(x) / 2.);
                maximum_cylinder_error = maximum_cylinder_error.max(
                    f64::from(x) / 2. * (1. - (std::f64::consts::PI / f64::from(segments)).cos()),
                );
                let mesh = Cylinder::new(x / 2., y).mesh().resolution(segments).build();
                let Some(VertexAttributeValues::Float32x3(vertices)) =
                    mesh.attribute(Mesh::ATTRIBUTE_POSITION)
                else {
                    unreachable!()
                };
                let vertices: Vec<_> = vertices
                    .iter()
                    .map(|p| transform.transform_point(Vec3::from_array(*p)).to_array())
                    .collect();
                let indices: Vec<u32> = mesh.indices().unwrap().iter().map(|i| i as u32).collect();
                let (vertices, triangles) = weld(vertices, indices);
                let index = pool.len();
                pool.push(json!({"vertices":vertices,"indices":triangles}));
                bodies.push(json!({"kind":"mesh","mesh":index,"affine":[1,0,0,0,0,1,0,0,0,0,1,0,0,0,0,1],"convex":true}));
                mesh
            }
        };
        glb.add(&object.name, mesh, transform, 1)?;
    }
    for path in &zone.paths {
        let material = if path.kind == theia_zone::paths::PathKind::Road {
            2
        } else {
            2 + usize::from(guide_kinds[0])
        };
        glb.add(
            &path.name,
            paths::ribbon(zone, path),
            Transform::default(),
            material,
        )?;
    }
    let plant_material = 2 + guide_kinds.iter().filter(|v| **v).count();
    for plant in &zone.plants {
        for (index, transform) in vegetation::parts(zone, plant) {
            glb.add(
                &plant.name,
                vegetation::mesh(index),
                transform,
                plant_material + index,
            )?;
        }
    }
    let visual_triangles = glb.triangles;
    let visual = glb.finish(guide_kinds, !zone.plants.is_empty(), painted)?;
    let profile = json!({"schema":4,"source_origin":[0,0,0],"bounds":[[-512,-512],[512,512]],"spawns":[zone.local(zone.spawn)?],
        "visual_sha256":sha256(&visual),"triangles":triangles,"static_shapes":{"meshes":pool,"objects":bodies}});
    let report = json!({"schema":1,"zone_name":zone.name,"authoring_origin":zone.origin,"visual_triangles":visual_triangles,"guide_paths":zone.paths.len(),"authoring_markers":zone.markers.len(),"greybox_plants":zone.plants.len(),"painted_terrain":painted,"terrain_sample_metres":zone.terrain.as_ref().map(|p|p.sample_metres()),
        "maximum_cylinder_radial_error_metres":maximum_cylinder_error,"collision_weld_tolerance_metres":0.0001,"scope":"existing bounded GLB/schema-4 content; requires authority admission; default game world unchanged"});
    Ok((visual, profile, report))
}
pub(super) fn publish(
    zone: &Zone,
    path: &Path,
    content_id: &str,
    revision: u32,
) -> Result<Value, String> {
    if path.exists() {
        return Err("Export requires a new output directory".into());
    }
    let (visual, profile, mut report) = geometry(zone)?;
    let profile = serde_json::to_vec(&profile).map_err(|e| e.to_string())?;
    // Run the actual headless runtime decoder/spawn validator before writing.
    let terrain = Terrain::decode(&profile).map_err(|e| e.to_string())?;
    if !terrain.has_spawns() {
        return Err("Export needs a valid capsule spawn".into());
    }
    let manifest=serde_json::to_vec_pretty(&json!({"schema":1,"content_id":content_id,"revision":revision,"collision_schema":4,
        "collision_bytes":profile.len(),"collision_sha256":sha256(&profile),"visual_bytes":visual.len(),"visual_sha256":sha256(&visual)})).map_err(|e|e.to_string())?;
    let checked = Manifest::decode(&manifest).map_err(|e| e.to_string())?;
    report["identity"] = json!(checked.identity());
    report["content_id"] = json!(content_id);
    report["revision"] = json!(revision);
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_nanos();
    let stage = parent.join(format!(".zone-export-{}-{nonce}", std::process::id()));
    fs::create_dir(&stage).map_err(|e| e.to_string())?;
    let mut destination_created = false;
    let result: Result<(), String> = (|| {
        fs::write(stage.join("terrain.glb"), visual).map_err(|e| e.to_string())?;
        fs::write(stage.join("terrain.json"), profile).map_err(|e| e.to_string())?;
        fs::write(stage.join("world.manifest.json"), manifest).map_err(|e| e.to_string())?;
        zone.save(&stage.join("authoring.zone.json"))?;
        fs::write(
            stage.join("export.json"),
            serde_json::to_vec_pretty(&report).unwrap(),
        )
        .map_err(|e| e.to_string())?;
        let loaded = Terrain::load(&stage.join("terrain.json")).map_err(|e| e.to_string())?;
        loaded
            .verify_visual(&stage.join("terrain.glb"))
            .map_err(|e| e.to_string())?;
        // Atomic directory reservation refuses a racing/existing destination.
        // Move verified artifacts first and publish the required manifest last.
        fs::create_dir(path).map_err(|e| e.to_string())?;
        destination_created = true;
        for name in [
            "terrain.glb",
            "terrain.json",
            "authoring.zone.json",
            "export.json",
            "world.manifest.json",
        ] {
            fs::rename(stage.join(name), path.join(name)).map_err(|e| e.to_string())?;
        }
        let _ = fs::remove_dir(&stage);
        Ok(())
    })();
    if result.is_err() {
        let _ = fs::remove_dir_all(&stage);
        if destination_created {
            let _ = fs::remove_dir_all(path);
        }
    }
    result?;
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        path::PathBuf,
        sync::atomic::{AtomicU64, Ordering},
        time::Duration,
    };
    use theia_network::{client::State, renet_runtime as runtime, simulation::World, wire::Intent};
    use tokio::sync::watch;
    static NEXT: AtomicU64 = AtomicU64::new(0);
    struct Folder(PathBuf);
    impl Folder {
        fn new() -> Self {
            let p = std::env::temp_dir().join(format!(
                "theia-zone-export-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir(&p).unwrap();
            Self(p)
        }
    }
    impl Drop for Folder {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    fn fixture() -> Zone {
        let mut editor = Editor::new(Zone::default()).unwrap();
        for line in include_str!("../../../content/zones/greybox.commands.jsonl").lines() {
            editor.execute(serde_json::from_str(line).unwrap()).unwrap();
        }
        editor.zone().clone()
    }
    #[test]
    fn exported_primitives_use_existing_admission_and_preserve_source_and_identity() {
        let folder = Folder::new();
        let path = folder.0.join("world");
        let zone = fixture();
        let report = publish(&zone, &path, "theia.test.authored-zone", 1).unwrap();
        assert!(
            report["maximum_cylinder_radial_error_metres"]
                .as_f64()
                .unwrap()
                <= 0.001
        );
        assert_eq!(Zone::load(&path.join("authoring.zone.json")).unwrap(), zone);
        let loaded = Terrain::load(&path.join("terrain.json")).unwrap();
        loaded.verify_visual(&path.join("terrain.glb")).unwrap();
        assert!(loaded.has_spawns());
        // Exact sphere encoded as zero-length capsule, plus bounded cylinder hull.
        assert!(
            !loaded
                .physics
                .line_clear(Vec3::new(-4., 0.75, 1.), Vec3::new(-4., 0.75, -6.))
        );
        assert!(
            !loaded
                .physics
                .line_clear(Vec3::new(4., 1., 1.), Vec3::new(4., 1., -6.))
        );
        assert!(loaded.physics.ground([0., 0.], 8.).unwrap().0 > 0.29);
        let old = fs::read(path.join("terrain.json")).unwrap();
        assert!(publish(&zone, &path, "theia.test.authored-zone", 2).is_err());
        assert_eq!(fs::read(path.join("terrain.json")).unwrap(), old);
        // Headless runtime still loads the admitted world without GLB I/O.
        fs::remove_file(path.join("terrain.glb")).unwrap();
        assert_eq!(
            Terrain::load(&path.join("terrain.json")).unwrap().identity,
            loaded.identity
        );
    }
    #[test]
    fn cylinder_collision_copy_is_closed_and_export_is_deterministic() {
        let folder = Folder::new();
        let zone = fixture();
        let (_, profile, _) = geometry(&zone).unwrap();
        for mesh in profile["static_shapes"]["meshes"].as_array().unwrap() {
            let vertices = mesh["vertices"].as_array().unwrap();
            let mut edges = std::collections::BTreeMap::new();
            let mut used = std::collections::BTreeSet::new();
            let faces = mesh["indices"].as_array().unwrap();
            for face in faces {
                let ids: Vec<_> = face
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|i| i.as_u64().unwrap())
                    .collect();
                for i in 0..3 {
                    let mut edge = [ids[i], ids[(i + 1) % 3]];
                    edge.sort();
                    *edges.entry(edge).or_insert(0) += 1;
                    used.insert(ids[i]);
                }
            }
            assert_eq!(used.len(), vertices.len());
            assert!(edges.values().all(|&count| count == 2));
            assert_eq!(vertices.len() + faces.len(), edges.len() + 2);
        }
        let first = folder.0.join("first");
        let second = folder.0.join("second");
        let a = publish(&zone, &first, "theia.test.deterministic-zone", 1).unwrap();
        let b = publish(&zone, &second, "theia.test.deterministic-zone", 1).unwrap();
        assert_eq!(a, b);
        for name in ["terrain.glb", "terrain.json", "world.manifest.json"] {
            assert_eq!(
                fs::read(first.join(name)).unwrap(),
                fs::read(second.join(name)).unwrap()
            );
        }
    }
    #[test]
    fn invalid_spawn_metadata_and_large_visual_budget_refuse_before_publication() {
        let folder = Folder::new();
        let path = folder.0.join("world");
        let mut zone = fixture();
        zone.spawn = [0., 0.1, -6.];
        assert!(publish(&zone, &path, "theia.test.authored-zone", 1).is_err());
        assert!(!path.exists());
        zone = fixture();
        assert!(publish(&zone, &path, "INVALID", 0).is_err());
        assert!(!path.exists());
        let sphere = zone
            .objects
            .iter()
            .find(|o| o.primitive == Primitive::Sphere)
            .unwrap()
            .clone();
        zone.objects = (1..=1024)
            .map(|id| Object {
                id,
                ..sphere.clone()
            })
            .collect();
        assert!(
            publish(&zone, &path, "theia.test.authored-zone", 1)
                .unwrap_err()
                .contains("triangle budget")
        );
        assert!(!path.exists());
    }
    #[test]
    fn far_authoring_centimetres_and_depressed_terrain_survive_local_export() {
        let folder = Folder::new();
        for (index, sign) in [-1., 1.].into_iter().enumerate() {
            let origin = [sign * 128_000_000., 0., -sign * 128_000_000.];
            let chunk = theia_core::coordinates::GlobalPosition::new(origin)
                .unwrap()
                .chunk(128)
                .unwrap();
            let mut patch = theia_zone::terrain::HeightPatch::flat([chunk.x, chunk.z]);
            patch.heights[16 * 33 + 16] = -2.;
            let zone = Zone {
                schema: 2,
                origin,
                spawn: [origin[0] + 4., 0.1, origin[2] + 4.],
                terrain: Some(patch),
                objects: vec![Object {
                    id: 1,
                    name: "Far block".into(),
                    primitive: Primitive::Box,
                    pose: Pose {
                        position: [origin[0] + 0.01, 0.5, origin[2] - 2.],
                        size: [1.; 3],
                        yaw_degrees: 0.,
                    },
                }],
                ..Zone::default()
            };
            let path = folder.0.join(format!("world-{index}"));
            publish(&zone, &path, "theia.test.far-zone", 1).unwrap();
            let loaded = Terrain::load(&path.join("terrain.json")).unwrap();
            assert!((loaded.physics.ground([64., 64.], 4.).unwrap().0 + 2.).abs() < 0.0001);
            assert!(loaded.physics.ground([0.01, -2.], 4.).unwrap().0 > 0.99);
            let source = Zone::load(&path.join("authoring.zone.json")).unwrap();
            assert_eq!(source, zone);
            let profile: Value =
                serde_json::from_slice(&fs::read(path.join("terrain.json")).unwrap()).unwrap();
            let centre = profile["static_shapes"]["objects"]
                .as_array()
                .unwrap()
                .last()
                .unwrap()["center"][0]
                .as_f64()
                .unwrap();
            assert!((centre - 0.01).abs() < 0.000001);
        }
    }
    #[test]
    fn fine_terrain_exports_all_samples_and_passes_existing_runtime_admission() {
        let mut zone = fixture();
        zone.schema = 2;
        zone.terrain = Some(theia_zone::terrain::HeightPatch::flat([0, 0]));
        let mut editor = Editor::new(zone).unwrap();
        editor
            .execute(Command::Apply {
                expected_revision: 0,
                edits: vec![
                    Edit::ResampleTerrain {
                        resolution: theia_zone::terrain::Resolution::TwoMetres,
                    },
                    Edit::PaintTerrain {
                        paint: theia_zone::painting::Paint {
                            centre: [64., 64.],
                            radius_metres: 16.,
                            surface: theia_zone::painting::Surface::Dirt,
                            blend: 1.,
                        },
                    },
                ],
            })
            .unwrap();
        let (_, profile, report) = geometry(editor.zone()).unwrap();
        assert_eq!(profile["triangles"].as_array().unwrap().len(), 8192);
        assert_eq!(report["terrain_sample_metres"], 2.);
        let folder = Folder::new();
        let destination = folder.0.join("fine");
        publish(editor.zone(), &destination, "theia.test.fine", 1).unwrap();
        assert_eq!(
            Zone::load(&destination.join("authoring.zone.json")).unwrap(),
            *editor.zone()
        );
        let terrain = Terrain::load(&destination.join("terrain.json")).unwrap();
        terrain
            .verify_visual(&destination.join("terrain.glb"))
            .unwrap();
        assert!(terrain.physics.ground([64., 64.], 10.).unwrap().0.abs() < 0.0001);
    }
    #[test]
    fn painted_terrain_exports_linear_vertex_colours_and_preserves_collision() {
        let mut zone = fixture();
        zone.schema = 2;
        zone.terrain = Some(theia_zone::terrain::HeightPatch::flat([0, 0]));
        let (base, profile, _) = geometry(&zone).unwrap();
        let mut editor = Editor::new(zone).unwrap();
        editor
            .execute(Command::Apply {
                expected_revision: 0,
                edits: vec![Edit::PaintTerrain {
                    paint: theia_zone::painting::Paint {
                        centre: [64., 64.],
                        radius_metres: 16.,
                        surface: theia_zone::painting::Surface::Dirt,
                        blend: 1.,
                    },
                }],
            })
            .unwrap();
        let (visual, after, report) = geometry(editor.zone()).unwrap();
        assert_ne!(visual, base);
        assert_eq!(profile["triangles"], after["triangles"]);
        assert_eq!(profile["static_shapes"], after["static_shapes"]);
        assert_eq!(report["painted_terrain"], true);
        let mesh = terrain::surface_mesh(editor.zone()).unwrap();
        let VertexAttributeValues::Float32x4(colours) =
            mesh.attribute(Mesh::ATTRIBUTE_COLOR).unwrap()
        else {
            panic!("Expected RGBA")
        };
        let expected = Color::srgb(0.38, 0.24, 0.11).to_linear().to_f32_array();
        assert_eq!(colours[16 * 33 + 16], expected);
        let json_size = u32::from_le_bytes(visual[12..16].try_into().unwrap()) as usize;
        let document: Value = serde_json::from_slice(&visual[20..20 + json_size]).unwrap();
        let primitive = document["meshes"]
            .as_array()
            .unwrap()
            .iter()
            .find(|m| m["name"] == "Height patch")
            .unwrap()["primitives"][0]
            .clone();
        let accessor =
            &document["accessors"][primitive["attributes"]["COLOR_0"].as_u64().unwrap() as usize];
        assert_eq!(accessor["count"], 1089);
        assert_eq!(accessor["type"], "VEC4");
        assert_eq!(
            document["materials"][primitive["material"].as_u64().unwrap() as usize]["pbrMetallicRoughness"]
                ["baseColorFactor"],
            json!([1., 1., 1., 1.])
        );
        let folder = Folder::new();
        let destination = folder.0.join("paint");
        publish(editor.zone(), &destination, "theia.test.paint", 1).unwrap();
        assert_eq!(
            Zone::load(&destination.join("authoring.zone.json")).unwrap(),
            *editor.zone()
        );
        Terrain::load(&destination.join("terrain.json"))
            .unwrap()
            .verify_visual(&destination.join("terrain.glb"))
            .unwrap();
    }
    #[test]
    fn greybox_vegetation_exports_standard_visuals_and_unchanged_collision() {
        let mut zone = fixture();
        let (before, profile, report) = geometry(&zone).unwrap();
        zone.schema = 5;
        zone.plants = vec![
            theia_zone::vegetation::Plant {
                id: 100,
                name: "Tree".into(),
                kind: theia_zone::vegetation::PlantKind::Tree,
                position: [-4., 0., 6.],
                yaw_degrees: 45.,
                scale: 1.,
            },
            theia_zone::vegetation::Plant {
                id: 101,
                name: "Shrub".into(),
                kind: theia_zone::vegetation::PlantKind::Shrub,
                position: [4., 0., 6.],
                yaw_degrees: 15.,
                scale: 1.5,
            },
        ];
        let (after, after_profile, after_report) = geometry(&zone).unwrap();
        assert_ne!(before, after);
        assert_eq!(profile["static_shapes"], after_profile["static_shapes"]);
        assert_eq!(profile["triangles"], after_profile["triangles"]);
        assert_eq!(after_report["greybox_plants"], 2);
        assert!(
            after_report["visual_triangles"].as_u64().unwrap()
                > report["visual_triangles"].as_u64().unwrap()
        );
        let folder = Folder::new();
        let destination = folder.0.join("plants");
        publish(&zone, &destination, "theia.test.plants", 1).unwrap();
        assert_eq!(
            Zone::load(&destination.join("authoring.zone.json")).unwrap(),
            zone
        );
        Terrain::load(&destination.join("terrain.json"))
            .unwrap()
            .verify_visual(&destination.join("terrain.glb"))
            .unwrap();
        // An ellipsoid's outward normal is its implicit-surface gradient.
        // Inspect emitted GLB data rather than reusing the transform expression.
        let mut glb = Glb::new();
        glb.add(
            "Ellipsoid",
            vegetation::mesh(1),
            Transform::from_scale(Vec3::new(2., 1., 3.)),
            0,
        )
        .unwrap();
        let read = |view: usize| -> Vec<Vec3> {
            let start = glb.views[view]["byteOffset"].as_u64().unwrap() as usize;
            let length = glb.views[view]["byteLength"].as_u64().unwrap() as usize;
            glb.bytes[start..start + length]
                .as_chunks::<12>()
                .0
                .iter()
                .map(|v| {
                    Vec3::new(
                        f32::from_le_bytes(v[0..4].try_into().unwrap()),
                        f32::from_le_bytes(v[4..8].try_into().unwrap()),
                        f32::from_le_bytes(v[8..12].try_into().unwrap()),
                    )
                })
                .collect()
        };
        for (position, normal) in read(0).iter().zip(read(1)) {
            let gradient = Vec3::new(position.x / 4., position.y, position.z / 9.).normalize();
            assert!(normal.dot(gradient) > 0.99999);
        }
    }
    #[test]
    fn markers_preserve_authoring_metadata_without_publishing_gameplay_or_geometry() {
        let mut zone = fixture();
        let (visual, profile, _) = geometry(&zone).unwrap();
        zone.schema = 4;
        zone.markers = vec![theia_zone::markers::Marker {
            id: 500,
            name: "Guide".into(),
            kind: theia_zone::markers::MarkerKind::Npc,
            position: [0., 0., 3.],
            yaw_degrees: 45.,
            radius_metres: 1.,
            definition: Some("human.guide".into()),
        }];
        let (after, after_profile, report) = geometry(&zone).unwrap();
        assert_eq!(after, visual);
        assert_eq!(after_profile, profile);
        assert_eq!(report["authoring_markers"], 1);
        let folder = Folder::new();
        let destination = folder.0.join("markers");
        publish(&zone, &destination, "theia.test.markers", 1).unwrap();
        assert_eq!(
            Zone::load(&destination.join("authoring.zone.json")).unwrap(),
            zone
        );
    }
    #[test]
    fn guide_ribbons_export_visually_and_never_enter_authoritative_collision() {
        use theia_core::time::TickRate;
        use theia_network::kinematic::{self, Motion};
        use theia_zone::paths::{GuidePath, PathKind};
        let mut zone = fixture();
        let (_, base_profile, base_report) = geometry(&zone).unwrap();
        zone.schema = 3;
        zone.paths = vec![
            GuidePath {
                id: 100,
                name: "Road guide".into(),
                kind: PathKind::Road,
                width_metres: 2.,
                points: vec![[-3., 1., 4.], [3., 1., 4.]],
            },
            GuidePath {
                id: 101,
                name: "Stream guide".into(),
                kind: PathKind::Stream,
                width_metres: 2.,
                points: vec![[-3., 1., 2.], [3., 1., 2.]],
            },
        ];
        let (_, profile, report) = geometry(&zone).unwrap();
        assert_eq!(report["guide_paths"], 2);
        assert_eq!(
            report["visual_triangles"].as_u64().unwrap(),
            base_report["visual_triangles"].as_u64().unwrap() + 4
        );
        assert_eq!(profile["static_shapes"], base_profile["static_shapes"]);
        assert_eq!(profile["triangles"], base_profile["triangles"]);
        let before = Terrain::decode(&serde_json::to_vec(&base_profile).unwrap()).unwrap();
        let after = Terrain::decode(&serde_json::to_vec(&profile).unwrap()).unwrap();
        assert_eq!(
            before.physics.colliders.len(),
            after.physics.colliders.len()
        );
        assert_ne!(before.identity, after.identity);
        let mut a = before.spawn(0).unwrap();
        let mut b = after.spawn(0).unwrap();
        let mut ma = Motion::default();
        let mut mb = Motion::default();
        for _ in 0..45 {
            let intent = Intent {
                axes: [0, -1000],
                ..default()
            };
            a = kinematic::advance(&before.physics, &mut ma, a, intent, TickRate::Hz30).unwrap();
            b = kinematic::advance(&after.physics, &mut mb, b, intent, TickRate::Hz30).unwrap();
        }
        assert_eq!(a, b);
        assert!(a.metres()[2] < 1.);
        let folder = Folder::new();
        let path = folder.0.join("guides");
        publish(&zone, &path, "theia.test.guides", 1).unwrap();
        assert_eq!(Zone::load(&path.join("authoring.zone.json")).unwrap(), zone);
    }
    async fn until(
        state: &mut watch::Receiver<State>,
        predicate: impl Fn(&State) -> bool,
    ) -> State {
        tokio::time::timeout(Duration::from_secs(6), async {
            loop {
                let current = state.borrow().clone();
                if predicate(&current) {
                    return current;
                }
                state.changed().await.unwrap();
            }
        })
        .await
        .unwrap()
    }
    #[tokio::test]
    async fn exported_zone_admits_two_encrypted_clients_blocks_wall_and_rejects_old_revision() {
        let folder = Folder::new();
        let zone = fixture();
        let old_path = folder.0.join("old");
        publish(&zone, &old_path, "theia.test.net-zone", 1).unwrap();
        let old = Terrain::load(&old_path.join("terrain.json"))
            .unwrap()
            .identity;
        let path = folder.0.join("current");
        publish(&zone, &path, "theia.test.net-zone", 2).unwrap();
        let terrain = Terrain::load(&path.join("terrain.json")).unwrap();
        let expected = terrain.identity;
        fs::remove_file(path.join("terrain.glb")).unwrap();
        let socket = std::net::UdpSocket::bind("127.0.0.1:0").unwrap();
        let address = socket.local_addr().unwrap();
        let tickets = folder.0.join("tickets");
        let transport = runtime::issue_local(socket, &tickets).unwrap();
        let (server_stop, shutdown) = watch::channel(false);
        let server = tokio::spawn(runtime::run_server(
            transport,
            shutdown,
            World::with_terrain(terrain),
        ));
        let mut peers = vec![];
        for index in 1..=2 {
            let bytes = fs::read(tickets.join(format!("client-{index}.token"))).unwrap();
            let (input, intents) = watch::channel(Intent::default());
            let (tx, states) = watch::channel(State::default());
            let (stop, shutdown) = watch::channel(false);
            let worker = tokio::spawn(async move {
                runtime::run_client(address, &bytes, intents, tx, shutdown, expected).await
            });
            peers.push((input, states, stop, worker));
        }
        let state = until(&mut peers[0].1, |s| {
            s.snapshot.as_ref().is_some_and(|s| s.players.len() == 2)
        })
        .await;
        let first = state.id.unwrap();
        let second = until(&mut peers[1].1, |s| {
            s.snapshot.as_ref().is_some_and(|s| s.players.len() == 2)
        })
        .await
        .id
        .unwrap();
        assert_ne!(first, second);
        peers[0].0.send_replace(Intent {
            axes: [0, -1000],
            ..default()
        });
        let walked = until(&mut peers[0].1, |s| {
            s.local_view()
                .ok()
                .and_then(|s| s.snapshot)
                .is_some_and(|s| s.players.iter().any(|p| p.id == first && p.xz[1] < -5.))
        })
        .await;
        peers[0].0.send_replace(Intent::default());
        let own = walked
            .local_view()
            .unwrap()
            .snapshot
            .unwrap()
            .players
            .into_iter()
            .find(|p| p.id == first)
            .unwrap();
        assert!(own.xz[1] > -5.3);
        until(&mut peers[1].1, |s| {
            s.local_view()
                .ok()
                .and_then(|s| s.snapshot)
                .is_some_and(|s| s.players.iter().any(|p| p.id == first && p.xz[1] < -5.))
        })
        .await;
        let (_input, input) = watch::channel(Intent::default());
        let (tx, _state) = watch::channel(State::default());
        let (_stop, shutdown) = watch::channel(false);
        let stale = fs::read(tickets.join("client-3.token")).unwrap();
        let error = tokio::time::timeout(
            Duration::from_secs(6),
            runtime::run_client(address, &stale, input, tx, shutdown, old),
        )
        .await
        .unwrap()
        .unwrap_err();
        assert!(error.to_string().contains("world mismatch"));
        for (_, _, stop, worker) in peers {
            stop.send_replace(true);
            worker.await.unwrap().unwrap();
        }
        server_stop.send_replace(true);
        server.await.unwrap().unwrap();
    }
}
