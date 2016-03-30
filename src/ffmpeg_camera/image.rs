
use ffmpeg_camera::ffmpeg_utils::FfmpegError;
use std::io;
use nalgebra as na;


#[derive(Copy, Clone)]
pub struct Pixel {
    pub y : u8,
    pub cb : u8,
    pub cr : u8,
}

pub trait Image : Clone {
    fn width(&self) -> usize; // width in pixels
    fn height(&self) -> usize; // height in pixels
    fn data(&self) -> &Vec<u8>; // raw data buffer

    fn index(&self, col : usize, row : usize) -> Pixel;
    fn save_jpeg(&self, save_fname : &str) -> Result<(), FfmpegError>;
    fn save_pgm(&self, save_fname: &str) -> io::Result<()>;

    fn contains(&self, col : usize, row : usize) -> bool {
         col < self.width() &&
         row < self.height()
    }

    fn opengl_coords_for_point(&self, point: na::Vec2<f32>) -> na::Vec2<f32> {
        let px = point[0];
        let py = point[1];

        // // Half-pixel offsets:
        // let ox = 0.5 / width;
        // let oy = 0.5 / height;

        // Normalised coordinates in [0, 1]:
        let nx = (px + 0.5) / self.width() as f32;
        let ny = (py + 0.5) / self.height() as f32;

        // OpenGL coordinates in [-1, 1]:
        let rx = nx * 2.0 - 1.0;
        let ry = ny * 2.0 - 1.0;
        na::Vec2::new(rx, -ry)
    }

    fn opengl_coords_for_index(&self, index: [usize; 2]) -> na::Vec2<f32> {
        let px = index[0] as f32;
        let py = index[1] as f32;

        self.opengl_coords_for_point(na::Vec2::new(px, py))
    }
}
