extern crate ffmpeg_sys;
extern crate libc;
extern crate arrayfire as af;
use ffmpeg_camera::ffmpeg_utils;
use ffmpeg_camera::ffmpeg_utils::FfmpegError;
// use nalgebra as na;
use std;
use ffmpeg_camera;
use ffmpeg_camera::image::Pixel;

#[derive(Clone)]
pub struct Image {
    pub width : usize, // width in pixels
    pub height : usize, // height in pixels

    pub local_data : Vec<u8>,
    pub af_grey : af::Array,
}

impl ffmpeg_camera::Image for Image {
    fn from_raw_parts(width: usize, height: usize, data: Vec<u8>) -> Image {
        // Data is an array containing raw interleaved YCbCrA data.
        // let shape = [height as u64, width as u64, 4, 1];
        let shape = [width as u64, height as u64, 1, 1];
        let img_nv12 = af::Array::new(&data, af::Dim4::new(&shape)).unwrap();
        // // i.e. [YCbCrA YCbCrA YCbCrA] -> [YYY CbCb CrCr AAA]
        // let img_nv12 = af::reorder(&img_nv12, af::Dim4::new(&[2, 1, 0, 3])).unwrap();

        Image {
            width: width,
            height: height,
            // data: data,
            local_data: data,
            af_grey: img_nv12,
        }
    }

    fn width(&self) -> usize {
        self.width
    }
    fn height(&self) -> usize {
        self.height
    }
    // fn data(&self) -> &Vec<u8> {
    //     &self.data
    // }

    fn index(&self, col : usize, row : usize) -> Pixel {
        if !self.contains(col, row) {
            panic!("Image index out of bounds.");
        }

        let p_i = (row * self.width + col);

        let y_i = p_i + 0;
        // let cb_i = p_i + 1;
        // let cr_i = p_i + 2;

        unsafe {
            Pixel {
                y: *self.local_data.get_unchecked(y_i),
                cb: 128, // *self.local_data.get_unchecked(cb_i),
                cr: 128, // *self.local_data.get_unchecked(cr_i),
            }
        }

        // Pixel {
        //     y: self.local_data[y_i],
        //     cb: self.local_data[y_i + 1],
        //     cr: self.local_data[y_i + 2],
        // }
    }

    // Based on: https://lists.libav.org/pipermail/libav-user/2010-August/005159.html
    fn save_jpeg(&self, save_fname : &str) -> Result<(), FfmpegError> {
        unsafe {
            // let mut data = self.to_byte_vector();

            let mut av_frame = try!(
                ffmpeg_utils::make_avframe(
                    self.width,
                    self.height,
                    // ffmpeg_sys::AV_PIX_FMT_BGR32,
                    ffmpeg_sys::AV_PIX_FMT_YUV420P,
                    // ffmpeg_sys::AV_PIX_FMT_YUV422P,
                    // &self.data
                    // &data
                    &self.local_data
                )
            );
            try!(ffmpeg_utils::save_frame_to_jpeg(av_frame, save_fname));
            ffmpeg_sys::av_frame_free(&mut av_frame);
        }

        Ok(())
    }
}

impl ffmpeg_camera::AfImage for Image {
    fn af_grey(&self) -> af::Array {
        self.af_grey.clone()
    }
}
