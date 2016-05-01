extern crate ffmpeg_sys;
extern crate libc;
extern crate arrayfire as af;
use ffmpeg_camera::ffmpeg_utils;
use ffmpeg_camera::ffmpeg_utils::FfmpegError;
// use nalgebra as na;
use std;
use ffmpeg_camera;
use ffmpeg_camera::image::Pixel;
use utility;

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
    // pub fn to_byte_vector(&self) -> Vec<u8> {
    //     // af::print(&self.data);
    //     // println!("elements: {:?}", self.data.elements().unwrap());
    //     // let mut tmp_data = utility::make_uninitialised_vec::<f32>(self.data.elements().unwrap() as usize);
    //     // self.data.host(&mut tmp_data);
    //     // let data_ptr = tmp_data.as_mut_ptr() as *mut u8;
    //     // let num_bytes = tmp_data.len() * std::mem::size_of::<f32>();
    //     // println!("num_bytes: {:?}", num_bytes);
    //     // unsafe {
    //     //     std::mem::forget(tmp_data); // Don't run tmp_data's destructor.
    //     //     Vec::from_raw_parts(data_ptr, num_bytes, num_bytes)
    //     // }
    //
    //     match self.af_data.get_type().unwrap() {
    //         af::Aftype::U8 => {
    //             let mut tmp_data = utility::make_uninitialised_vec::<u8>(self.af_data.elements().unwrap() as usize);
    //             self.af_data.host(&mut tmp_data);
    //             tmp_data
    //         },
    //         _ => unimplemented!(),
    //     }
    // }

    pub fn from_af_array(rgb_array: af::Array) -> Image {

        // rgb array is in the format loaded from af::load_image.
        // e.g. a 3x1 image looks like [bbb ggg rrr].

        let dims = rgb_array.dims().unwrap();
        let height = dims[0];
        let width = dims[1];

        let zeroes_dims = af::Dim4::new(&[height, width, 1, 1]);
        let zeroes = af::constant::<f32>(0.0, zeroes_dims).unwrap();

        println!("dims: {:?}, zeroes_dims: {:?}", dims, zeroes_dims);

        println!("from_af_array, join:");
        // Append an alpha channel:
        // [bbb ggg rrr] -> [bbb ggg rrr aaa]
        let data_array = af::join(2, &rgb_array, &zeroes).unwrap();
        println!("data_array, shape: {}", data_array.dims().unwrap());
        println!("data_array, elements: {}", data_array.elements().unwrap());

        // println!("from_af_array, host:");
        // let num_bytes = match data_array.get_type().unwrap() {
        //     af::Aftype::U8 => {
        //         data_array.elements().unwrap() as usize
        //     },
        //     af::Aftype::F32 => {
        //         std::mem::size_of::<f32>() * data_array.elements().unwrap() as usize
        //     },
        //     _ => unimplemented!(),
        // };

        let img_32bit = data_array.cast::<u8>().unwrap();
        // let img_32bit = af::transpose(&img_32bit, false).unwrap();
        // Convert arrayfire image format into an interleaved format:
        // i.e. [bbb ggg rrr aaa] -> [bgra bgra bgra]
        let img_32bit = af::reorder(&img_32bit, af::Dim4::new(&[2, 1, 0, 3])).unwrap();
        let num_bytes = img_32bit.elements().unwrap() as usize;
        let mut local_data = utility::make_uninitialised_vec::<u8>(num_bytes);
        println!("local_data: {:?}", local_data.as_mut_ptr());
        img_32bit.host(&mut local_data);

        println!("image:");
        Image {
            width: width as usize,
            height: height as usize,
            local_data: local_data,
            af_data: data_array,
        }
    }
}


impl ffmpeg_camera::Image for Image {
    fn from_raw_parts(width: usize, height: usize, data: Vec<u8>) -> Image {
        // Data is an array containing raw interleaved YCbCrA data.
        // let shape = [height as u64, width as u64, 4, 1];
        let shape = [4, width as u64, height as u64, 1];
        let img_32bit = af::Array::new(&data, af::Dim4::new(&shape)).unwrap();
        // i.e. [YCbCrA YCbCrA YCbCrA] -> [YYY CbCb CrCr AAA]
        let img_32bit = af::reorder(&img_32bit, af::Dim4::new(&[2, 1, 0, 3])).unwrap();

        Image {
            width: width,
            height: height,
            // data: data,
            local_data: data,
            af_data: img_32bit,
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

        let p_i = (row * self.width + col) * 4;

        let y_i = p_i + 0;
        let cb_i = p_i + 1;
        let cr_i = p_i + 2;

        unsafe {
            Pixel {
                y: *self.local_data.get_unchecked(y_i),
                cb: *self.local_data.get_unchecked(cb_i),
                cr: *self.local_data.get_unchecked(cr_i),
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

impl ffmpeg_camera::AfImage for Image {
    fn af_grey(&self) -> af::Array {
        let img_ycbcra = &self.af_data;
        // println!("shape: {}", img_ycbcra.dims().unwrap());
        let img_grey = af::slice(img_ycbcra, 0).unwrap();
        img_grey.cast::<f32>().unwrap()
    }
}
