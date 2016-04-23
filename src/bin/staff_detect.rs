#[macro_use]
extern crate glium;
extern crate nalgebra as na;
extern crate time;
// extern crate image;

// use std::f32;

extern crate optical_music_recognition as omr;
use omr::ffmpeg_camera::ffmpeg_camera;
use omr::ffmpeg_camera::image::Image;
use omr::drawing;
use omr::math;
use omr::geometry;
use omr::geometry as gm;
use omr::detection::ransac::staff_cross::StaffCrossLineModel;
use omr::detection::scanning::staff_cross::StaffCross;

// use std::io::Cursor;
use glium::DisplayBuild;
use glium::Surface;
use std::cmp;

use time::Duration;
use time::SteadyTime;

use std::cell::RefCell;

fn main() {
    // let mut camera =
    //     ffmpeg_camera::FfmpegCamera::get_default()
    //         .expect("Failed to open camera.");


    // let (img_w, img_h) = (320, 240);
    // let (img_w, img_h) = (640, 480);
    // let (img_w, img_h) = (1280, 720);
    let (img_w, img_h) = (1920, 1080);

    let mut camera = ffmpeg_camera::FfmpegCamera::get_best((img_w, img_h))
        .expect("Failed to open camera.");

    let display = glium::glutin::WindowBuilder::new()
        // .with_dimensions(1280, 720)
        .with_dimensions(img_w as u32*4, img_h as u32*4)
        .with_title(format!("OMR"))
        .build_glium()
        .unwrap();

    // let image = image::load(Cursor::new(&include_bytes!("../../curved-3.jpg")[..]),
    //                         image::JPEG).unwrap().to_rgba();
    // let image_dimensions = image.dimensions();
    // let image = glium::texture::RawImage2d::from_raw_rgba_reversed(image.into_raw(), image_dimensions);
    // let texture = glium::texture::Texture2d::new(&display, image).unwrap();

    // let mut draw_ctx = drawing::context::DrawingContext::new(&display);
    // draw_ctx.set_view_matrices(img_w, img_h);
    // let mut draw_ctx = drawing::context::DrawingContext::new(&display);
    // draw_ctx.set_window_dims((img_w, img_h));
    // let mut draw_frame = draw_ctx.get_default_frame();
    let window_dims = (img_w, img_h);
    let mut draw_ctx = RefCell::new(drawing::context::DrawingContext::new(&display));
    draw_ctx.borrow_mut().set_window_dims(window_dims);
    let mut draw_frame = drawing::context::DrawingContext::get_default_frame(&draw_ctx);
    draw_frame.set_view_matrices();


    // let img_pane = drawing::image_pane::ImagePane::new(&display);
    // let rect_buff = drawing::rectangle_buffer::RectangleBuffer::new(&display);

    let mut frame_start_time = SteadyTime::now();
    loop {

        let webcam_frame = camera.get_image_uyvy().unwrap();
        // let webcam_frame = camera.get_image_ycbcr().unwrap();

        // webcam_frame.save_pgm("image.pgm").unwrap();
        // webcam_frame.save_jpeg("image_yuv422.jpg").unwrap();

        let ycbcr_frame = draw_ctx.borrow_mut().convert_preprocess_uyvy_ycbcr(&webcam_frame).unwrap();

        // ycbcr_frame.save_jpeg("image_ycbcr.jpg").unwrap();

        let mut target = display.draw();
        target.clear_color(0.0, 0.0, 1.0, 1.0);

        // draw_frame.draw_image_uyvy(&mut target, &webcam_frame);
        draw_frame.draw_image_ycbcr(&mut target, &ycbcr_frame);

        let staff_cross_col = [0.6, 0.2, 0.0, 1.0];

        let fitting_start_time = SteadyTime::now();

        // Scan entire image for StaffCross points:
        let num_scan_lines = std::cmp::min(640, img_w / 2);
        let cross_points = omr::detection::scanning::staff_cross::scan_entire_image(&ycbcr_frame, num_scan_lines);

        // Draw detected StaffCross points:
        // for cross in &cross_points {
        //     draw_frame.draw_staff_cross(&mut target, &ycbcr_frame, &cross, staff_cross_col);
        // }
        draw_frame.draw_staff_crosses(&mut target, &ycbcr_frame, &cross_points, staff_cross_col);


        // // Draw segments:
        // let segments = omr::detection::scanning::segment::scan_entire_image(&ycbcr_frame, num_scan_lines);
        // // let x = 256;
        // for segment in segments.iter() {
        //     // let col = [(i*71 % 255) as f32 / 255.0, 0.5 * (i*333 % 255) as f32 / 255.0, 0.0, 0.0];
        //     let col = [1.0, 0.0, 0.0, 1.0];
        //     let x = segment.x;
        //
        //     let pix_h = 2.0 * (1.0 / ycbcr_frame.width as f32);
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
        //     // draw_frame.draw_line(&mut target, p1, p2, pix_h * 5.0, col);
        //     draw_frame.draw_line(&mut target, p1, p2, pix_h, col);
        // }

        let num_iterations = omr::detection::ransac::calculate_num_iterations(
            cross_points.len(), // num_points
            cross_points.len() / 20, // num_inliers
            2, // points_per_model
            0.75 // success_probability
        );
        // println!("cross_points.len(): {:?}", cross_points.len());
        // println!("num_iterations: {:?}", num_iterations);

        // Run RANSAC on the StaffCross points to find a line:
        let params = omr::detection::ransac::RansacParams {
            num_iterations: num_iterations,
            max_distance: 111.0, // not currently used
            min_inliers: 15,
        };
        // let maybe_line = omr::detection::ransac::ransac::<StaffCrossLineModel,_,_>(params, &cross_points);
        // let state = omr::detection::ransac::ransac::<StaffCrossLineModel,_,_>(&params, &cross_points);
        // draw_frame.draw_ransac_state(&mut target, &ycbcr_frame, &state);

        let states = omr::detection::ransac::ransac_multiple::<StaffCrossLineModel,_,_>(&params, &cross_points);
        for state in &states {
            // draw_frame.draw_ransac_state(&mut target, &ycbcr_frame, &state);

            let centres : Vec<na::Vector2<f32>> = state.inliers.iter()
                // .take(5)
                .map(|c| c.centre()).collect();

            let best_line = math::fit_line(&centres);
            let p1 = ycbcr_frame.opengl_coords_for_point(best_line.a);
            let p2 = ycbcr_frame.opengl_coords_for_point(best_line.b);
            // draw_frame.draw_line_extended(&mut target, p1, p2, 3.0, [0.3, 0.3, 0.1, 1.0]);

            let mut is_staff = false;
            if let Some(ref detected_line) = state.model {
                // is_staff = omr::detection::refinement::refine_detected_staff(&detected_line, &state.inliers);

                // detected_line
                // best_line
                let inliers = &state.inliers;

                let (t_min, t_max) = best_line.screen_entry_exit_times(ycbcr_frame.width as f32, ycbcr_frame.height as f32);
                // let p_min = ycbcr_frame.opengl_coords_for_point(best_line.point_at_time(t_min));
                // let p_max = ycbcr_frame.opengl_coords_for_point(best_line.point_at_time(t_max));
                // draw_frame.draw_line(&mut target, p_min, p_min*0.9+p_max*0.1, 10.0, [1.0, 1.0, 0.5, 1.0]);
                // draw_frame.draw_line(&mut target, p_max, p_min*0.1+p_max*0.9, 10.0, [1.0, 1.0, 0.5, 1.0]);

                let normal = best_line.normal();

                let avg_space_width = inliers.iter().fold(0.0, |sum, pt| sum + pt.average_space_width(&best_line)) / inliers.len() as f32;
                let avg_line_width = inliers.iter().fold(0.0, |sum, pt| sum + pt.average_line_width(&best_line)) / inliers.len() as f32;

                let line_sep = avg_space_width + avg_line_width;

                let staff = geometry::staff::Staff::new(
                    best_line.a,
                    best_line.b,
                    avg_line_width,
                    avg_space_width
                );

                // let num = 100;
                // for i in 0..num {
                let mut t = t_min;
                let step_size = staff.line_sep() * 0.5;
                while t + step_size < t_max {
                    t += step_size;

                    // Sample lines:
                    let mut line_avg = 0.0;
                    for pt in staff.perpendicular_samples(t, 5, line_sep) {
                        let brightness = ycbcr_frame.sample_point(pt).y as f32 / 255.0;
                        line_avg += brightness.round();

                        let draw_pt = ycbcr_frame.opengl_coords_for_point(pt);

                        let colour = if brightness > 0.5 {[0.0, 0.5, 0.0, 1.0]} else {[0.0, 0.0, 0.5, 1.0]};
                        // draw_frame.draw_point(&mut target, draw_pt, 1.0, colour);
                    }
                    line_avg /= 5.0;

                    // Sample spaces:
                    let mut space_avg = 0.0;
                    for pt in staff.perpendicular_samples(t, 4, line_sep) {
                        let brightness = ycbcr_frame.sample_point(pt).y as f32 / 255.0;
                        space_avg += brightness.round();

                        let draw_pt = ycbcr_frame.opengl_coords_for_point(pt);
                        // draw_frame.draw_point(&mut target, draw_pt, 1.0, [0.8, 0.0, 1.0, 1.0]);
                    }
                    space_avg /= 4.0;

                    // Blank spaces:
                    let num_samples = 20;
                    let sample_sep = 1.2 * line_sep * 2.0 / (num_samples as f32 * 0.5);
                    let mut blank_avg = 0.0;
                    for pt in staff.perpendicular_samples(t, num_samples, sample_sep) {
                        let brightness = ycbcr_frame.sample_point(pt).y as f32 / 255.0;
                        blank_avg += brightness.round();
                        let draw_pt = ycbcr_frame.opengl_coords_for_point(pt);
                        // draw_frame.draw_point(&mut target, draw_pt, 1.0, [0.2, 0.2, 0.2, 1.0]);
                    }
                    blank_avg /= num_samples as f32;

                    let class = omr::detection::refinement::classify_staff_sample(line_avg, space_avg, blank_avg);

                    let p_t = best_line.point_at_time(t);
                    let draw_pt1 = ycbcr_frame.opengl_coords_for_point(p_t+normal*line_sep*2.0);
                    let draw_pt2 = ycbcr_frame.opengl_coords_for_point(p_t-normal*line_sep*2.0);

                    if class == omr::detection::refinement::StaffEvidenceClass::Blank {
                        // draw_frame.draw_line(&mut target, draw_pt1, draw_pt2, 1.0, [0.0, 0.0, 0.0, 1.0]);
                    }
                    if class == omr::detection::refinement::StaffEvidenceClass::Strong {
                        // draw_frame.draw_line(&mut target, draw_pt1, draw_pt2, 1.0, [1.0, 0.3, 0.0, 1.0]);
                    }
                    if class == omr::detection::refinement::StaffEvidenceClass::Partial {
                        // draw_frame.draw_line(&mut target, draw_pt1, draw_pt2, 1.0, [1.0, 0.8, 0.0, 1.0]);
                    }
                    if class == omr::detection::refinement::StaffEvidenceClass::Weak {
                        // draw_frame.draw_line(&mut target, draw_pt1, draw_pt2, 1.0, [0.3, 0.6, 0.0, 1.0]);
                    }
                    if class == omr::detection::refinement::StaffEvidenceClass::Negative {
                        // draw_frame.draw_line(&mut target, draw_pt1, draw_pt2, 1.0, [0.0, 0.0, 1.0, 1.0]);
                    }
                    // if class == omr::detection::refinement::StaffEvidenceClass::None {
                    //     draw_frame.draw_line(&mut target, draw_pt1, draw_pt2, 1.0, [0.0, 0.5, 1.0, 1.0]);
                    // }
                }

                let (candidate_segments, blank_segments) = omr::detection::refinement::partition_staff(&ycbcr_frame, &staff);
                for part in &candidate_segments {
                    let staff_pt1 = part.point_at_time(0.0);
                    let staff_pt2 = part.point_at_time(part.length);
                    let draw_pt1 = ycbcr_frame.opengl_coords_for_point(staff_pt1);
                    let draw_pt2 = ycbcr_frame.opengl_coords_for_point(staff_pt2);
                    draw_frame.draw_line(&mut target, draw_pt1, draw_pt2, 1.0, [1.0, 1.0, 1.0, 1.0]);
                }
                for part in blank_segments {
                    let staff_pt1 = part.point_at_time(0.0);
                    let staff_pt2 = part.point_at_time(part.length);
                    let draw_pt1 = ycbcr_frame.opengl_coords_for_point(staff_pt1);
                    let draw_pt2 = ycbcr_frame.opengl_coords_for_point(staff_pt2);
                    draw_frame.draw_line(&mut target, draw_pt1, draw_pt2, 1.0, [0.0, 0.0, 0.0, 1.0]);
                }

                let staff_segments = candidate_segments.iter()
                    .filter(|segment| omr::detection::refinement::staff_segment_is_valid(&ycbcr_frame, &segment));
                for segment in staff_segments {
                    draw_frame.draw_staff_in_image(&mut target, &ycbcr_frame, &segment, [0.8, 0.3, 1.0, 1.0]);
                }
            }
        }

        let frame_duration = SteadyTime::now() - frame_start_time;
        let fitting_duration = SteadyTime::now() - fitting_start_time;
        frame_start_time = SteadyTime::now();
        let mspf = frame_duration.num_milliseconds();
        let fps = 1000.0 / (mspf as f32);
        let time_str = format!("{} ms/frame, {} fps", mspf, fps);
        draw_frame.draw_string(
            &mut target,
            &time_str,
            na::Vector2::<f32>::new(-1.0, -1.0),
            0.03,
            (0.0,0.0,0.0,1.0)
        );

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
