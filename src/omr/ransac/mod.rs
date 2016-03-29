pub mod staff_cross;

use rand::Rng;
use rand;

// Fit stafflines using RANSAC:

pub trait RansacModel<Model, Point> {
    // Find the best model that best fits a minimal set of points
    fn fit_inliers(&Vec<Point>) -> Model;

    // Number of points required for fitInliers
    // Note: this is a method due to the following:
    //      error: associated constants are experimental (see issue #29646)
    fn num_required() -> usize;

    // Find the number of points within a given threshold of the model
    fn find_inliers(f32, &Vec<Point>, &Model) -> Vec<Point>;

    // Find the model that best fits a large set of points
    fn fit_model(&Vec<Point>) -> Option<Model>;
  }

pub struct RansacParams {
    // Number of attempted model fits
    num_iterations : u32,

    // Minimum distance to model to count as an inlier
    max_distance : f32,

    // Minimum number of inliers required for a model to be accepted
    min_inliers : u32,
}

struct RansacState<Model, Point> {
    samples : Vec<Point>,
    model : Option<Model>,
    inliers : Vec<Point>,
}

pub fn ransac<RM, Model, Point>(params: RansacParams, data: Vec<Point>) -> Option<Model>
    where RM: RansacModel<Model, Point>
        , Point: Clone {
    let mut rng = rand::thread_rng();

    let mut best_state = RansacState::<Model, Point> {
        samples: Vec::new(),
        model: None,
        inliers: Vec::new(),
    };

    for i in 0..params.num_iterations {
        // Randomly select points:
        let samples = rand::sample(&mut rng, data.iter().cloned(), RM::num_required());

        // Fit the model:
        let current_fit = RM::fit_inliers(&samples);

        // Find the set of inliers:
        let current_inliers = RM::find_inliers(params.max_distance, &data, &current_fit);

        // If the current fit is better than the current best fit:
        if current_inliers.len() > best_state.inliers.len() {
            // Replace the best model:
            best_state = RansacState {
                samples: samples,
                model: Some(current_fit),
                inliers: current_inliers,
            };
        }
    }

    best_state.model
}
