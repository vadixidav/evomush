use CellContainer;
use rand::Rng;
use cell::Cell;
use CellGraph;
use zoom::{self, BasicParticle, Toroid};
use nalgebra::Vector2;
use num::Zero;

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
    zoom::Box{origin: Vector2::new(0.0, 0.0), offset: Vector2::new(500.0, 500.0)}
}

pub fn compute_hooke_coefficient(edge: (f64, f64)) -> f64 {
    HOOKE_STATIC + HOOKE_DYNAMIC * (edge.0 * edge.1).sqrt()
}

pub fn random_point<R: Rng>(rng: &mut R) -> Vector2<f64> {
    let mut central_rand = || 2.0 * rng.next_f64() - 1.0;
    area_box().origin + Vector2::new(area_box().offset.x * central_rand(), area_box().offset.y * central_rand())
}

pub fn generate_cells<R: Rng>(graph: &mut CellGraph, rng: &mut R) {
    if rng.next_f64() < CELL_SPAWN_PROBABILITY {
        let particle = BasicParticle::new(NEWTON_RESTING, random_point(rng), Vector2::zero(), INERTIA);
        graph.add_node(CellContainer{
            cell: Cell::new_rand(rng, particle),
            delta: None,
            prev_delta: None,
        });
    }
}
