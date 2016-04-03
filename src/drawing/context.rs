
use drawing::rectangle_buffer::RectangleBuffer;
use drawing::rectangle_buffer::RotatedRectangle;
use drawing::image_pane::ImagePane;

use ffmpeg_camera::image_uyvy;
use ffmpeg_camera::image_ycbcr;
use ffmpeg_camera::image::Image;
use glium;
use nalgebra as na;

use omr::scanning::staff_cross::StaffCross;

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

    pub fn draw_image_ycbcr(&self, target : &mut glium::Frame, image : &image_ycbcr::Image) {
        self.image_pane.draw_image_ycbcr(target, image)
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

    pub fn draw_line_extended(&self, target : &mut glium::Frame, p1 : na::Vec2<f32>, p2 : na::Vec2<f32>, lw : f32, colour : [f32; 4]) {
        let dir = na::normalize(&(p2 - p1));
        let avg = (p1 + p2) / 2.0;

        let e1 = avg + dir * 5.0;
        let e2 = avg - dir * 5.0;

        self.draw_line(target, e1, e2, lw, colour);
    }

    pub fn draw_staff(&self, target: &mut glium::Frame, p1: na::Vec2<f32>, p2: na::Vec2<f32>, sw: f32, lw: f32, colour: [f32; 4]) {
        let dir = na::normalize(&(p2 - p1));
        let norm = na::Vec2::new(dir[1], -dir[0]);

        let line_space = lw + sw;

        for i in -2..3 {
            let d = line_space * (i as f32);

            let e1 = p1 + norm * d;
            let e2 = p2 + norm * d;

            self.draw_line(target, e1, e2, lw, colour);
        }
    }

    pub fn draw_staff_cross(&self, mut target: &mut glium::Frame, ycbcr_frame : &image_ycbcr::Image, cross: &StaffCross, colour: [f32; 4]) {
        let pix_h = 2.0 * (1.0 / ycbcr_frame.height as f32);
        let scan_draw_cols = 1.0;

        let x = cross.x;
        for span in cross.spans() {
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

            self.draw_line(&mut target, p1, p2, pix_h * scan_draw_cols, colour);
            // self.draw_line(&mut target, p1, p2, (p2[1] - p1[1]).abs(), colour);
        }
    }

    pub fn convert_uyvy_ycbcr(&self, uyvy_image : &image_uyvy::Image)
        -> Result<image_ycbcr::Image, glium::framebuffer::ValidationError> {
        self.image_pane.convert_uyvy_ycbcr(uyvy_image)
    }

}
