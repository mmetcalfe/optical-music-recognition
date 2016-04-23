extern crate arrayfire as af;
use utility;

pub fn host_to_vec(array: &af::Array) -> Vec<f32> {
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
            // println!("host_to_vec, unimplemented:");
            unimplemented!()
        },
    }
}
