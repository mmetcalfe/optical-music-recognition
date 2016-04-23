extern crate core;

pub mod af_util;
use std::mem;

use self::core::default::Default;

pub fn make_uninitialised_vec<T: Clone + Default>(length : usize) -> Vec<T> {
    vec![T::default(); length]
}

// pub fn make_uninitialised_vec<T>(length : usize) -> Vec<T> {
//     // See: https://doc.rust-lang.org/std/vec/struct.Vec.html#method.from_raw_parts
//     let mut tmp_data = Vec::<T>::with_capacity(length);
//     let data_ptr = tmp_data.as_mut_ptr();
//     unsafe {
//         mem::forget(tmp_data); // Don't run tmp_data's destructor.
//         // Create a full vector of uninitialised values:
//         Vec::from_raw_parts(data_ptr, length, length)
//     }
// }

pub fn vec_to_bytes<T>(mut input_vec: Vec<T>) -> Vec<u8> {
    // See: https://doc.rust-lang.org/std/vec/struct.Vec.html#method.from_raw_parts
    let size = mem::size_of::<T>()*input_vec.len();
    let data_ptr = input_vec.as_mut_ptr();
    unsafe {
        mem::forget(input_vec); // Don't run input_vec's destructor.
        // Create a full vector of uninitialised values:
        Vec::from_raw_parts(data_ptr as *mut u8, size, size)
    }
}
