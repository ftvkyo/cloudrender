use std::time::{Duration, Instant};

use anyhow::Result;
use cgmath::{Point2, Point3};

use crate::app::App;

pub type Position = Point3<f32>;
pub type TexCoord = Point2<f32>;

mod app;

pub fn main() -> Result<()> {
    let mut app = App::init()?;

    let frames_per_second = 60;
    let frame_duration = Duration::new(0, 1_000_000_000u32 / frames_per_second);

    let points = vec![
        Position::new(0.0, 0.0, 0.0),
        Position::new(0.5, 0.0, 0.0),
        Position::new(0.0, 0.5, 0.0),
    ];

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

        app.render_frame(&points)?;

        let instant_end = Instant::now();
        let duration_rendering = instant_end - instant_start;
        let duration_to_sleep = frame_duration.saturating_sub(duration_rendering);

        ::std::thread::sleep(duration_to_sleep);
    }

    Ok(())
}
