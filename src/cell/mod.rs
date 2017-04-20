mod brain;

use zoom::*;
use nalgebra as na;
use rand::Rng;
use gapush::simple::{SimpleInstruction, PlainOp};

const INIT_ENERGY: usize = 1 << 20;
const SIZE_TO_ENERGY_RATIO: f64 = 0.05;
const ENERGY_TO_EXECUTION_RATIO: f64 = 20.0;
const CELL_SIGMOID_COEFFICIENT: f64 = 0.01;

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

    pub fn create_state(&self, connections: Vec<ConnectionState>) -> StateParameters {
        StateParameters {
            position: self.particle.position,
            energy: self.energy,
            connections: connections,
        }
    }

    pub fn cycle(&mut self, state: StateParameters) -> Delta {
        let cycle_cycles = self.brain.run_cycle(state.energy as f64);
        let (connection_deltas, connection_cycles) = state.connections.into_iter()
            .map(|cs| self.brain.run_connection(cs.length, cs.incoming))
            .map(|(elasticity, signal, sever, cycles)|
                (cell_sigmoid(elasticity.unwrap_or(0)),
                signal.unwrap_or(SimpleInstruction::PlainOp(PlainOp::Nop)),
                sever.unwrap_or(false),
                cycles))
            .map(|(elasticity, signal, sever, cycles)| (ConnectionDelta{
                elasticity: elasticity,
                signal: signal,
                sever: sever,
            },
            cycles))
            .fold((Vec::new(), 0), |(mut v, tcycles), (delta, cycles)| {v.push(delta); (v, tcycles + cycles)});
        let (repulsion, repulsion_cycles) = self.brain.run_repulsion();
        let repulsion = cell_sigmoid(repulsion.unwrap_or(0));
        let (die, die_cycles) = self.brain.run_die();
        let die = die.unwrap_or(false);
        let (divide, divide_cycles) = self.brain.run_divide();
        let divide = divide.unwrap_or(false);
        self.energy = self.energy.checked_sub((ENERGY_TO_EXECUTION_RATIO *
            (cycle_cycles + connection_cycles + repulsion_cycles + die_cycles + divide_cycles) as f64) as usize)
            .unwrap_or(0);
        Delta{
            connections: connection_deltas,
            repulsion: repulsion,
            // Also consider that an energy of 0 indicates death.
            die: die || self.energy == 0,
            divide: divide,
        }
    }
}

pub struct ConnectionState {
    pub incoming: SimpleInstruction,
    pub length: f64,
}

pub struct StateParameters {
    pub position: na::Vector2<f64>,
    pub energy: usize,
    pub connections: Vec<ConnectionState>,
}

pub struct ConnectionDelta {
    pub elasticity: f64,
    pub signal: SimpleInstruction,
    pub sever: bool,
}

pub struct Delta {
    pub connections: Vec<ConnectionDelta>,
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
