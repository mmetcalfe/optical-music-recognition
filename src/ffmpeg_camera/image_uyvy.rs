
// use std::ops;
// use std::cmp;
// use std::fs::File;

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
    pub data : Vec<u8>, // uyvy422 data buffer
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

        // u y v y u y v y ...
        let row_offset = row * self.width * 2;
        let y_i = row_offset + col*2 + 1;
        // let uv_offset = ((col*2) / 4) * 4;
        let uv_offset = (col / 2) * 4;
        let u_i = row_offset + uv_offset + 0;
        let v_i = row_offset + uv_offset + 2;

        unsafe {
            image::Pixel {
                y: *self.data.get_unchecked(y_i),
                cb: *self.data.get_unchecked(u_i),
                cr: *self.data.get_unchecked(v_i),
            }
        }
        // image::Pixel {
        //     y: self.data[y_i],
        //     cb: self.data[u_i],
        //     cr: self.data[v_i],
        // }
    }

    // Based on: https://lists.libav.org/pipermail/libav-user/2010-August/005159.html
    fn save_jpeg(&self, save_fname : &str) -> Result<(), FfmpegError> {
        unsafe {
            let mut yuyv422_frame = try!(ffmpeg_utils::make_avframe(self.width, self.height, ffmpeg_sys::AV_PIX_FMT_UYVY422, &self.data));
            try!(ffmpeg_utils::save_frame_to_jpeg(yuyv422_frame, save_fname));
            ffmpeg_sys::av_frame_free(&mut yuyv422_frame);
        }

        Ok(())
    }

    // pub fn index(&self, row : usize, col : usize) -> image::Pixel {
    //     // // Use nearest u and v:
    //     // // u y v y u y v y ...
    //     // // 1 1 1
    //     // //     2 2 2
    //     // let pos = (row * self.width + col) * 2;
    //     // let y_i = pos + 1;
    //     // // let u_i = pos + if even_col { 0 } else { 2 };
    //     // // let v_i = pos + if even_col { 2 } else { 0 };
    //     // let uv_offset = (col % 2) * 2;
    //     // let u_i = pos + uv_offset;
    //     // let v_i = pos + (2 - uv_offset);
    //
    //     // // Grouped yuyv:
    //     // // u y v y u y v y ...
    //     // // 1 1 1
    //     // // 2   2 2
    //     // let row_offset = row * self.width * 2;
    //     // let y_i = row_offset + col*2 + 1;
    //     // // let uv_offset = ((col*2) / 4) * 4;
    //     // let uv_offset = (col / 2) * 4;
    //     // let u_i = row_offset + uv_offset + 0;
    //     // let v_i = row_offset + uv_offset + 2;
    //
    //     // Grouped yuyv:
    //     // u y v y u y v y ...
    //     // 1 1 1
    //     // 2   2 2
    //     let row_offset = row * self.width * 2;
    //     let y_i = row_offset + col*2 + 1;
    //     // let uv_offset = ((col*2) / 4) * 4;
    //     let uv_offset = (col / 2) * 4;
    //     let u_i = row_offset + uv_offset + 0;
    //     let v_i = row_offset + uv_offset + 2;
    //
    //     unsafe {
    //         image::Pixel {
    //             y: *self.data.get_unchecked(y_i),
    //             u: *self.data.get_unchecked(u_i),
    //             v: *self.data.get_unchecked(v_i),
    //         }
    //     }
    // }

}
