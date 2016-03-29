use nalgebra as na;
use nalgebra::Norm;

pub struct Line {
    a: na::Vec2<f32>,
    b: na::Vec2<f32>,
}

impl Line {

    pub fn new(a: na::Vec2<f32>, b: na::Vec2<f32>) -> Line {
        Line {
            a: a,
            b: b,
        }
    }

    pub fn distance_to_point(&self, point: &na::Vec2<f32>) -> f32 {
        let dir = self.a - self.b;
        let norm = na::Vec2::new(dir[1], -dir[0]);

        let dot = na::dot(&dir, &norm);
        let norm_len = norm.norm();

        let dist = (dot).abs() / norm_len;

        dist
    }
}
