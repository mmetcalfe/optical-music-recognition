
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

#[derive(PartialEq, Clone, Copy)]
pub enum StaffEvidenceClass {
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

        if (has_all_lines && has_spaces) || (has_lines && has_all_spaces) {
            return StaffEvidenceClass::Partial;
        }

        if has_lines && has_spaces {
            return StaffEvidenceClass::Weak;
        }

        if !has_lines && !has_spaces {
            // Almost definitely not a staff. Probably just text, music notation superimposed on a
            // staff, or a badly misaligned staff due to curvature.
            return StaffEvidenceClass::Negative;
        }
    }

    // println!("is_gap: {:?}", is_gap);
    // println!("has_lines: {:?}", has_lines);
    // println!("has_spaces: {:?}", has_spaces);
    // println!("has_all_lines: {:?}", has_all_lines);
    // println!("has_all_spaces: {:?}", has_all_spaces);
    // unimplemented!();

    // There's something here, but there's no evidence to prove or disprove that it's a staff.
    StaffEvidenceClass::None
}

pub fn partition_staff<I: Image>(image: &I, staff: &Staff) -> (Vec<Staff>, Vec<Staff>) {

    // TODO: Replace all of this with a more generic classifier.

    let mut sample_classes = Vec::new();

    // Classify samples:
    let step_size = 2.0;
    let (t_min, t_max) = staff.screen_entry_exit_times(image.width() as f32, image.height() as f32);
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

        sample_classes.push(class);
    }

    // Use class information to partition samples:
    let mut sample_blank_spans = Vec::new();
    let mut sample_staff_spans = Vec::new();
    let staff_height = staff.line_width * 5.0 + staff.space_width * 4.0;
    let min_gap_width = staff_height;
    let min_gap_samples = std::cmp::max(2, ((min_gap_width) / step_size).round() as usize);
    let min_span_samples = min_gap_samples;
    let max_weak_gap_samples = min_gap_samples;
    let max_weak_span_samples = min_gap_samples;

    let mut gap_length = 0;
    let mut gap_start : isize = -1;
    let mut span_start : isize = -1;
    let mut weak_count = 0;
    let num_samples = sample_classes.len();
    for (i, class) in sample_classes.iter().enumerate() {
        let class = *class;
        if gap_start < 0 {
            let mut start_gap = false;
            if class == StaffEvidenceClass::Blank {
                start_gap = true;
            } else
            if // class == StaffEvidenceClass::None ||
                class == StaffEvidenceClass::Negative {
                weak_count += 1;
                if weak_count > max_weak_span_samples {
                    start_gap = true;
                }
            }

            if start_gap {
                gap_start = i as isize;
                weak_count = 0;

                let span_end = (i - 1) as isize;
                if span_end - span_start >= min_span_samples as isize {
                    sample_staff_spans.push((span_start as usize, span_end as usize));
                }
            }
        } else {
            let mut end_gap = false;

            if class == StaffEvidenceClass::Weak {
                weak_count += 1;
                if weak_count > max_weak_gap_samples {
                    end_gap = true;
                }
            } else
            if !(class == StaffEvidenceClass::Negative ||
                class == StaffEvidenceClass::Blank ||
                class == StaffEvidenceClass::None) {
                    end_gap = true;
                    weak_count = 0;
            }

            if end_gap || i == (num_samples - 1) {
                let gap_end = (i - 1) as isize;
                if gap_end - gap_start >= min_gap_samples as isize {
                    sample_blank_spans.push((gap_start as usize, gap_end as usize));
                }
                weak_count = 0;
                gap_start = -1;
                span_start = i as isize;
            }
        }
    }

    // Use sample spans to generate staff segments:
    let convert_sample_spans_to_segments = |sample_spans : Vec<(usize, usize)>| {
        let mut staff_segments = Vec::new();
        for (start, end) in sample_spans {
            let t_start = t_min + (1 + start) as f32 * step_size;
            let t_end = t_min + (1 + end) as f32 * step_size;

            let p_start = staff.point_at_time(t_start);
            let p_end = staff.point_at_time(t_end);

            let segment = Staff::new(p_start, p_end, staff.line_width, staff.space_width);
            staff_segments.push(segment);
        }
        staff_segments
    };

    let blank_segments = convert_sample_spans_to_segments(sample_blank_spans);
    let staff_segments = convert_sample_spans_to_segments(sample_staff_spans);

    (staff_segments, blank_segments)
}

pub fn staff_segment_is_valid<I: Image>(image: &I, staff: &Staff) -> bool {
    true
}
