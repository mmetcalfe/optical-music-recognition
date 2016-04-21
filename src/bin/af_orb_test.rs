// file: af_bug_mwe.rs
extern crate arrayfire as af;
use std::mem;
fn main() {
    af::set_device(0);
    af::info();

    // Load the test image: http://www.smufl.org/wp-content/uploads/M3.jpeg
    // let file_name = String::from("../../smufl-november2-test.jpg");
    // let file_name = String::from("../../image_uyvy.jpg");
    let file_name = String::from("image_uyvy.jpg");
    let img_col = af::load_image(file_name, true).unwrap();
    println!("shape: {}", img_col.dims().unwrap());

    let img = af::color_space(&img_col, af::ColorSpace::GRAY, af::ColorSpace::RGB).unwrap();

    {
        // let features = af::fast(&img_grey, 20.0, 9, true, 0.05, 10).unwrap();
        // let num_features = features.num_features().unwrap() as usize;
        // println!("num_features: {:?}", num_features);

        let fast_thr = 20.0;
        let max_feat = 400;
        let scl_fctr = 1.5;
        let levels = 4;
        let blur_img = false;

        let orb_result = af::orb(
            &img,
            fast_thr,
            max_feat,
            scl_fctr,
            levels,
            blur_img
        );

        if let Ok((orb_features, orb_arr)) = orb_result {
            let num_features = orb_features.num_features().unwrap() as usize;
            println!("num_features: {:?}", num_features);

            println!("End features scope.");
        } else {
            if let Err(err) = orb_result {
                println!("ORB FAILED: {:?}.", err);
            }
        }

        // let af_xpos = features.xpos().unwrap();
        // println!("num_elements: {:?}", af_xpos.elements().unwrap());

        // unsafe { std::mem::forget(af_xpos); }

    }
    println!("End program.");
}
