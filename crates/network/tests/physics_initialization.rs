//! A cold query scene must initialize even when callers occupy Bevy's workers.
use bevy::{
    prelude::*,
    tasks::{ComputeTaskPool, TaskPoolBuilder},
};
use std::{
    sync::{Arc, Barrier, mpsc},
    time::Duration,
};

#[test]
fn cold_shared_scene_initializes_under_concurrent_bevy_schedules() {
    ComputeTaskPool::get_or_init(|| TaskPoolBuilder::new().num_threads(2).build());
    let barrier = Arc::new(Barrier::new(8));
    let (complete, result) = mpsc::sync_channel(8);
    let mut workers = Vec::new();
    for _ in 0..8 {
        let barrier = barrier.clone();
        let complete = complete.clone();
        workers.push(std::thread::spawn(move || {
            let mut app = App::new();
            app.add_systems(Update, || {
                let ground = theia_network::world::physics_scene()
                    .ground([0., 0.], 5.)
                    .unwrap();
                assert!(ground.0.abs() < 0.001 && ground.1);
            });
            barrier.wait();
            app.update();
            complete.send(()).unwrap();
        }));
    }
    // Do not join blocked threads on failure: the harness can report the timeout
    // and exit instead of hanging indefinitely on this initialization regression.
    for _ in 0..8 {
        result
            .recv_timeout(Duration::from_secs(10))
            .expect("Cold scene initialization deadlocked");
    }
    for worker in workers {
        worker.join().unwrap();
    }
}
