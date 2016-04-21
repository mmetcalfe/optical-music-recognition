// file: af_bug_mwe.rs
extern crate arrayfire as af;
use std::mem;
fn main() {
    af::set_device(0);
    af::info();

    // Load the test image: http://www.smufl.org/wp-content/uploads/M3.jpeg
    let file_name = String::from("../../smufl-november2-test.jpg");
    let img_grey = af::load_image(file_name, false).unwrap();
    {
        let features = af::fast(&img_grey, 20.0, 9, true, 0.05, 10).unwrap();
        let num_features = features.num_features().unwrap() as usize;
        println!("num_features: {:?}", num_features);

        let af_xpos = features.xpos().unwrap();
        println!("num_elements: {:?}", af_xpos.elements().unwrap());

        // unsafe { std::mem::forget(af_xpos); }

        println!("End features scope.");
    }
    println!("End program.");
}

/* Pull request text:

Currently, calling accessor methods on a Features object causes a crash due to a double free.

A (reasonably) minimal example of this problem is demonstrated in the following program:

```rust
// file: af_bug_mwe.rs
extern crate arrayfire as af;
use std::mem;
fn main() {
    af::set_device(0);
    af::info();

    // Load the test image: http://www.smufl.org/wp-content/uploads/M3.jpeg
    let file_name = String::from("../../smufl-november2-test.jpg");
    let img_grey = af::load_image(file_name, false).unwrap();
    {
        let features = af::fast(&img_grey, 20.0, 9, true, 0.05, 10).unwrap();
        let num_features = features.num_features().unwrap() as usize;
        println!("num_features: {:?}", num_features);

        let af_xpos = features.xpos().unwrap();
        println!("num_elements: {:?}", af_xpos.elements().unwrap());

        // unsafe { mem::forget(af_xpos); }

        println!("End features scope.");
    }
    println!("End program.");
}
```

Using the current devel branch of arrayfire-rust (f22a5fb799a5b8a2892a10cdee762992970ced50) the program outputs the following:

    ArrayFire v3.2.2 (OpenCL, 64-bit Mac OSX, build 7507b61)
    [0] APPLE   : HD Graphics 4000
    -1- APPLE   : GeForce GT 650M
    num_features: 3827
    num_elements: 3827
    End features scope.
    af_bug_mwe(18660,0x7fff7aa27300) malloc: *** error for object 0x7facabcf43e0: pointer being freed was not allocated
    *** set a breakpoint in malloc_error_break to debug
    An unknown error occurred

    To learn more, run the command again with --verbose.

Un-commenting the unsafe line in the above example prevents the error.

This pull request changes the feat_func_def macro in the vision module to retain the arrays returned from af_get_features_* functions, in the same way that ArrayFire's [current c++ implementation](https://github.com/arrayfire/arrayfire/blob/devel/src/api/cpp/features.cpp#L59) does.

With this change applied the output of the above program is as expected:

    ArrayFire v3.2.2 (OpenCL, 64-bit Mac OSX, build 7507b61)
    [0] APPLE   : HD Graphics 4000
    -1- APPLE   : GeForce GT 650M
    num_features: 3827
    num_elements: 3827
    End features scope.
    End program.

*/
