extern crate core;

use std::cell::RefCell;

use drawing::rectangle_buffer::RectangleBuffer;
use geometry::RotatedRectangle;
use drawing::image_pane::ImagePane;
use drawing::text_helper::TextHelper;

use ffmpeg_camera::image_uyvy;
use ffmpeg_camera::image_ycbcr;
use ffmpeg_camera::Image;
use ffmpeg_camera::ToTexture;
use glium;
use nalgebra as na;
use nalgebra::Transpose;
use nalgebra::Inverse;

use detection::scanning::staff_cross::StaffCross;
use detection::ransac::staff_cross::StaffCrossLine;
use detection::ransac::RansacState;
use geometry as gm;

use utility;

use self::core::ops::Mul;
use self::core::convert::From;

// use std::f32;

/// Manages all drawing operations.
pub struct DrawingContext<'a> {
    image_pane : ImagePane<'a>,
    rectangle_buffer : RectangleBuffer,
    text_helper : TextHelper,
    window_dims: (usize, usize),
}

/// Provides a coordinate system in which to draw.
pub struct DrawingFrame<'a> {
    /// Context to use for drawing.
    // ctx: &'a mut DrawingContext<'a>,
    ctx: &'a RefCell<DrawingContext<'a>>,

    /// Position and size of the frame within the base frame.
    pub rect: gm::RotatedRectangle,

    /// Virtual pixel dimensions of the frame. This determines the frame's scale.
    pub frame_dims: na::Vector2<f32>,

    /// Displacement in the frame's viewport transform.
    pub frame_centre: na::Vector2<f32>,
}

impl<'a> DrawingContext<'a> {
    pub fn new(display : &glium::Display) -> DrawingContext {
        DrawingContext {
            image_pane: ImagePane::new(display),
            rectangle_buffer: RectangleBuffer::new(display.clone()),
            text_helper: TextHelper::new(display),
            window_dims: (100, 100),
        }
    }

    pub fn set_window_dims(&mut self, dims: (usize, usize)) {
        self.window_dims = dims;
    }

    pub fn get_default_frame(self_: &'a RefCell<Self>) -> DrawingFrame<'a> {
        DrawingFrame::<'a> {
            ctx: self_,
            rect: gm::RotatedRectangle {
                position: [0.0, 0.0],
                size: [2.0, 2.0],
                angle: 0.0,
            },
            frame_dims: na::Vector2::new(2.0, 2.0),
            frame_centre: na::Vector2::new(0.0, 0.0),
        }
    }

    pub fn get_conversion_frame(self_: &'a RefCell<Self>) -> DrawingFrame<'a> {
        DrawingFrame::<'a> {
            ctx: self_,
            rect: gm::RotatedRectangle {
                position: [0.0, 0.0],
                size: [2.0, 2.0],
                angle: 0.0,
            },
            frame_dims: na::Vector2::new(2.0, 2.0),
            frame_centre: na::Vector2::new(-1.0, -1.0),
        }
    }
}

impl<'a> DrawingFrame<'a> {

    pub fn convert_uyvy_ycbcr(&self, uyvy_image : &image_uyvy::Image)
        -> Result<image_ycbcr::Image, glium::framebuffer::ValidationError> {
        self.set_view_matrices();
        self.ctx.borrow_mut().image_pane.convert_uyvy_ycbcr(uyvy_image)
    }

    pub fn convert_preprocess_uyvy_ycbcr(&self, uyvy_image : &image_uyvy::Image)
        -> Result<image_ycbcr::Image, glium::framebuffer::ValidationError> {
        self.set_view_matrices();
        self.ctx.borrow_mut().image_pane.convert_preprocess_uyvy_ycbcr(uyvy_image)
    }

    fn make_scale_transform(&self) -> [[f32; 4]; 4] {
        // let (width, height) = self.frame_dims;
        // let xs = 2.0 / self.frame_dims[0];
        // let ys = 2.0 / self.frame_dims[1];
        let xs = 1.0 / self.frame_dims[0];
        let ys = 1.0 / self.frame_dims[1];
        let scale_transform = [
            [xs, 0.0, 0.0, 0.0],
            [0.0, ys, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0f32],
        ];

        scale_transform
    }

    fn make_viewport_transform(&self) -> [[f32; 4]; 4] {
        let xc = self.frame_centre[0];
        let yc = self.frame_centre[1];
        [
            [2.0, 0.0, 0.0, 0.0],
            [0.0, 2.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            // [0.0, 0.0, 0.0, 1.0f32],
            [xc, yc, 0.0, 1.0f32],
        ]
    }

    pub fn local_to_parent(&self, point: &na::Vector2<f32>) -> na::Vector2<f32> {
        let frame_transform = self.rect.get_transform();
        let viewport_transform = self.make_viewport_transform();
        let scale_transform = self.make_scale_transform();

        let frame_mat = na::Matrix4::<f32>::from(&frame_transform);
        let vp_mat = na::Matrix4::<f32>::from(&viewport_transform);
        let scale_mat = na::Matrix4::<f32>::from(&scale_transform);
        let trans_mat = frame_mat.mul(vp_mat.mul(scale_mat));

        let point_4 = na::Vector4::<f32>::new(point[0], point[1], 0.0, 1.0);

        let trans_4 = trans_mat.mul(point_4);

        // na::Vector2::<f32>::new(1.0 + trans_4[0], 1.0 - trans_4[1])
        // na::Vector2::<f32>::new(1.0 + trans_4[0], 1.0 + trans_4[1])
        na::Vector2::<f32>::new(trans_4[0], trans_4[1])
    }

    pub fn draw_string(&self, target: &mut glium::Frame, string: &str, pos: na::Vector2<f32>, scale: f32, colour: (f32, f32, f32, f32)) {
        self.set_view_matrices();
        self.ctx.borrow_mut().text_helper.draw_string(target, string, pos, scale, colour)
    }

    pub fn make_view_matrix(&self) -> na::Matrix4<f32> {
        // xo, yo = (0, 0)
        // w, h = framebufferSize
        // vpMat = np.matrix([
        // [w/2.0, 0, 0, w/2.0 + xo],
        // [0, h/2.0, 0, h/2.0 + yo],
        // [0, 0, 1, 0],
        // [0, 0, 0, 1]
        // ], np.float32)

        let frame_transform = self.rect.get_transform();
        let viewport_transform = self.make_viewport_transform();
        let scale_transform = self.make_scale_transform();

        let frame_mat = na::Matrix4::<f32>::from(&frame_transform);
        let vp_mat = na::Matrix4::<f32>::from(&viewport_transform);
        let scale_mat = na::Matrix4::<f32>::from(&scale_transform);
        let trans_mat = frame_mat.mul(vp_mat.mul(scale_mat));

        trans_mat
    }

    pub fn set_view_matrices(&self) {
        let trans_mat = self.make_view_matrix();

        let mut ctx = self.ctx.borrow_mut();
        ctx.rectangle_buffer.set_view_matrix(trans_mat.as_ref());
        // ctx.image_pane.set_view_matrix(frame_mat.as_ref());
        ctx.image_pane.set_view_matrix(trans_mat.as_ref());
    }

    pub fn draw_rectangle(&self, target : &mut glium::Frame, rect : &RotatedRectangle, colour : [f32; 4]) {
        self.set_view_matrices();
        self.ctx.borrow_mut().rectangle_buffer.draw_rectangle(target, rect, colour)
    }

    // pub fn draw_image_uyvy(&self, target : &mut glium::Frame, image : &image_uyvy::Image) {
    //     self.set_view_matrices();
    //     self.ctx.borrow_mut().image_pane.draw_image_uyvy(target, image)
    // }

    // pub fn draw_image_ycbcr(&self, target : &mut glium::Frame, image : &image_ycbcr::Image) {
    //     self.set_view_matrices();
    //     self.ctx.borrow_mut().image_pane.draw_image_ycbcr(target, image)
    // }

    pub fn draw_image<I: Image + ToTexture>(&self, target : &mut glium::Frame, image : &I) {
        self.set_view_matrices();
        self.ctx.borrow_mut().image_pane.draw_image(target, image)
    }

    pub fn draw_image_homog<I: Image + ToTexture>(&self,
        target : &mut glium::Frame,
        image : &I,
        reference: &Self,
        homog: &na::Matrix3<f32>) {
        self.set_view_matrices();
        let reference_scale = na::Matrix4::<f32>::from(&reference.make_scale_transform());
        let view = reference.make_view_matrix();
        // self.ctx.borrow_mut().image_pane.draw_image_ycbcr(target, image)
        self.ctx.borrow_mut().image_pane.draw_image_homog(
            target,
            image,
            &view,
            &reference_scale,
            homog
        )
    }

    pub fn draw_point(&self, target : &mut glium::Frame, pt : na::Vector2<f32>, lw : f32, colour : [f32; 4]) {
        self.set_view_matrices();
        let rect = RotatedRectangle {
            position: [pt[0], pt[1]],
            size: [lw, lw],
            angle: 0.0,
        };
        self.ctx.borrow_mut().rectangle_buffer.draw_rectangle(target, &rect, colour)
    }

    pub fn draw_line(&self, target : &mut glium::Frame, p1 : na::Vector2<f32>, p2 : na::Vector2<f32>, lw : f32, colour : [f32; 4]) {
        self.set_view_matrices();
        let rect = gm::RotatedRectangle::from_line(&gm::Line::new(p1, p2), lw);
        self.ctx.borrow_mut().rectangle_buffer.draw_rectangle(target, &rect, colour)
    }

    pub fn draw_lines(&self, target : &mut glium::Frame, lines: &[gm::Line], lw : f32, colour : [f32; 4]) {
        self.set_view_matrices();
        let rects : Vec<_> = lines.iter().map(|l| gm::RotatedRectangle::from_line(&l, lw)).collect();

        self.ctx.borrow_mut().rectangle_buffer.draw_rectangles(target, &rects, colour)
    }

    pub fn draw_line_extended(&self, target : &mut glium::Frame, p1 : na::Vector2<f32>, p2 : na::Vector2<f32>, lw : f32, colour : [f32; 4]) {
        let dir = na::normalize(&(p2 - p1));
        let avg = (p1 + p2) / 2.0;

        let e1 = avg + dir * 10000.0;
        let e2 = avg - dir * 10000.0;

        self.draw_line(target, e1, e2, lw, colour);
    }

    pub fn draw_staff_from_parts(&self, target: &mut glium::Frame, p1: na::Vector2<f32>, p2: na::Vector2<f32>, sw: f32, lw: f32, colour: [f32; 4]) {
        self.set_view_matrices();
        let dir = na::normalize(&(p2 - p1));
        let norm = na::Vector2::new(dir[1], -dir[0]);

        let line_space = lw + sw;

        // let mut lines = Vec::<gm::Line>::new();
        for i in -2..3 {
            let d = line_space * (i as f32);

            let e1 = p1 + norm * d;
            let e2 = p2 + norm * d;

            self.draw_line(target, e1, e2, lw, colour);
            // lines.push(gm::Line::new(e1, e2));
        }

        // self.draw_lines(target, &lines, lw, colour);
    }

    pub fn draw_staff_in_image<I: Image>(&self, target: &mut glium::Frame, ycbcr_frame : &I, staff: &gm::staff::Staff, colour: [f32; 4]) {
        self.set_view_matrices();
        let start_pt = staff.point_at_time(0.0);
        let end_pt = staff.point_at_time(staff.length);
        let p1 = ycbcr_frame.opengl_coords_for_point(start_pt);
        let p2 = ycbcr_frame.opengl_coords_for_point(end_pt);
        self.draw_staff_from_parts(
            target,
            p1,
            p2,
            staff.space_width,
            staff.line_width,
            colour
        )
    }

    pub fn draw_staff_cross(&self, mut target: &mut glium::Frame, ycbcr_frame : &image_ycbcr::Image, cross: &StaffCross, colour: [f32; 4]) {
        self.set_view_matrices();
        let pix_h = 1.0; // 2.0 * (1.0 / ycbcr_frame.height as f32);
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

    pub fn draw_staff_crosses<I: Image>(&self, mut target: &mut glium::Frame, ycbcr_frame : &I, crosses: &[StaffCross], colour: [f32; 4]) {
        self.set_view_matrices();
        let pix_h = 1.0;
        let lw = 1.0;

        let mut lines = Vec::<gm::Line>::new();

        for cross in crosses {
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

                lines.push(gm::Line::new(p1, p2));
            }
        }

        self.draw_lines(target, &lines, lw, colour);
    }

    pub fn draw_ransac_state<I: Image>(
        &self,
        mut target: &mut glium::Frame,
        ycbcr_frame : &I,
        state: &RansacState<StaffCrossLine, StaffCross>) {
            self.set_view_matrices();

        if let Some(ref line) = state.model {
            let line = gm::Line::new(line.a.centre(), line.b.centre());

            // Draw inliers:
            let inliers_col = [1.0, 0.0, 0.0, 1.0];
            // for cross in state.inliers.iter() {
            //     self.draw_staff_cross(&mut target, &ycbcr_frame, &cross, inliers_col);
            // }
            self.draw_staff_crosses(&mut target, ycbcr_frame, &state.inliers, inliers_col);

            // Draw staff lines:
            let mut space_width_sum = 0.0;
            for pt in &state.inliers {
                let pt_space_width = pt.average_space_width(&line);
                space_width_sum += pt_space_width
            }
            let avg_space_width = space_width_sum / state.inliers.len() as f32;

            let mut line_width_sum = 0.0;
            for pt in &state.inliers {
                let pt_line_width = pt.average_line_width(&line);
                line_width_sum += pt_line_width
            }
            let avg_line_width = line_width_sum / state.inliers.len() as f32;

            // let pix_h = 1.0; //2.0 * (1.0 / ycbcr_frame.height as f32);
            let p1 = ycbcr_frame.opengl_coords_for_point(line.a);
            let p2 = ycbcr_frame.opengl_coords_for_point(line.b);

            let staff_col = [0.4, 0.6, 0.4, 1.0];
            self.draw_staff_from_parts(&mut target, p1, p2,
                avg_space_width,
                avg_line_width,
                staff_col
            );

            // Draw centre line:
            let lw = 1.0;
            let col_ext = [0.0, 1.0, 0.0, 1.0];
            self.draw_line_extended(&mut target, p1, p2, lw, col_ext);
            let col = [0.0, 0.0, 1.0, 1.0];
            self.draw_line(&mut target, p1, p2, lw, col);
        }
    }
}
