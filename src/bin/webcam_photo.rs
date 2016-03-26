extern crate optical_music_recognition;

use optical_music_recognition::ffmpeg_camera::ffmpeg_camera;
// use optical_music_recognition::ffmpeg_camera::ffmpeg_utils;
// use optical_music_recognition::ffmpeg_camera::image_ycbcr;

fn main() {

    let mut camera = ffmpeg_camera::FfmpegCamera::get_default().expect("Failed to open camera.");

    let image = camera.get_image().unwrap();

    image.save_pgm("image.pgm").unwrap();
    image.save_jpeg("image.jpg").unwrap();
}
