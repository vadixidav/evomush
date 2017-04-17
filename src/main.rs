#![feature(conservative_impl_trait)]
#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate rand;
extern crate heapsize;
extern crate sdl2;
extern crate gapush;
extern crate glowygraph as gg;
extern crate glium;
extern crate glium_sdl2;
extern crate zoom;
extern crate nalgebra;
extern crate petgraph;

mod circle;
mod cell;

use gg::render2::*;

fn main() {
    use glium_sdl2::DisplayBuild;
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let gl_subsystem = video_subsystem.gl_attr();
    gl_subsystem.set_context_major_version(3);
    gl_subsystem.set_context_minor_version(1);

    let display = video_subsystem
        .window("My window", 800, 600)
        .resizable()
        .build_glium()
        .unwrap();
    let glowy = Renderer::new(&display);

    let mut running = true;
    let mut event_pump = sdl_context.event_pump().unwrap();

    while running {
        use glium::Surface;

        // Get dimensions each frame.
        let dims = display.get_framebuffer_dimensions();
        let hscale = dims.1 as f32 / dims.0 as f32;

        // Begin draw.
        let mut target = display.draw();
        // Clear screen.
        target.clear_color(0.0, 0.0, 0.0, 1.0);
        // Draw circles.
        glowy.render_qbeziers_flat(&mut target,
                                   [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
                                   [[hscale, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
                                   &circle::make_circle([1.0, 0.0, 0.0, 1.0]).collect::<Vec<_>>());
        // End draw.
        target.finish().unwrap();

        // Handle events.
        for event in event_pump.poll_iter() {
            use sdl2::event::Event;

            match event {
                Event::Quit { .. } => {
                    running = false;
                }
                _ => (),
            }
        }
    }
}

struct CellContainer {
    pub cell: cell::Cell,
    /// The current delta.
    pub delta: Option<cell::Delta>,
    /// The previous delta.
    pub prev_delta: Option<cell::Delta>,
}
