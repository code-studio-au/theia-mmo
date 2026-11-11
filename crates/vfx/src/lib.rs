//! Helpers for future verified particle packages. This crate does not spawn effects.
pub mod chunk_overlay;
pub mod simulation;
use bevy::{
    asset::{AssetServer, Assets, RenderAssetUsages},
    image::{CompressedImageFormats, Image, ImageLoaderSettings, ImageSampler, ImageType},
    prelude::{Handle, Quat, Transform, Vec3},
};
use bevy_sprinkles::{
    TextureRef,
    asset::{DrawPassMaterial, ParticlesAsset, versions},
    budget::emitter_counts,
};
use std::{
    collections::{BTreeMap, BTreeSet},
    io::Read,
    path::{Component, Path},
};
use theia_core::coordinates::{ChunkId, INITIAL_CHUNK_METRES, LocalFrame, LocalPosition};
use theia_network::{particle_content::Catalog, shared_images::ColorSpace};

/// A catalog-identified effect decoded from the bytes whose hash was checked.
pub struct CatalogEffect {
    pub id: String,
    pub asset: ParticlesAsset,
}

/// One catalog placement projected into the current render frame.
pub struct CatalogPlacement {
    pub id: u64,
    pub effect: String,
    pub transform: Transform,
}

/// Conservative replicated counts, not a complete CPU or GPU memory estimate.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CatalogCounts {
    pub placements: u64,
    pub emitters: u64,
    pub vertices: u64,
    pub indices: u64,
    pub particle_slots: u64,
    pub trail_history_entries: u64,
    pub sub_emitter_words: u64,
}

/// Count every emitter for every placement, including disabled emitters and
/// potentially shared meshes. Textures, render targets, shader data and other
/// allocations are not included. These counts alone do not admit playback.
pub fn count_catalog_resources(
    catalog: &Catalog,
    effects: &[CatalogEffect],
) -> Result<CatalogCounts, String> {
    let by_id = catalog_effects_by_id(catalog, effects)?;
    let mut totals = CatalogCounts::default();
    for placement in catalog.placements() {
        totals.placements = totals
            .placements
            .checked_add(1)
            .ok_or("Particle count overflow")?;
        let asset = by_id[placement.effect.as_str()];
        for (index, emitter) in asset.emitters.iter().enumerate() {
            let count = emitter_counts(emitter).ok_or("Invalid particle allocation arithmetic")?;
            let sub_words = emitter
                .sub_emitter
                .as_ref()
                .and_then(|sub| {
                    (sub.target_emitter != index)
                        .then(|| asset.emitters.get(sub.target_emitter))
                        .flatten()
                        .map(|target| 4 + 12 * u64::from(target.emission.particles_amount))
                })
                .unwrap_or(0);
            for (total, value) in [
                (&mut totals.emitters, 1),
                (&mut totals.vertices, u64::from(count.vertices)),
                (
                    &mut totals.indices,
                    u64::try_from(count.indices).map_err(|_| "Particle count overflow")?,
                ),
                (&mut totals.particle_slots, u64::from(count.particle_slots)),
                (
                    &mut totals.trail_history_entries,
                    u64::from(count.trail_history_entries),
                ),
                (&mut totals.sub_emitter_words, sub_words),
            ] {
                *total = total.checked_add(value).ok_or("Particle count overflow")?;
            }
        }
    }
    Ok(totals)
}

impl CatalogCounts {
    /// Add conservative counts without wrapping.
    pub fn checked_add(self, other: Self) -> Option<Self> {
        Some(Self {
            placements: self.placements.checked_add(other.placements)?,
            emitters: self.emitters.checked_add(other.emitters)?,
            vertices: self.vertices.checked_add(other.vertices)?,
            indices: self.indices.checked_add(other.indices)?,
            particle_slots: self.particle_slots.checked_add(other.particle_slots)?,
            trail_history_entries: self
                .trail_history_entries
                .checked_add(other.trail_history_entries)?,
            sub_emitter_words: self
                .sub_emitter_words
                .checked_add(other.sub_emitter_words)?,
        })
    }

    /// Require each count to stay within an explicit caller-provided limit.
    pub fn fits_within(self, limits: Self) -> bool {
        self.placements <= limits.placements
            && self.emitters <= limits.emitters
            && self.vertices <= limits.vertices
            && self.indices <= limits.indices
            && self.particle_slots <= limits.particle_slots
            && self.trail_history_entries <= limits.trail_history_entries
            && self.sub_emitter_words <= limits.sub_emitter_words
    }
}

fn catalog_effects_by_id<'a>(
    catalog: &Catalog,
    effects: &'a [CatalogEffect],
) -> Result<BTreeMap<&'a str, &'a ParticlesAsset>, String> {
    let mut by_id = BTreeMap::new();
    for effect in effects {
        if by_id.insert(effect.id.as_str(), &effect.asset).is_some() {
            return Err("Duplicate loaded particle effect ID".into());
        }
    }
    if by_id.len() != catalog.effects().len()
        || catalog
            .effects()
            .iter()
            .any(|effect| !by_id.contains_key(effect.id.as_str()))
    {
        return Err("Loaded particle effects do not match the catalog".into());
    }
    Ok(by_id)
}

/// Partition a verified source catalog by the half-open owner of each anchor.
/// Effects must be loaded from that catalog with `particles` as their package
/// path. This only drafts catalogs; callers must verify and publish their bytes,
/// dependencies and containing chunk manifests transactionally.
pub fn partition_catalog(
    catalog: &Catalog,
    effects: &[CatalogEffect],
    source_frame: LocalFrame,
    available: &BTreeSet<ChunkId>,
) -> Result<BTreeMap<ChunkId, Vec<u8>>, String> {
    let effects_by_id = catalog_effects_by_id(catalog, effects)?;
    let mut placements: BTreeMap<ChunkId, Vec<serde_json::Value>> = BTreeMap::new();
    for placement in catalog.placements() {
        let global = source_frame
            .to_global(LocalPosition::new(placement.position).ok_or("Invalid particle position")?)
            .ok_or("Particle global position is outside coordinate bounds")?;
        let owner = global
            .chunk(INITIAL_CHUNK_METRES)
            .ok_or("Invalid particle chunk")?;
        if !available.contains(&owner) {
            return Err("Particle anchor has no output chunk".into());
        }
        let frame = LocalFrame::new(
            owner
                .origin(INITIAL_CHUNK_METRES)
                .ok_or("Invalid particle chunk origin")?,
        );
        let position = frame
            .to_local(global)
            .ok_or("Particle outside chunk frame")?
            .metres();
        placements
            .entry(owner)
            .or_default()
            .push(serde_json::json!({
                "id":placement.id,"effect":placement.effect,"position":position,
                "yaw_degrees":placement.yaw_degrees,"scale":placement.scale
            }));
    }
    let mut output = BTreeMap::new();
    for (owner, rows) in placements {
        let used: BTreeSet<_> = rows
            .iter()
            .map(|row| row["effect"].as_str().expect("authored effect"))
            .collect();
        let mut texture_keys = BTreeSet::new();
        for id in &used {
            for emitter in &effects_by_id[id].emitters {
                let DrawPassMaterial::Standard(material) = &emitter.draw_pass.material else {
                    return Err("Custom particle shader unsupported".into());
                };
                for (texture, color) in [
                    (&material.base_color_texture, ColorSpace::Srgb),
                    (&material.emissive_texture, ColorSpace::Srgb),
                    (&material.metallic_roughness_texture, ColorSpace::Linear),
                    (&material.normal_map_texture, ColorSpace::Linear),
                    (&material.occlusion_texture, ColorSpace::Linear),
                ] {
                    match texture {
                        None | Some(TextureRef::Preset(_)) => {}
                        Some(TextureRef::Asset(path)) => {
                            texture_keys.insert((path.clone(), color));
                        }
                        _ => return Err("Non-package particle texture unsupported".into()),
                    }
                }
            }
        }
        let selected_effects: Vec<_> = catalog.effects().iter().filter(|effect| used.contains(effect.id.as_str())).map(|effect| serde_json::json!({"id":effect.id,"bytes":effect.bytes,"sha256":effect.sha256})).collect();
        let mut textures = Vec::new();
        for texture in catalog.textures() {
            let key = (
                format!("particles/textures/{}", texture.filename()),
                texture.color_space,
            );
            if texture_keys.remove(&key) {
                textures.push(serde_json::json!({"sha256":texture.sha256,"bytes":texture.bytes,"color_space":match texture.color_space { ColorSpace::Srgb => "srgb", ColorSpace::Linear => "linear" }}));
            }
        }
        if !texture_keys.is_empty() {
            return Err("Particle texture reference missing from catalog".into());
        }
        let bytes = serde_json::to_vec(&serde_json::json!({"schema":1,"effects":selected_effects,"placements":rows,"textures":textures})).map_err(|e|e.to_string())?;
        Catalog::decode(&bytes).map_err(|e| e.to_string())?;
        output.insert(owner, bytes);
    }
    Ok(output)
}

/// Immutable bytes for a future chunk-package publication.
pub struct ParticleChunkDraft {
    pub catalog: Vec<u8>,
    pub effects: Vec<(String, Vec<u8>)>,
    pub textures: Vec<(String, Vec<u8>)>,
}

/// Bind a drafted catalog to a new revision of a schema-1 or schema-2 chunk.
/// Schema 3 has projected-material requirements not represented by schema 4.
/// The caller must verify the original package and all drafted dependencies,
/// update the index to the returned identity and publish the result atomically.
pub fn bind_chunk_catalog(
    manifest_bytes: &[u8],
    catalog_bytes: &[u8],
) -> Result<(Vec<u8>, [u8; 32]), String> {
    revise_chunk_catalog(manifest_bytes, Some(catalog_bytes), false)
}

/// Replace or remove the particle catalog of a previously verified chunk.
/// Removing uses schema 2, which preserves schema-4 shared-image semantics.
pub fn replace_chunk_catalog(
    manifest_bytes: &[u8],
    catalog_bytes: Option<&[u8]>,
) -> Result<(Vec<u8>, [u8; 32]), String> {
    revise_chunk_catalog(manifest_bytes, catalog_bytes, true)
}

fn revise_chunk_catalog(
    manifest_bytes: &[u8],
    catalog_bytes: Option<&[u8]>,
    replace: bool,
) -> Result<(Vec<u8>, [u8; 32]), String> {
    let original =
        theia_network::content::Manifest::decode(manifest_bytes).map_err(|e| e.to_string())?;
    if original.has_particle_catalog() && !replace {
        return Err("Chunk already contains particles".into());
    }
    if catalog_bytes.is_none() && !original.has_particle_catalog() {
        return Err("Chunk has no particle catalog to remove".into());
    }
    if let Some(catalog_bytes) = catalog_bytes {
        Catalog::decode(catalog_bytes).map_err(|e| e.to_string())?;
    }
    let revision = original
        .revision()
        .checked_add(1)
        .ok_or("Chunk revision overflow")?;
    let mut document: serde_json::Value =
        serde_json::from_slice(manifest_bytes).map_err(|e| e.to_string())?;
    if !matches!(document["schema"].as_u64(), Some(1 | 2 | 4)) {
        return Err("Chunk material profile schema cannot be combined with particles".into());
    }
    document["revision"] = serde_json::json!(revision);
    if let Some(catalog_bytes) = catalog_bytes {
        document["schema"] = serde_json::json!(4);
        document["particle_bytes"] = serde_json::json!(catalog_bytes.len());
        document["particle_sha256"] =
            serde_json::json!(theia_network::terrain::sha256(catalog_bytes));
    } else {
        document["schema"] = serde_json::json!(2);
        let object = document.as_object_mut().ok_or("Invalid manifest")?;
        object.remove("particle_bytes");
        object.remove("particle_sha256");
    }
    let bytes = serde_json::to_vec(&document).map_err(|e| e.to_string())?;
    let revised = theia_network::content::Manifest::decode(&bytes).map_err(|e| e.to_string())?;
    if let Some(catalog_bytes) = catalog_bytes {
        revised
            .verify_particle_catalog(catalog_bytes)
            .map_err(|e| e.to_string())?;
    }
    Ok((bytes, revised.identity()))
}

/// Draft each chunk from one verified snapshot of every dependency. This does
/// not modify files or bind chunk identities; callers must publish atomically.
/// The total limit includes copies needed by multiple output chunks.
pub fn draft_particle_chunks(
    catalog: &Catalog,
    effect_directory: &Path,
    texture_directory: &Path,
    source_frame: LocalFrame,
    available: &BTreeSet<ChunkId>,
    max_output_bytes: usize,
) -> Result<BTreeMap<ChunkId, ParticleChunkDraft>, String> {
    let effect_bytes = catalog
        .read_effects(effect_directory)
        .map_err(|e| e.to_string())?;
    let effects = catalog_effects_from_buffers(catalog, &effect_bytes, Path::new("particles"))?;
    for effect in &effects {
        simulation::validate(&effect.asset).map_err(|error| format!("{}: {error}", effect.id))?;
    }
    let texture_bytes = catalog
        .read_textures(texture_directory)
        .map_err(|e| e.to_string())?;
    for (texture, bytes) in catalog.textures().iter().zip(&texture_bytes) {
        Image::from_buffer(
            bytes,
            ImageType::Extension("png"),
            CompressedImageFormats::NONE,
            texture.color_space == ColorSpace::Srgb,
            ImageSampler::default(),
            RenderAssetUsages::default(),
        )
        .map_err(|error| format!("Invalid particle texture {}: {error}", texture.filename()))?;
    }
    let catalogs = partition_catalog(catalog, &effects, source_frame, available)?;
    let mut used_bytes = 0usize;
    let mut result = BTreeMap::new();
    for (id, bytes) in catalogs {
        let selected = Catalog::decode(&bytes).map_err(|e| e.to_string())?;
        let mut chunk_effects = Vec::new();
        let mut chunk_textures = Vec::new();
        used_bytes = used_bytes
            .checked_add(bytes.len())
            .ok_or("Particle draft size overflow")?;
        for effect in selected.effects() {
            let index = catalog
                .effects()
                .iter()
                .position(|source| source.id == effect.id)
                .ok_or("Draft effect missing")?;
            let data = &effect_bytes[index];
            used_bytes = used_bytes
                .checked_add(data.len())
                .ok_or("Particle draft size overflow")?;
            if used_bytes > max_output_bytes {
                return Err("Particle draft exceeds output byte limit".into());
            }
            chunk_effects.push((format!("effect-{}.ron", effect.id), data.clone()));
        }
        for texture in selected.textures() {
            let index = catalog
                .textures()
                .iter()
                .position(|source| {
                    source.sha256 == texture.sha256 && source.color_space == texture.color_space
                })
                .ok_or("Draft texture missing")?;
            let data = &texture_bytes[index];
            used_bytes = used_bytes
                .checked_add(data.len())
                .ok_or("Particle draft size overflow")?;
            if used_bytes > max_output_bytes {
                return Err("Particle draft exceeds output byte limit".into());
            }
            chunk_textures.push((texture.filename(), data.clone()));
        }
        result.insert(
            id,
            ParticleChunkDraft {
                catalog: bytes,
                effects: chunk_effects,
                textures: chunk_textures,
            },
        );
    }
    Ok(result)
}

/// Project immutable catalog placements from their content frame into the
/// current render frame, then compose each effect's initial transform. Recompute
/// from the catalog after an origin shift rather than accumulating f32 offsets.
/// This does not spawn effects or validate their simulation parameters.
pub fn project_catalog_placements(
    catalog: &Catalog,
    effects: &[CatalogEffect],
    content_frame: LocalFrame,
    render_frame: LocalFrame,
) -> Result<Vec<CatalogPlacement>, String> {
    let by_id = catalog_effects_by_id(catalog, effects)?;
    let mut projected = Vec::with_capacity(catalog.placements().len());
    for placement in catalog.placements() {
        let effect = by_id[placement.effect.as_str()];
        let local =
            LocalPosition::new(placement.position).ok_or("Invalid catalog particle position")?;
        let global = content_frame
            .to_global(local)
            .ok_or("Particle placement exceeds global coordinate bounds")?;
        let render = render_frame
            .to_local(global)
            .ok_or("Particle placement is outside the render frame")?;
        let initial = &effect.initial_transform;
        if !initial.translation.is_finite()
            || !initial.rotation.is_finite()
            || !initial.scale.is_finite()
        {
            return Err(format!(
                "{}: non-finite particle initial transform",
                placement.effect
            ));
        }
        let anchor = Transform::from_translation(Vec3::from_array(render.metres()))
            .with_rotation(Quat::from_rotation_y(placement.yaw_degrees.to_radians()))
            .with_scale(Vec3::splat(placement.scale));
        let transform = anchor.mul_transform(initial.to_transform());
        if LocalPosition::new(transform.translation.to_array()).is_none()
            || !transform.rotation.is_finite()
            || !transform.scale.is_finite()
        {
            return Err(format!(
                "{}: particle transform is outside the render frame",
                placement.effect
            ));
        }
        projected.push(CatalogPlacement {
            id: placement.id,
            effect: placement.effect.clone(),
            transform,
        });
    }
    Ok(projected)
}

/// Particle snapshots prepared for one immutable chunk residency candidate.
pub struct PreparedResidency {
    pub chunks: Vec<(ChunkId, PreparedCatalog)>,
    pub counts: CatalogCounts,
    pub decoded_image_bytes: usize,
}

/// Admit a candidate while the previous residency is still alive. Retained
/// chunks reuse existing runtime assets and are not bound a second time. The
/// caller must supply only immutable, identity-matched chunks it will actually
/// reuse. CPU images count both complete snapshots because candidate images are
/// already decoded even when their runtime assets are retained.
pub fn check_residency_staging_budget(
    active_counts: CatalogCounts,
    active_image_bytes: usize,
    candidate: &PreparedResidency,
    retained: &BTreeSet<ChunkId>,
    limits: CatalogCounts,
    max_decoded_image_bytes: usize,
) -> Result<(), String> {
    let mut total = CatalogCounts::default();
    let mut seen = BTreeSet::new();
    for (id, catalog) in &candidate.chunks {
        if !seen.insert(*id) {
            return Err("Duplicate particle chunk".into());
        }
        total = total
            .checked_add(catalog.counts)
            .ok_or("Candidate particle count overflow")?;
    }
    if total != candidate.counts || !total.fits_within(limits) {
        return Err("Candidate particles exceed supplied count limits".into());
    }
    let mut counts = active_counts;
    for (id, catalog) in &candidate.chunks {
        if !retained.contains(id) {
            counts = counts
                .checked_add(catalog.counts)
                .ok_or("Staged particle count overflow")?;
        }
    }
    if !counts.fits_within(limits) {
        return Err("Active and staged particles exceed supplied count limits".into());
    }
    let image_bytes = active_image_bytes
        .checked_add(candidate.decoded_image_bytes)
        .ok_or("Staged particle image size overflow")?;
    if image_bytes > max_decoded_image_bytes {
        return Err("Active and staged particles exceed decoded image limit".into());
    }
    Ok(())
}

/// Verify every visual in an already identity-checked residency and prepare
/// its manifest-bound particle catalogs. Positions are projected from each
/// chunk's own f64 origin to the index frame. Limits apply to the entire
/// candidate, including repeated effects across chunks. This performs blocking
/// I/O and image decoding; it does not publish or render any candidate and
/// callers must separately account for old-plus-new staging resources.
pub fn prepare_resident_particles(
    residency: &theia_network::chunks::Residency,
    limits: CatalogCounts,
    max_decoded_image_bytes: usize,
) -> Result<PreparedResidency, String> {
    let mut chunks = Vec::new();
    let mut counts = CatalogCounts::default();
    let mut decoded_image_bytes = 0usize;
    for id in residency.loaded() {
        let directory = residency.index().directory(id);
        let root = directory
            .parent()
            .ok_or("Chunk directory has no asset root")?;
        let manifest = residency
            .manifest(id)
            .ok_or("Loaded chunk has no manifest")?;
        if !manifest.has_particle_catalog() {
            manifest
                .verify_visual_in(&directory.join("terrain.glb"), root)
                .map_err(|e| e.to_string())?;
            continue;
        }
        let mut catalog_bytes = Vec::new();
        std::fs::File::open(directory.join("particles/catalog.json"))
            .map_err(|e| e.to_string())?
            .take(theia_network::particle_content::MAX_CATALOG_BYTES + 1)
            .read_to_end(&mut catalog_bytes)
            .map_err(|e| e.to_string())?;
        let catalog = manifest
            .verify_particle_catalog(&catalog_bytes)
            .map_err(|e| e.to_string())?;
        manifest
            .verify_visual_in_for_particle_catalog(
                &directory.join("terrain.glb"),
                root,
                &catalog_bytes,
            )
            .map_err(|e| e.to_string())?;
        let origin = id
            .origin(INITIAL_CHUNK_METRES)
            .ok_or("Invalid chunk origin")?;
        let remaining = max_decoded_image_bytes
            .checked_sub(decoded_image_bytes)
            .ok_or("Particle residency exceeds decoded image limit")?;
        let prepared = prepare_catalog_with_image_limit(
            &catalog,
            &directory.join("particles/effects"),
            &directory.join("particles/textures"),
            Path::new("particles"),
            LocalFrame::new(origin),
            residency.index().frame(),
            limits,
            remaining,
        )?;
        counts = counts
            .checked_add(prepared.counts)
            .ok_or("Particle residency count overflow")?;
        if !counts.fits_within(limits) {
            return Err("Particle residency exceeds supplied count limits".into());
        }
        for texture in &prepared.textures {
            decoded_image_bytes = decoded_image_bytes
                .checked_add(texture.image.data.as_ref().map_or(0, Vec::len))
                .ok_or("Particle residency image size overflow")?;
        }
        if decoded_image_bytes > max_decoded_image_bytes {
            return Err("Particle residency exceeds decoded image limit".into());
        }
        chunks.push((id, prepared));
    }
    Ok(PreparedResidency {
        chunks,
        counts,
        decoded_image_bytes,
    })
}

/// A cataloged PNG decoded from the verified bytes with its authored color role.
pub struct CatalogTexture {
    pub filename: String,
    pub color_space: ColorSpace,
    pub image: Image,
}

/// A CPU-side, catalog-consistent snapshot for a future client integration.
pub struct PreparedCatalog {
    pub effects: Vec<CatalogEffect>,
    pub textures: Vec<CatalogTexture>,
    pub placements: Vec<CatalogPlacement>,
    pub counts: CatalogCounts,
}

/// Validate and prepare one already manifest-verified catalog without creating
/// any render entities. The caller supplies explicit count limits. These limits
/// cover only [`CatalogCounts`]; remaining simulation and total CPU/GPU resource
/// admission is still required. Blocking I/O and PNG decoding must run off the
/// simulation and render update paths. Failure returns no partial snapshot.
pub fn prepare_catalog(
    catalog: &Catalog,
    effect_directory: &Path,
    texture_directory: &Path,
    package: &Path,
    content_frame: LocalFrame,
    render_frame: LocalFrame,
    limits: CatalogCounts,
) -> Result<PreparedCatalog, String> {
    prepare_catalog_with_image_limit(
        catalog,
        effect_directory,
        texture_directory,
        package,
        content_frame,
        render_frame,
        limits,
        usize::MAX,
    )
}

/// As [`prepare_catalog`], but checks the verified PNG header sizes against the
/// caller's aggregate decoded-image limit before invoking the decoder.
#[allow(clippy::too_many_arguments)]
pub fn prepare_catalog_with_image_limit(
    catalog: &Catalog,
    effect_directory: &Path,
    texture_directory: &Path,
    package: &Path,
    content_frame: LocalFrame,
    render_frame: LocalFrame,
    limits: CatalogCounts,
    max_decoded_image_bytes: usize,
) -> Result<PreparedCatalog, String> {
    let effects = load_catalog_effects(catalog, effect_directory, package)?;
    for effect in &effects {
        simulation::validate(&effect.asset).map_err(|error| format!("{}: {error}", effect.id))?;
    }
    let counts = count_catalog_resources(catalog, &effects)?;
    if !counts.fits_within(limits) {
        return Err("Particle catalog exceeds the supplied count limits".into());
    }
    let placements = project_catalog_placements(catalog, &effects, content_frame, render_frame)?;
    let textures =
        decode_catalog_textures_with_limit(catalog, texture_directory, max_decoded_image_bytes)?;
    let decoded = textures
        .iter()
        .try_fold(0usize, |sum, texture| {
            sum.checked_add(texture.image.data.as_ref().map_or(0, Vec::len))
        })
        .ok_or("Particle decoded image size overflow")?;
    if decoded > max_decoded_image_bytes {
        return Err("Particle decoded images exceed supplied limit".into());
    }
    Ok(PreparedCatalog {
        effects,
        textures,
        placements,
        counts,
    })
}

/// A prepared effect installed in Bevy's asset stores.
pub struct BoundEffect {
    pub id: String,
    pub handle: Handle<ParticlesAsset>,
}

/// Runtime handles own the verified images for as long as the catalog is used.
pub struct BoundCatalog {
    pub effects: Vec<BoundEffect>,
    pub textures: Vec<Handle<Image>>,
    pub placements: Vec<CatalogPlacement>,
    pub counts: CatalogCounts,
}

/// Install the exact decoded images in a prepared snapshot and replace the
/// packaged paths in its effects with strong runtime handles. Validate every
/// reference before inserting any assets. This does not admit simulation
/// parameters or spawn effects; the caller must enforce those policies.
pub fn bind_prepared_catalog(
    mut prepared: PreparedCatalog,
    package: &Path,
    images: &mut Assets<Image>,
    particles: &mut Assets<ParticlesAsset>,
) -> Result<BoundCatalog, String> {
    validate_package_path(package)?;
    let mut expected = BTreeSet::new();
    for texture in &prepared.textures {
        let filename = Path::new(&texture.filename);
        if filename.components().count() != 1
            || !matches!(filename.components().next(), Some(Component::Normal(_)))
        {
            return Err("Invalid prepared particle texture filename".into());
        }
        let path = package
            .join("textures")
            .join(filename)
            .to_str()
            .ok_or("Particle package path must be UTF-8")?
            .replace('\\', "/");
        if !expected.insert((path, texture.color_space)) {
            return Err("Duplicate prepared particle texture".into());
        }
    }
    let mut used = BTreeSet::new();
    for effect in &prepared.effects {
        for emitter in &effect.asset.emitters {
            let DrawPassMaterial::Standard(material) = &emitter.draw_pass.material else {
                return Err("Packaged particle effect uses a custom shader".into());
            };
            for (texture, role) in [
                (&material.base_color_texture, ColorSpace::Srgb),
                (&material.emissive_texture, ColorSpace::Srgb),
                (&material.metallic_roughness_texture, ColorSpace::Linear),
                (&material.normal_map_texture, ColorSpace::Linear),
                (&material.occlusion_texture, ColorSpace::Linear),
            ] {
                match texture {
                    None | Some(TextureRef::Preset(_)) => {}
                    Some(TextureRef::Asset(path)) if expected.contains(&(path.clone(), role)) => {
                        used.insert((path.clone(), role));
                    }
                    _ => return Err("Prepared particle texture reference is invalid".into()),
                }
            }
        }
    }
    if used != expected {
        return Err("Prepared particle texture is unreferenced".into());
    }
    let mut handles = BTreeMap::new();
    for texture in prepared.textures {
        let path = package
            .join("textures")
            .join(texture.filename)
            .to_str()
            .expect("validated UTF-8 package path")
            .replace('\\', "/");
        handles.insert((path, texture.color_space), images.add(texture.image));
    }
    for effect in &mut prepared.effects {
        for emitter in &mut effect.asset.emitters {
            let DrawPassMaterial::Standard(material) = &mut emitter.draw_pass.material else {
                unreachable!()
            };
            for (texture, role) in [
                (&mut material.base_color_texture, ColorSpace::Srgb),
                (&mut material.emissive_texture, ColorSpace::Srgb),
                (&mut material.metallic_roughness_texture, ColorSpace::Linear),
                (&mut material.normal_map_texture, ColorSpace::Linear),
                (&mut material.occlusion_texture, ColorSpace::Linear),
            ] {
                if let Some(TextureRef::Asset(path)) = texture {
                    *texture = Some(TextureRef::Runtime(handles[&(path.clone(), role)].clone()));
                }
            }
        }
    }
    let textures = handles.into_values().collect();
    let effects = prepared
        .effects
        .into_iter()
        .map(|effect| BoundEffect {
            id: effect.id,
            handle: particles.add(effect.asset),
        })
        .collect();
    Ok(BoundCatalog {
        effects,
        textures,
        placements: prepared.placements,
        counts: prepared.counts,
    })
}

/// Read and decode cataloged PNGs using Bevy's standard image decoder. This
/// performs blocking file I/O and CPU decoding. It does not register assets at
/// package paths, upload textures, enforce a GPU budget or start playback.
pub fn decode_catalog_textures(
    catalog: &Catalog,
    directory: &Path,
) -> Result<Vec<CatalogTexture>, String> {
    decode_catalog_textures_with_limit(catalog, directory, usize::MAX)
}

fn decode_catalog_textures_with_limit(
    catalog: &Catalog,
    directory: &Path,
    max_decoded_image_bytes: usize,
) -> Result<Vec<CatalogTexture>, String> {
    let buffers = catalog
        .read_textures_with_rgba_limit(
            directory,
            u64::try_from(max_decoded_image_bytes).unwrap_or(u64::MAX),
        )
        .map_err(|e| e.to_string())?;
    catalog
        .textures()
        .iter()
        .zip(buffers)
        .map(|(texture, bytes)| {
            let image = Image::from_buffer(
                &bytes,
                ImageType::Extension("png"),
                CompressedImageFormats::NONE,
                texture.color_space == ColorSpace::Srgb,
                ImageSampler::default(),
                RenderAssetUsages::default(),
            )
            .map_err(|error| format!("Invalid particle texture {}: {error}", texture.filename()))?;
            Ok(CatalogTexture {
                filename: texture.filename(),
                color_space: texture.color_space,
                image,
            })
        })
        .collect()
}

/// Read catalog effects, verify their exact bytes, migrate their original RON
/// format, check allocation arithmetic and rebase their texture references.
/// This is not admission for playback: callers must independently verify and
/// decode texture files, validate all simulation parameters and enforce actual
/// CPU/GPU budgets before creating render entities. It performs blocking file
/// I/O and belongs off the simulation and render update paths.
pub fn load_catalog_effects(
    catalog: &Catalog,
    directory: &Path,
    package: &Path,
) -> Result<Vec<CatalogEffect>, String> {
    let buffers = catalog.read_effects(directory).map_err(|e| e.to_string())?;
    catalog_effects_from_buffers(catalog, &buffers, package)
}

fn catalog_effects_from_buffers(
    catalog: &Catalog,
    buffers: &[Vec<u8>],
    package: &Path,
) -> Result<Vec<CatalogEffect>, String> {
    if buffers.len() != catalog.effects().len() {
        return Err("Particle effect snapshot count mismatch".into());
    }
    let mut assets = Vec::with_capacity(buffers.len());
    for (effect, bytes) in catalog.effects().iter().zip(buffers) {
        let asset = versions::migrate(bytes)
            .map_err(|error| format!("{}: invalid particle effect: {error}", effect.id))?
            .asset;
        for (index, emitter) in asset.emitters.iter().enumerate() {
            if emitter_counts(emitter).is_none() {
                return Err(format!(
                    "{}: emitter {index} has invalid allocation arithmetic",
                    effect.id
                ));
            }
        }
        assets.push(asset);
    }
    let assets = rebase_effect_textures(catalog, &assets, package)?;
    Ok(catalog
        .effects()
        .iter()
        .zip(assets)
        .map(|(effect, asset)| CatalogEffect {
            id: effect.id.clone(),
            asset,
        })
        .collect())
}

/// Queue catalog texture variants using standard Bevy image loading. `package`
/// is a relative package directory under the configured Bevy asset root; an
/// empty path selects the root package. The caller must verify the catalog and
/// texture files first and keep these handles alive until effects are removed.
/// Each variant needs its own asset path, and these must be the first requests
/// for those paths so Bevy applies the intended loader settings.
pub fn preload_textures(
    server: &AssetServer,
    catalog: &Catalog,
    package: &Path,
) -> Result<Vec<Handle<Image>>, String> {
    validate_package_path(package)?;
    let mut handles = Vec::with_capacity(catalog.textures().len());
    for texture in catalog.textures() {
        let path = package.join("textures").join(texture.filename());
        let path = path
            .to_str()
            .ok_or("Particle package path must be UTF-8")?
            .replace('\\', "/");
        let srgb = texture.color_space == ColorSpace::Srgb;
        handles.push(
            server
                .load_builder()
                .with_settings(move |settings: &mut ImageLoaderSettings| {
                    settings.is_srgb = srgb;
                })
                .load(path),
        );
    }
    Ok(handles)
}

fn validate_package_path(package: &Path) -> Result<(), String> {
    if package.components().any(|part| match part {
        Component::Normal(name) => name.to_str().is_none_or(|name| {
            name.is_empty()
                || !name
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.'))
        }),
        _ => true,
    }) {
        return Err("Particle package path must be relative and contain no traversal".into());
    }
    Ok(())
}

/// Validate catalog texture references and rebase them under the package asset
/// path for Sprinkles. This does not verify file bytes, simulation parameters,
/// resource budgets or effect identities; callers must do those separately.
/// The returned clones are safe to modify without changing source assets.
pub fn rebase_effect_textures(
    catalog: &Catalog,
    effects: &[ParticlesAsset],
    package: &Path,
) -> Result<Vec<ParticlesAsset>, String> {
    validate_package_path(package)?;
    let expected: BTreeSet<_> = catalog
        .textures()
        .iter()
        .map(|texture| {
            (
                format!("textures/{}", texture.filename()),
                texture.color_space,
            )
        })
        .collect();
    let mut used = BTreeSet::new();
    let mut rebased = effects.to_vec();
    for effect in &mut rebased {
        if !effect.sprinkles_editor.is_empty() {
            return Err("Packaged particle effect contains editor paths".into());
        }
        for emitter in &mut effect.emitters {
            let DrawPassMaterial::Standard(material) = &mut emitter.draw_pass.material else {
                return Err("Packaged particle effect uses a custom shader".into());
            };
            for (texture, role) in [
                (&mut material.base_color_texture, ColorSpace::Srgb),
                (&mut material.emissive_texture, ColorSpace::Srgb),
                (&mut material.metallic_roughness_texture, ColorSpace::Linear),
                (&mut material.normal_map_texture, ColorSpace::Linear),
                (&mut material.occlusion_texture, ColorSpace::Linear),
            ] {
                let Some(texture) = texture else { continue };
                match texture {
                    TextureRef::Preset(_) => {}
                    TextureRef::Local(_) | TextureRef::Runtime(_) => {
                        return Err(
                            "Packaged particle texture cannot use a local path or runtime handle"
                                .into(),
                        );
                    }
                    TextureRef::Asset(path) => {
                        let key = (path.clone(), role);
                        if !expected.contains(&key) {
                            return Err("Particle texture reference is absent from the catalog or has the wrong color space".into());
                        }
                        used.insert(key);
                        *path = package
                            .join(&*path)
                            .to_str()
                            .ok_or("Particle package path must be UTF-8")?
                            .replace('\\', "/");
                    }
                }
            }
        }
    }
    if used != expected {
        return Err("Particle catalog contains an unreferenced texture".into());
    }
    Ok(rebased)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::{
        asset::{AssetApp, AssetPlugin, Assets},
        image::{CompressedImageFormats, ImageLoader, ImagePlugin},
        prelude::*,
    };
    use serde_json::json;
    use std::{
        fs,
        sync::atomic::{AtomicU64, Ordering},
        time::{Duration, Instant},
    };
    use theia_network::terrain::sha256;
    static NEXT: AtomicU64 = AtomicU64::new(0);
    const PNG: &[u8] = include_bytes!("../../network/tests/fixtures/shared-image.png");

    #[test]
    fn prepares_a_manifest_bound_snapshot_and_rejects_partial_or_over_limit_data() {
        use bevy_sprinkles::prelude::{EmitterData, ParticlesDimension};
        use theia_core::coordinates::GlobalPosition;
        use theia_network::{content::Manifest, particle_content::Texture};

        let root = std::env::temp_dir().join(format!(
            "theia-vfx-prepared-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        let effect_dir = root.join("effects");
        let texture_dir = root.join("textures");
        fs::create_dir_all(&effect_dir).unwrap();
        fs::create_dir_all(&texture_dir).unwrap();
        let texture = Texture {
            sha256: sha256(PNG),
            bytes: PNG.len() as u64,
            color_space: ColorSpace::Srgb,
        };
        fs::write(texture_dir.join(texture.filename()), PNG).unwrap();
        let linear = Texture {
            color_space: ColorSpace::Linear,
            ..texture
        };
        fs::write(texture_dir.join(linear.filename()), PNG).unwrap();
        let mut emitter = EmitterData::default();
        let DrawPassMaterial::Standard(material) = &mut emitter.draw_pass.material else {
            unreachable!()
        };
        material.base_color_texture = Some(TextureRef::Asset(format!(
            "textures/{}",
            texture.filename()
        )));
        material.normal_map_texture =
            Some(TextureRef::Asset(format!("textures/{}", linear.filename())));
        let asset = ParticlesAsset::new(
            "Fire".into(),
            ParticlesDimension::D3,
            default(),
            vec![emitter],
            vec![],
            false,
            default(),
        );
        let effect = ron::to_string(&asset).unwrap().into_bytes();
        let effect_path = effect_dir.join("effect-vfx.fire.ron");
        fs::write(&effect_path, &effect).unwrap();
        let catalog_bytes = serde_json::to_vec(&json!({"schema":1,
            "effects":[{"id":"vfx.fire","bytes":effect.len(),"sha256":sha256(&effect)}],
            "placements":[{"id":7,"effect":"vfx.fire","position":[0.125,1.,2.],"yaw_degrees":0.,"scale":1.}],
            "textures":[{"sha256":sha256(PNG),"bytes":PNG.len(),"color_space":"srgb"},
                        {"sha256":sha256(PNG),"bytes":PNG.len(),"color_space":"linear"}]
        })).unwrap();
        let manifest = Manifest::decode(
            &serde_json::to_vec(&json!({"schema":4,"content_id":"world.test","revision":1,
                "collision_schema":4,"collision_bytes":1,"collision_sha256":sha256(b"c"),
                "visual_bytes":1,"visual_sha256":sha256(b"v"),
                "particle_bytes":catalog_bytes.len(),"particle_sha256":sha256(&catalog_bytes)
            }))
            .unwrap(),
        )
        .unwrap();
        let catalog = manifest.verify_particle_catalog(&catalog_bytes).unwrap();
        let frame = LocalFrame::new(
            GlobalPosition::new([128_000_000_000., 4., -128_000_000_000.]).unwrap(),
        );
        let effects =
            load_catalog_effects(&catalog, &effect_dir, Path::new("world/revision")).unwrap();
        let limits = count_catalog_resources(&catalog, &effects).unwrap();
        let prepare = |limits, render| {
            prepare_catalog(
                &catalog,
                &effect_dir,
                &texture_dir,
                Path::new("world/revision"),
                frame,
                render,
                limits,
            )
        };
        let prepared = prepare(limits, frame).unwrap();
        assert_eq!(prepared.effects.len(), 1);
        assert_eq!(prepared.textures.len(), 2);
        assert_eq!(prepared.placements[0].id, 7);
        assert_eq!(prepared.counts, limits);
        assert!(
            prepared.textures[0]
                .image
                .texture_descriptor
                .format
                .is_srgb()
        );
        let mut images = Assets::<Image>::default();
        let mut particle_assets = Assets::<ParticlesAsset>::default();
        let mismatch = prepare(limits, frame).unwrap();
        assert!(
            bind_prepared_catalog(
                mismatch,
                Path::new("wrong/package"),
                &mut images,
                &mut particle_assets
            )
            .is_err()
        );
        assert_eq!(images.len(), 0);
        assert_eq!(particle_assets.len(), 0);
        let original_pixels = prepared.textures[0].image.data.clone();
        let mut too_small = limits;
        too_small.particle_slots -= 1;
        assert!(prepare(too_small, frame).is_err());
        let far = LocalFrame::new(GlobalPosition::new([0.; 3]).unwrap());
        assert!(prepare(limits, far).is_err());
        fs::write(&effect_path, b"changed").unwrap();
        assert!(prepare(limits, frame).is_err());
        fs::write(&effect_path, effect).unwrap();
        fs::write(texture_dir.join(texture.filename()), b"changed").unwrap();
        assert!(prepare(limits, frame).is_err());
        let bound = bind_prepared_catalog(
            prepared,
            Path::new("world/revision"),
            &mut images,
            &mut particle_assets,
        )
        .unwrap();
        assert_eq!(bound.textures.len(), 2);
        let installed = particle_assets.get(&bound.effects[0].handle).unwrap();
        let DrawPassMaterial::Standard(material) = &installed.emitters[0].draw_pass.material else {
            unreachable!()
        };
        let Some(TextureRef::Runtime(handle)) = &material.base_color_texture else {
            panic!("expected verified image handle")
        };
        assert!(bound.textures.iter().any(|image| image.id() == handle.id()));
        let color_image = images.get(handle).unwrap();
        assert_eq!(color_image.data, original_pixels);
        assert!(color_image.texture_descriptor.format.is_srgb());
        let Some(TextureRef::Runtime(normal)) = &material.normal_map_texture else {
            panic!("expected verified linear image handle")
        };
        assert_ne!(normal.id(), handle.id());
        assert!(
            !images
                .get(normal)
                .unwrap()
                .texture_descriptor
                .format
                .is_srgb()
        );
        assert!(
            ron::to_string(installed).is_err(),
            "runtime-only handles must never be serialized"
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn catalog_counts_include_repeated_placements_disabled_emitters_and_sub_buffers() {
        use bevy_sprinkles::{
            asset::{EmitterEmission, SubEmitterConfig},
            prelude::{EmitterData, ParticlesDimension},
        };
        let document = json!({"schema":1,
            "effects":[{"id":"vfx.fire","bytes":4,"sha256":sha256(b"fire")}],
            "placements":[
                {"id":1,"effect":"vfx.fire","position":[0.,0.,0.],"yaw_degrees":0.,"scale":1.},
                {"id":2,"effect":"vfx.fire","position":[1.,0.,0.],"yaw_degrees":0.,"scale":1.}
            ]
        });
        let catalog = Catalog::decode(&serde_json::to_vec(&document).unwrap()).unwrap();
        let first = EmitterData {
            sub_emitter: Some(SubEmitterConfig {
                target_emitter: 1,
                ..default()
            }),
            ..default()
        };
        let second = EmitterData {
            enabled: false,
            emission: EmitterEmission {
                particles_amount: 5,
                ..default()
            },
            ..default()
        };
        let first_count = emitter_counts(&first).unwrap();
        let second_count = emitter_counts(&second).unwrap();
        let asset = ParticlesAsset::new(
            "Fire".into(),
            ParticlesDimension::D3,
            default(),
            vec![first, second],
            vec![],
            false,
            default(),
        );
        let mut effects = vec![CatalogEffect {
            id: "vfx.fire".into(),
            asset,
        }];
        let counts = count_catalog_resources(&catalog, &effects).unwrap();
        assert_eq!(counts.placements, 2);
        assert_eq!(counts.emitters, 4);
        assert_eq!(
            counts.vertices,
            2 * u64::from(first_count.vertices + second_count.vertices)
        );
        assert_eq!(
            counts.particle_slots,
            2 * u64::from(first_count.particle_slots + second_count.particle_slots)
        );
        assert_eq!(counts.sub_emitter_words, 2 * (4 + 12 * 5));
        assert!(counts.fits_within(counts));
        let mut limits = counts;
        limits.particle_slots -= 1;
        assert!(!counts.fits_within(limits));
        effects[0].asset.emitters[0].emission.particles_amount = u32::MAX;
        assert!(count_catalog_resources(&catalog, &effects).is_err());
        effects[0].id = "vfx.other".into();
        assert!(count_catalog_resources(&catalog, &effects).is_err());
    }

    #[test]
    fn catalog_placements_reproject_from_global_without_accumulating_offsets() {
        use bevy_sprinkles::prelude::{EmitterData, ParticlesDimension};
        use theia_core::coordinates::GlobalPosition;

        let document = json!({"schema":1,
            "effects":[{"id":"vfx.fire","bytes":4,"sha256":sha256(b"fire")}],
            "placements":[{"id":7,"effect":"vfx.fire","position":[0.125,1.,2.],"yaw_degrees":90.,"scale":2.}]
        });
        let catalog = Catalog::decode(&serde_json::to_vec(&document).unwrap()).unwrap();
        let mut asset = ParticlesAsset::new(
            "Fire".into(),
            ParticlesDimension::D3,
            default(),
            vec![EmitterData::default()],
            vec![],
            false,
            default(),
        );
        asset.initial_transform.translation = Vec3::X;
        asset.initial_transform.scale = Vec3::splat(2.);
        let mut effects = vec![CatalogEffect {
            id: "vfx.fire".into(),
            asset,
        }];
        let content = LocalFrame::new(
            GlobalPosition::new([128_000_000_000., 4., -128_000_000_000.]).unwrap(),
        );
        let shifted = LocalFrame::new(
            GlobalPosition::new([128_000_000_128., 0., -128_000_000_128.]).unwrap(),
        );
        let original = project_catalog_placements(&catalog, &effects, content, content).unwrap();
        let moved = project_catalog_placements(&catalog, &effects, content, shifted).unwrap();
        assert_eq!(original[0].id, 7);
        assert_eq!(original[0].effect, "vfx.fire");
        assert!(
            original[0]
                .transform
                .translation
                .abs_diff_eq(Vec3::new(0.125, 1., 0.), 0.0001)
        );
        assert!(
            original[0]
                .transform
                .scale
                .abs_diff_eq(Vec3::splat(4.), 0.0001)
        );
        let global = |frame: LocalFrame, placement: &CatalogPlacement| {
            frame
                .to_global(LocalPosition::new(placement.transform.translation.to_array()).unwrap())
                .unwrap()
                .metres()
        };
        assert_eq!(global(content, &original[0]), global(shifted, &moved[0]));
        let restored = project_catalog_placements(&catalog, &effects, content, content).unwrap();
        assert_eq!(original[0].transform, restored[0].transform);
        let far = LocalFrame::new(GlobalPosition::new([0.; 3]).unwrap());
        assert!(project_catalog_placements(&catalog, &effects, content, far).is_err());
        effects[0].asset.initial_transform.translation = Vec3::splat(f32::MAX);
        assert!(project_catalog_placements(&catalog, &effects, content, content).is_err());
        effects[0].asset.initial_transform.translation = Vec3::ZERO;
        effects[0].asset.initial_transform.rotation.x = f32::NAN;
        assert!(project_catalog_placements(&catalog, &effects, content, content).is_err());
        effects[0].id = "vfx.other".into();
        assert!(project_catalog_placements(&catalog, &effects, content, content).is_err());
    }

    #[test]
    fn catalog_loading_hashes_parses_and_rebases_the_same_effect_bytes() {
        let root = std::env::temp_dir().join(format!(
            "theia-vfx-effect-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&root).unwrap();
        let path = root.join("effect-vfx.fire.ron");
        let example = include_bytes!(
            "../../../third_party/sprinkles/crates/bevy_sprinkles_editor/src/assets/examples/magic-puff.ron"
        );
        let catalog_for = |bytes: &[u8], textures: serde_json::Value| {
            Catalog::decode(&serde_json::to_vec(&json!({"schema":1,
                "effects":[{"id":"vfx.fire","bytes":bytes.len(),"sha256":sha256(bytes)}],
                "placements":[{"id":1,"effect":"vfx.fire","position":[0.,0.,0.],"yaw_degrees":0.,"scale":1.}],
                "textures":textures
            })).unwrap()).unwrap()
        };
        let catalog = catalog_for(example, json!([]));
        fs::write(&path, example).unwrap();
        let effects = load_catalog_effects(&catalog, &root, Path::new("world/revision")).unwrap();
        assert_eq!(effects.len(), 1);
        assert_eq!(effects[0].id, "vfx.fire");
        assert_eq!(effects[0].asset.emitters.len(), 6);
        fs::write(&path, b"changed").unwrap();
        assert_eq!(effects[0].asset.emitters.len(), 6);
        assert!(load_catalog_effects(&catalog, &root, Path::new("world")).is_err());

        let malformed = b"not RON";
        fs::write(&path, malformed).unwrap();
        assert!(
            load_catalog_effects(
                &catalog_for(malformed, json!([])),
                &root,
                Path::new("world")
            )
            .is_err()
        );

        let mut asset = versions::migrate(example).unwrap().asset;
        asset.emitters[0].draw_pass.material = DrawPassMaterial::CustomShader {
            vertex_shader: None,
            fragment_shader: None,
        };
        let custom = ron::to_string(&asset).unwrap().into_bytes();
        fs::write(&path, &custom).unwrap();
        assert!(
            load_catalog_effects(&catalog_for(&custom, json!([])), &root, Path::new("world"))
                .is_err()
        );

        let mut asset = versions::migrate(example).unwrap().asset;
        asset.emitters[0].emission.particles_amount = u32::MAX;
        let oversized = ron::to_string(&asset).unwrap().into_bytes();
        fs::write(&path, &oversized).unwrap();
        assert!(
            load_catalog_effects(
                &catalog_for(&oversized, json!([])),
                &root,
                Path::new("world")
            )
            .is_err()
        );

        let mut asset = versions::migrate(example).unwrap().asset;
        let texture = json!({"sha256":sha256(PNG),"bytes":PNG.len(),"color_space":"srgb"});
        let texture_catalog = catalog_for(example, json!([texture.clone()]));
        let relative = format!("textures/{}", texture_catalog.textures()[0].filename());
        let DrawPassMaterial::Standard(material) = &mut asset.emitters[0].draw_pass.material else {
            unreachable!()
        };
        material.base_color_texture = Some(TextureRef::Asset(relative.clone()));
        let textured = ron::to_string(&asset).unwrap().into_bytes();
        fs::write(&path, &textured).unwrap();
        let catalog = catalog_for(&textured, json!([texture]));
        let effects = load_catalog_effects(&catalog, &root, Path::new("world/revision")).unwrap();
        let DrawPassMaterial::Standard(material) = &effects[0].asset.emitters[0].draw_pass.material
        else {
            unreachable!()
        };
        assert_eq!(
            material.base_color_texture,
            Some(TextureRef::Asset(format!("world/revision/{relative}")))
        );
        assert!(load_catalog_effects(&catalog, &root, Path::new("../escape")).is_err());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn rebasing_requires_cataloged_role_specific_references_and_no_editor_paths() {
        use bevy_sprinkles::prelude::{EmitterData, ParticlesDimension};
        let document = json!({"schema":1,
            "effects":[{"id":"vfx.fire","bytes":4,"sha256":sha256(b"fire")}],
            "placements":[{"id":1,"effect":"vfx.fire","position":[0.,0.,0.],"yaw_degrees":0.,"scale":1.}],
            "textures":[{"sha256":sha256(PNG),"bytes":PNG.len(),"color_space":"srgb"},
                        {"sha256":sha256(PNG),"bytes":PNG.len(),"color_space":"linear"}]
        });
        let catalog = Catalog::decode(&serde_json::to_vec(&document).unwrap()).unwrap();
        let color = format!("textures/{}", catalog.textures()[0].filename());
        let linear = format!("textures/{}", catalog.textures()[1].filename());
        let mut effect = ParticlesAsset::new(
            "Fire".into(),
            ParticlesDimension::D3,
            default(),
            vec![EmitterData::default()],
            vec![],
            false,
            default(),
        );
        let DrawPassMaterial::Standard(material) = &mut effect.emitters[0].draw_pass.material
        else {
            unreachable!()
        };
        material.base_color_texture = Some(TextureRef::Asset(color.clone()));
        material.normal_map_texture = Some(TextureRef::Asset(linear.clone()));
        let result =
            rebase_effect_textures(&catalog, &[effect.clone()], Path::new("world/revision"))
                .unwrap();
        let DrawPassMaterial::Standard(material) = &result[0].emitters[0].draw_pass.material else {
            unreachable!()
        };
        assert_eq!(
            material.base_color_texture,
            Some(TextureRef::Asset(format!("world/revision/{color}")))
        );
        assert_eq!(
            material.normal_map_texture,
            Some(TextureRef::Asset(format!("world/revision/{linear}")))
        );
        let DrawPassMaterial::Standard(original) = &effect.emitters[0].draw_pass.material else {
            unreachable!()
        };
        assert_eq!(
            original.base_color_texture,
            Some(TextureRef::Asset(color.clone()))
        );
        assert!(
            rebase_effect_textures(&catalog, &[effect.clone()], Path::new("../escape")).is_err()
        );
        let mut invalid = effect.clone();
        invalid
            .sprinkles_editor
            .assets_folder
            .push("/private/".into());
        assert!(rebase_effect_textures(&catalog, &[invalid], Path::new("world")).is_err());
        let mut invalid = effect.clone();
        let DrawPassMaterial::Standard(material) = &mut invalid.emitters[0].draw_pass.material
        else {
            unreachable!()
        };
        material.base_color_texture = Some(TextureRef::Local("/private/file.png".into()));
        assert!(rebase_effect_textures(&catalog, &[invalid], Path::new("world")).is_err());
        let mut invalid = effect.clone();
        let DrawPassMaterial::Standard(material) = &mut invalid.emitters[0].draw_pass.material
        else {
            unreachable!()
        };
        material.base_color_texture = Some(TextureRef::Asset(linear));
        assert!(rebase_effect_textures(&catalog, &[invalid], Path::new("world")).is_err());
        let mut invalid = effect;
        let DrawPassMaterial::Standard(material) = &mut invalid.emitters[0].draw_pass.material
        else {
            unreachable!()
        };
        material.normal_map_texture = None;
        assert!(rebase_effect_textures(&catalog, &[invalid], Path::new("world")).is_err());
    }

    #[test]
    fn package_variants_load_with_distinct_color_spaces_and_safe_paths() {
        let root = std::env::temp_dir().join(format!(
            "theia-vfx-load-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(root.join("chunk_0_0/textures")).unwrap();
        let document = json!({"schema":1,
            "effects":[{"id":"vfx.fire","bytes":4,"sha256":sha256(b"fire")}],
            "placements":[{"id":1,"effect":"vfx.fire","position":[0.,0.,0.],"yaw_degrees":0.,"scale":1.}],
            "textures":[{"sha256":sha256(PNG),"bytes":PNG.len(),"color_space":"srgb"},
                        {"sha256":sha256(PNG),"bytes":PNG.len(),"color_space":"linear"}]
        });
        let catalog = Catalog::decode(&serde_json::to_vec(&document).unwrap()).unwrap();
        for texture in catalog.textures() {
            fs::write(
                root.join("chunk_0_0/textures").join(texture.filename()),
                PNG,
            )
            .unwrap();
        }
        catalog
            .verify_textures(&root.join("chunk_0_0/textures"))
            .unwrap();
        let decoded = decode_catalog_textures(&catalog, &root.join("chunk_0_0/textures")).unwrap();
        assert_eq!(decoded.len(), 2);
        assert_eq!(decoded[0].filename, catalog.textures()[0].filename());
        assert!(decoded[0].image.texture_descriptor.format.is_srgb());
        assert!(!decoded[1].image.texture_descriptor.format.is_srgb());
        assert_eq!(decoded[0].color_space, ColorSpace::Srgb);
        let first_path = root
            .join("chunk_0_0/textures")
            .join(catalog.textures()[0].filename());
        fs::write(&first_path, b"changed").unwrap();
        assert!(decode_catalog_textures(&catalog, &root.join("chunk_0_0/textures")).is_err());
        assert!(
            decoded[0]
                .image
                .data
                .as_ref()
                .is_some_and(|data| !data.is_empty())
        );
        fs::write(&first_path, PNG).unwrap();

        let mut corrupt = PNG.to_vec();
        let idat = corrupt
            .windows(4)
            .position(|bytes| bytes == b"IDAT")
            .unwrap();
        corrupt[idat + 5] ^= 0xff;
        let invalid = json!({"schema":1,
            "effects":[{"id":"vfx.fire","bytes":4,"sha256":sha256(b"fire")}],
            "placements":[{"id":1,"effect":"vfx.fire","position":[0.,0.,0.],"yaw_degrees":0.,"scale":1.}],
            "textures":[{"sha256":sha256(&corrupt),"bytes":corrupt.len(),"color_space":"srgb"}]
        });
        let invalid = Catalog::decode(&serde_json::to_vec(&invalid).unwrap()).unwrap();
        fs::write(
            root.join("chunk_0_0/textures")
                .join(invalid.textures()[0].filename()),
            &corrupt,
        )
        .unwrap();
        // The hash and bounded header match; actual image decoding still fails.
        invalid
            .read_textures(&root.join("chunk_0_0/textures"))
            .unwrap();
        assert!(decode_catalog_textures(&invalid, &root.join("chunk_0_0/textures")).is_err());
        let mut app = App::new();
        app.add_plugins((
            MinimalPlugins,
            AssetPlugin {
                file_path: root.to_str().unwrap().into(),
                ..default()
            },
            ImagePlugin::default(),
        ));
        app.register_asset_loader(ImageLoader::new(CompressedImageFormats::NONE));
        app.finish();
        app.cleanup();
        let server = app.world().resource::<AssetServer>();
        for invalid in [
            "../escape",
            "/absolute",
            "chunk_0_0/../other",
            "a\\..\\other",
            "source:other",
        ] {
            assert!(
                preload_textures(server, &catalog, Path::new(invalid)).is_err(),
                "{invalid}"
            );
        }
        let handles = preload_textures(server, &catalog, Path::new("chunk_0_0")).unwrap();
        assert_ne!(handles[0].id(), handles[1].id());
        for (texture, handle) in catalog.textures().iter().zip(&handles) {
            let path = format!("chunk_0_0/textures/{}", texture.filename());
            assert_eq!(server.load::<Image>(path).id(), handle.id());
        }
        let until = Instant::now() + Duration::from_secs(5);
        loop {
            app.update();
            let images = app.world().resource::<Assets<Image>>();
            if let (Some(color), Some(data)) = (images.get(&handles[0]), images.get(&handles[1])) {
                assert!(color.texture_descriptor.format.is_srgb());
                assert!(!data.texture_descriptor.format.is_srgb());
                break;
            }
            assert!(Instant::now() < until, "Image loading did not complete");
            std::thread::sleep(Duration::from_millis(5));
        }
        drop(app);
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn partition_keeps_only_each_chunks_effect_and_texture_dependencies() {
        use bevy_sprinkles::prelude::{EmitterData, ParticlesDimension};
        use theia_core::coordinates::GlobalPosition;
        use theia_network::particle_content::Texture;
        let texture = Texture {
            sha256: sha256(PNG),
            bytes: PNG.len() as u64,
            color_space: ColorSpace::Srgb,
        };
        let document = json!({"schema":1,
            "effects":[{"id":"vfx.textured","bytes":1,"sha256":vec![0;32]}, {"id":"vfx.plain","bytes":1,"sha256":vec![1;32]}],
            "placements":[{"id":1,"effect":"vfx.textured","position":[-0.25,0.,1.],"yaw_degrees":0.,"scale":1.},
                          {"id":2,"effect":"vfx.plain","position":[128.,0.,1.],"yaw_degrees":0.,"scale":1.}],
            "textures":[{"sha256":texture.sha256,"bytes":texture.bytes,"color_space":"srgb"}]});
        let catalog = Catalog::decode(&serde_json::to_vec(&document).unwrap()).unwrap();
        let mut emitter = EmitterData::default();
        let DrawPassMaterial::Standard(material) = &mut emitter.draw_pass.material else {
            unreachable!()
        };
        material.base_color_texture = Some(TextureRef::Asset(format!(
            "particles/textures/{}",
            texture.filename()
        )));
        let make = |id: &str, emitter| CatalogEffect {
            id: id.into(),
            asset: ParticlesAsset::new(
                id.into(),
                ParticlesDimension::D3,
                default(),
                vec![emitter],
                vec![],
                false,
                default(),
            ),
        };
        let effects = vec![
            make("vfx.textured", emitter),
            make("vfx.plain", EmitterData::default()),
        ];
        let ids = [ChunkId { x: -1, z: 0 }, ChunkId { x: 1, z: 0 }];
        let result = partition_catalog(
            &catalog,
            &effects,
            LocalFrame::new(GlobalPosition::new([0.; 3]).unwrap()),
            &ids.into_iter().collect(),
        )
        .unwrap();
        let textured = Catalog::decode(&result[&ids[0]]).unwrap();
        assert_eq!(textured.placements()[0].position[0], 127.75);
        assert_eq!(textured.effects()[0].id, "vfx.textured");
        assert_eq!(textured.textures().len(), 1);
        let plain = Catalog::decode(&result[&ids[1]]).unwrap();
        assert_eq!(plain.placements()[0].position[0], 0.);
        assert_eq!(plain.effects()[0].id, "vfx.plain");
        assert!(plain.textures().is_empty());
    }

    #[test]
    fn resident_particles_are_manifest_bound_projected_and_aggregate_limited() {
        use bevy_sprinkles::prelude::{EmitterData, ParticlesDimension};
        use theia_network::{
            chunks::{Index, Residency},
            content::Manifest,
            particle_content::Texture,
        };
        let root = std::env::temp_dir().join(format!(
            "theia-vfx-residency-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&root).unwrap();
        let base = 1_000_000_000i64;
        let texture = Texture {
            sha256: sha256(PNG),
            bytes: PNG.len() as u64,
            color_space: ColorSpace::Srgb,
        };
        let mut emitter = EmitterData::default();
        let DrawPassMaterial::Standard(material) = &mut emitter.draw_pass.material else {
            unreachable!()
        };
        material.base_color_texture = Some(TextureRef::Asset(format!(
            "textures/{}",
            texture.filename()
        )));
        let asset = ParticlesAsset::new(
            "chunk".into(),
            ParticlesDimension::D3,
            default(),
            vec![emitter],
            vec![],
            false,
            default(),
        );
        let effect = ron::to_string(&asset).unwrap().into_bytes();
        let catalog = serde_json::to_vec(&json!({"schema":1,
            "effects":[{"id":"vfx.test","bytes":effect.len(),"sha256":sha256(&effect)}],
            "placements":[{"id":1,"effect":"vfx.test","position":[1.,0.,2.],"yaw_degrees":0.,"scale":1.}],
            "textures":[{"sha256":sha256(PNG),"bytes":PNG.len(),"color_space":"srgb"}]})).unwrap();
        let mut json_chunk = br#"{"asset":{"version":"2.0"}}"#.to_vec();
        while !json_chunk.len().is_multiple_of(4) {
            json_chunk.push(b' ');
        }
        let mut visual = b"glTF".to_vec();
        visual.extend(2u32.to_le_bytes());
        visual.extend((20u32 + json_chunk.len() as u32).to_le_bytes());
        visual.extend((json_chunk.len() as u32).to_le_bytes());
        visual.extend(b"JSON");
        visual.extend(json_chunk);
        let mut entries = Vec::new();
        for offset in 0..3 {
            let id = ChunkId {
                x: base + offset,
                z: base,
            };
            let dir = root.join(format!("chunk_{}_{}", id.x, id.z));
            fs::create_dir_all(dir.join("particles/effects")).unwrap();
            fs::create_dir_all(dir.join("particles/textures")).unwrap();
            fs::write(dir.join("terrain.glb"), &visual).unwrap();
            let profile = serde_json::to_vec(&json!({"schema":2,"bounds":[[-1.,-1.],[129.,129.]],"source_origin":[0.,0.,0.],
                "visual_sha256":sha256(&visual),"triangles":[[[-1.,0.,-1.],[-1.,0.,129.],[129.,0.,-1.]],
                [[129.,0.,-1.],[-1.,0.,129.],[129.,0.,129.]]]})).unwrap();
            fs::write(dir.join("terrain.json"), &profile).unwrap();
            let mut document = json!({"schema":1,"content_id":format!("theia.test.chunk.x{}", id.x),"revision":1,
                "collision_schema":2,"collision_bytes":profile.len(),"collision_sha256":sha256(&profile),
                "visual_bytes":visual.len(),"visual_sha256":sha256(&visual)});
            if offset < 2 {
                document["schema"] = json!(4);
                document["particle_bytes"] = json!(catalog.len());
                document["particle_sha256"] = json!(sha256(&catalog));
                fs::write(dir.join("particles/catalog.json"), &catalog).unwrap();
                fs::write(dir.join("particles/effects/effect-vfx.test.ron"), &effect).unwrap();
                fs::write(dir.join("particles/textures").join(texture.filename()), PNG).unwrap();
            }
            let manifest = serde_json::to_vec(&document).unwrap();
            fs::write(dir.join("world.manifest.json"), &manifest).unwrap();
            entries.push(
                json!({"id":[id.x,id.z],"package":Manifest::decode(&manifest).unwrap().identity()}),
            );
        }
        fs::write(
            root.join("world.chunks.json"),
            serde_json::to_vec(&json!({"schema":1,"chunk_metres":128,
            "query_origin":[base as f64 * 128. + 8.,0.,base as f64 * 128. - 8.],"chunks":entries}))
            .unwrap(),
        )
        .unwrap();
        let mut residency = Residency::new(Index::load(&root).unwrap(), 3).unwrap();
        let ids: Vec<_> = (0..3)
            .map(|offset| ChunkId {
                x: base + offset,
                z: base,
            })
            .collect();
        residency.replace(&ids, &[]).unwrap();
        let mut source: serde_json::Value = serde_json::from_slice(&catalog).unwrap();
        source["placements"] = json!([
            {"id":1,"effect":"vfx.test","position":[127.,0.,2.],"yaw_degrees":10.,"scale":1.},
            {"id":2,"effect":"vfx.test","position":[128.,0.,2.],"yaw_degrees":20.,"scale":2.}
        ]);
        let source = Catalog::decode(&serde_json::to_vec(&source).unwrap()).unwrap();
        let source_effects = load_catalog_effects(
            &source,
            &residency
                .index()
                .directory(ids[0])
                .join("particles/effects"),
            Path::new("particles"),
        )
        .unwrap();
        let frame = LocalFrame::new(ids[0].origin(INITIAL_CHUNK_METRES).unwrap());
        let available: BTreeSet<_> = ids.iter().copied().collect();
        let partitioned = partition_catalog(&source, &source_effects, frame, &available).unwrap();
        assert_eq!(partitioned.len(), 2);
        let original_bytes = fs::read(
            residency
                .index()
                .directory(ids[2])
                .join("world.manifest.json"),
        )
        .unwrap();
        let original = Manifest::decode(&original_bytes).unwrap();
        let (new_bytes, identity) =
            bind_chunk_catalog(&original_bytes, &partitioned[&ids[0]]).unwrap();
        let revised = Manifest::decode(&new_bytes).unwrap();
        assert_eq!(revised.revision(), original.revision() + 1);
        assert_ne!(identity, original.identity());
        revised
            .verify_particle_catalog(&partitioned[&ids[0]])
            .unwrap();
        assert!(bind_chunk_catalog(&new_bytes, &partitioned[&ids[0]]).is_err());
        assert!(bind_chunk_catalog(&original_bytes, b"not a catalog").is_err());
        let mut projected: serde_json::Value = serde_json::from_slice(&original_bytes).unwrap();
        projected["schema"] = json!(3);
        projected["shared_images"] = json!([
            {"sha256":sha256(PNG),"bytes":PNG.len(),"color_space":"srgb","sampler":[10497,10497,0,0]},
            {"sha256":sha256(PNG),"bytes":PNG.len(),"color_space":"linear","sampler":[33071,33071,9728,9728]}
        ]);
        let projected = serde_json::to_vec(&projected).unwrap();
        Manifest::decode(&projected).unwrap();
        assert!(
            bind_chunk_catalog(&projected, &partitioned[&ids[0]])
                .unwrap_err()
                .contains("material profile")
        );
        for (id, expected_x) in [(ids[0], 127.), (ids[1], 0.)] {
            let chunk = Catalog::decode(&partitioned[&id]).unwrap();
            assert_eq!(chunk.placements()[0].position, [expected_x, 0., 2.]);
            assert_eq!(chunk.effects().len(), 1);
            assert_eq!(chunk.textures().len(), 1);
        }
        assert!(
            partition_catalog(
                &source,
                &source_effects,
                frame,
                &[ids[0]].into_iter().collect()
            )
            .is_err()
        );
        assert!(
            residency.verify_visuals().is_err(),
            "legacy path must still reject particle chunks"
        );
        let particle_dir = residency.index().directory(ids[0]).join("particles");
        let drafts = draft_particle_chunks(
            &source,
            &particle_dir.join("effects"),
            &particle_dir.join("textures"),
            frame,
            &available,
            1_000_000,
        )
        .unwrap();
        assert_eq!(drafts.len(), 2);
        for draft in drafts.values() {
            assert_eq!(draft.effects[0].1, effect);
            assert_eq!(draft.textures[0].1, PNG);
        }
        assert!(
            draft_particle_chunks(
                &source,
                &particle_dir.join("effects"),
                &particle_dir.join("textures"),
                frame,
                &available,
                1
            )
            .is_err()
        );
        let texture_path = particle_dir.join("textures").join(texture.filename());
        fs::write(&texture_path, b"tampered").unwrap();
        assert!(
            draft_particle_chunks(
                &source,
                &particle_dir.join("effects"),
                &particle_dir.join("textures"),
                frame,
                &available,
                1_000_000
            )
            .is_err()
        );
        fs::write(texture_path, PNG).unwrap();
        let limits = CatalogCounts {
            placements: 2,
            emitters: 2,
            vertices: 1_000_000,
            indices: 3_000_000,
            particle_slots: 128,
            trail_history_entries: 1024,
            sub_emitter_words: 1024,
        };
        let prepared = prepare_resident_particles(&residency, limits, 1024).unwrap();
        assert_eq!(prepared.chunks.len(), 2);
        assert_eq!(prepared.counts.placements, 2);
        assert!(prepared.decoded_image_bytes > 0);
        assert!(
            check_residency_staging_budget(
                CatalogCounts::default(),
                0,
                &prepared,
                &BTreeSet::new(),
                limits,
                1024
            )
            .is_ok()
        );
        assert!(
            check_residency_staging_budget(
                prepared.counts,
                prepared.decoded_image_bytes,
                &prepared,
                &BTreeSet::new(),
                limits,
                1024
            )
            .is_err()
        );
        assert!(
            check_residency_staging_budget(
                CatalogCounts::default(),
                usize::MAX,
                &prepared,
                &BTreeSet::new(),
                limits,
                usize::MAX
            )
            .is_err()
        );
        let retained: BTreeSet<_> = prepared.chunks.iter().map(|(id, _)| *id).collect();
        assert!(
            check_residency_staging_budget(prepared.counts, 0, &prepared, &retained, limits, 1024)
                .is_ok()
        );
        let retained: BTreeSet<_> = [prepared.chunks[0].0].into_iter().collect();
        assert!(
            check_residency_staging_budget(
                prepared.chunks[0].1.counts,
                0,
                &prepared,
                &retained,
                limits,
                1024
            )
            .is_ok()
        );
        assert!(
            check_residency_staging_budget(prepared.counts, 0, &prepared, &retained, limits, 1024)
                .is_err()
        );
        assert_eq!(
            prepared.chunks[0].1.placements[0].transform.translation,
            Vec3::new(-7., 0., 10.)
        );
        assert_eq!(
            prepared.chunks[1].1.placements[0].transform.translation,
            Vec3::new(121., 0., 10.)
        );
        assert!(prepare_resident_particles(&residency, limits, 0).is_err());
        assert!(
            prepare_resident_particles(&residency, limits, prepared.decoded_image_bytes).is_ok()
        );
        assert!(
            prepare_resident_particles(&residency, limits, prepared.decoded_image_bytes - 1)
                .is_err()
        );
        let mut too_small = limits;
        too_small.placements = 1;
        assert!(prepare_resident_particles(&residency, too_small, 1024).is_err());
        let overflow = CatalogCounts {
            placements: u64::MAX,
            ..default()
        };
        assert!(
            overflow
                .checked_add(CatalogCounts {
                    placements: 1,
                    ..default()
                })
                .is_none()
        );
        let second = residency.index().directory(ids[1]);
        fs::write(
            second.join("particles/effects/effect-vfx.test.ron"),
            b"tampered",
        )
        .unwrap();
        assert!(prepare_resident_particles(&residency, limits, 1024).is_err());
        assert_eq!(residency.loaded(), ids);
        fs::remove_dir_all(root).unwrap();
    }
}
