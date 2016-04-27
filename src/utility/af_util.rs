use arrayfire as af;
use nalgebra as na;
use utility;
use std::mem;

pub fn host_to_mat3_f32(array: &af::Array) -> na::Matrix3<f32> {
    // let num_elements = array.elements().unwrap() as usize;
    let dims = array.dims().unwrap();
    if dims[0] != 3 || dims[1] != 3 || dims[2] != 1 || dims[3] != 1 {
        panic!("host_to_mat3_f32, input array must have shape [3, 3, 1, 1].")
    }

    match array.get_type().unwrap() {
        af::Aftype::F32 => {
            let mut matrix = [[0.0; 3]; 3];
            unsafe {
                let arr = mem::transmute::<&mut [[f32; 3]; 3], &mut [f32; 9]>(&mut matrix);
                array.host(arr);
            }
            na::Matrix3::from(&matrix)
        },
        _ => {
            panic!("host_to_mat3_f32, array type was not F32")
        },
    }
}

pub fn host_to_vec_f32(array: &af::Array) -> Vec<f32> {
    // println!("host_to_vec, get_type: {:?}", array.get());
    match array.get_type().unwrap() {
        af::Aftype::F32 => {
            // println!("host_to_vec, data_vec:");
            let num_elements = array.elements().unwrap() as usize;
            // println!("host_to_vec, elements: {}", num_elements);
            let mut data_vec = utility::make_uninitialised_vec::<f32>(num_elements);
            // println!("host_to_vec, array.host: {:?}", data_vec.len());
            array.host(&mut data_vec);
            data_vec
        },
        _ => {
            panic!("host_to_vec, array type was not F32")
        },
    }
}

pub fn host_to_vec_u32(array: &af::Array) -> Vec<u32> {
    // println!("host_to_vec, get_type: {:?}", array.get());
    match array.get_type().unwrap() {
        af::Aftype::U32 => {
            // println!("host_to_vec, data_vec:");
            let num_elements = array.elements().unwrap() as usize;
            // println!("host_to_vec, elements: {}", num_elements);
            let mut data_vec = utility::make_uninitialised_vec::<u32>(num_elements);
            // println!("host_to_vec, array.host: {:?}", data_vec.len());
            array.host(&mut data_vec);
            data_vec
        },
        _ => {
            panic!("host_to_vec, array type was not U32")
        },
    }
}

pub fn host_to_vec_bool(array: &af::Array) -> Vec<bool> {
    // println!("host_to_vec, get_type: {:?}", array.get());
    match array.get_type().unwrap() {
        af::Aftype::B8 => {
            // println!("host_to_vec, data_vec:");
            let num_elements = array.elements().unwrap() as usize;
            // println!("host_to_vec, elements: {}", num_elements);
            let mut data_vec = utility::make_uninitialised_vec::<bool>(num_elements);
            // println!("host_to_vec, array.host: {:?}", data_vec.len());
            array.host(&mut data_vec);
            data_vec
        },
        _ => {
            panic!("host_to_vec, array type was not B8")
        },
    }
}
