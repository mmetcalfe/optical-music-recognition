use ffmpeg_camera::image_ycbcr;

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
