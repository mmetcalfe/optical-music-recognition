pub mod staff_cross;

use rand;
use rand::SeedableRng;
use rand::Rng;
use rand::distributions::{IndependentSample, Range};
use std;

pub fn choose(n: usize, k: usize) -> usize {
    // https://en.wikipedia.org/wiki/Binomial_coefficient#Multiplicative_formula

    if n == 0 {
        return 1;
    }

    let mut product = 1.0;
    for i in 1..k+1 {
        product *= (n + 1 - i) as f64 / i as f64
    }

    product.round() as usize
}

pub fn calculate_num_iterations(
    // Total number of points in the dataset:
    num_points: usize,

    // Estimated number of points belonging to the model that we wish to find:
    // Note: This is the set of very accurate points, any 'num_points' of which would fit an
    // acceptable model.
    // We assume that the rest of the dataset is noise.
    num_inliers: usize,

    // Points required to fit a candidate model:
    points_per_model: usize,

    // The required proability of finding the model:
    success_probability: f32
    ) -> usize {

    if num_inliers > num_points {
        panic!("Cannot have more inliers than data points!")
    }

    if num_inliers < points_per_model {
        return 0;
        // panic!("Cannot have fewer inliers than points required!")
    }

    let n = num_points;
    let k = num_inliers;
    let m = points_per_model;
    let p = success_probability as f64;

    // Ways to choose source points correctly:
    let k_m = choose(k, m);

    // Ways to choose source points overall:
    let n_m = choose(n, m);

    // Probability of choosing correctly in a single iteration:
    let f = k_m as f64 / n_m as f64;

    // Iterations required to have probability p of choosing correctly at least once:
    (1.0 - p).log(1.0 - f) as usize
}


// Fit stafflines using RANSAC:

pub trait RansacModel<Model, Point> {
    // Find the best model that best fits a minimal set of points
    fn fit_inliers(&[&Point]) -> Model;

    // Number of points required for fitInliers
    // Note: this is a method due to the following:
    //      error: associated constants are experimental (see issue #29646)
    fn num_required() -> usize;

    // Find the number of points within a given threshold of the model
    fn find_inliers(f32, &Vec<Point>, &Model) -> Vec<Point>;

    // Return all points that are not within a given threshold of the model
    fn find_outliers(f32, &Vec<Point>, &Model) -> Vec<Point>;

    // Find the model that best fits a large set of points
    fn fit_model(&Vec<Point>) -> Option<Model>;
  }

#[derive(Clone, Copy)]
pub struct RansacParams {
    // Number of attempted model fits
    pub num_iterations : usize,

    // Minimum distance to model to count as an inlier
    pub max_distance : f32,

    // Minimum number of inliers required for a model to be accepted
    pub min_inliers : usize,
}

pub struct RansacState<Model, Point> {
    // samples : Vec<Point>,
    pub model : Option<Model>,
    pub inliers : Vec<Point>,
}

pub fn ransac<RM, Model, Point, R: Rng>(params: &RansacParams, data: &Vec<Point>, rng: &mut R)
    -> RansacState<Model, Point>
    // -> Option<Model>
    where RM: RansacModel<Model, Point>
        , Point: Clone {

    let mut best_state = RansacState::<Model, Point> {
        // samples: Vec::new(),
        model: None,
        inliers: Vec::new(),
    };

    // If there are too few points, just return None:
    if data.len() < RM::num_required() {
        // return None;
        return best_state;
    }

    let between = rand::distributions::Range::new(0, data.len());
    for _ in 0..params.num_iterations {
        // Randomly select points:
        // Note: Using rand::sample is *much* slower than just sampling two random indices.
        // let samples = rand::sample(rng, data, RM::num_required());
        let a = between.ind_sample(rng);
        let b = between.ind_sample(rng);
        let samples = [&data[a], &data[b]];
        if a == b {
            continue;
        }

        // Fit the model:
        // println!("Fit the model:");
        let current_fit = RM::fit_inliers(&samples);

        // Find the set of inliers:
        // println!("Find the set of inliers:");
        let current_inliers = RM::find_inliers(params.max_distance, &data, &current_fit);

        // If the current fit is better than the current best fit:
        // println!("If the current fit is better than the current best fit:");
        if current_inliers.len() > best_state.inliers.len() {
            if current_inliers.len() >= params.min_inliers {
                // Replace the best model:
                best_state = RansacState {
                    // samples: samples,
                    model: Some(current_fit),
                    inliers: current_inliers,
                };
            }
        }
    }

    // best_state.model
    best_state
}

pub fn ransac_multiple<RM, Model, Point>(params: &RansacParams, data: &Vec<Point>)
    -> Vec<RansacState<Model, Point>>
    // -> Option<Model>
    where RM: RansacModel<Model, Point>
        , Point: Clone {

    // let mut thread_rng = rand::thread_rng();
    // let mut rng = rand::XorShiftRng::from_seed(thread_rng.gen::<[u32; 4]>());
    let mut rng = rand::XorShiftRng::new_unseeded();

    let mut states = Vec::new();
    let mut new_data = data.clone();

    loop {
        let num_iterations = calculate_num_iterations(
            new_data.len(), // num_points
            std::cmp::min(new_data.len() / 2, data.len() / 20), // num_inliers
            RM::num_required(), // points_per_model
            0.75 // success_probability
        );
        let mut new_params = params.clone();
        new_params.num_iterations = num_iterations;
        // println!("num_iterations: {:?}", num_iterations);

        let state = ransac::<RM, Model, Point, _>(&new_params, &new_data, &mut rng);
        // let state = ransac::<RM, Model, Point>(params, &new_data);

        if state.model.is_some() {
            if let Some(ref model) = state.model {
                new_data = RM::find_outliers(params.max_distance, &new_data, &model);
            }
            states.push(state);
        } else {
            break;
        }
    }

    states
}
