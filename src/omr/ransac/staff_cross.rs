
use omr::ransac::RansacModel;
use omr::scanning::staff_cross::StaffCross;
// use nalgebra as na;
use geometry as gm;
use nalgebra as na;

#[derive(Debug)]
pub struct StaffCrossLine {
    pub a: StaffCross,
    pub b: StaffCross,
}
impl StaffCrossLine {
    pub fn new(a: StaffCross, b: StaffCross) -> StaffCrossLine {
        StaffCrossLine {
            a: a,
            b: b,
        }
    }

    pub fn average_space_width(&self) -> f32 {
        let line = gm::Line::new(self.a.centre(), self.b.centre());

        let avg_a = self.a.average_space_width(&line);
        let avg_b = self.b.average_space_width(&line);

        (avg_a + avg_b) / 2.0
    }

    pub fn average_line_width(&self) -> f32 {
        let line = gm::Line::new(self.a.centre(), self.b.centre());

        let avg_a = self.a.average_line_width(&line);
        let avg_b = self.b.average_line_width(&line);

        (avg_a + avg_b) / 2.0
    }
}

pub struct StaffCrossLineModel;
impl RansacModel<StaffCrossLine, StaffCross> for StaffCrossLineModel {

    fn fit_inliers(data: &Vec<StaffCross>) -> StaffCrossLine {
        let pt1 = &data[0];
        let pt2 = &data[1];
        // StaffCrossLine::new(pt1.centre(), pt2.centre())
        StaffCrossLine::new(pt1.clone(), pt2.clone())
    }

    fn num_required() -> usize {
        2
    }

    fn find_inliers(max_dist: f32, data: &Vec<StaffCross>, model: &StaffCrossLine) -> Vec<StaffCross> {
        let space_width = model.average_space_width();
        let line_width = model.average_line_width();
        let line = gm::Line::new(model.a.centre(), model.b.centre());
        let line_dir = na::normalize(&(line.b - line.a));

        let mut inliers = Vec::new();

        for pt in data {
            // Ignore if sample is not close enough to the model.
            let dist = line.distance_to_point(&pt.centre());
            if dist > space_width * 0.5 {
                continue;
            }

            // Ignore if the sample scan direction is too close to the line direction.
            // Note: This prevents multiple samples from a single scan-line across multiple staves
            // being considered in the same model.
            let scan_dir = pt.scan_direction();
            let cos_angle = na::dot(&scan_dir, &line_dir);
            if cos_angle.abs() > 0.8 {
                continue;
            }

            // Ignore if sample has a staff-line spacing too different from the model.
            let pt_space_width = pt.average_space_width(&line);
            let space_error = (pt_space_width - space_width).abs() / space_width;
            if space_error > 0.5 {
                continue;
            }

            // Ignore if sample has a staff-line width too different from the model.
            let pt_line_width = pt.average_line_width(&line);
            let line_error = (pt_line_width - line_width).abs() / line_width;
            if line_error > 1.0 {
                continue;
            }

            inliers.push(pt.clone());
        }

        inliers
    }

    fn fit_model(data: &Vec<StaffCross>) -> Option<StaffCrossLine> {
        None
    }

}
