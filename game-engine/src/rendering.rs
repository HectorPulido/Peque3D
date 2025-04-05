use super::camera3d::Camera3d;
use super::object3d::Object3d;
use na::Matrix4;
use piston_window::{clear, line, Context, G2d, PistonWindow, WindowSettings};

pub struct Windows {
    pub window: PistonWindow,
    camera: Camera3d,
    camera_size_x: u32,
    camera_size_y: u32,
}

impl Windows {
    pub fn new(
        camera_size_x: u32,
        camera_size_y: u32,
        window_title: &str,
        camera: Camera3d,
    ) -> Self {
        let window: PistonWindow =
            WindowSettings::new(window_title, [camera_size_x, camera_size_y])
                .exit_on_esc(true)
                .build()
                .unwrap();

        Self {
            window,
            camera,
            camera_size_x,
            camera_size_y,
        }
    }

    pub fn draw(&mut self, event: &piston_window::Event, objects: &mut [Object3d]) {
        let proj_matrix = self.camera.projection_matrix();
        let screen_center_x = (self.camera_size_x as f32) / 2.0;
        let screen_center_y = (self.camera_size_y as f32) / 2.0;
        self.window.draw_2d(event, |context, graphics, _| {
            clear([0.0, 0.0, 0.0, 1.0], graphics);
            Self::draw_scene(
                context,
                graphics,
                objects,
                &self.camera,
                &proj_matrix,
                (screen_center_x, screen_center_y),
            );
        });
    }

    fn draw_scene(
        context: Context,
        graphics: &mut G2d,
        objects: &mut [Object3d],
        camera: &Camera3d,
        proj_matrix: &Matrix4<f32>,
        screen_center: (f32, f32),
    ) {
        let (cx, cy) = screen_center;

        let mut all_lines: Vec<[f64; 4]> = Vec::new();
        for object in objects.iter_mut() {
            if !object.render {
                continue;
            }

            let transformed = object.transform_points();
            let proj = proj_matrix;
            let pts2d: Vec<[f64; 2]> = transformed
                .iter()
                .map(|p| {
                    if let Some((x, y)) = camera.project_point_with(p, proj) {
                        [
                            cx as f64 + (x as f64) * 100.0,
                            cy as f64 - (y as f64) * 100.0,
                        ]
                    } else {
                        [0.0, 0.0]
                    }
                })
                .collect();

            for edge in &object.edges {
                for window_edge in edge.windows(2) {
                    let a = pts2d[window_edge[0]];
                    let b = pts2d[window_edge[1]];
                    all_lines.push([a[0], a[1], b[0], b[1]]);
                }
            }
        }

        // Dibujamos todas las líneas de una vez:
        for line_coords in all_lines.iter() {
            line(
                [1.0, 1.0, 1.0, 1.0],
                1.0,
                *line_coords,
                context.transform,
                graphics,
            );
        }
    }
}
