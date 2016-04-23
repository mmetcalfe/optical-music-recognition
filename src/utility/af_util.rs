extern crate arrayfire as af;
use utility;

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
