use ffmpeg_camera::image_ycbcr;
use ffmpeg_camera::image::Image;
use ffmpeg_camera::image;

pub struct Segment {
    pub x: usize,
    pub y_min: usize,
    pub y_max: usize,
}

// impl Segment {
//     fn empty() -> Segment {
//         Segment {
//             x: 0,
//             y_min: 0,
//             y_max: 0,
//         }
//     }
// }

// Scans across an image, classifying pixels as either white or black, and returning a sequence of
// black line segments.
pub struct SegmentScanner<'a> {
    image : &'a image_ycbcr::Image,
    curr_point : [usize; 2],

    // Is the current point white?
    is_white : bool,
    last_white_point: [usize; 2],
    curr_pixel: image::Pixel,
}

impl<'a> SegmentScanner<'a> {
    pub fn new(image : &'a image_ycbcr::Image, start_point : [usize; 2]) -> SegmentScanner {
        SegmentScanner {
            image: image,
            curr_point: start_point,
            is_white: true,
            last_white_point: start_point,
            curr_pixel: image.index(start_point[0], start_point[1]),
        }
    }
}

impl<'a> Iterator for SegmentScanner<'a> {
    type Item = Segment;

    fn next(&mut self) -> Option<Segment> {

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
                // self.curr_cross.add(self.last_white_point, next_point);

                return Some(Segment {
                    x: next_point[0],
                    y_min: self.last_white_point[1] + 1,
                    y_max: next_point[1] - 1,
                })

                // if self.curr_cross.is_complete() {
                //     let complete_cross = self.curr_cross.clone();
                //     self.curr_cross = Segment::empty();
                //     return Some(complete_cross);
                // }
            }
        }
    }
}
