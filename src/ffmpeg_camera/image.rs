
use ffmpeg_camera::ffmpeg_utils::FfmpegError;
use std::io;
use nalgebra as na;

use std::io::prelude::*;
use std::fs::OpenOptions;
use std::cmp;


#[derive(Copy, Clone)]
pub struct Pixel {
    pub y : u8,
    pub cb : u8,
    pub cr : u8,
}

pub trait Image : Clone {
    fn from_raw_parts(width: usize, height: usize, data: Vec<u8>) -> Self;

    fn width(&self) -> usize; // width in pixels
    fn height(&self) -> usize; // height in pixels
    // fn data(&self) -> &Vec<u8>; // raw data buffer

    fn index(&self, col : usize, row : usize) -> Pixel;
    // fn sample_point(&self, pt: na::Vector2<f32>) -> Pixel {
    //     let col = pt[0].floor() as usize;
    //     let row = pt[1].floor() as usize;
    //     self.index(col, row)
    // }
    fn sample_point(&self, pt: na::Vector2<f32>) -> Pixel {
        let raw_col = pt[0].floor() as usize;
        let raw_row = pt[1].floor() as usize;

        let col = cmp::max(0, cmp::min(self.width() - 1, raw_col));
        let row = cmp::max(0, cmp::min(self.height() - 1, raw_row));

        self.index(col, row)
    }

    fn save_jpeg(&self, save_fname : &str) -> Result<(), FfmpegError>;

    fn contains(&self, col : usize, row : usize) -> bool {
         col < self.width() &&
         row < self.height()
    }

    fn opengl_coords_for_point(&self, point: na::Vector2<f32>) -> na::Vector2<f32> {
        let px = point[0];
        let py = point[1];

        // // // Half-pixel offsets:
        // // let ox = 0.5 / width;
        // // let oy = 0.5 / height;
        //
        // // Normalised coordinates in [0, 1]:
        // let nx = (px + 0.5) / self.width() as f32;
        // let ny = (py + 0.5) / self.height() as f32;
        //
        // // OpenGL coordinates in [-1, 1]:
        // let rx = nx * 2.0 - 1.0;
        // let ry = ny * 2.0 - 1.0;
        // na::Vector2::new(rx, -ry)

        na::Vector2::new(px as f32, -py as f32)
    }

    fn opengl_coords_for_index(&self, index: [usize; 2]) -> na::Vector2<f32> {
        let px = index[0] as f32;
        let py = index[1] as f32;

        self.opengl_coords_for_point(na::Vector2::new(px, py))
    }

    fn save_pgm(&self, save_fname: &str) -> io::Result<()> {
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(save_fname)
            .unwrap();
        try!(write!(file, "P6\n{} {}\n{}\n", self.width(), self.height(), 255));
        for y in 0..self.height() {
            for x in 0..self.width() {
                let px = self.index(x, y);
                try!(file.write(&[px.y]));
                try!(file.write(&[px.cb]));
                try!(file.write(&[px.cr]));
            }
        }
        try!(writeln!(file, ""));
        Ok(())
    }
}
