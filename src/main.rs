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
mod auxillary;

use auxillary::*;
use gg::render2::*;
use boolinator::Boolinator;
use std::iter::once;
use petgraph::visit::{EdgeRef, IntoEdgeReferences};
use nalgebra::Norm;

/// Create the graph which is used to store the cells and all their connections.
/// The cell it goes out from is the first weight and vice versa.
type CellGraph = petgraph::stable_graph::StableGraph<CellContainer, (cell::ConnectionDelta, cell::ConnectionDelta)>;

const SIZE_SCALE: f64 = 0.6;

const SEED: [u64; 4] = [0, 1, 2, 3];
const CIRCLE_SCALE: f32 = 0.015 / SIZE_SCALE as f32;
const DYNAMIC_ENERGY_GAIN_COEFFICIENT: f64 = 1.0;
const RENDER_LENGTH_LIMIT: f64 = 1000.0;

const CENTER_BAND_RATIO: f64 = 0.5;
const CENTER_BAND_ACCELERATION: f64 = 100.0;

fn main() {
    use glium_sdl2::DisplayBuild;
    use rand::SeedableRng;
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let gl_subsystem = video_subsystem.gl_attr();
    gl_subsystem.set_context_major_version(3);
    gl_subsystem.set_context_minor_version(1);

    let display = video_subsystem
        .window("Evomush", 640, 640)
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
            let y = graph[nix].cell.position().y;
            if y.abs() < area_box().offset.y * CENTER_BAND_RATIO {
                graph[nix].cell.impulse(nalgebra::Vector2::new(
                    y / (area_box().offset.y * CENTER_BAND_RATIO) * CENTER_BAND_ACCELERATION, 0.0));
            }
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
            (er.weight().0.sever || er.weight().0.sever).as_some(er.id())
        }).collect::<Vec<_>>() {
            graph.remove_edge(eix);
        }

        // Give everybody food based on closest distance squared.
        for nix in graph.node_indices().collect::<Vec<_>>() {
            let add_energy = graph[nix]
                    .cell
                    .closest_distance_squared()
                    .map(|d| (d * DYNAMIC_ENERGY_GAIN_COEFFICIENT) as usize)
                    .unwrap_or(0);
            let new_energy = graph[nix].cell.energy() + add_energy;
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
                                         .filter(|&(p0, p1)| (p0 - p1).norm_squared() < RENDER_LENGTH_LIMIT.powi(2))
                                         .flat_map(|(p0, p1)| once(Node{position: [p0.x as f32, p0.y as f32],
                                            inner_color: [0.0, 0.0, 0.0, 1.0],
                                            falloff: 0.2,
                                            falloff_color: [0.0, 0.35, 0.0, 1.0],
                                            falloff_radius: CIRCLE_SCALE * area_box().offset.y as f32,
                                            inner_radius: 0.0}).chain(once(
                                                Node{position: [p1.x as f32, p1.y as f32],
                                            inner_color: [0.0, 0.0, 0.0, 1.0],
                                            falloff: 0.2,
                                            falloff_color: [0.0, 0.35, 0.0, 1.0],
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
