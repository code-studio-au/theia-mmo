//! Checked allocation counts; policy limits belong to the host application.
use crate::{asset::EmitterData, mesh::checked_mesh_counts, spawning::checked_emitter_layout};

/// Counts per emitter instance, before mesh sharing or GPU/CPU memory accounting.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EmitterCounts {
    /// Replicated particle mesh vertices, or one base mesh if the amount is zero.
    pub vertices: u32,
    /// Replicated particle mesh indices, or one base mesh if the amount is zero.
    pub indices: usize,
    /// Particle records per simulation buffer.
    pub particle_slots: u32,
    /// Trail history records.
    pub trail_history_entries: u32,
}

/// Returns `None` if the runtime's allocation arithmetic would overflow.
pub fn emitter_counts(emitter: &EmitterData) -> Option<EmitterCounts> {
    let amount = emitter.emission.particles_amount;
    let (particle_slots, frames) = checked_emitter_layout(emitter)?;
    let (vertices, indices) = checked_mesh_counts(&emitter.draw_pass.mesh, amount.max(1))?;
    Some(EmitterCounts {
        vertices,
        indices,
        particle_slots,
        trail_history_entries: amount.checked_mul(frames)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::asset::{ParticleMesh, RibbonTrailShape};

    #[test]
    fn accounts_for_trails_zero_particles_and_overflow() {
        let mut emitter = EmitterData::default();
        let base = emitter_counts(&emitter).unwrap();
        assert_eq!(base.particle_slots, 8);
        assert_eq!(base.trail_history_entries, 0);
        emitter.emission.particles_amount = 0;
        let zero = emitter_counts(&emitter).unwrap();
        assert!(zero.vertices > 0 && zero.indices > 0);
        assert_eq!(zero.particle_slots, 0);
        emitter.emission.particles_amount = 5;
        emitter.trail.enabled = true;
        emitter.trail.stretch_time = 0.1;
        emitter.time.fixed_fps = 60;
        emitter.draw_pass.mesh = ParticleMesh::RibbonTrail {
            size: 1.,
            sections: 4,
            section_rings: 1,
            shape: RibbonTrailShape::Flat,
        };
        let trail = emitter_counts(&emitter).unwrap();
        assert_eq!(trail.particle_slots, 20);
        assert_eq!(trail.trail_history_entries, 30);
        emitter.emission.particles_amount = u32::MAX;
        assert!(emitter_counts(&emitter).is_none());
    }
}
