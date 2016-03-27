#[macro_use]
extern crate glium;
// extern crate image;

extern crate optical_music_recognition;
use optical_music_recognition::ffmpeg_camera::ffmpeg_camera;
use optical_music_recognition::drawing;

// use std::io::Cursor;
use glium::DisplayBuild;
use glium::Surface;

fn main() {
    // let mut camera =
    //     ffmpeg_camera::FfmpegCamera::get_default()
    //         .expect("Failed to open camera.");
    let mut camera =
        ffmpeg_camera::FfmpegCamera::get_camera("default", "29.970000", (1280, 720))
            .expect("Failed to open camera.");

    let display = glium::glutin::WindowBuilder::new().build_glium().unwrap();

    // let image = image::load(Cursor::new(&include_bytes!("../../curved-3.jpg")[..]),
    //                         image::JPEG).unwrap().to_rgba();
    // let image_dimensions = image.dimensions();
    // let image = glium::texture::RawImage2d::from_raw_rgba_reversed(image.into_raw(), image_dimensions);
    // let texture = glium::texture::Texture2d::new(&display, image).unwrap();

    let img_pane = drawing::ImagePane::new(&display);

    loop {
        let webcam_frame = camera.get_image().unwrap();

        // webcam_frame.save_pgm("image.pgm").unwrap();
        // webcam_frame.save_jpeg("image.jpg").unwrap();

        let mut target = display.draw();
        target.clear_color(0.0, 0.0, 1.0, 1.0);

        img_pane.draw_image(&mut target, &webcam_frame);

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
