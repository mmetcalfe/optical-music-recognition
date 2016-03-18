
pub mod ffmpeg_camera;
pub mod image_ycbcr;


fn main() {

    let mut camera = ffmpeg_camera::FfmpegCamera::get_default().expect("Failed to open camera.");

    camera.get_image().unwrap();
    camera.get_image().unwrap();
    camera.get_image().unwrap();
    let image = camera.get_image().unwrap();

    unsafe {
        image.save_jpeg("another.jpg");
    }
}
