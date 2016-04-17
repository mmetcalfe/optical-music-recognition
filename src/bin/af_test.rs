extern crate arrayfire as af;
use af::*; // Ideally, you should use only names you are about to use, for example purposes * is fine.

fn main() {
    let num_rows: u64 = 5;
    let num_cols: u64 = 3;
    let dims = Dim4::new(&[num_rows, num_cols, 1, 1]);
    println!("Create a 5-by-3 matrix of random floats on the GPU");
    let a = match randu(dims, Aftype::F32) {
        Ok(value) => value,
        Err(error) => panic!("{}", error),
    };
    print(&a);
}
