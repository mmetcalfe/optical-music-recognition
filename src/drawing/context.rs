use drawing::rectangle_buffer::RectangleBuffer;
use drawing::rectangle_buffer::RotatedRectangle;
use drawing::image_pane::ImagePane;

use ffmpeg_camera::image_uyvy;
// use ffmpeg_camera::image::Image;
use glium;
use nalgebra as na;

// use std::f32;

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

    pub fn draw_image_uyvy(&self, target : &mut glium::Frame, image : &image_uyvy::Image) {
        self.image_pane.draw_image_uyvy(target, image)
    }

    pub fn draw_line(&self, target : &mut glium::Frame, p1 : na::Vec2<f32>, p2 : na::Vec2<f32>, lw : f32, colour : [f32; 4]) {
        let x1 = p1[0];
        let y1 = p1[1];
        let x2 = p2[0];
        let y2 = p2[1];

        let xd = x2 - x1;
        let yd = y2 - y1;
        let len = (xd*xd + yd*yd).sqrt();

        let xa = (x1 + x2) / 2.0;
        let ya = (y1 + y2) / 2.0;

        let angle = -f32::atan2(yd, xd);

        let rect = RotatedRectangle {
            position: [xa, ya],
            size: [len, lw],
            angle: angle,
        };
        self.rectangle_buffer.draw_rectangle(target, &rect, colour)
    }
}
