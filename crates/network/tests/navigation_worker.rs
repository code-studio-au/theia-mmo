//! Bounded query transport/publication contracts, not creature gameplay.
use std::{
    num::NonZeroU64,
    thread,
    time::{Duration, Instant},
};
use theia_core::{
    coordinates::GlobalPosition,
    identity::{EntityId, EntityKey, WorldEpoch},
    player_life::LifeKey,
};
use theia_network::navigation::{
    Agent, Detail, Tile,
    query::Query,
    worker::{Reply, Request, SubmitError, Tag, Worker},
};

fn key(id: u64) -> EntityKey {
    EntityKey {
        epoch: WorldEpoch::new(7).unwrap(),
        entity: EntityId::new(id).unwrap(),
    }
}
fn point(x: f64, z: f64) -> GlobalPosition {
    GlobalPosition::new([7936. + x, 0., 7936. + z]).unwrap()
}
fn fixture() -> (Worker, Request) {
    let vertices = vec![
        [0., 0., 0.],
        [0., 0., 128.],
        [128., 0., 128.],
        [128., 0., 0.],
    ];
    let tile = Tile {
        schema: 1,
        id: [62, 62],
        source: [8; 32],
        agent: Agent::development(),
        vertices: vertices.clone(),
        polygons: vec![vec![0, 1, 2, 3]],
        detail: Detail {
            vertices,
            triangles: vec![[0, 1, 2], [0, 2, 3]],
            polygons: vec![[0, 4, 0, 2]],
        },
    };
    let worker = Worker::spawn(Query::new(&[tile]).unwrap(), [9; 32]).unwrap();
    let request = Request {
        tag: Tag {
            creature: key(1),
            target: Some(LifeKey {
                actor: key(2),
                generation: NonZeroU64::new(1).unwrap(),
            }),
            revision: 1,
        },
        start: point(10., 10.),
        end: point(90., 90.),
        submitted: 10,
        expires: 40,
    };
    (worker, request)
}
fn receive(worker: &mut Worker) -> Reply {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        if let Some(reply) = worker.poll().unwrap() {
            return reply;
        }
        assert!(Instant::now() < deadline, "worker failed to publish");
        thread::sleep(Duration::from_millis(1));
    }
}
#[test]
fn one_outstanding_query_and_exact_incarnation_revision_time_publication() {
    let (mut worker, request) = fixture();
    assert_eq!(worker.poll().unwrap().map(|_| ()), None);
    assert_eq!(worker.submit(request), Ok(()));
    assert_eq!(worker.pending(), Some(request));
    assert_eq!(worker.submit(request), Err(SubmitError::Busy));
    let reply = receive(&mut worker);
    assert_eq!(worker.pending(), None);
    let route = reply.current([9; 32], request.tag, 10).unwrap();
    assert!(route.len() >= 2);
    assert!((route.last().unwrap().metres()[0] - request.end.metres()[0]).abs() < 0.01);
    assert!(reply.current([9; 32], request.tag, 40).is_some());
    for now in [9, 41, u64::MAX] {
        assert!(reply.current([9; 32], request.tag, now).is_none());
    }
    assert!(reply.current([7; 32], request.tag, 20).is_none());
    let mut tag = request.tag;
    tag.revision += 1;
    assert!(reply.current([9; 32], tag, 20).is_none());
    let mut tag = request.tag;
    tag.target.as_mut().unwrap().generation = NonZeroU64::new(2).unwrap();
    assert!(reply.current([9; 32], tag, 20).is_none());
    let mut tag = request.tag;
    tag.creature = key(3);
    assert!(reply.current([9; 32], tag, 20).is_none());
    let mut tag = request.tag;
    tag.creature.epoch = WorldEpoch::new(8).unwrap();
    assert!(reply.current([9; 32], tag, 20).is_none());
    let next = Request {
        tag: Tag {
            revision: 2,
            ..request.tag
        },
        ..request
    };
    worker.submit(next).unwrap();
    assert!(
        receive(&mut worker)
            .current([9; 32], next.tag, 20)
            .is_some()
    );
}
#[test]
fn invalid_and_unavailable_destinations_never_publish_a_direct_line_fallback() {
    let (mut worker, request) = fixture();
    for bad in [
        Request {
            expires: 10,
            ..request
        },
        Request {
            expires: 9,
            ..request
        },
        Request {
            expires: 131,
            ..request
        },
        Request {
            tag: Tag {
                revision: 0,
                ..request.tag
            },
            ..request
        },
        Request {
            tag: Tag {
                creature: EntityKey {
                    epoch: WorldEpoch::new(8).unwrap(),
                    ..key(1)
                },
                ..request.tag
            },
            ..request
        },
    ] {
        assert_eq!(worker.submit(bad), Err(SubmitError::Invalid));
        assert_eq!(worker.pending(), None);
    }
    let missing = Request {
        end: point(200., 90.),
        ..request
    };
    worker.submit(missing).unwrap();
    let reply = receive(&mut worker);
    assert!(reply.route.is_err());
    assert!(reply.current([9; 32], request.tag, 20).is_none());
    worker
        .submit(Request {
            tag: Tag {
                target: None,
                revision: 2,
                ..request.tag
            },
            ..request
        })
        .unwrap();
    assert!(receive(&mut worker).route.is_ok());
}

use theia_network::navigation::scheduler::{MAX_REQUESTERS, Scheduler};

fn receive_scheduled(scheduler: &mut Scheduler, creature: EntityKey, now: u64) -> Reply {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        scheduler.pump(now).unwrap();
        if let Some(reply) = scheduler.take_reply(creature, now) {
            return reply;
        }
        assert!(Instant::now() < deadline, "scheduler failed to publish");
        thread::sleep(Duration::from_millis(1));
    }
}
fn for_creature(request: Request, id: u64) -> Request {
    Request {
        tag: Tag {
            creature: key(id),
            ..request.tag
        },
        ..request
    }
}
#[test]
fn scheduler_bounds_admission_and_delivers_each_creature_without_consuming_other_replies() {
    let (worker, request) = fixture();
    let mut scheduler = Scheduler::new(worker);
    for id in 1..=MAX_REQUESTERS as u64 {
        scheduler.submit(for_creature(request, id)).unwrap();
    }
    assert_eq!(scheduler.registered(), MAX_REQUESTERS);
    assert_eq!(scheduler.queued(), MAX_REQUESTERS);
    assert_eq!(
        scheduler.submit(for_creature(request, 100)),
        Err(SubmitError::Busy)
    );
    // Waiting for the last actor lets all earlier replies accumulate in their
    // bounded independent mailboxes. Taking its reply must not consume theirs.
    let reply = receive_scheduled(&mut scheduler, key(MAX_REQUESTERS as u64), 10);
    assert!(
        reply
            .current(scheduler.navigation(), reply.request.tag, 10)
            .is_some()
    );
    assert_eq!(scheduler.queued(), 0);
    for id in 1..MAX_REQUESTERS as u64 {
        assert_eq!(
            scheduler.take_reply(key(id), 10).unwrap().request,
            for_creature(request, id)
        );
        assert!(scheduler.take_reply(key(id), 10).is_none());
    }
    scheduler.retire(key(1));
    scheduler.submit(for_creature(request, 100)).unwrap();
    assert!(
        receive_scheduled(&mut scheduler, key(100), 10)
            .route
            .is_ok()
    );
}
#[test]
fn scheduler_replaces_active_goals_without_starving_another_actor() {
    let (worker, request) = fixture();
    let mut scheduler = Scheduler::new(worker);
    scheduler.submit(request).unwrap();
    scheduler.submit(for_creature(request, 3)).unwrap();
    scheduler.pump(10).unwrap(); // Actor 1 is in flight, actor 3 must be next.
    let replacement = Request {
        tag: Tag {
            revision: 2,
            ..request.tag
        },
        end: point(50., 50.),
        ..request
    };
    scheduler.submit(replacement).unwrap();
    assert_eq!(scheduler.submit(request), Err(SubmitError::Invalid));
    assert_eq!(scheduler.pending(key(1)), Some(replacement));
    let other = receive_scheduled(&mut scheduler, key(3), 10);
    assert!(other.route.is_ok());
    // That pump dispatches the replacement, but cannot have polled it yet.
    assert!(scheduler.take_reply(key(1), 10).is_none());
    let reply = receive_scheduled(&mut scheduler, key(1), 10);
    assert_eq!(reply.request, replacement);
    assert!(
        reply
            .current(scheduler.navigation(), replacement.tag, 10)
            .is_some()
    );
}
#[test]
fn scheduler_retirement_discards_inflight_life_even_when_key_is_registered_again() {
    let (worker, request) = fixture();
    let mut scheduler = Scheduler::new(worker);
    scheduler.submit(request).unwrap();
    scheduler.pump(10).unwrap();
    scheduler.retire(key(1));
    let fresh = Request {
        end: point(40., 40.),
        ..request
    };
    scheduler.submit(fresh).unwrap();
    assert_eq!(receive_scheduled(&mut scheduler, key(1), 10).request, fresh);
    scheduler.retire(key(1));
    assert!(scheduler.pending(key(1)).is_none());
    scheduler.submit(for_creature(request, 4)).unwrap();
    assert_eq!(
        receive_scheduled(&mut scheduler, key(4), 10)
            .request
            .tag
            .creature,
        key(4)
    );
}
#[test]
fn scheduler_expires_queued_active_and_ready_work_and_preserves_route_errors() {
    let (worker, request) = fixture();
    let mut scheduler = Scheduler::new(worker);
    scheduler.submit(request).unwrap();
    scheduler.submit(for_creature(request, 3)).unwrap();
    scheduler.pump(10).unwrap();
    scheduler.pump(41).unwrap();
    assert_eq!(scheduler.queued(), 0);
    assert!(scheduler.take_reply(key(1), 41).is_none());
    assert!(scheduler.take_reply(key(3), 41).is_none());
    assert_eq!(scheduler.submit(request), Err(SubmitError::Invalid));
    assert_eq!(scheduler.pump(40), Err(SubmitError::Invalid));
    let missing = Request {
        tag: Tag {
            revision: 2,
            ..request.tag
        },
        end: point(200., 90.),
        submitted: 41,
        expires: 60,
        ..request
    };
    scheduler.submit(missing).unwrap();
    let reply = receive_scheduled(&mut scheduler, key(1), 41);
    assert_eq!(reply.request, missing);
    assert!(reply.route.is_err()); // The pursuit policy needs this failure.
    let next = Request {
        tag: Tag {
            revision: 3,
            ..request.tag
        },
        submitted: 41,
        expires: 60,
        ..request
    };
    scheduler.submit(next).unwrap();
    scheduler.pump(41).unwrap(); // Dispatch actor 1 before queuing actor 4.
    scheduler.submit(for_creature(next, 4)).unwrap();
    receive_scheduled(&mut scheduler, key(4), 41);
    assert!(scheduler.pending(key(1)).is_some()); // Ready, unconsumed reply.
    assert!(scheduler.take_reply(key(1), 61).is_none());
    assert!(scheduler.pending(key(1)).is_none());
}
