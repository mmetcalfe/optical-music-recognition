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
use omr::geometry as gm;
use omr::detection::ransac::staff_cross::StaffCrossLineModel;
use omr::detection::scanning::staff_cross::StaffCross;

// use std::io::Cursor;
use glium::DisplayBuild;
use glium::Surface;
use std::cmp;

use time::Duration;
use time::SteadyTime;

use std::cell::RefCell;

fn get_fake_webcam_frame() -> image_ycbcr::Image {
    // Fake webcam frame:
    println!("Fake image:");
    let file_name = String::from("image_uyvy.jpg");
    let load_result = af::load_image(file_name, true);
    if load_result.is_err() {
        panic!("LOAD FAILED!");
    }
    let rgb_image = load_result.unwrap();
    println!("shape: {}", rgb_image.dims().unwrap());
    let ycbcr_frame = image_ycbcr::Image::from_af_array(rgb_image);
    println!("save_jpeg:");
    // ycbcr_frame.save_jpeg("image_ycbcr.jpg").unwrap();

    ycbcr_frame

    // // Reconstruct webcam frame vector from its af_array.
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
}

fn draw_orb_features<I: Image>(target: &mut glium::Frame, frame: &omr::drawing::context::DrawingFrame, orb_features: &af::Features, image: &I) {
    let num_features = orb_features.num_features().unwrap() as usize;

    if num_features > 0 {
        let af_xpos = orb_features.xpos().unwrap();
        // println!("xpos host_to_vec:");
        let xpos = omr::utility::af_util::host_to_vec(&af_xpos);

        let af_ypos = orb_features.ypos().unwrap();
        let ypos = omr::utility::af_util::host_to_vec(&af_ypos);

        for i in 0..num_features {
            let x = xpos[i];
            let y = ypos[i];
            // println!("Feature {}: {:?}.", i, (x, y));
            let pt = na::Vector2::new(x, y);
            let draw_pt = image.opengl_coords_for_point(pt);
            frame.draw_point(target, draw_pt, 5.0, [1.0, 0.2, 0.2, 1.0]);
        }
    }
}

fn main() {
    // af::set_device(0);
    // af::info();
    println!("Available ArrayFire backends: {:?}", af::get_available_backends());

    // let (img_w, img_h) = (320, 240);
    // let (img_w, img_h) = (640, 480);
    let (img_w, img_h) = (1280, 720);
    // let (img_w, img_h) = (1920, 1080);

    let mut camera = ffmpeg_camera::FfmpegCamera::get_best((img_w, img_h))
        .expect("Failed to open camera.");

    let window_dims = (img_w, img_h);
    println!("Create display:");
    let display = glium::glutin::WindowBuilder::new()
        // .with_dimensions(1280, 720)
        // .with_dimensions(img_w as u32*4, img_h as u32*4)
        .with_dimensions(window_dims.0 as u32, window_dims.1 as u32)
        .with_title(format!("OMR"))
        .build_glium()
        .unwrap();

    println!("Create drawing context:");
    let mut draw_ctx = RefCell::new(drawing::context::DrawingContext::new(&display));
    draw_ctx.borrow_mut().set_window_dims(window_dims);
    let mut window_frame = drawing::context::DrawingContext::get_default_frame(&draw_ctx);

    let mut photo_frame = drawing::context::DrawingContext::get_default_frame(&draw_ctx);
    photo_frame.rect = gm::RotatedRectangle {
        position: [0.5, 0.5],
        size: [1.0, 1.0],
        angle: 0.0,
    };

    let mut video_frame = drawing::context::DrawingContext::get_default_frame(&draw_ctx);
    video_frame.rect = gm::RotatedRectangle {
        position: [-0.5, 0.5],
        size: [1.0, 1.0],
        angle: 0.0,
    };

    let mut frame_start_time = SteadyTime::now();

    let mut captured_frame: Option<image_ycbcr::Image> = None;
    let mut captured_frame_features: Option<(af::Features, af::Array)> = None;
    let mut take_photo = false;

    loop {
    // {
        // Get webcam frame:
        let webcam_frame = camera.get_image_uyvy().unwrap();
        // webcam_frame.save_jpeg("image_uyvy.jpg").unwrap();
        let ycbcr_frame = draw_ctx.borrow_mut().convert_uyvy_ycbcr(&webcam_frame).unwrap();
        // // Fake webcam frame:
        // let ycbcr_frame = get_fake_webcam_frame();

        // println!("save:");
        ycbcr_frame.save_jpeg("image_ycbcr.jpg").unwrap();

        let mut target = display.draw();
        target.clear_color(0.0, 0.0, 1.0, 1.0);
        video_frame.draw_image_ycbcr(&mut target, &ycbcr_frame);
        if let Some(ref frame) = captured_frame {
            photo_frame.draw_image_ycbcr(&mut target, &frame);
        }

        // Obtain greyscale image:
        let img_grey = {
            let img_ycbcra = &ycbcr_frame.af_data;
            // println!("shape: {}", img_ycbcra.dims().unwrap());
            let img_grey = af::slice(img_ycbcra, 0).unwrap();
            img_grey.cast::<f32>().unwrap()
        };

        if take_photo {
            captured_frame_features = None;
        }

        {
            // Begin ORB detection:
            let fast_thr = 80.0;
            let max_feat = 16;
            let scl_fctr = 1.5;
            let levels = 4;
            let blur_img = true;

            let orb_result = af::orb(
                &img_grey,
                fast_thr,
                max_feat,
                scl_fctr,
                levels,
                blur_img
            );

            if let Ok((orb_features, orb_descriptors)) = orb_result {
                draw_orb_features(&mut target, &video_frame, &orb_features, &ycbcr_frame);

                print!("ORB SUCCEDED: ");

                if let Some((ref frame_features, ref frame_descriptors)) = captured_frame_features {
                    draw_orb_features(&mut target, &photo_frame, &frame_features, &ycbcr_frame);
                    //
                    // let num_current_features = orb_features.num_features().unwrap() as usize;
                    // let num_frame_features = frame_features.num_features().unwrap() as usize;
                    // println!("num_features: {:?}", num_features);
                    // if num_features > 6 && num_frame_features > 6 {
                    //
                    // }
                }

                if take_photo {
                    captured_frame_features = Some((orb_features, orb_descriptors));
                }
            } else {
                print!("ORB FAILED:");
                if let Err(err) = orb_result {
                    println!("{:?}.", err);
                }
            }
        }

        let frame_duration = SteadyTime::now() - frame_start_time;
        frame_start_time = SteadyTime::now();
        let mspf = frame_duration.num_milliseconds();
        let fps = 1000.0 / (mspf as f32);
        let time_str = format!("{} ms/frame, {} fps", mspf, fps);
        window_frame.draw_string(
            &mut target,
            &time_str,
            na::Vector2::<f32>::new(-1.0, -1.0),
            0.03,
            (0.0,0.0,0.0,1.0)
        );

        target.finish().unwrap();

        if take_photo {
            println!("TAKE PHOTO!");
            ycbcr_frame.save_jpeg("captured_frame.jpg").unwrap();
            take_photo = false;

            captured_frame = Some(ycbcr_frame);
            continue;
        }

        // listing the events produced by the window and waiting to be received
        for ev in display.poll_events() {
            match ev {
                // the window has been closed by the user
                glium::glutin::Event::Closed => return,
                glium::glutin::Event::ReceivedCharacter(ch) => {
                    if ch == ' ' {
                        take_photo = true;
                    }
                },
                _ => ()
            }
        }
    }
}
