extern crate ffmpeg_sys;
extern crate libc;
extern crate arrayfire as af;
use ffmpeg_camera::ffmpeg_utils;
use ffmpeg_camera::ffmpeg_utils::FfmpegError;
// use nalgebra as na;
use std;
use ffmpeg_camera::image;

#[derive(Clone)]
pub struct Image {
    pub width : usize, // width in pixels
    pub height : usize, // height in pixels

    // YCbCrA data buffer.
    // Note: We'd like this buffer to just contain YCbCr data, so that there'd only be 24 bits per
    // pixel, but reading data in that format back from the graphics card is extremely slow.
    // Note: local_data remains as a copy of the image data because ArrayFire does not appear to
    // allow easy indexing of arrays.
    // TODO: Remove the need for performing and storing multiple copies of the image data.
    pub local_data : Vec<u8>,
    pub af_data : af::Array,
}

impl Image {
    pub fn to_byte_vector(&self) -> Vec<u8> {
        // af::print(&self.data);
        // println!("elements: {:?}", self.data.elements().unwrap());
        // let mut tmp_data = ffmpeg_utils::make_uninitialised_vec::<f32>(self.data.elements().unwrap() as usize);
        // self.data.host(&mut tmp_data);
        // let data_ptr = tmp_data.as_mut_ptr() as *mut u8;
        // let num_bytes = tmp_data.len() * std::mem::size_of::<f32>();
        // println!("num_bytes: {:?}", num_bytes);
        // unsafe {
        //     std::mem::forget(tmp_data); // Don't run tmp_data's destructor.
        //     Vec::from_raw_parts(data_ptr, num_bytes, num_bytes)
        // }

        let mut tmp_data = ffmpeg_utils::make_uninitialised_vec::<u8>(self.af_data.elements().unwrap() as usize);
        self.af_data.host(&mut tmp_data);
        tmp_data
    }
}


impl image::Image for Image {
    fn from_raw_parts(width: usize, height: usize, data: Vec<u8>) -> Image {
        // let shape = [height as u64, width as u64, 4, 1];
        let shape = [width as u64, height as u64, 4, 1];
        let array = af::Array::new(af::Dim4::new(&shape), &data, af::Aftype::U8).unwrap();

        Image {
            width: width,
            height: height,
            // data: data,
            local_data: data,
            af_data: array,
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
                y: *self.local_data.get_unchecked(y_i),
                cb: *self.local_data.get_unchecked(cb_i),
                cr: *self.local_data.get_unchecked(cr_i),
            }
        }

        // image::Pixel {
        //     y: self.local_data[y_i],
        //     cb: self.local_data[y_i + 1],
        //     cr: self.local_data[y_i + 2],
        // }
    }

    // Based on: https://lists.libav.org/pipermail/libav-user/2010-August/005159.html
    fn save_jpeg(&self, save_fname : &str) -> Result<(), FfmpegError> {
        unsafe {
            // let mut data = self.to_byte_vector();

            let mut yuyv422_frame = try!(
                ffmpeg_utils::make_avframe(
                    self.width,
                    self.height,
                    ffmpeg_sys::AV_PIX_FMT_BGR32,
                    // &self.data
                    // &data
                    &self.local_data
                )
            );
            try!(ffmpeg_utils::save_frame_to_jpeg(yuyv422_frame, save_fname));
            ffmpeg_sys::av_frame_free(&mut yuyv422_frame);
        }

        Ok(())
    }
}
