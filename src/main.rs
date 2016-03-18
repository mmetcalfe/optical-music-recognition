pub mod ffmpeg_camera;
pub mod ffmpeg_utils;
pub mod image_ycbcr;


fn main() {

    let mut camera = ffmpeg_camera::FfmpegCamera::get_default().expect("Failed to open camera.");

    let image = camera.get_image().unwrap();

    image.save_jpeg("another.jpg").unwrap();
}
