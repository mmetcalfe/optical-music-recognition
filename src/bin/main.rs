#[macro_use]
extern crate glium;
extern crate nalgebra as na;
// extern crate image;

// use std::f32;

extern crate optical_music_recognition;
use optical_music_recognition::ffmpeg_camera::ffmpeg_camera;
use optical_music_recognition::drawing;
use optical_music_recognition::omr;

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

    let display = glium::glutin::WindowBuilder::new().build_glium().unwrap();

    // let image = image::load(Cursor::new(&include_bytes!("../../curved-3.jpg")[..]),
    //                         image::JPEG).unwrap().to_rgba();
    // let image_dimensions = image.dimensions();
    // let image = glium::texture::RawImage2d::from_raw_rgba_reversed(image.into_raw(), image_dimensions);
    // let texture = glium::texture::Texture2d::new(&display, image).unwrap();

    let draw_ctx = drawing::context::DrawingContext::new(&display);
    // let img_pane = drawing::image_pane::ImagePane::new(&display);
    // let rect_buff = drawing::rectangle_buffer::RectangleBuffer::new(&display);

    loop {
        let webcam_frame = camera.get_image().unwrap();

        // webcam_frame.save_pgm("image.pgm").unwrap();
        // webcam_frame.save_jpeg("image.jpg").unwrap();

        let mut target = display.draw();
        target.clear_color(0.0, 0.0, 1.0, 1.0);

        draw_ctx.draw_image(&mut target, &webcam_frame);

        // let rect = drawing::rectangle_buffer::RotatedRectangle {
        //     position: [0.5, 0.0],
        //     size: [1.0, 0.25],
        //     angle: f32::consts::PI/4.0,
        // };
        // draw_ctx.draw_rectangle(&mut target, &rect, [1.0, 0.0, 1.0, 0.0]);

        // let x = 256;
        for x in (0..webcam_frame.width).filter(|x| x % 10 == 0) {
            let scanner = omr::scanning::StaffScanner::new(&webcam_frame, [x, 0]);
            // println!("Crosses:");
            for (i, cross) in scanner.filter(|c| c.is_plausible()).enumerate() {
                // let col = [(i*71 % 255) as f32 / 255.0, 0.5 * (i*333 % 255) as f32 / 255.0, 0.0, 0.0];
                let col = [1.0, 0.0, 0.0, 1.0];
                // println!("{:?}", cross);
                for span in cross.spans() {
                    let pix_w = 2.0 * (1.0 / webcam_frame.width as f32);
                    let p1 = webcam_frame.opengl_coords_for_index([x, span[0]]);
                    let p2 = webcam_frame.opengl_coords_for_index([x, span[1]]);
                    draw_ctx.draw_line(&mut target, p1, p2, pix_w * 5.0, col);
                }
            }
        }

        // Scan entire image for StaffCross points:

        // Draw detected StaffCross points:

        // Run RANSAC on the StaffCross points to find a line:

        // Draw the detected line:



        // for cross in scanner {
        //     println!("{:?}", cross);
        // }


        // draw_ctx.draw_line(&mut target, [-0.5, 0.5], [0.0, -0.5], 0.01, [1.0, 0.0, 1.0, 0.0]);
        // draw_ctx.draw_line(&mut target, [-0.5, 0.0], [0.5, 1.0], 0.02, [1.0, 1.0, 0.0, 0.0]);
        // draw_ctx.draw_line(&mut target, [0.5, 1.0], [-0.5, 0.0], 0.01, [0.0, 1.0, 0.0, 0.0]);

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
