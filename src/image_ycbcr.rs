
// use std::ops;

// use std::io::prelude::*;
// use std::fs::File;
// use std::fs::OpenOptions;

pub struct Pixel {
    pub y : u8,
    pub cb : u8,
    pub cr : u8,
}

pub struct Image {
    pub width : usize,
    pub height : usize,
    pub data : Vec<u8>,
}

impl Image {

    pub fn index(&self, row : usize, col : usize) -> Pixel {
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
    }

    // pub fn save

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
