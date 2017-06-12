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
extern crate boolinator;

mod circle;
mod cell;
mod aux;

use aux::*;
use gg::render2::*;
use boolinator::Boolinator;
use std::iter::once;
use petgraph::visit::{EdgeRef, IntoEdgeReferences};

/// Create the graph which is used to store the cells and all their connections.
/// The cell it goes out from is the first weight and vice versa.
type CellGraph = petgraph::stable_graph::StableGraph<CellContainer, (cell::ConnectionDelta, cell::ConnectionDelta)>;

const SEED: [u64; 4] = [0, 1, 2, 3];
const CIRCLE_SCALE: f32 = 0.01;

fn main() {
    use glium_sdl2::DisplayBuild;
    use rand::SeedableRng;
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let gl_subsystem = video_subsystem.gl_attr();
    gl_subsystem.set_context_major_version(3);
    gl_subsystem.set_context_minor_version(1);

    let display = video_subsystem
        .window("My window", area_box().offset.x as u32 * 2, area_box().offset.y as u32 * 2)
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
        for nix in graph.node_indices().collect::<Vec<_>>() {
            use petgraph::Direction::*;

            let out_states = compute_connection_states(&mut graph, nix, Outgoing);
            let in_states = compute_connection_states(&mut graph, nix, Incoming);

            let cc = graph.node_weight_mut(nix).unwrap();
            let state = cc.cell.create_state(out_states, in_states);
            cc.delta = Some(cc.cell.cycle(state));
        }

        // Update all edge deltas.
        for nix in graph.node_indices().collect::<Vec<_>>() {
            use petgraph::Direction::*;

            // Handle the connection deltas.
            update_deltas(&mut graph, nix, Outgoing);
            update_deltas(&mut graph, nix, Incoming);
        }

        // Handle cell physics interations.
        cell_physics_interactions(&mut graph);

        // Advance physics
        for nix in graph.node_indices().collect::<Vec<_>>() {
            graph[nix].cell.update_physics();
        }

        // Handle division
        for nix in graph.node_indices().collect::<Vec<_>>() {
            if graph[nix].delta.as_ref().map(|d| d.divide).unwrap_or(false) {
                divide_cell(&mut graph, nix, &mut rng);
            }
        }

        // Handle death
        for nix in graph.node_indices().collect::<Vec<_>>() {
            if graph[nix].delta.as_ref().map(|d| d.die).unwrap_or(false) {
                graph.remove_node(nix);
            }
        }

        for eix in graph.edge_references().filter_map(|er| {
            use petgraph::visit::EdgeRef;
            (er.weight().0.sever || er.weight().0.sever).as_some(er.id())
        }).collect::<Vec<_>>() {
            graph.remove_edge(eix);
        }

        // Give everybody food based on connections just cuz.
        for nix in graph.node_indices().collect::<Vec<_>>() {
            let count = graph.edges(nix).count();
            let new_energy = graph[nix].cell.energy() + count * (1 << 15);
            graph[nix].cell.set_energy(new_energy);
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
                                   [[1.0, 0.0, 0.0],
                                   [0.0, 1.0, 0.0],
                                   [0.0, 0.0, 1.0]],
                                   [[hscale / area_box().offset.x as f32, 0.0, 0.0],
                                    [0.0, 1.0 / area_box().offset.y as f32, 0.0],
                                        [0.0, 0.0, 1.0]],
                                   &graph.node_indices()
                                         .map(|nix| graph[nix].cell.position())
                                         .flat_map(|p| circle::make_circle([0.0, 1.0, 1.0, 1.0]).map(move |mut qb| {
                                             qb.falloff_radius0 *= CIRCLE_SCALE * area_box().offset.y as f32;
                                             qb.falloff_radius1 *= CIRCLE_SCALE * area_box().offset.y as f32;

                                             qb.position0[0] *= CIRCLE_SCALE * area_box().offset.x as f32;
                                             qb.position0[0] += p.x as f32;
                                             qb.position0[1] *= CIRCLE_SCALE * area_box().offset.y as f32;
                                             qb.position0[1] += p.y as f32;
                                             qb.position1[0] *= CIRCLE_SCALE * area_box().offset.x as f32;
                                             qb.position1[0] += p.x as f32;
                                             qb.position1[1] *= CIRCLE_SCALE * area_box().offset.y as f32;
                                             qb.position1[1] += p.y as f32;
                                             qb.position2[0] *= CIRCLE_SCALE * area_box().offset.x as f32;
                                             qb.position2[0] += p.x as f32;
                                             qb.position2[1] *= CIRCLE_SCALE * area_box().offset.y as f32;
                                             qb.position2[1] += p.y as f32;
                                             qb
                                         }))
                                         .collect::<Vec<_>>());
        // Draw edges.
        glowy.render_edges_round(&mut target,
                                   [[1.0, 0.0, 0.0],
                                   [0.0, 1.0, 0.0],
                                   [0.0, 0.0, 1.0]],
                                   [[hscale / area_box().offset.x as f32, 0.0, 0.0],
                                    [0.0, 1.0 / area_box().offset.y as f32, 0.0],
                                        [0.0, 0.0, 1.0]],
                                   &graph.edge_references()
                                         .map(|er| (graph[er.source()].cell.position(), graph[er.target()].cell.position()))
                                         .flat_map(|(p0, p1)| once(Node{position: [p0.x as f32, p0.y as f32],
                                            inner_color: [0.0, 0.0, 0.0, 1.0],
                                            falloff: 0.25,
                                            falloff_color: [0.0, 1.0, 0.0, 1.0],
                                            falloff_radius: CIRCLE_SCALE * area_box().offset.y as f32,
                                            inner_radius: 0.0}).chain(once(
                                                Node{position: [p1.x as f32, p1.y as f32],
                                            inner_color: [0.0, 0.0, 0.0, 1.0],
                                            falloff: 0.25,
                                            falloff_color: [0.0, 1.0, 0.0, 1.0],
                                            falloff_radius: CIRCLE_SCALE * area_box().offset.y as f32,
                                            inner_radius: 0.0}
                                            )))
                                         .collect::<Vec<_>>());

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
