extern crate nalgebra as na;
extern crate rgsl;

use rgsl::blas;
use rgsl::cblas;

use std::cmp;
use geometry as gm;

fn least_squares_solve(a: &rgsl::MatrixF64, b: &rgsl::VectorF64) -> rgsl::VectorF64 {

    let qr = a.clone().unwrap();
    let tau = rgsl::VectorF64::new(cmp::min(a.size1(), a.size2())).unwrap();

    let qr_error = rgsl::linear_algebra::QR_decomp(&qr, &tau);
    if qr_error != rgsl::Value::Success {
        println!("qr_error: {:?}", qr_error);
        panic!();
    }

    // println!("qr: {:?}", qr);
    // println!("tau: {:?}", tau);

    let x = rgsl::VectorF64::new(a.size1()).unwrap();
    let residual = rgsl::VectorF64::new(a.size1()).unwrap();

    let solve_error = rgsl::linear_algebra::QR_lssolve(&qr, &tau, &b, &x, &residual);
    if solve_error != rgsl::Value::Success {
        println!("solve_error: {:?}", solve_error);
        panic!();
    }

    x
}

pub fn fit_line(points: &Vec<na::Vector2<f32>>) -> gm::Line {
    let m = points.len();
    let n = 2;
    let x = rgsl::MatrixF64::new(m, 2).unwrap();
    let y = rgsl::MatrixF64::new(m, 1).unwrap();

    for (i, pt) in points.iter().enumerate() {
        x.set(i, 0, 1.0 as f64);
        x.set(i, 1, pt[0] as f64);
        y.set(i, 0, pt[1] as f64);
    }

    // Solve X.t()*X*beta = X.t()*y

    let mut gramian = rgsl::MatrixF64::new(n, n).unwrap();
    let mut xt_y_mat = rgsl::MatrixF64::new(n, 1).unwrap();

    // println!("x: {:?}", x);
    // println!("y: {:?}", y);
    // println!("gramian: {:?}", gramian);
    // println!("xt_y_mat: {:?}", xt_y_mat);

    // println!("gramian");

    // dgemm: C = \alpha op(A) op(B) + \beta C
    let dgemm_error = blas::level3::dgemm(
        cblas::Transpose::Trans,
        cblas::Transpose::NoTrans,
        1.0, // \alpha
        &x, // A
        &x, // B
        0.0, // \beta
        &mut gramian // C
    );
    if dgemm_error != rgsl::Value::Success {
        println!("dgemm_error, gramian: {:?}", dgemm_error);
        panic!();
    }

    // println!("xt_y_mat");

    let dgemm_error = blas::level3::dgemm(
        cblas::Transpose::Trans,
        cblas::Transpose::NoTrans,
        1.0, // \alpha
        &x, // A
        &y, // B
        0.0, // \beta
        &mut xt_y_mat // C
    );
    if dgemm_error != rgsl::Value::Success {
        println!("dgemm_error, xt_y_mat: {:?}", dgemm_error);
        panic!();
    }

    let (xt_y, _) = xt_y_mat.get_col(0).unwrap();

    let beta = least_squares_solve(&gramian, &xt_y);

    // println!("beta: {:?}", beta);
    let b = beta.get(0) as f32;
    let m = beta.get(1) as f32;
    let l0 = na::Vector2::<f32>::new(0.0, b);
    let l1 = na::Vector2::<f32>::new(1.0, b + m);
    gm::Line::new(l0, l1)
}
