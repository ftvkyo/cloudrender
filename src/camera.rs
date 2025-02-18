use cgmath::{Deg, Matrix4, Rad, Vector3};

pub struct AppCamera {
    pub distance: f32,
    pub fovy: Rad<f32>,
    pub aspect: f32,
}

impl AppCamera {
    pub fn new(aspect: f32) -> Self {
        Self {
            distance: 3.0,
            fovy: Deg(45.0).into(),
            aspect,
        }
    }

    pub fn view(&self) -> Matrix4<f32> {
        Matrix4::<f32>::from_translation(Vector3::new(0.0, 0.0, -self.distance))
    }

    pub fn projection(&self) -> Matrix4<f32> {
        cgmath::perspective(self.fovy, self.aspect, 0.01, 100.0)
    }
}
