#[macro_use]
extern crate glium;
extern crate nalgebra as na;
// extern crate image;

// use std::f32;

extern crate optical_music_recognition;
use optical_music_recognition::ffmpeg_camera::ffmpeg_camera;
use optical_music_recognition::ffmpeg_camera::image::Image;
use optical_music_recognition::drawing;
use optical_music_recognition::omr;
// use optical_music_recognition::geometry;
use optical_music_recognition::omr::ransac::staff_cross::StaffCrossLineModel;
// use std::io::Cursor;
use glium::DisplayBuild;
use glium::Surface;

fn main() {
    // let mut camera =
    //     ffmpeg_camera::FfmpegCamera::get_default()
    //         .expect("Failed to open camera.");
    // let mut camera =
    //     ffmpeg_camera::FfmpegCamera::get_camera("default", "29.970000", (1280, 720))
    //         .expect("Failed to open camera.");

    let mut camera =
        ffmpeg_camera::FfmpegCamera::get_camera("HD Pro Webcam C920", "30.000030", (1280, 720))
            .expect("Failed to open camera.");

    let display = glium::glutin::WindowBuilder::new()
        .with_dimensions(1280, 720)
        .with_title(format!("OMR"))
        .build_glium()
        .unwrap();

    // let image = image::load(Cursor::new(&include_bytes!("../../curved-3.jpg")[..]),
    //                         image::JPEG).unwrap().to_rgba();
    // let image_dimensions = image.dimensions();
    // let image = glium::texture::RawImage2d::from_raw_rgba_reversed(image.into_raw(), image_dimensions);
    // let texture = glium::texture::Texture2d::new(&display, image).unwrap();

    let draw_ctx = drawing::context::DrawingContext::new(&display);
    // let img_pane = drawing::image_pane::ImagePane::new(&display);
    // let rect_buff = drawing::rectangle_buffer::RectangleBuffer::new(&display);

    loop {
        let webcam_frame = camera.get_image_uyvy().unwrap();
        // let webcam_frame = camera.get_image_ycbcr().unwrap();

        // webcam_frame.save_pgm("image.pgm").unwrap();
        webcam_frame.save_jpeg("image_yuv422.jpg").unwrap();

        let ycbcr_frame = draw_ctx.convert_uyvy_ycbcr(&webcam_frame).unwrap();

        ycbcr_frame.save_jpeg("image_ycbcr.jpg").unwrap();


        let mut target = display.draw();
        target.clear_color(0.0, 0.0, 1.0, 1.0);

        // draw_ctx.draw_image_uyvy(&mut target, &webcam_frame);
        draw_ctx.draw_image_ycbcr(&mut target, &ycbcr_frame);

        // Scan entire image for StaffCross points:
        let num_scan_lines = 100;
        let cross_points = omr::scanning::staff_cross::scan_entire_image(&ycbcr_frame, num_scan_lines);

        // Draw detected StaffCross points:
        for cross in cross_points.iter() {
            // let col = [(i*71 % 255) as f32 / 255.0, 0.5 * (i*333 % 255) as f32 / 255.0, 0.0, 0.0];
            let col = [1.0, 0.0, 0.0, 1.0];
            let x = cross.x;
            for span in cross.spans() {
                let pix_w = 2.0 * (1.0 / ycbcr_frame.width as f32);
                let pix_h = 2.0 * (1.0 / ycbcr_frame.height as f32);
                let mut p1 = ycbcr_frame.opengl_coords_for_index([x, span[0]]);
                let mut p2 = ycbcr_frame.opengl_coords_for_index([x, span[1]]);

                // Draw from the top of the first pixel to the bottom of the second:
                if p2[1] < p1[1] {
                        p1[1] += pix_h / 2.0;
                        p2[1] -= pix_h / 2.0;
                    } else {
                        p1[1] -= pix_h / 2.0;
                        p2[1] += pix_h / 2.0;
                    }

                draw_ctx.draw_line(&mut target, p1, p2, pix_w * 5.0, col);
            }
        }

        // // Draw segments:
        // let segments = omr::scanning::segment::scan_entire_image(&ycbcr_frame, num_scan_lines);
        // // let x = 256;
        // for segment in segments.iter() {
        //     // let col = [(i*71 % 255) as f32 / 255.0, 0.5 * (i*333 % 255) as f32 / 255.0, 0.0, 0.0];
        //     let col = [1.0, 0.0, 0.0, 1.0];
        //     let x = segment.x;
        //
        //     let pix_w = 2.0 * (1.0 / ycbcr_frame.width as f32);
        //     let pix_h = 2.0 * (1.0 / ycbcr_frame.height as f32);
        //     let mut p1 = ycbcr_frame.opengl_coords_for_index([x, segment.y_min]);
        //     let mut p2 = ycbcr_frame.opengl_coords_for_index([x, segment.y_max]);
        //
        //     // Draw from the top of the first pixel to the bottom of the second:
        //     if p2[1] < p1[1] {
        //         p1[1] += pix_h / 2.0;
        //         p2[1] -= pix_h / 2.0;
        //     } else {
        //         p1[1] -= pix_h / 2.0;
        //         p2[1] += pix_h / 2.0;
        //     }
        //
        //     // draw_ctx.draw_line(&mut target, p1, p2, pix_w * 5.0, col);
        //     draw_ctx.draw_line(&mut target, p1, p2, pix_w, col);
        // }

        // Run RANSAC on the StaffCross points to find a line:
        let params = omr::ransac::RansacParams {
            num_iterations: 100,
            max_distance: 111.0, // not currently used
            min_inliers: 10,
        };
        let maybe_line = omr::ransac::ransac::<StaffCrossLineModel,_,_>(params, &cross_points);

        // Draw the detected line:
        if let Some(line) = maybe_line {
            let pix_w = 2.0 * (1.0 / ycbcr_frame.width as f32);
            let col = [0.0, 0.0, 1.0, 1.0];
            let col_ext = [0.0, 1.0, 0.0, 1.0];
            let p1 = ycbcr_frame.opengl_coords_for_point(line.a.centre());
            let p2 = ycbcr_frame.opengl_coords_for_point(line.b.centre());
            draw_ctx.draw_line_extended(&mut target, p1, p2, pix_w * 5.0, col_ext);
            draw_ctx.draw_line(&mut target, p1, p2, pix_w * 5.0, col);
        }

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
