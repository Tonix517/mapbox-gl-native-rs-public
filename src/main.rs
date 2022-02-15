extern crate image;

#[macro_use]
extern crate gfx;
extern crate gfx_text;
extern crate gfx_window_glutin;
extern crate glutin;

mod config;
mod mapbox;
mod painter;

use painter::painter::Painter;
use std::cell::RefCell;
use std::sync::Arc;
use std::time::{Duration, SystemTime};

fn main() {
    let map_arc = Arc::new(RefCell::new(mapbox::map::Map::new()));
    map_arc
        .borrow_mut()
        .set_center(config::MAP_CENTER_LATLONG.0, config::MAP_CENTER_LATLONG.1);

    let mut events_loop = glutin::EventsLoop::new();
    let mut painter = Painter::new(map_arc.clone(), &events_loop);

    // Issue map load after painter is ready. Otherwise it is possible that
    // vector tiles are loaded and parsed before painter is ready and then
    // nothing will show up on map.
    map_arc.borrow_mut().load_map(config::STYLESHEET_BASE);

    // Main Event Loop
    let mut ts = SystemTime::now();
    let mut frame_count = 0 as u32;

    let mut running = true;
    while running {
        events_loop.poll_events(|event| {
            if let glutin::Event::WindowEvent { event, .. } = event {
                match event {
                    glutin::WindowEvent::CloseRequested => running = false,
                    glutin::WindowEvent::KeyboardInput {
                        input:
                            glutin::KeyboardInput {
                                virtual_keycode: Some(glutin::VirtualKeyCode::Tab),
                                state: glutin::ElementState::Released,
                                ..
                            },
                        ..
                    } => painter.toggle_show_grid(),
                    _ => {}
                }
            }
        });

        painter.render();
        frame_count += 1;

        let now = SystemTime::now();
        let elapsed = now
            .duration_since(ts)
            .expect("Clock may have gone backwards");
        if elapsed >= Duration::new(1, 0) {
            println!(
                "[FPS]: {}",
                (frame_count as f32 / (elapsed.as_millis() as f32 / 1000.0)) as f32
            );
            ts = now;
            frame_count = 0;
        }
    }
}

// TODO: add unit tests
