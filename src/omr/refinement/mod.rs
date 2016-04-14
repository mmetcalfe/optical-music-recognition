
use omr::scanning::staff_cross::StaffCross;
use omr::ransac::staff_cross::StaffCrossLine;
use geometry::staff::Staff;
use ffmpeg_camera::image::Image;
use std;

pub fn staff_sample_average<I: Image>(image: &I, staff: &Staff, t: f32, num_samples: usize, sample_sep: f32) -> f32 {
    let mut blank_avg = 0.0;
    for pt in staff.perpendicular_samples(t, num_samples, sample_sep) {
        let brightness = image.sample_point(pt).y as f32 / 255.0;
        blank_avg += brightness.round();
    }
    blank_avg /= num_samples as f32;

    blank_avg
}

#[derive(PartialEq)]
pub enum StaffEvidenceClass {
    // TODO: Improve classes when evidence is present.
    Blank, // Strong evidence for a blank/featureless surface.
    Negative, // No evidence for staff-lines or staff-spaces.
    None, // Weak evidence for staff-lines or staff-spaces.
    Weak, // Weak evidence for staff-lines and staff-spaces.
    Partial, // Good evidence for staff-lines or staff-spaces.
    Strong, // Good evidence for staff-lines and staff-spaces.
}

pub fn classify_staff_sample(line_avg: f32, space_avg: f32, blank_avg: f32) -> StaffEvidenceClass {

    let has_lines = line_avg < 0.5;
    let has_spaces = space_avg > 0.5;
    let has_all_lines = line_avg < 0.1;
    let has_all_spaces = space_avg > 0.9;
    // let is_gap = (space_avg*4.0 + line_avg*5.0) / 9.0 > 0.95;
    let is_gap = blank_avg > 0.95;

    if is_gap {
        // Definitely not a staff.
        return StaffEvidenceClass::Blank;
    } else {
        if has_all_lines && has_all_spaces {
            return StaffEvidenceClass::Strong;
        }

        if has_all_lines || has_all_spaces {
            return StaffEvidenceClass::Partial;
        }

        if has_lines && has_spaces {
            return StaffEvidenceClass::Weak;
        }

        if !has_lines && !has_spaces {
            // Almost definitely not a staff.
            // Probably just text, or a badly misaligned staff due to curvature.
            return StaffEvidenceClass::Negative;
        }
    }

    // println!("is_gap: {:?}", is_gap);
    // println!("has_lines: {:?}", has_lines);
    // println!("has_spaces: {:?}", has_spaces);
    // println!("has_all_lines: {:?}", has_all_lines);
    // println!("has_all_spaces: {:?}", has_all_spaces);
    // unimplemented!();

    StaffEvidenceClass::None
}

pub fn partition_staff<I: Image>(image: &I, staff: &Staff) -> Vec<Staff> {
    let (t_min, t_max) = staff.screen_entry_exit_times(image.width() as f32, image.height() as f32);

    let staff_height = staff.line_width * 5.0 + staff.space_width * 4.0;
    let step_size = 2.0;
    // let gap_samples = std::cmp::max(2, ((staff_height * 0.5) / step_size).round() as usize);

    // let mut time_spans = Vec::new();
    let mut span_start = t_min;


    let mut t = t_min;
    while t + step_size < t_max {
        t += step_size;

        let mut line_avg = staff_sample_average(image, staff, t, 5, staff.line_sep());
        let mut space_avg = staff_sample_average(image, staff, t, 4, staff.line_sep());

        // Blank spaces:
        let blank_samples = 20;
        let blank_sep = 1.2 * staff.line_sep() * 2.0 / (blank_samples as f32 * 0.5);
        let blank_avg = staff_sample_average(image, staff, t, blank_samples, blank_sep);

        let class = classify_staff_sample(line_avg, space_avg, blank_avg);

    }

    Vec::new()
}

// pub fn refine_detected_staff(line: &StaffCrossLine, inliers: &Vec<StaffCross>) -> bool {
//     true
// }
