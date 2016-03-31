extern crate ffmpeg_sys;
extern crate libc;
use ffmpeg_camera::ffmpeg_utils;
use ffmpeg_camera::ffmpeg_utils::FfmpegError;
// use nalgebra as na;

use ffmpeg_camera::image;

#[derive(Clone)]
pub struct Image {
    pub width : usize, // width in pixels
    pub height : usize, // height in pixels

    // YCbCrA data buffer.
    // Note: We'd like this buffer to just contain YCbCr data, so that there'd only be 24 bits per
    // pixel, but reading data in that format back from the graphics card is extremely slow.
    pub data : Vec<u8>,
}

impl image::Image for Image {
    fn from_raw_parts(width: usize, height: usize, data: Vec<u8>) -> Image {
        Image {
            width: width,
            height: height,
            data: data,
        }
    }

    fn width(&self) -> usize {
        self.width
    }
    fn height(&self) -> usize {
        self.height
    }
    fn data(&self) -> &Vec<u8> {
        &self.data
    }

    fn index(&self, col : usize, row : usize) -> image::Pixel {
        if !self.contains(col, row) {
            panic!("Image index out of bounds.");
        }

        let p_i = (row * self.width + col) * 4;

        let y_i = p_i + 0;
        let cb_i = p_i + 1;
        let cr_i = p_i + 2;

        unsafe {
            image::Pixel {
                y: *self.data.get_unchecked(y_i),
                cb: *self.data.get_unchecked(cb_i),
                cr: *self.data.get_unchecked(cr_i),
            }
        }

        // image::Pixel {
        //     y: self.data[y_i],
        //     cb: self.data[y_i + 1],
        //     cr: self.data[y_i + 2],
        // }
    }

    // Based on: https://lists.libav.org/pipermail/libav-user/2010-August/005159.html
    fn save_jpeg(&self, save_fname : &str) -> Result<(), FfmpegError> {
        unsafe {
            let mut yuyv422_frame = try!(ffmpeg_utils::make_avframe(self.width, self.height, ffmpeg_sys::AV_PIX_FMT_BGR32, &self.data));
            try!(ffmpeg_utils::save_frame_to_jpeg(yuyv422_frame, save_fname));
            ffmpeg_sys::av_frame_free(&mut yuyv422_frame);
        }

        Ok(())
    }
}
