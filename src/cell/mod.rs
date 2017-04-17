mod brain;

use zoom::*;
use nalgebra as na;
use rand::Rng;

const INIT_ENERGY: usize = 1 << 20;
const SIZE_TO_ENERGY_RATIO: f64 = 0.05;

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
}

fn energy_to_size(energy: usize) -> usize {
    (energy as f64 * SIZE_TO_ENERGY_RATIO) as usize
}
