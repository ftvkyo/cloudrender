use std::time::{Duration, Instant};

use anyhow::Result;
use cgmath::{Matrix4, Point2, Point3, SquareMatrix, Vector3};
use cloud::Cloud;

use crate::{app::App, camera::AppCamera};

pub type Position = Point3<f32>;
pub type TexCoord = Point2<f32>;
pub type Direction = Vector3<f32>;

pub mod app;
pub mod camera;
pub mod cloud;

pub fn main() -> Result<()> {
    let w = 800;
    let h = 600;
    let aspect = 800 as f32 / 600 as f32;

    let mut app = App::init(w, h)?;

    let frames_per_second = 60;
    let frame_duration = Duration::new(0, 1_000_000_000u32 / frames_per_second);

    let cloud = Cloud::<20>::new();

    let camera = AppCamera::new(aspect);
    let model = Matrix4::<f32>::identity();

    'quit: loop {
        {
            use sdl3::event::Event;
            use sdl3::keyboard::Keycode;

            for event in app.poll_iter() {
                if let Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } = event
                {
                    break 'quit;
                }
            }
        }

        let instant_start = Instant::now();

        app.update_uniforms(&model, &camera)?;
        app.render_frame(&cloud.points)?;

        let instant_end = Instant::now();
        let duration_rendering = instant_end - instant_start;
        let duration_to_sleep = frame_duration.saturating_sub(duration_rendering);

        ::std::thread::sleep(duration_to_sleep);
    }

    Ok(())
}
