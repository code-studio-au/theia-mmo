//! Real encrypted loopback plus synthetic Bevy widget/drag events. No OS window,
//! input device, renderer, provider or SQL; not native drag qualification.
use super::*;
use bevy::picking::{backend::HitData, events::Pointer, pointer::Location};
use std::{net::UdpSocket, path::PathBuf, sync::Arc, time::Duration};
use theia_network::{client::State, renet_runtime as net};
use tokio::sync::watch;
struct Ticket(PathBuf);
impl Drop for Ticket {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
struct Stop(watch::Sender<bool>);
impl Drop for Stop {
    fn drop(&mut self) {
        self.0.send_replace(true);
    }
}
async fn until(app: &mut App, predicate: impl Fn(&State) -> bool) -> State {
    let result = tokio::time::timeout(Duration::from_secs(6), async {
        loop {
            app.update();
            let state = app.world().resource::<crate::network::Network>().state();
            assert!(
                state.error.is_none(),
                "Headless encrypted peer failed: {:?}",
                state.error
            );
            if predicate(&state) {
                return state;
            }
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    })
    .await;
    result.unwrap_or_else(|_| {
        panic!(
            "Headless encrypted ability-bar condition timed out: {:?}",
            app.world().resource::<crate::network::Network>().state()
        )
    })
}
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn real_encrypted_cast_after_book_drop_and_bar_swap_keeps_original_server_cooldown() {
    let data = include_bytes!("../../../../../content/gameplay/development-abilities.json");
    let catalog = Catalog::from_json(data).unwrap();
    let id = catalog.abilities()[0].id().clone();
    let mut world = theia_network::simulation::World::with_procedural([8060., 8000.], 1.).unwrap();
    let target = world.spawn_development_target([8068., 8000.]).unwrap();
    world.enable_combat(data, 100, 5000).unwrap();
    let identity = world.identity();
    let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
    let address = socket.local_addr().unwrap();
    let (transport, issuer) = net::secure_issuer(socket, vec![address]).unwrap();
    let issued = issuer.issue(1, None, 30).unwrap();
    let directory = std::env::temp_dir().join(format!(
        "theia-bar-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir(&directory).unwrap();
    let ticket = Ticket(directory);
    let path = ticket.0.join("ticket.bin");
    std::fs::write(&path, issued.bytes()).unwrap();
    let (stop, shutdown) = watch::channel(false);
    let stop = Stop(stop);
    let server = tokio::spawn(net::run_server_measured(transport, shutdown, world));
    let network = crate::network::Network::from_args(
        &[
            "--renet-connect".into(),
            address.to_string(),
            path.to_str().unwrap().into(),
        ],
        None,
        false,
        identity,
        true,
        None,
        None,
        None,
        None,
        Some(Arc::new(catalog.clone())),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        false,
        false,
        None,
        None,
        None,
        None,
        None,
    )
    .unwrap();
    std::fs::remove_file(path).unwrap();
    network.publish_chunks(
        theia_network::procedural::neighbours(theia_core::coordinates::ChunkId { x: 62, z: 62 }, 1)
            .into_iter()
            .collect(),
    );
    let mut app = super::super::ui::tests::widget_app_catalog(true, catalog);
    app.insert_resource(network)
        .init_resource::<super::super::book::Ui>();
    install(&mut app);
    until(&mut app, |s| {
        s.snapshot.is_some() && s.chunk_ack.is_some() && s.combat.is_some()
    })
    .await;
    app.world()
        .resource::<crate::network::Network>()
        .request_target(Some(target.entity));
    until(&mut app, |s| {
        s.targeting
            .as_ref()
            .is_some_and(|v| v.selected == Some(target.entity))
    })
    .await;
    app.world_mut()
        .resource_mut::<super::super::book::Ui>()
        .open = true;
    let source = app
        .world_mut()
        .spawn((super::super::book::BookEntry(0), Node::default()))
        .id();
    let window = app
        .world_mut()
        .query_filtered::<Entity, With<Window>>()
        .single(app.world())
        .unwrap();
    let pointer = || {
        Pointer::new(
            PointerId::Mouse,
            Location {
                target: bevy::camera::RenderTarget::Window(bevy::window::WindowRef::Entity(window))
                    .normalize(Some(window))
                    .unwrap(),
                position: Vec2::new(100., 100.),
            },
        )
    };
    let destination = app
        .world_mut()
        .query::<(Entity, &super::super::ui::ActionSlot)>()
        .iter(app.world())
        .find(|(_, s)| s.0 == 11)
        .unwrap()
        .0;
    app.world_mut().trigger(PointerDragStart {
        entity: source,
        pointer: pointer(),
        button: PointerButton::Primary,
        hit: HitData::new(source, 1., None, None),
    });
    assert!(app.world().resource::<Drag>().pending.is_some());
    app.world_mut().trigger(PointerDragDrop {
        entity: destination,
        pointer: pointer(),
        button: PointerButton::Primary,
        dropped: source,
        hit: HitData::new(destination, 1., None, None),
    });
    app.world_mut().trigger(PointerDragEnd {
        entity: source,
        pointer: pointer(),
        button: PointerButton::Primary,
        distance: Vec2::new(20., 20.),
    });
    app.update();
    assert_eq!(
        app.world().resource::<Layout>().slots[11].as_ref(),
        Some(&id)
    );
    assert!(app.world().resource::<Drag>().pending.is_none());
    app.world_mut()
        .resource_mut::<super::super::book::Ui>()
        .open = false;
    let button = app
        .world_mut()
        .query::<(Entity, &super::super::ui::ActionButton)>()
        .iter(app.world())
        .find(|(_, s)| s.0 == 11)
        .unwrap()
        .0;
    app.world_mut()
        .trigger(bevy::ui_widgets::Activate { entity: button });
    app.update();
    let (slot, _) = app.world().resource::<super::super::ui::Input>().0.unwrap();
    assert_eq!(slot, 11);
    let config = app.world().resource::<super::super::Config>();
    let ability = app
        .world()
        .resource::<Layout>()
        .ability(&config.0, slot)
        .unwrap()
        .id()
        .clone();
    assert!(
        app.world()
            .resource::<crate::network::Network>()
            .request_cast(ability, target.entity)
    );
    let casting = until(&mut app, |s| {
        s.combat
            .as_ref()
            .is_some_and(|c| c.cast.as_ref().is_some_and(|cast| cast.ability == id))
    })
    .await;
    let deadline = casting
        .combat
        .as_ref()
        .unwrap()
        .cooldowns
        .iter()
        .find(|(ability, _)| ability == &id)
        .unwrap()
        .1;
    // Another binding for the same action follows the same server deadline.
    app.world_mut()
        .resource_mut::<super::super::book::Ui>()
        .open = true;
    let bar_slot = |app: &mut App, index| {
        app.world_mut()
            .query::<(Entity, &super::super::ui::ActionSlot)>()
            .iter(app.world())
            .find(|(_, slot)| slot.0 == index)
            .unwrap()
            .0
    };
    let from = bar_slot(&mut app, 0);
    let to = bar_slot(&mut app, 10);
    app.world_mut().trigger(PointerDragStart {
        entity: from,
        pointer: pointer(),
        button: PointerButton::Primary,
        hit: HitData::new(from, 1., None, None),
    });
    assert!(app.world().resource::<Drag>().pending.is_some());
    app.world_mut().get_mut::<Window>(window).unwrap().focused = false;
    app.update();
    assert!(app.world().resource::<Drag>().pending.is_none());
    assert_eq!(
        app.world().resource::<Layout>().slots[0].as_ref(),
        Some(&id)
    );
    app.world_mut().get_mut::<Window>(window).unwrap().focused = true;
    app.world_mut().trigger(PointerDragStart {
        entity: from,
        pointer: pointer(),
        button: PointerButton::Primary,
        hit: HitData::new(from, 1., None, None),
    });
    app.world_mut().trigger(PointerDragDrop {
        entity: to,
        pointer: pointer(),
        button: PointerButton::Primary,
        dropped: from,
        hit: HitData::new(to, 1., None, None),
    });
    app.world_mut().trigger(PointerDragEnd {
        entity: from,
        pointer: pointer(),
        button: PointerButton::Primary,
        distance: Vec2::new(20., 20.),
    });
    assert!(app.world().resource::<Layout>().slots[0].is_none());
    app.world_mut()
        .resource_mut::<super::super::book::Ui>()
        .open = false;
    app.update();
    let after = app.world().resource::<crate::network::Network>().state();
    assert_eq!(
        after
            .combat
            .as_ref()
            .unwrap()
            .cooldowns
            .iter()
            .find(|(ability, _)| ability == &id)
            .unwrap()
            .1,
        deadline
    );
    until(&mut app, |s| {
        s.combat.as_ref().is_some_and(|c| {
            c.targets
                .iter()
                .any(|h| h.entity == target.entity && h.current == 80)
        })
    })
    .await;
    let config = app.world().resource::<super::super::Config>();
    let ability = app
        .world()
        .resource::<Layout>()
        .ability(&config.0, 10)
        .unwrap()
        .id()
        .clone();
    assert_eq!(ability, id);
    assert!(
        app.world()
            .resource::<crate::network::Network>()
            .request_cast(ability, target.entity)
    );
    let refused = until(&mut app, |s| {
        s.combat.as_ref().is_some_and(|c| {
            c.reply.as_ref().is_some_and(|r| {
                matches!(
                    r.outcome,
                    theia_network::simulation::combat::Outcome::Rejected(
                        theia_network::simulation::combat::Reason::Unavailable
                    )
                )
            })
        })
    })
    .await;
    assert_eq!(
        refused
            .combat
            .as_ref()
            .unwrap()
            .cooldowns
            .iter()
            .find(|(ability, _)| ability == &id)
            .unwrap()
            .1,
        deadline
    );
    drop(app);
    stop.0.send_replace(true);
    let stats = server.await.unwrap().unwrap();
    assert_eq!(stats.cast_requests_accepted, 1);
    assert_eq!(stats.cast_requests_rejected, 1);
}
