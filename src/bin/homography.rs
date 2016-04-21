#[macro_use]
extern crate glium;
extern crate nalgebra as na;
extern crate time;
extern crate arrayfire as af;

// extern crate image;

// use std::f32;

extern crate optical_music_recognition as omr;
use omr::ffmpeg_camera::ffmpeg_camera;
use omr::ffmpeg_camera::ffmpeg_utils;
use omr::ffmpeg_camera::image::Image;
use omr::ffmpeg_camera::image_ycbcr;
use omr::drawing;
use omr::math;
use omr::geometry;
use omr::geometry as gm;
use omr::detection::ransac::staff_cross::StaffCrossLineModel;
use omr::detection::scanning::staff_cross::StaffCross;

// use std::io::Cursor;
use glium::DisplayBuild;
use glium::Surface;
use std::cmp;

use time::Duration;
use time::SteadyTime;

fn main() {
    // af::set_device(0);
    // af::info();
    println!("Available ArrayFire backends: {:?}", af::get_available_backends());

    // println!("Join test:");
    // let values = [
    //     1.0, 2.0,   3.0, 4.0,    5.0,  6.0,
    //     7.0, 8.0,   9.0, 10.0,  11.0, 12.0,
    //     13.0, 14.0,  15.0, 16.0,  17.0, 18.0,
    // ];
    // let test_array = af::Array::new::<f32>(&values, af::Dim4::new(&[2, 3, 3, 1])).unwrap();
    // af::print(&test_array);
    // let seqs = &[
    //     af::Seq::new(0.0, 1.0, 1.0),
    //     af::Seq::new(0.0, 0.0, 1.0),
    //     af::Seq::new(0.0, 0.0, 1.0),
    // ];
    // let pix_array = af::index(&test_array, seqs).unwrap();
    // af::print(&pix_array);
    //
    // return;
    //

    // let (img_w, img_h) = (320, 240);
    // let (img_w, img_h) = (640, 480);
    let (img_w, img_h) = (1280, 720);
    // let (img_w, img_h) = (1920, 1080);

    let mut camera = ffmpeg_camera::FfmpegCamera::get_best((img_w, img_h))
        .expect("Failed to open camera.");

    println!("Create display:");
    let display = glium::glutin::WindowBuilder::new()
        // .with_dimensions(1280, 720)
        // .with_dimensions(img_w as u32*4, img_h as u32*4)
        .with_dimensions(img_w as u32, img_h as u32)
        .with_title(format!("OMR"))
        .build_glium()
        .unwrap();

    println!("Create drawing context:");
    let mut draw_ctx = drawing::context::DrawingContext::new(&display);
    draw_ctx.set_view_matrices_for_image_dimensions(img_w, img_h);

    let mut frame_start_time = SteadyTime::now();

    // // Fake webcam frame:
    // println!("Fake image:");
    // let file_name = String::from("image_uyvy.jpg");
    // let load_result = af::load_image(file_name, true);
    // if load_result.is_err() {
    //     println!("LOAD FAILED!");
    //     return;
    // }
    // let rgb_image = load_result.unwrap();
    // println!("shape: {}", rgb_image.dims().unwrap());
    // let ycbcr_frame = image_ycbcr::Image::from_af_array(rgb_image);
    // println!("save_jpeg:");
    // // ycbcr_frame.save_jpeg("image_ycbcr.jpg").unwrap();

    // return;

    loop {
    // {
        // Get webcam frame:
        let webcam_frame = camera.get_image_uyvy().unwrap();
        // webcam_frame.save_jpeg("image_uyvy.jpg").unwrap();
        let ycbcr_frame = draw_ctx.convert_uyvy_ycbcr(&webcam_frame).unwrap();


        // println!("Sequence:");
        // let seqs = &[
        //     af::Seq::default(),
        //     af::Seq::default(),
        //     af::Seq::new(0.0, 2.0, 1.0),
        // ];
        // let pix_array = af::index(&ycbcr_frame.af_data, seqs).unwrap();
        // let pix_array = pix_array.cast::<f32>().unwrap();
        // println!("from_af_array:");
        // let ycbcr_frame = image_ycbcr::Image::from_af_array(pix_array);

        println!("save:");
        ycbcr_frame.save_jpeg("image_ycbcr.jpg").unwrap();

        let mut target = display.draw();
        target.clear_color(0.0, 0.0, 1.0, 1.0);
        draw_ctx.draw_image_ycbcr(&mut target, &ycbcr_frame);

        // Begin ORB detection:

        let img_ycbcra = &ycbcr_frame.af_data;
        println!("shape: {}", img_ycbcra.dims().unwrap());

        // Slice:
        println!("Slice:");
        let img_grey = af::slice(img_ycbcra, 0).unwrap();
        let img_grey = img_grey.cast::<f32>().unwrap();

        // // Colour convert:
        // println!("Colour convert:");
        // let seqs = &[
        //     af::Seq::default(),
        //     af::Seq::default(),
        //     // Seq::new(1.0, 3.0, 1.0)
        //     // af::Seq::new(1.0, 3.0, 1.0),
        //     // af::Seq::new(1.0, 3.0, 1.0),
        //     af::Seq::new(0.0, 2.0, 1.0)
        // ];
        // let img_ycbcr = af::index(img_ycbcra, seqs).unwrap();
        // // af::print(&img_ycbcr);
        // println!("shape: {}", img_ycbcr.dims().unwrap());
        // let img_grey = af::color_space(&img_ycbcr, af::ColorSpace::GRAY, af::ColorSpace::RGB).unwrap();

        println!("backend: {:?}", img_grey.get_backend());
        println!("shape: {:?}", img_grey.dims().unwrap());
        match img_grey.get_type().unwrap() {
            af::Aftype::F32 => println!("type: f32"),
            _ => println!("type: UNKNOWN"),
        }
        println!("numdims: {:?}", img_grey.numdims().unwrap());

        // let num_rows: u64 = 200;
        // let num_cols: u64 = 200;
        // let dims = af::Dim4::new(&[num_rows, num_cols, 1, 1]);
        // let img_grey = af::randu::<f32>(dims).unwrap();


        let fast_thr = 20.0;
        let max_feat = 200;
        let scl_fctr = 1.5;
        let levels = 4;
        let blur_img = false;
        {
            println!("orb:");
            // TODO: Find out why af::orb only succeeds for 320x240 images.
            let orb_result = af::orb(
                &img_grey,
                fast_thr,
                max_feat,
                scl_fctr,
                levels,
                blur_img
            );

            if let Ok((orb_features, orb_arr)) = orb_result {
                print!("ORB SUCCEDED: ");
                let num_features = orb_features.num_features().unwrap() as usize;
                println!("num_features: {:?}", num_features);
                if num_features > 0 {
                    let af_xpos = orb_features.xpos().unwrap();
                    // af::print(&af_xpos);
                    println!("af_xpos.elements(): {:?}", af_xpos.elements().unwrap());

                    match af_xpos.get_type().unwrap() {
                        af::Aftype::F32 => println!("type: f32"),
                        _ => println!("type: UNKNOWN"),
                    }
                    let mut xpos = ffmpeg_utils::make_uninitialised_vec::<f32>(af_xpos.elements().unwrap() as usize);
                    af_xpos.host(&mut xpos);

                    let af_ypos = orb_features.ypos().unwrap();
                    let mut ypos = ffmpeg_utils::make_uninitialised_vec::<f32>(af_ypos.elements().unwrap() as usize);
                    af_ypos.host(&mut ypos);

                    for i in 0..num_features {
                        let x = xpos[i];
                        let y = ypos[i];
                        // println!("Feature {}: {:?}.", i, (x, y));
                        let pt = na::Vec2::new(x, y);
                        let draw_pt = ycbcr_frame.opengl_coords_for_point(pt);
                        draw_ctx.draw_point(&mut target, draw_pt, 5.0, [1.0, 0.2, 0.2, 1.0]);
                    }
                    println!("End printing scope.");
                } else {
                    println!("NO FEATURES DETECTED.");
                }
            } else {
                print!("ORB FAILED:");
                if let Err(err) = orb_result {
                    println!("{:?}.", err);
                }
            }
            println!("End orb scope.");
        }
        println!("After orb scope.");

        let frame_duration = SteadyTime::now() - frame_start_time;
        frame_start_time = SteadyTime::now();
        let mspf = frame_duration.num_milliseconds();
        let fps = 1000.0 / (mspf as f32);
        let time_str = format!("{} ms/frame, {} fps", mspf, fps);
        draw_ctx.draw_string(
            &mut target,
            &time_str,
            na::Vec2::<f32>::new(-1.0, -1.0),
            0.03,
            (0.0,0.0,0.0,1.0)
        );

        target.finish().unwrap();

        // listing the events produced by the window and waiting to be received
        for ev in display.poll_events() {
            match ev {
                glium::glutin::Event::Closed => return,   // the window has been closed by the user
                _ => ()
            }
        }
    }
}
