use nalgebra as na;
use nalgebra::Norm;

#[derive(Debug)]
pub struct Line {
    pub a: na::Vec2<f32>,
    pub b: na::Vec2<f32>,
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

        let norm_len = norm.norm();

        let ap = *point - self.a;
        let dot = na::dot(&ap, &norm);

        let dist = (dot).abs() / norm_len;

        dist
    }
}
