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
extern crate num;
extern crate itertools;

mod circle;
mod cell;
mod aux;

use aux::*;
use gg::render2::*;

/// Create the graph which is used to store the cells and all their connections.
/// The cell it goes out from is the first weight and vice versa.
type CellGraph = petgraph::Graph<CellContainer, (cell::ConnectionDelta, cell::ConnectionDelta)>;


const SEED: [u64; 4] = [0, 1, 2, 3];

fn main() {
    use glium_sdl2::DisplayBuild;
    use rand::SeedableRng;
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
    let mut event_pump = sdl_context.event_pump().unwrap();

    let mut graph = CellGraph::new();
    let mut rng = rand::Isaac64Rng::from_seed(&SEED);

    loop {
        use glium::Surface;

        // Generate cells randomly.
        generate_cells(&mut graph, &mut rng);

        // Compute cell deltas.
        for nix in graph.node_indices() {
            use petgraph::Direction::*;

            let out_states = compute_connection_states(&mut graph, nix, Outgoing);
            let in_states = compute_connection_states(&mut graph, nix, Incoming);

            let cc = graph.node_weight_mut(nix).unwrap();
            let state = cc.cell.create_state(out_states, in_states);
            cc.delta = Some(cc.cell.cycle(state));
        }

        // Update all edge elasticities.
        for nix in graph.node_indices() {
            use petgraph::Direction::*;

            // Handle the connection deltas.
            update_deltas(&mut graph, nix, Outgoing);
            update_deltas(&mut graph, nix, Incoming);
        }

        // Handle cell physics interations.
        cell_physics_interactions(&mut graph);

        // Advance physics
        for nix in graph.node_indices() {
            graph[nix].cell.update_physics();
        }

        // Handle death
        for nix in graph.node_indices() {
            if graph[nix].delta.as_ref().map(|d| d.die).unwrap_or(false) {
                graph.remove_node(nix);
            }
        }

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
                    return;
                }
                _ => (),
            }
        }
    }
}

pub struct CellContainer {
    pub cell: cell::Cell,
    /// The current delta.
    pub delta: Option<cell::Delta>,
}
