use cgmath::{EuclideanSpace, InnerSpace, Point3, Vector3, Zero};
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
        const RECENTER: f32 = 1.0;
        const INTERACTION: f32 = 0.1;

        let origin = Point3::origin();

        let mut center_of_mass = Vector3::zero();
        for point in self.points {
            center_of_mass += origin - point;
        }

        let mut points_after = self.points.clone();

        for (p, point) in points_after.iter_mut().enumerate() {
            let mut shift = center_of_mass * delta * RECENTER;

            for (o, other) in self.points.iter().enumerate() {
                if p == o {
                    continue;
                }

                let interaction_vector = other - *point;
                let interaction_force = interaction_vector.magnitude().log2();
                shift += interaction_vector.normalize() * interaction_force * delta * INTERACTION;
            }

            *point += shift;
        }

        self.points = points_after;
    }
}
