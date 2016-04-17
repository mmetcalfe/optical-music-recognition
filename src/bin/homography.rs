#[macro_use]
extern crate glium;
extern crate nalgebra as na;
extern crate time;
// extern crate image;

// use std::f32;

extern crate optical_music_recognition as omr;
use omr::ffmpeg_camera::ffmpeg_camera;
use omr::ffmpeg_camera::image::Image;
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
    // let (img_w, img_h) = (320, 240);
    // let (img_w, img_h) = (640, 480);
    // let (img_w, img_h) = (1280, 720);
    let (img_w, img_h) = (1920, 1080);
    let mut camera = ffmpeg_camera::FfmpegCamera::get_best((img_w, img_h))
        .expect("Failed to open camera.");

    let display = glium::glutin::WindowBuilder::new()
        // .with_dimensions(1280, 720)
        .with_dimensions(img_w as u32*4, img_h as u32*4)
        .with_title(format!("OMR"))
        .build_glium()
        .unwrap();

    let mut draw_ctx = drawing::context::DrawingContext::new(&display);
    draw_ctx.set_view_matrices_for_image_dimensions(img_w, img_h);

    let mut frame_start_time = SteadyTime::now();
    loop {
        let webcam_frame = camera.get_image_uyvy().unwrap();
        let ycbcr_frame = draw_ctx.convert_uyvy_ycbcr(&webcam_frame).unwrap();

        let mut target = display.draw();
        target.clear_color(0.0, 0.0, 1.0, 1.0);

        draw_ctx.draw_image_ycbcr(&mut target, &ycbcr_frame);

        let processing_start_time = SteadyTime::now();



        let frame_duration = SteadyTime::now() - frame_start_time;
        let processing_duration = SteadyTime::now() - processing_start_time;
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
