
use omr::ransac::RansacModel;
use omr::scanning::StaffCross;
use nalgebra as na;
use geometry as gm;

pub struct StaffCrossLineModel;

impl RansacModel<gm::Line, StaffCross> for StaffCrossLineModel {

    fn fit_inliers(data: &Vec<StaffCross>) -> gm::Line {
        let pt1 = &data[0];
        let pt2 = &data[1];
        gm::Line::new(pt1.centre(), pt2.centre())
    }

    fn num_required() -> usize {
        2
    }

    fn find_inliers(max_dist: f32, data: &Vec<StaffCross>, model: &gm::Line) -> Vec<StaffCross> {
        let mut inliers = Vec::new();

        for pt in data {
            let dist = model.distance_to_point(&pt.centre());

            if dist < max_dist {
                inliers.push(pt.clone());
            }
        }

        inliers
    }

    fn fit_model(data: &Vec<StaffCross>) -> Option<gm::Line> {
        None
    }

}
