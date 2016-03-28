use ffmpeg_camera::image_ycbcr;

extern crate core;
// use self::core::slice::Iter;

// Represents a the intersection of a straight line with a set of 5 staff lines.
// Stores the coordinates of each of the 5 line intersections in image coordinates.
#[derive(Debug, Clone)]
pub struct StaffCross {
    // Note: Currently, to keep things simple, we only scan downward, so we store each span of ink
    // as the starting and ending y-coordinate. (later we'll either store start+stop points, or add
    // a transform from SE(2) to represent the scan coorinate system)

    spans : Vec<[usize; 2]>,
}

impl StaffCross {
    fn empty() -> StaffCross {
        StaffCross {
            spans: Vec::new(),
        }
    }

    fn add(&mut self, point_a : [usize; 2], point_b : [usize; 2]) {
        self.spans.push([point_a[1], point_b[1]])
    }

    fn is_complete(&self) -> bool {
        self.spans.len() >= 5
    }

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
    image : &'a image_ycbcr::Image,
    curr_point : [usize; 2],
    curr_cross : StaffCross,

    // Is the current point white?
    is_white : bool,
    last_white_point: [usize; 2],
    curr_pixel: image_ycbcr::Pixel,
}

impl<'a> StaffScanner<'a> {
    pub fn new(image : &'a image_ycbcr::Image, start_point : [usize; 2]) -> StaffScanner {
        StaffScanner {
            image: image,
            curr_point: start_point,
            curr_cross: StaffCross::empty(),
            is_white: true,
            last_white_point: start_point,
            curr_pixel: image.index(start_point[0], start_point[1]),
        }
    }
}

impl<'a> Iterator for StaffScanner<'a> {
    type Item = StaffCross;

    fn next(&mut self) -> Option<StaffCross> {

        loop {
            let next_point = [self.curr_point[0], self.curr_point[1] + 1];
            if !self.image.contains(next_point[0], next_point[1]) {
                return None;
            }

            let next_pixel = self.image.index(next_point[0], next_point[1]);

            if self.is_white && next_pixel.y < 128 {
                self.is_white = false;
                self.last_white_point = self.curr_point;
            }

            self.curr_point = next_point;
            self.curr_pixel = next_pixel;

            if !self.is_white && next_pixel.y > 128 {
                self.is_white = true;
                self.curr_cross.add(self.last_white_point, next_point);

                if self.curr_cross.is_complete() {
                    let complete_cross = self.curr_cross.clone();
                    self.curr_cross = StaffCross::empty();
                    return Some(complete_cross);
                }
            }
        }
    }
}
