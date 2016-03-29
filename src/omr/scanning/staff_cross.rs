extern crate core;
// use self::core::slice::Iter;

use ffmpeg_camera::image_ycbcr;
use nalgebra as na;
use omr::scanning::segment;
use std::collections::LinkedList;

// Represents a the intersection of a straight line with a set of 5 staff lines.
// Stores the coordinates of each of the 5 line intersections in image coordinates.
#[derive(Debug, Clone)]
pub struct StaffCross {
    // Note: Currently, to keep things simple, we only scan downward, so we store each span of ink
    // as the starting and ending y-coordinate. (later we'll either store start+stop points, or add
    // a transform from SE(2) to represent the scan coorinate system)

    pub x : usize,
    spans : Vec<[usize; 2]>,
}

impl StaffCross {

    fn empty() -> StaffCross {
        StaffCross {
            x: 0,
            spans: Vec::new(),
        }
    }

    pub fn centre(&self) -> na::Vec2<f32> {
        let sum_y : f32 = self.spans.iter().fold(0.0, |a, s| a + (s[0] + s[1]) as f32 / 2.0);
        let avg_y = sum_y / self.spans.len() as f32;
        let avg_x = self.x as f32;

        na::Vec2::new(avg_x, avg_y)
    }

    fn add_segment(&mut self, segment: &segment::Segment) {
        self.x = segment.x;
        self.spans.push([segment.y_min, segment.y_max])
    }

    // fn is_complete(&self) -> bool {
    //     self.spans.len() >= 5
    // }

    pub fn spans(&self) -> self::core::slice::Iter<[usize; 2]> {
        self.spans.iter()
    }

    // Returns whether the sequence of spans is regular enough that it could plausibly be a
    // cross-section of a staff.
    pub fn is_plausible(&self) -> bool {
        let mut len_sum = 0;
        let mut gap_sum = 0;
        let mut last_end = 0;
        for span in self.spans() {
            if last_end > 0 {
                gap_sum += span[0] - last_end;
            }

            len_sum += span[1] - span[0];

            last_end = span[1];
        }

        let avg_gap = gap_sum as f32 / 4.0;
        let avg_len = len_sum as f32 / 5.0;

        // If stafflines are thicker than the spaces between them.
        if avg_len > avg_gap {
            return false;
        }

        let mut last_end = 0;
        for span in self.spans() {
            if last_end > 0 {
                let gap_len = span[0] - last_end;
                let gap_rel_err = (gap_len as f32 - avg_gap).abs() / avg_gap;
                if gap_rel_err > 0.5 {
                    return false;
                }
            }

            let curr_len = span[1] - span[0];
            let len_rel_err = (curr_len as f32 - avg_len).abs() / avg_len;
            if len_rel_err > 0.5 {
                return false;
            }

            last_end = span[1];
        }

        true
    }
}

// Scans across an image, returning a sequence of detected StaffPoints.
pub struct StaffScanner<'a> {
    // image: &'a image_ycbcr::Image,
    segment_scanner: segment::SegmentScanner<'a>,
    segment_queue: LinkedList<segment::Segment>,
}

impl<'a> StaffScanner<'a> {
    pub fn new(image : &'a image_ycbcr::Image, start_point : [usize; 2]) -> StaffScanner {
        StaffScanner {
            // image: image,
            segment_scanner: segment::SegmentScanner::new(image, start_point),
            segment_queue: LinkedList::new(),
        }
    }
}

impl<'a> Iterator for StaffScanner<'a> {
    type Item = StaffCross;

    fn next(&mut self) -> Option<StaffCross> {

        loop {
            let maybe_segment = self.segment_scanner.next();
            if let Some(segment) = maybe_segment {
                self.segment_queue.push_back(segment);
                if self.segment_queue.len() > 5 {
                    self.segment_queue.pop_front();
                }

                if self.segment_queue.len() == 5 {
                    let mut staff_cross = StaffCross::empty();
                    for segment in &self.segment_queue {
                        staff_cross.add_segment(&segment);
                    }

                    return Some(staff_cross)
                }
            } else {
                return None
            }
        }
    }
}

pub fn scan_entire_image(image: &image_ycbcr::Image, step: usize) -> Vec<StaffCross> {
    let mut results = Vec::new();

    for x in (0..image.width).filter(|x| x % step == 0) {
        let scanner = StaffScanner::new(&image, [x, 0]);

        let crosses = scanner.filter(|c| c.is_plausible());

        results.extend(crosses)
    }

    results
}
