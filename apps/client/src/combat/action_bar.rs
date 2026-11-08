//! Session-local presentation bindings. Authoritative casts still use ability IDs.
use bevy::prelude::*;
use theia_core::{
    abilities::{Ability, Catalog},
    content::ContentId,
};
pub(crate) const SLOTS: usize = 12;
#[derive(Resource)]
pub(crate) struct Layout {
    slots: [Option<ContentId>; SLOTS],
    pub(crate) revision: u64,
}
impl Layout {
    pub(crate) fn new(catalog: &Catalog) -> Self {
        Self {
            revision: 0,
            slots: std::array::from_fn(|slot| {
                catalog.abilities().get(slot).map(|a| a.id().clone())
            }),
        }
    }
    pub(crate) fn ability<'a>(&self, catalog: &'a Catalog, slot: usize) -> Option<&'a Ability> {
        catalog.ability(self.slots.get(slot)?.as_ref()?.as_str())
    }
    pub(crate) fn copy(&mut self, catalog: &Catalog, id: &ContentId, to: usize) -> bool {
        if to >= SLOTS || catalog.ability(id.as_str()).is_none() {
            return false;
        }
        if self.slots[to].as_ref() == Some(id) {
            return true;
        }
        let Some(revision) = self.revision.checked_add(1) else {
            return false;
        };
        self.slots[to] = Some(id.clone());
        self.revision = revision;
        true
    }
    pub(crate) fn swap(&mut self, from: usize, to: usize) -> bool {
        if from >= SLOTS || to >= SLOTS {
            return false;
        }
        if self.slots[from] == self.slots[to] {
            return true;
        }
        let Some(revision) = self.revision.checked_add(1) else {
            return false;
        };
        self.slots.swap(from, to);
        self.revision = revision;
        true
    }
}
pub(super) fn prepare(mut commands: Commands, config: Res<super::Config>) {
    commands.insert_resource(Layout::new(&config.0));
}

use bevy::picking::{
    events::{PointerCancel, PointerDragDrop, PointerDragEnd, PointerDragStart},
    pointer::{PointerButton, PointerId},
};
use theia_core::identity::{EntityId, EntityKey, WorldEpoch};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Context {
    owner: EntityKey,
    world: [u8; 32],
}
fn context(network: Option<&crate::network::Network>) -> Option<Context> {
    let network = network?;
    if !network.connected() {
        return None;
    }
    let state = network.state();
    context_from_state(&state)
}
fn context_from_state(state: &theia_network::client::State) -> Option<Context> {
    let welcome = state.welcome.as_ref()?;
    let snapshot = state.snapshot.as_ref()?;
    if state.error.is_some()
        || state.id != Some(welcome.id)
        || welcome.world == [0; 32]
        || snapshot.epoch != welcome.epoch
        || !snapshot.players.iter().any(|p| p.id == welcome.id)
    {
        return None;
    }
    Some(Context {
        owner: EntityKey {
            epoch: WorldEpoch::new(welcome.epoch)?,
            entity: EntityId::new(welcome.id)?,
        },
        world: welcome.world,
    })
}
#[derive(Clone, Copy, Debug)]
enum Origin {
    Book { page: usize, row: usize },
    Bar(usize),
}
struct Pending {
    entity: Entity,
    id: ContentId,
    context: Context,
    origin: Origin,
}
#[derive(Resource, Default)]
pub(crate) struct Drag {
    pending: Option<Pending>,
}
impl Drag {
    fn apply(
        &mut self,
        layout: &mut Layout,
        catalog: &Catalog,
        current: Context,
        page: usize,
        dropped: Entity,
        to: usize,
    ) -> bool {
        let Some(pending) = self.pending.take() else {
            return false;
        };
        if pending.entity != dropped || pending.context != current {
            return false;
        }
        match pending.origin {
            Origin::Book { page: old, row }
                if old == page
                    && row < super::book::PAGE_SIZE
                    && old < super::book::pages(catalog.abilities().len())
                    && catalog
                        .abilities()
                        .get(old * super::book::PAGE_SIZE + row)
                        .is_some_and(|a| a.id() == &pending.id) =>
            {
                layout.copy(catalog, &pending.id, to)
            }
            Origin::Bar(from)
                if layout
                    .ability(catalog, from)
                    .is_some_and(|a| a.id() == &pending.id) =>
            {
                layout.swap(from, to)
            }
            _ => false,
        }
    }
}
pub(super) fn install(app: &mut App) {
    app.init_resource::<Drag>()
        .add_observer(start)
        .add_observer(drop_on_slot)
        .add_observer(end)
        .add_observer(pointer_cancel)
        .add_systems(PreUpdate, cancel.after(super::book::keyboard));
}
type EditSource<'w> = (
    Res<'w, super::book::Ui>,
    Res<'w, super::Config>,
    Res<'w, Layout>,
);
fn start(
    mut event: On<PointerDragStart>,
    windows: Query<&Window>,
    (book, config, layout): EditSource<'_>,
    nodes: Query<(
        Option<&super::book::BookEntry>,
        Option<&super::ui::ActionSlot>,
    )>,
    network: Option<Res<crate::network::Network>>,
    mut drag: ResMut<Drag>,
) {
    if event.pointer.id != PointerId::Mouse || event.button != PointerButton::Primary {
        return;
    }
    drag.pending = None;
    if !book.open || !windows.single().is_ok_and(|w| w.focused) {
        return;
    }
    let Some(context) = context(network.as_deref()) else {
        return;
    };
    let Ok((entry, slot)) = nodes.get(event.entity) else {
        return;
    };
    let (origin, ability) = if let Some(entry) = entry {
        (
            Origin::Book {
                page: book.page(),
                row: entry.0,
            },
            book.ability_index(entry.0)
                .and_then(|i| config.0.abilities().get(i)),
        )
    } else if let Some(slot) = slot {
        (Origin::Bar(slot.0), layout.ability(&config.0, slot.0))
    } else {
        return;
    };
    if let Some(ability) = ability {
        event.propagate(false);
        drag.pending = Some(Pending {
            entity: event.entity,
            id: ability.id().clone(),
            context,
            origin,
        });
    }
}
fn drop_on_slot(
    mut event: On<PointerDragDrop>,
    windows: Query<&Window>,
    (book, config): (Res<super::book::Ui>, Res<super::Config>),
    mut layout: ResMut<Layout>,
    slots: Query<&super::ui::ActionSlot>,
    network: Option<Res<crate::network::Network>>,
    mut drag: ResMut<Drag>,
) {
    if event.pointer.id != PointerId::Mouse || event.button != PointerButton::Primary {
        return;
    }
    if !book.open || !windows.single().is_ok_and(|w| w.focused) {
        drag.pending = None;
        return;
    }
    let Some(context) = context(network.as_deref()) else {
        drag.pending = None;
        return;
    };
    let Ok(slot) = slots.get(event.entity) else {
        return;
    };
    event.propagate(false);
    drag.apply(
        &mut layout,
        &config.0,
        context,
        book.page(),
        event.dropped,
        slot.0,
    );
}
fn end(event: On<PointerDragEnd>, mut drag: ResMut<Drag>) {
    if event.pointer.id == PointerId::Mouse
        && event.button == PointerButton::Primary
        && drag
            .pending
            .as_ref()
            .is_some_and(|p| p.entity == event.entity)
    {
        drag.pending = None;
    }
}
fn pointer_cancel(event: On<PointerCancel>, mut drag: ResMut<Drag>) {
    if event.pointer.id == PointerId::Mouse {
        drag.pending = None;
    }
}
fn cancel(
    windows: Query<&Window>,
    book: Res<super::book::Ui>,
    network: Option<Res<crate::network::Network>>,
    mut drag: ResMut<Drag>,
) {
    if !book.open
        || !windows.single().is_ok_and(|w| w.focused)
        || drag.pending.as_ref().is_some_and(|p| {
            Some(p.context) != context(network.as_deref())
                || matches!(p.origin,Origin::Book{page,..} if page!=book.page())
        })
    {
        drag.pending = None;
    }
}
pub(crate) fn drag_hint(drag: Option<&Drag>) -> Option<String> {
    drag?.pending.as_ref().map(|p| {
        format!(
            "Placing {} — drop on an action slot",
            super::ui::ability_name(p.id.as_str())
        )
    })
}

/// Read-only local layout/geometry in the existing explicit qualification mode.
pub(crate) fn snapshot(world: &mut World) -> serde_json::Value {
    let Some(layout) = world.get_resource::<Layout>() else {
        return serde_json::Value::Null;
    };
    let bindings: Vec<_> = layout
        .slots
        .iter()
        .map(|id| id.as_ref().map(|id| id.as_str().to_owned()))
        .collect();
    let revision = layout.revision;
    let dragging = world
        .get_resource::<Drag>()
        .and_then(|drag| drag_hint(Some(drag)));
    let scale = world.resource::<bevy::ui::UiScale>().0;
    let slots: Vec<_>=world.query::<(&super::ui::ActionSlot,&ComputedNode,&bevy::ui::UiGlobalTransform)>()
        .iter(world).map(|(slot,node,transform)|serde_json::json!({"slot":slot.0,"point":(transform.affine().translation*crate::fantasy_ui::window_factor(node,scale)).to_array()})).collect();
    serde_json::json!({"bindings":bindings,"revision":revision,"dragging":dragging,"slots":slots})
}

#[cfg(test)]
mod tests {
    use super::*;
    fn catalog() -> Catalog {
        Catalog::from_json(include_bytes!(
            "../../../../content/gameplay/development-classic-abilities.json"
        ))
        .unwrap()
    }
    fn context() -> Context {
        Context {
            owner: EntityKey {
                epoch: WorldEpoch::new(7).unwrap(),
                entity: EntityId::new(3).unwrap(),
            },
            world: [1; 32],
        }
    }
    fn pending(origin: Origin, id: ContentId) -> Pending {
        let mut world = World::new();
        let entity = world.spawn_empty().id();
        Pending {
            entity,
            id,
            context: context(),
            origin,
        }
    }
    #[test]
    fn drag_context_requires_current_admission_owner_world_and_snapshot_membership() {
        let (original, _) = crate::quest_journal::tests::fixture();
        assert_eq!(context_from_state(&original), Some(context()));
        for case in 0..7 {
            let mut state = original.clone();
            match case {
                0 => state.error = Some("Disconnected".into()),
                1 => state.id = Some(99),
                2 => state.welcome.as_mut().unwrap().world = [0; 32],
                3 => state.snapshot.as_mut().unwrap().epoch += 1,
                4 => state.snapshot.as_mut().unwrap().players.clear(),
                5 => state.snapshot = None,
                6 => state.welcome = None,
                _ => unreachable!(),
            }
            assert!(context_from_state(&state).is_none(), "case {case}");
        }
    }
    #[test]
    fn copied_and_swapped_slots_resolve_original_ability_ids_and_timing_with_empty_bounds() {
        let catalog = catalog();
        let mut layout = Layout::new(&catalog);
        let spell = catalog.abilities()[0].id().clone();
        let sword = catalog.abilities()[1].id().clone();
        assert!(layout.copy(&catalog, &spell, 11));
        assert_eq!(layout.ability(&catalog, 11).unwrap().id(), &spell);
        assert_eq!(
            layout.ability(&catalog, 11).unwrap().cooldown_duration(),
            catalog.abilities()[0].cooldown_duration()
        );
        assert!(layout.swap(11, 1));
        assert_eq!(layout.ability(&catalog, 11).unwrap().id(), &sword);
        assert_eq!(layout.ability(&catalog, 1).unwrap().id(), &spell);
        assert!(layout.swap(0, 10));
        assert!(layout.ability(&catalog, 0).is_none());
        assert_eq!(layout.ability(&catalog, 10).unwrap().id(), &spell);
        let before = layout.slots.clone();
        let revision = layout.revision;
        assert!(!layout.copy(&catalog, &ContentId::new("theia.unknown").unwrap(), 0));
        assert!(!layout.copy(&catalog, &spell, 12));
        assert!(!layout.swap(12, 0));
        assert_eq!(layout.slots, before);
        assert_eq!(layout.revision, revision);
        assert!(layout.copy(&catalog, &spell, 10));
        assert_eq!(layout.revision, revision);
        layout.revision = u64::MAX;
        assert!(!layout.copy(&catalog, &sword, 0));
        assert!(!layout.swap(1, 11));
        assert_eq!(layout.slots, before);
    }
    #[test]
    fn drag_copy_and_swap_require_original_source_page_entity_and_admitted_context() {
        let catalog = catalog();
        let spell = catalog.abilities()[0].id().clone();
        for case in 0..7 {
            let mut layout = Layout::new(&catalog);
            let origin = if case == 6 {
                Origin::Bar(0)
            } else {
                Origin::Book { page: 0, row: 0 }
            };
            let p = pending(origin, spell.clone());
            let entity = p.entity;
            let mut current = context();
            let mut page = 0;
            let mut dropped = entity;
            match case {
                1 => current.world = [2; 32],
                2 => current.owner.epoch = WorldEpoch::new(8).unwrap(),
                3 => current.owner.entity = EntityId::new(4).unwrap(),
                4 => page = 1,
                5 => {
                    let mut world = World::new();
                    world.spawn_empty();
                    dropped = world.spawn_empty().id();
                }
                6 => {
                    layout.swap(0, 1);
                }
                _ => {}
            }
            let mut drag = Drag { pending: Some(p) };
            assert_eq!(
                drag.apply(&mut layout, &catalog, current, page, dropped, 11),
                case == 0,
                "case {case}"
            );
            assert!(drag.pending.is_none());
            if case == 0 {
                assert_eq!(layout.ability(&catalog, 11).unwrap().id(), &spell);
            } else {
                assert!(layout.ability(&catalog, 11).is_none());
            }
        }
        let mut layout = Layout::new(&catalog);
        let p = pending(Origin::Bar(0), spell.clone());
        let entity = p.entity;
        let mut drag = Drag { pending: Some(p) };
        assert!(drag.apply(&mut layout, &catalog, context(), 0, entity, 11));
        assert!(layout.ability(&catalog, 0).is_none());
        assert_eq!(layout.ability(&catalog, 11).unwrap().id(), &spell);
    }
    #[test]
    fn closed_book_or_missing_admission_cancels_pending_drag_without_changing_layout() {
        let catalog = catalog();
        let mut app = App::new();
        let original = Layout::new(&catalog);
        let before = original.slots.clone();
        app.insert_resource(original)
            .init_resource::<super::super::book::Ui>()
            .insert_resource(Drag {
                pending: Some(pending(Origin::Bar(0), catalog.abilities()[0].id().clone())),
            })
            .add_systems(Update, cancel);
        app.world_mut().spawn(Window {
            focused: true,
            ..default()
        });
        app.update();
        assert!(app.world().resource::<Drag>().pending.is_none());
        app.world_mut()
            .resource_mut::<super::super::book::Ui>()
            .open = true;
        app.world_mut().resource_mut::<Drag>().pending =
            Some(pending(Origin::Bar(0), catalog.abilities()[0].id().clone()));
        app.update();
        assert!(app.world().resource::<Drag>().pending.is_none());
        assert_eq!(app.world().resource::<Layout>().slots, before);
        assert!(super::context(None).is_none());
    }
}

#[cfg(test)]
#[path = "action_bar/renet_tests.rs"]
mod renet_tests;
