extern crate nalgebra as na;
use na::{Matrix4, Vector3};

/// Cámara 3D con proyección en perspectiva completa.
pub struct Camera3d {
    fov: f32,    // campo de visión en radianes
    aspect: f32, // relación de aspecto (ancho / alto)
    near: f32,
    far: f32,
}

impl Camera3d {
    /// Crea una nueva cámara. Se ingresa el campo de visión en grados.
    pub fn new(fov_deg: f32, aspect: f32, near: f32, far: f32) -> Self {
        let fov = fov_deg.to_radians();
        Camera3d {
            fov,
            aspect,
            near,
            far,
        }
    }

    /// Proyecta un punto 3D a 2D usando una matriz de proyección perspectiva.
    pub fn projection_matrix(&self) -> Matrix4<f32> {
        let f = 1.0 / (self.fov / 2.0).tan();
        Matrix4::new(
            f / self.aspect,
            0.0,
            0.0,
            0.0,
            0.0,
            f,
            0.0,
            0.0,
            0.0,
            0.0,
            (self.far + self.near) / (self.near - self.far),
            (2.0 * self.far * self.near) / (self.near - self.far),
            0.0,
            0.0,
            -1.0,
            0.0,
        )
    }

    /// Proyecta un punto 3D a 2D utilizando la matriz de proyección proporcionada.
    pub fn project_point_with(
        &self,
        point: &Vector3<f32>,
        proj: &Matrix4<f32>,
    ) -> Option<(f32, f32)> {
        let mut point4 = na::Vector4::new(point.x, point.y, point.z, 1.0);
        point4 = proj * point4;
        if point4.w.abs() < 0.001 {
            return None;
        }
        point4 /= point4.w;
        Some((point4.x, point4.y))
    }
}
