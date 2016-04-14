use std::f32;
use std::cmp;
use nalgebra as na;
use nalgebra::Norm;
use geometry as gm;

#[derive(Debug)]
pub struct Staff {
    pub pos: na::Vec2<f32>,
    pub dir: na::Vec2<f32>,
    pub length: f32,
    pub line_width: f32,
    pub space_width: f32,
}

impl Staff {

    #[inline(never)]
    pub fn new(a: na::Vec2<f32>, b: na::Vec2<f32>, line_width: f32, space_width: f32) -> Staff {
        let dir = b - a;
        let norm_dir = na::normalize(&dir);
        let length = dir.norm();

        Staff {
            pos: a,
            dir: norm_dir,
            length: length,
            line_width: line_width,
            space_width: space_width,
        }
    }

    pub fn line_sep(&self) -> f32 {
        self.line_width + self.space_width
    }

    pub fn normal(&self) -> na::Vec2<f32> {
        na::Vec2::new(self.dir[1], -self.dir[0])
    }

    #[inline(never)]
    pub fn distance_to_point(&self, point: &na::Vec2<f32>) -> f32 {
        self.signed_distance_to_point(point).abs()
    }

    #[inline(never)]
    pub fn signed_distance_to_point(&self, point: &na::Vec2<f32>) -> f32 {
        let norm = self.normal();

        let norm_len = 1.0;

        let ap = *point - self.pos;
        let dot = na::dot(&ap, &norm);

        let dist = dot / norm_len;

        dist
    }

    pub fn point_at_time(&self, time: f32) -> na::Vec2<f32> {
        self.pos + self.dir * time
    }

    pub fn perpendicular_samples(&self, time: f32, num_samples: usize, sample_sep: f32) -> Vec<na::Vec2<f32>>{
        let mut samples = Vec::new();

        let p_t = self.point_at_time(time);

        let normal = self.normal();
        let min_i = -(num_samples as isize/2);
        let max_i = if num_samples % 2 == 0 {num_samples/2} else {num_samples/2 + 1} as isize;
        let i_mod = if num_samples % 2 == 0 {0.5} else {0.0};
        let mut blank_avg = 0.0;
        for i in min_i..max_i {
            let d = sample_sep * (i as f32 + i_mod);
            let pt = p_t + normal * d;

            samples.push(pt);

            // let brightness = ycbcr_frame.sample_point(pt).y as f32 / 255.0;
            // blank_avg += brightness.round();
            // let draw_pt = ycbcr_frame.opengl_coords_for_point(pt);
            // draw_ctx.draw_point(&mut target, draw_pt, 1.0, [0.2, 0.2, 0.2, 1.0]);
        }

        samples
    }

    #[inline(never)]
    pub fn screen_entry_exit_times(&self, width: f32, height: f32) -> (f32, f32) {
        let line = gm::Line::new(self.pos, self.pos + self.dir);

        line.screen_entry_exit_times(width, height)
    }
}
