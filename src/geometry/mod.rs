pub mod staff;

use std::f32;
use std::cmp;
use nalgebra as na;
use nalgebra::Norm;

#[derive(Debug)]
pub struct Line {
    pub a: na::Vec2<f32>,
    pub b: na::Vec2<f32>,
}

impl Line {

    #[inline(never)]
    pub fn new(a: na::Vec2<f32>, b: na::Vec2<f32>) -> Line {
        Line {
            a: a,
            b: b,
        }
    }

    pub fn normal(&self) -> na::Vec2<f32> {
        let dir = self.a - self.b;
        let norm = na::Vec2::new(dir[1], -dir[0]);
        na::normalize(&norm)
    }

    #[inline(never)]
    pub fn distance_to_point(&self, point: &na::Vec2<f32>) -> f32 {
        self.signed_distance_to_point(point).abs()
    }

    #[inline(never)]
    pub fn signed_distance_to_point(&self, point: &na::Vec2<f32>) -> f32 {
        let dir = self.a - self.b;
        let norm = na::Vec2::new(dir[1], -dir[0]);

        let norm_len = norm.norm();

        let ap = *point - self.a;
        let dot = na::dot(&ap, &norm);

        let dist = dot / norm_len;

        dist
    }

    pub fn point_at_time(&self, time: f32) -> na::Vec2<f32> {
        let d = self.b - self.a;

        self.a + d * time
    }

    #[inline(never)]
    pub fn screen_entry_exit_times(&self, width: f32, height: f32) -> (f32, f32) {
        // TODO: Improve this method.

        let d = self.b - self.a;

        let tx0 = (0.0 - self.a[1]) / d[1];
        let ty0 = (0.0 - self.a[0]) / d[0];
        let txh = (height - self.a[1]) / d[1];
        let tyw = (width - self.a[0]) / d[0];

        let mut t_min = f32::INFINITY;
        let mut t_max = f32::NEG_INFINITY;

        let x0 = self.a[0] + d[0] * tx0;
        if 0.0 <= x0 && x0 <= width {
            t_min = if t_min < x0 {t_min} else {tx0};
            t_max = if t_max > x0 {t_max} else {tx0}
        }

        let xh = self.a[0] + d[0] * txh;
        if 0.0 <= xh && xh <= width {
            t_min = if t_min < xh {t_min} else {txh};
            t_max = if t_max > xh {t_max} else {txh}
        }

        let y0 = self.a[1] + d[1] * ty0;
        if 0.0 <= y0 && y0 <= height {
            t_min = if t_min < y0 {t_min} else {ty0};
            t_max = if t_max > y0 {t_max} else {ty0}
        }

        let yw = self.a[1] + d[1] * tyw;
        if 0.0 <= yw && yw <= height {
            t_min = if t_min < yw {t_min} else {tyw};
            t_max = if t_max > yw {t_max} else {tyw}
        }

        (t_min, t_max)
    }
}
