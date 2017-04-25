use CellContainer;
use rand::Rng;
use cell::{Cell, ConnectionState};
use CellGraph;
use zoom::{self, BasicParticle, Toroid};
use nalgebra::Vector2;
use num::Zero;
use petgraph::graph::{NodeIndex, EdgeIndex};
use petgraph::Direction;

// Spring attraction force.
const HOOKE_DYNAMIC: f64 = 0.1;
const HOOKE_STATIC: f64 = 0.1;
const HOOKE_RESTING: f64 = HOOKE_STATIC + 0.5 * HOOKE_DYNAMIC;
// Gravitational repeling force.
const NEWTON_DYNAMIC: f64 = 0.1;
const NEWTON_STATIC: f64 = 0.1;
const NEWTON_RESTING: f64 = NEWTON_STATIC + 0.5 * NEWTON_DYNAMIC;

const INERTIA: f64 = 1.0;

const CELL_SPAWN_PROBABILITY: f64 = 0.01;

pub fn area_box() -> zoom::Box<Vector2<f64>> {
    zoom::Box {
        origin: Vector2::new(0.0, 0.0),
        offset: Vector2::new(500.0, 500.0),
    }
}

pub fn compute_hooke_coefficient(edge: (f64, f64)) -> f64 {
    HOOKE_STATIC + HOOKE_DYNAMIC * (edge.0 * edge.1).sqrt()
}

pub fn random_point<R: Rng>(rng: &mut R) -> Vector2<f64> {
    let mut central_rand = || 2.0 * rng.next_f64() - 1.0;
    area_box().origin +
    Vector2::new(area_box().offset.x * central_rand(),
                 area_box().offset.y * central_rand())
}

pub fn generate_cells<R: Rng>(graph: &mut CellGraph, rng: &mut R) {
    if rng.next_f64() < CELL_SPAWN_PROBABILITY {
        let particle =
            BasicParticle::new(NEWTON_RESTING, random_point(rng), Vector2::zero(), INERTIA);
        graph.add_node(CellContainer {
                           cell: Cell::new_rand(rng, particle),
                           delta: None,
                           prev_delta: None,
                       });
    }
}

fn compute_connection_state(graph: &mut CellGraph,
                            source_position: Vector2<f64>,
                            direction: Direction,
                            target_edge: EdgeIndex<u32>,
                            target_node: NodeIndex<u32>)
                            -> ConnectionState {
    use nalgebra::Norm;
    let sent = match (direction, graph.edge_weight(target_edge).unwrap()) {
        (Direction::Outgoing, e) => e.1.signal.clone(),
        (Direction::Incoming, e) => e.0.signal.clone(),
    };
    let length = (graph.node_weight(target_node).unwrap().cell.position() - source_position).norm();
    ConnectionState {
        incoming: sent,
        length: length,
    }
}

pub fn compute_connection_states(graph: &mut CellGraph,
                                 nix: NodeIndex<u32>,
                                 direction: Direction)
                                 -> Vec<ConnectionState> {
    let pos = graph.node_weight(nix).unwrap().cell.position();
    let mut walker = graph.neighbors_directed(nix, direction).detach();
    let mut counter = 0..;
    let mut states = Vec::new();
    while let Some((eix, tnix)) = walker.next(&graph) {
        states.push(compute_connection_state(graph, pos, direction, eix, tnix));
    }
    states
}

pub fn update_deltas(graph: &mut CellGraph, nix: NodeIndex<u32>, direction: Direction) {
    let deltas = graph
        .node_weight(nix)
        .and_then(|container| container.delta.as_ref())
        .map(|delta| match direction {
                 Direction::Outgoing => delta.out_connections.clone(),
                 Direction::Incoming => delta.in_connections.clone(),
             });
    let mut walker = graph.neighbors_directed(nix, direction).detach();
    let mut counter = 0..;
    while let (Some(ix), Some(eix)) = (counter.next(), walker.next_edge(&graph)) {
        graph.edge_weight_mut(eix).unwrap().0 = deltas
            .as_ref()
            .map(|deltas| deltas[ix].clone())
            .unwrap_or_default();
    }
}
