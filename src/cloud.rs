use cgmath::EuclideanSpace;
use rand::Rng;

use crate::Position;

pub struct Cloud<const C: usize> {
    pub points: [Position; C],
}

impl<const C: usize> Cloud<C> {
    pub fn new() -> Self {
        let mut rng = rand::rng();

        let mut points = [Position::origin(); C];

        for point in &mut points {
            point.x = rng.random_range(-1.0..=1.0);
            point.y = rng.random_range(-1.0..=1.0);
            point.z = rng.random_range(-1.0..=1.0);
        }

        points.sort_by(|a, b| a.z.partial_cmp(&b.z).unwrap());

        Self {
            points
        }
    }

    pub fn step(&mut self, delta: f32) {
        todo!()
    }
}
