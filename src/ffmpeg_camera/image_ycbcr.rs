
// use std::ops;

// use std::cmp;
use std::io;
use std::io::prelude::*;
// use std::fs::File;
use std::fs::OpenOptions;

extern crate ffmpeg_sys;
extern crate libc;
use ffmpeg_camera::ffmpeg_utils;
use ffmpeg_camera::ffmpeg_utils::FfmpegError;

#[derive(Copy, Clone)]
pub struct Pixel {
    pub y : u8,
    pub cb : u8,
    pub cr : u8,
}

#[derive(Clone)]
pub struct Image {
    pub width : usize, // width in pixels
    pub height : usize, // height in pixels
    pub data : Vec<u8>, // uyvy422 data buffer
}

impl Image {

    pub fn contains(&self, col : usize, row : usize) -> bool {
         col < self.width &&
         row < self.height
    }

    pub fn index(&self, col : usize, row : usize) -> Pixel {
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
            Pixel {
                y: *self.data.get_unchecked(y_i),
                cb: *self.data.get_unchecked(u_i),
                cr: *self.data.get_unchecked(v_i),
            }
        }
        // Pixel {
        //     y: self.data[y_i],
        //     cb: self.data[u_i],
        //     cr: self.data[v_i],
        // }
    }

    // Based on: https://lists.libav.org/pipermail/libav-user/2010-August/005159.html
    pub fn save_jpeg(&self, save_fname : &str) -> Result<(), FfmpegError> {
        unsafe {
            let mut yuyv422_frame = try!(ffmpeg_utils::make_avframe(self.width, self.height, &self.data));
            try!(ffmpeg_utils::save_yuyv422_frame_to_jpeg(yuyv422_frame, save_fname));
            ffmpeg_sys::av_frame_free(&mut yuyv422_frame);
        }

        Ok(())
    }

    pub fn save_pgm(&self, save_fname: &str) -> io::Result<()> {
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(save_fname)
            .unwrap();
        try!(write!(file, "P6\n{} {}\n{}\n", self.width, self.height, 255));
        for y in 0..self.height {
            for x in 0..self.width {
                let px = self.index(x, y);
                try!(file.write(&[px.y]));
                try!(file.write(&[px.cb]));
                try!(file.write(&[px.cr]));
            }
        }
        try!(writeln!(file, ""));
        Ok(())
        //     // libc::fwrite(buffer.offset((i * stride) as isize) as *const libc::c_void, 1, img_w*3, f);
    }

    // pub fn index(&self, row : usize, col : usize) -> Pixel {
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
    //         Pixel {
    //             y: *self.data.get_unchecked(y_i),
    //             u: *self.data.get_unchecked(u_i),
    //             v: *self.data.get_unchecked(v_i),
    //         }
    //     }
    // }

}
