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
