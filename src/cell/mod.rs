mod brain;

use zoom::*;
use nalgebra as na;
use rand::Rng;
use gapush::simple::{SimpleInstruction, PlainOp};
use aux::area_box;
use zoom::particle;

const INIT_ENERGY: usize = 1 << 20;
const SIZE_TO_ENERGY_RATIO: f64 = 0.05;
const ENERGY_TO_EXECUTION_RATIO: f64 = 20.0;
const CELL_SIGMOID_COEFFICIENT: f64 = 0.01;

const DRAG_COEFFICIENT: f64 = 0.001;
const PHYSICS_DELTA: f64 = 50.0;
const GRAVITATE_RADIUS: f64 = 0.0001;

const RANDOM_SHIFT_OFFSET: f64 = 0.2;

#[derive(Clone)]
pub struct Cell {
    energy: usize,
    particle: particle::BasicParticle<na::Vector2<f64>, f64>,
    brain: brain::Brain,
}

impl Cell {
    pub fn new_rand<R: Rng>(rng: &mut R,
                            particle: particle::BasicParticle<na::Vector2<f64>, f64>)
                            -> Cell {
        Cell {
            energy: INIT_ENERGY,
            particle: particle,
            brain: brain::Brain::new_rand(energy_to_size(INIT_ENERGY), rng),
        }
    }

    pub fn set_energy(&mut self, energy: usize) {
        self.energy = energy;
        self.brain.set_size(energy_to_size(energy));
    }

    pub fn energy(&self) -> usize {
        self.energy
    }

    pub fn create_state(&self,
                        out_connections: Vec<ConnectionState>,
                        in_connections: Vec<ConnectionState>)
                        -> StateParameters {
        StateParameters {
            position: self.particle.position,
            energy: self.energy,
            out_connections: out_connections,
            in_connections: in_connections,
        }
    }

    pub fn run_connection(&mut self,
                          connection_states: Vec<ConnectionState>)
                          -> (Vec<ConnectionDelta>, usize) {
        connection_states
            .into_iter()
            .map(|cs| self.brain.run_connection(cs.length, cs.incoming))
            .map(|(elasticity, signal, sever, cycles)| {
                     (cell_sigmoid(elasticity.unwrap_or(0)),
                      signal.unwrap_or(SimpleInstruction::PlainOp(PlainOp::Nop)),
                      sever.unwrap_or(false),
                      cycles)
                 })
            .map(|(elasticity, signal, sever, cycles)| {
                     (ConnectionDelta {
                          elasticity: elasticity,
                          signal: signal,
                          sever: sever,
                      },
                      cycles)
                 })
            .fold((Vec::new(), 0), |(mut v, tcycles), (delta, cycles)| {
                v.push(delta);
                (v, tcycles + cycles)
            })
    }

    pub fn cycle(&mut self, state: StateParameters) -> Delta {
        let cycle_cycles = self.brain.run_cycle(state.energy as f64);
        let (out_connection_deltas, out_connection_cycles) =
            self.run_connection(state.out_connections);
        let (in_connection_deltas, in_connection_cycles) =
            self.run_connection(state.in_connections);
        let (repulsion, repulsion_cycles) = self.brain.run_repulsion();
        let repulsion = cell_sigmoid(repulsion.unwrap_or(0));
        let (die, die_cycles) = self.brain.run_die();
        let die = die.unwrap_or(false);
        let (divide, divide_cycles) = self.brain.run_divide();
        let divide = divide.unwrap_or(false);
        self.energy = self.energy
            .checked_sub((ENERGY_TO_EXECUTION_RATIO *
                          (cycle_cycles + out_connection_cycles + in_connection_cycles +
                           repulsion_cycles + die_cycles +
                           divide_cycles) as f64) as usize)
            .unwrap_or(0);
        Delta {
            out_connections: out_connection_deltas,
            in_connections: in_connection_deltas,
            repulsion: repulsion,
            // Also consider that an energy of 0 indicates death.
            die: die || self.energy == 0,
            divide: divide,
        }
    }

    pub fn mutate<R: Rng>(&mut self, rng: &mut R) {
        self.brain.mutate(rng);
    }

    pub fn random_shift<R: Rng>(&mut self, rng: &mut R) {
        let mut central_rand = || RANDOM_SHIFT_OFFSET * (2.0 * rng.next_f64() - 1.0);
        self.particle.position += na::Vector2::new(central_rand(), central_rand());
    }

    pub fn update_physics(&mut self) {
        self.particle.drag(DRAG_COEFFICIENT);
        self.particle.advance(PHYSICS_DELTA);
        self.particle.position = area_box().wrap_position(self.particle.position);
    }

    pub fn position(&self) -> na::Vector2<f64> {
        self.particle.position.clone()
    }

    pub fn interact_connection(&self, other: &Self, hooke: f64) {
        particle::hooke_delta(&self.particle, &other.particle, hooke, |(from, to)| area_box().wrap_delta(to - from));
    }

    pub fn interact_repel(&self, other: &Self, newton: f64) {
        particle::gravitate_radius_squared_delta(&self.particle, &other.particle,
                GRAVITATE_RADIUS * GRAVITATE_RADIUS,
                -newton,
                |(from, to)| area_box().wrap_delta(to - from));
    }
}

#[derive(Clone, Debug)]
pub struct ConnectionState {
    pub incoming: SimpleInstruction,
    pub length: f64,
}

#[derive(Clone, Debug)]
pub struct StateParameters {
    pub position: na::Vector2<f64>,
    pub energy: usize,
    pub out_connections: Vec<ConnectionState>,
    pub in_connections: Vec<ConnectionState>,
}

#[derive(Clone, Debug)]
pub struct ConnectionDelta {
    pub elasticity: f64,
    pub signal: SimpleInstruction,
    pub sever: bool,
}

impl Default for ConnectionDelta {
    fn default() -> ConnectionDelta {
        ConnectionDelta {
            elasticity: 0.5,
            signal: SimpleInstruction::PlainOp(PlainOp::Nop),
            sever: false,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Delta {
    pub out_connections: Vec<ConnectionDelta>,
    pub in_connections: Vec<ConnectionDelta>,
    pub repulsion: f64,
    pub die: bool,
    pub divide: bool,
}

fn energy_to_size(energy: usize) -> usize {
    (energy as f64 * SIZE_TO_ENERGY_RATIO) as usize
}

fn cell_sigmoid(n: i64) -> f64 {
    let t = n as f64 * CELL_SIGMOID_COEFFICIENT;
    1.0 / (1.0 + (-t).exp())
}
