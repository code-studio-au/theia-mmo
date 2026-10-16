//! Explicit test-only scripted input and a real rendered screenshot; no network mock.
use crate::movement::LocalPawn;
use bevy::{
    app::AppExit,
    prelude::*,
    render::{
        renderer::RenderAdapterInfo,
        view::window::screenshot::{Screenshot, ScreenshotCaptured},
    },
};
use serde_json::{Value, json};
use std::path::PathBuf;

#[derive(Resource)]
pub struct Run {
    pub frame: u32,
    output: PathBuf,
    checkpoints: Vec<Value>,
}
impl Run {
    pub fn from_args(
        mut args: impl Iterator<Item = String>,
    ) -> Result<Option<Self>, Box<dyn std::error::Error>> {
        match args.next().as_deref() {
            None => Ok(None),
            Some("--smoke-test") => {
                let output = PathBuf::from(
                    args.next()
                        .ok_or("--smoke-test needs an output directory")?,
                );
                if args.next().is_some() {
                    return Err("Unexpected argument".into());
                }
                std::fs::create_dir_all(&output)?;
                for name in ["report.json", "scene.png"] {
                    let path = output.join(name);
                    if path.exists() {
                        std::fs::remove_file(path)?;
                    }
                }
                Ok(Some(Self {
                    frame: 0,
                    output,
                    checkpoints: Vec::new(),
                }))
            }
            Some(arg) => Err(format!(
                "Unknown argument {arg}; use --smoke-test <directory> or no arguments"
            )
            .into()),
        }
    }
}

pub fn observe(
    mut commands: Commands,
    mut run: ResMut<Run>,
    pawn: Query<&Transform, With<LocalPawn>>,
    adapter: Res<RenderAdapterInfo>,
) {
    if matches!(run.frame, 59 | 119 | 179) {
        let position = pawn
            .single()
            .expect("One local pawn")
            .translation
            .to_array();
        let frame = run.frame;
        run.checkpoints
            .push(json!({"frame":frame,"position":position}));
    }
    if run.frame == 239 {
        let output = run.output.clone();
        let checkpoints = run.checkpoints.clone();
        let gpu = json!({"name":adapter.name,"backend":format!("{:?}",adapter.backend),"device_id":adapter.device});
        let capture = move |event: On<ScreenshotCaptured>, mut exit: MessageWriter<AppExit>| {
            event
                .image
                .clone()
                .try_into_dynamic()
                .expect("Screenshot pixels")
                .save(output.join("scene.png"))
                .expect("Save screenshot");
            let report = json!({
                "schema": 1, "completed": true, "offline": true,
                "checkpoints": checkpoints, "gpu": gpu,
                "extent": [event.image.width(), event.image.height()],
                "scope": "scripted local movement; rendered screenshot; no networking, collision, animation or gameplay authority"
            });
            std::fs::write(
                output.join("report.json"),
                serde_json::to_vec_pretty(&report).unwrap(),
            )
            .expect("Write smoke report");
            exit.write(AppExit::Success);
        };
        commands
            .spawn(Screenshot::primary_window())
            .observe(capture);
    }
    assert!(run.frame < 600, "Screenshot did not complete");
    run.frame += 1;
}
