use std::f32::consts::PI;

use cgmath::{EuclideanSpace, InnerSpace, Point3, Vector3, Zero};
use rand::Rng;

pub type Position = Point3<f32>;
pub type Velocity = Vector3<f32>;
pub type Acceleration = Vector3<f32>;
pub type Force = Vector3<f32>;

#[derive(Clone)]
pub struct Atom {
    pub position: Position,
    pub velocity: Velocity,
    pub mass: f32,
    pub charge: f32,
}

impl Atom {
    /// Gravitational constant
    const G: f32 = 5.0;
    /// Permeability of the medium to EM force
    #[allow(non_upper_case_globals)]
    const μ: f32 = 55.0;

    pub fn new(position: Position) -> Self {
        Self {
            position,
            velocity: Velocity::zero(),
            mass: 1.0,
            charge: 1.0,
        }
    }

    pub fn step(&mut self, force: Force, delta: f32) {
        self.velocity += delta * force / self.mass;
        self.position += delta * self.velocity;
    }

    pub fn find_gravity(&self, other: &Atom) -> Force {
        // By default, attract.
        let dir = other.position - self.position;
        let factor = Self::G * self.mass * other.mass / dir.magnitude2();
        return dir * factor;
    }

    pub fn find_magnetism(&self, other: &Atom) -> Force {
        // By default, repel.
        // If charges have opposing signs, this will turn into attraction.
        let dir = self.position - other.position;
        let factor = Self::μ * self.charge * other.charge / 4.0 / PI / dir.magnitude2();
        return dir * factor;
    }
}

pub struct Cloud {
    atoms: Vec<Atom>,
}

impl Cloud {
    pub fn new(count: usize) -> Self {
        let mut rng = rand::rng();

        let mut atoms = Vec::with_capacity(count);

        for _ in 0..count {
            let pos = Position::new(
                rng.random_range(-1.0..=1.0),
                rng.random_range(-1.0..=1.0),
                rng.random_range(-1.0..=1.0),
            );
            atoms.push(Atom::new(pos));
        }

        Self { atoms }
    }

    pub fn step(&mut self, delta: f32) {
        const RECENTER: f32 = 1.0;

        // 1. Nudge the center of mass towards the origin
        let mut center_of_mass = Vector3::zero();
        for atom in &self.atoms {
            center_of_mass -= atom.position.to_vec() * atom.mass;
        }
        for atom in &mut self.atoms {
            atom.position += center_of_mass * delta * RECENTER;
        }

        let atoms_tmp = self.atoms.clone();

        // 2. Apply forces
        for (a, atom) in self.atoms.iter_mut().enumerate() {
            let mut force = Force::zero();
            for (o, other) in atoms_tmp.iter().enumerate() {
                if a == o {
                    continue;
                }
                force += atom.find_gravity(other);
                force += atom.find_magnetism(other);
            }
            atom.step(force, delta);
        }

        self.sort();
    }

    fn sort(&mut self) {
        let s = |a: &Atom, b: &Atom| a.position.z.total_cmp(&b.position.z);
        self.atoms.sort_by(s);
    }

    pub fn positions(&self) -> Vec<Position> {
        self.atoms.iter().map(|a| a.position).collect()
    }
}
