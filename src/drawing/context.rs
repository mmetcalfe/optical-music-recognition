use drawing::rectangle_buffer::RectangleBuffer;
use drawing::rectangle_buffer::RotatedRectangle;
use drawing::image_pane::ImagePane;

use ffmpeg_camera::image_ycbcr;
use glium;


pub struct DrawingContext<'a> {
    image_pane : ImagePane<'a>,
    rectangle_buffer : RectangleBuffer,
}

impl<'a> DrawingContext<'a> {
    pub fn new(display : &glium::Display) -> DrawingContext {
        DrawingContext {
            image_pane: ImagePane::new(display),
            rectangle_buffer: RectangleBuffer::new(display)
        }
    }

    pub fn draw_rectangle(&self, target : &mut glium::Frame, rect : &RotatedRectangle, colour : [f32; 4]) {
        self.rectangle_buffer.draw_rectangle(target, rect, colour)
    }

    pub fn draw_image(&self, target : &mut glium::Frame, image : &image_ycbcr::Image) {
        self.image_pane.draw_image(target, image)
    }
}
