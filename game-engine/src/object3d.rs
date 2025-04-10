use na::{Matrix3, Vector3};
use rand::Rng;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct Object3d {
    pub object_name: String,
    pub random_id: u32,
    model: Vec<Vector3<f32>>,
    pub edges: Vec<Vec<usize>>,
    pub position: Vector3<f32>,
    pub rotation: f32,
    pub render: bool,
    transformed_cache: Vec<Vector3<f32>>,
}

impl Object3d {
    /// Create a new Object3d from a model path, position and rotation.
    pub fn new(model_path: &str, position: Vector3<f32>, rotation: f32) -> Self {
        let num = rand::thread_rng().gen_range(0..100000);
        let (model, edges) = Self::load_obj(model_path);
        let mut transformed_cache = Vec::with_capacity(model.len());
        transformed_cache.resize(model.len(), Vector3::zeros());
        Object3d {
            object_name: model_path.to_string(),
            random_id: num,
            model,
            edges,
            position,
            rotation,
            transformed_cache,
            render: true,
        }
    }

    /// Apply a translation to the object.
    pub fn transform_points(&mut self) -> &[Vector3<f32>] {
        let cos_theta = self.rotation.cos();
        let sin_theta = self.rotation.sin();
        let rotation_matrix = Matrix3::new(
            cos_theta, 0.0, sin_theta, 0.0, 1.0, 0.0, -sin_theta, 0.0, cos_theta,
        );

        for (i, p) in self.model.iter().enumerate() {
            self.transformed_cache[i] = rotation_matrix * p + self.position;
        }
        &self.transformed_cache
    }

    pub fn load_obj(file_path: &str) -> (Vec<Vector3<f32>>, Vec<Vec<usize>>) {
        let file = File::open(file_path).expect("Could not open the file");
        let reader = BufReader::new(file);
        let mut vertices = Vec::new();
        let mut edges = Vec::new();

        for line in reader.lines() {
            let line = line.expect("Error reading line");
            if line.starts_with("v ") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 4 {
                    let x: f32 = parts[1].parse().unwrap_or(0.0);
                    let y: f32 = parts[2].parse().unwrap_or(0.0);
                    let z: f32 = parts[3].parse().unwrap_or(0.0);
                    vertices.push(Vector3::new(x, y, z));
                }
            } else if line.starts_with("f ") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                let mut face = Vec::new();
                for part in parts.iter().skip(1) {
                    let index_str = part.split('/').next().unwrap();
                    let index: usize = index_str.parse().unwrap_or(0);
                    face.push(index - 1);
                }
                edges.push(face);
            }
        }
        (vertices, edges)
    }
}
