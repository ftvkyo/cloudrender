use cgmath::{Deg, Matrix4, Rad, Vector3};

use crate::Position;

pub struct AppCamera {
    pub position: Position,
    pub center: Position,
    pub fovy: Rad<f32>,
    pub aspect: f32,
}

impl AppCamera {
    pub fn new(aspect: f32) -> Self {
        Self {
            position: Position::new(1.0, 1.0, 10.0),
            center: Position::new(0.0, 0.0, 0.0),
            fovy: Deg(45.0).into(),
            aspect,
        }
    }

    pub fn view(&self) -> Matrix4<f32> {
        Matrix4::<f32>::from_translation(Vector3::new(0.0, 0.0, -4.0))
    }

    pub fn projection(&self) -> Matrix4<f32> {
        cgmath::perspective(self.fovy, self.aspect, 0.01, 100.0)
    }
}
