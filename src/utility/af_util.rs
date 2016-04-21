extern crate arrayfire as af;
use utility;

pub fn host_to_vec(array: &af::Array) -> Vec<f32> {
    match array.get_type().unwrap() {
        af::Aftype::F32 => {
            let mut data_vec = utility::make_uninitialised_vec::<f32>(array.elements().unwrap() as usize);
            array.host(&mut data_vec);
            data_vec
        },
        _ => unimplemented!(),
    }
}
