mod brain;

use zoom::*;
use nalgebra as na;

pub struct Cell {
    particle: particle::BasicParticle<na::Vector2<f64>, f64>,
}
