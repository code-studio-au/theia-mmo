//! Trusted local playtest spawns keyed by issuer client ID, never arrival order.
use crate::{Result, procedural::Source, simulation::World};
use serde::Deserialize;
use theia_core::coordinates::GlobalPosition;

pub const MAX_BYTES: usize = 16 * 1024;
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Document {
    schema: u32,
    world_identity: [u8; 32],
    players: Vec<Player>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Player {
    client_id: u64,
    feet: [f64; 3],
}
/// Parsed plan; no position is admitted until `prepare` checks actual collision.
pub struct Plan {
    identity: [u8; 32],
    centres: [Option<GlobalPosition>; 2],
}
/// All configured positions were checked against this exact source.
pub struct Prepared {
    identity: [u8; 32],
    centres: [Option<GlobalPosition>; 2],
}
fn identity(world: &World) -> Result<[u8; 32]> {
    let runtime = world
        .procedural()
        .ok_or("Test players require authored terrain")?;
    if !world.prediction_enabled()
        || !world.classic_enabled()
        || world.dev_tour
        || runtime.speed != 1.
    {
        return Err("Test players require normal-speed Classic prediction without a tour".into());
    }
    let Source::Authored(store) = &runtime.source else {
        return Err("Test players require an authored world".into());
    };
    Ok(store.identity())
}
impl Plan {
    pub fn from_json(bytes: &[u8], world: &World) -> Result<Self> {
        if bytes.len() > MAX_BYTES {
            return Err("Test player profile exceeds byte budget".into());
        }
        let document: Document = serde_json::from_slice(bytes)?;
        if document.schema != 1
            || !(1..=2).contains(&document.players.len())
            || document.world_identity != identity(world)?
        {
            return Err("Invalid test player schema, count or authored world identity".into());
        }
        let mut centres = [None; 2];
        for player in document.players {
            if !(1..=2).contains(&player.client_id) {
                return Err("Test player client ID must be 1 or 2".into());
            }
            let mut centre = player.feet;
            let feet = GlobalPosition::new(centre).ok_or("Invalid test player feet")?;
            if crate::procedural::owner(feet).is_none() {
                return Err("Test player feet leave supported terrain domain".into());
            }
            centre[1] += f64::from(crate::world::PAWN_Y);
            let centre = GlobalPosition::new(centre).ok_or("Invalid test player centre")?;
            if centres[(player.client_id - 1) as usize]
                .replace(centre)
                .is_some()
            {
                return Err("Duplicate test player client ID".into());
            }
        }
        Ok(Self {
            identity: document.world_identity,
            centres,
        })
    }
    /// Retain/prepare these patches outside the fixed simulation step.
    pub fn positions(&self) -> Vec<GlobalPosition> {
        self.centres.into_iter().flatten().collect()
    }
    pub fn prepare(self, world: &World) -> Result<Prepared> {
        let prepared = Prepared {
            identity: self.identity,
            centres: self.centres,
        };
        prepared.validate(world)?;
        Ok(prepared)
    }
}
impl Prepared {
    pub(super) fn validate(&self, world: &World) -> Result<()> {
        if self.identity != identity(world)? || world.completed_tick() != 0 || world.has_players() {
            return Err("Test player admission requires its inactive authored world".into());
        }
        for position in self.positions() {
            world.validate_restored_player(position)?;
        }
        Ok(())
    }
    pub(super) fn positions(&self) -> Vec<GlobalPosition> {
        self.centres.into_iter().flatten().collect()
    }
    pub(super) fn position(&self, client: u64) -> Option<GlobalPosition> {
        let index = usize::try_from(client.checked_sub(1)?).ok()?;
        self.centres.get(index).copied().flatten()
    }
}
