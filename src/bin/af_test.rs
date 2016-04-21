// // extern crate arrayfire as af;
// // use af::*; // Ideally, you should use only names you are about to use, for example purposes * is fine.
// //
// // fn main() {
// //     let num_rows: u64 = 5;
// //     let num_cols: u64 = 3;
// //     let dims = Dim4::new(&[num_rows, num_cols, 1, 1]);
// //     println!("Create a 5-by-3 matrix of random floats on the GPU");
// //     let a = match randu(dims, Aftype::F32) {
// //         Ok(value) => value,
// //         Err(error) => panic!("{}", error),
// //     };
// //     print(&a);
// // }
//

extern crate arrayfire as af;
// use std::mem;

fn main() {
    af::set_device(0);
    af::info();

    // let file_name = String::from("../../image.jpg");
    let file_name = String::from("../../smufl-november2-test.jpg");
    // let file_name = String::from("../../image_yuv422.jpg");
    let img_grey = af::load_image(file_name, false).unwrap();
    println!("shape: {}", img_grey.dims().unwrap());
    let dims = img_grey.dims().unwrap();
    let img_h = dims[0];
    let img_w = dims[1];

    let disp_img = img_grey.dims()
        .and_then(|x| af::constant(255 as f32, x))
        .and_then(|x| af::div(&img_grey, &x, false))
        .unwrap();

    let window_name = String::from("ORB test");
    let img_wnd = af::Window::new(img_w as i32, img_h as i32, window_name).unwrap();
    // let img_wnd = af::Window::new(480, 640, window_name).unwrap();
    img_wnd.draw_image(&disp_img, None);
    // loop {
    //     if img_wnd.is_closed().unwrap() == true { break; }
    // }

    // let fast_thr = 20.0;
    // let max_feat = 100;
    // let scl_fctr = 1.5;
    // let levels = 4;
    // let blur_img = false;
    {
        println!("orb:");
        // let (features, orb_arr) = af::orb(
        //     &img_grey,
        //     fast_thr,
        //     max_feat,
        //     scl_fctr,
        //     levels,
        //     blur_img
        // ).unwrap();

        let features = af::fast(&img_grey, 20.0, 9, true, 0.05, 10).unwrap();

        // let features = features.clone();

        {
            let af_xpos = features.xpos().unwrap();
            println!("handle 1: {:?}", af_xpos.get());
            let af_xpos_2 = features.xpos().unwrap();
            println!("handle 2: {:?}", af_xpos_2.get());
            // af_xpos.
            af::print(&af_xpos);
            // unsafe { std::mem::forget(af_xpos); }
            // unsafe { std::mem::forget(af_xpos_2); }
        //     let mut xpos = ffmpeg_utils::make_uninitialised_vec::<u8>(af_xpos.elements().unwrap() as usize);
        //     af_xpos.host(&mut xpos);
        //
        //     let af_ypos = features.ypos().unwrap();
        //     let mut ypos = ffmpeg_utils::make_uninitialised_vec::<u8>(af_ypos.elements().unwrap() as usize);
        //     af_ypos.host(&mut ypos);
        //
            let num_features = features.num_features().unwrap() as usize;
            println!("num_features: {:?}", num_features);

        //
        //     for i in 0..num_features {
        //         let x = xpos[i];
        //         let y = ypos[i];
        //         println!("Feature {}: {:?}.", i, (x, y));
        //     }
        //     println!("End printing scope.");
        }
        println!("End orb scope.");
    }
    println!("After orb scope.");
}


// extern crate arrayfire as af;
//
// use af::*;
// use std::env;
// use std::path::PathBuf;
//
// #[allow(unused_variables)]
// #[allow(unused_must_use)]
// fn main() {
//     set_device(0);
//     info();
//
//     let assets_dir = PathBuf::from("../../");
//
//     let img_wnd = match Window::new(480, 640, String::from("Input Image")) {
//         Ok(v) => { v.set_position(100, 100).unwrap(); v },
//         Err(e)=> panic!("Window creation failed, exiting: {}", e),
//     };
//
//     let hst_wnd = match Window::new(512, 512, String::from("Input Image Histogram")) {
//         Ok(v) => { v.set_position(600, 100).unwrap(); v },
//         Err(e)=> panic!("Window creation failed, exiting: {}", e),
//     };
//
//     let (man, hst) = match load_image(format!("{}/image_yuv422.jpg", assets_dir.display()), false) {
//         Ok(v) => match histogram(&v, 256, 0.0, 255.0) {
//             Ok(h) => (v, h),
//             Err(e)=> panic!("Histogram computation failed, exiting: {}", e),
//         },
//         Err(e)=> panic!("Image loading failed, exiting: {}", e),
//     };
//
//     let disp_img = man.dims()
//         .and_then(|x| constant(255 as f32, x))
//         .and_then(|x| div(&man, &x, false))
//         .unwrap();
//
//     loop {
//         img_wnd.draw_image(&disp_img, None);
//         hst_wnd.draw_hist(&hst, 0.0, 255.0, None);
//
//         if img_wnd.is_closed().unwrap() == true { break; }
//         if hst_wnd.is_closed().unwrap() == true { break; }
//     }
// }
