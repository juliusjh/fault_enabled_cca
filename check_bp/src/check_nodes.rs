// pub mod prob_check_node;
pub mod check_node;
// pub mod double_check_node;
// pub mod prob_double_check_node;

// pub use prob_check_node::*;
// pub use double_check_node::*;
// pub use prob_double_check_node::*;
use belief_propagation::Msg;
pub use check_node::*;

use rustfft::{num_complex::Complex, Fft, FftPlanner};

use std::sync::Arc;

use crate::check_msg::CheckMsg;

#[derive(Clone, Copy)]
pub enum CmpOperator {
    SmallerEq(i64),
    GreaterEq(i64),
}

fn pdf_le(data: &Vec<f64>, value: i64) -> f64 {
    data[..=(value + data.len() as i64 / 2) as usize]
        .iter()
        .sum()
}
fn pdf_ge(data: &Vec<f64>, value: i64) -> f64 {
    data[(value + data.len() as i64 / 2) as usize..]
        .iter()
        .sum()
}

fn multiply_pointwise(op0: &mut Vec<Complex<f64>>, op1: &Vec<Complex<f64>>) {
    // let prod: Vec<Complex<f64>> = op0
    //     .iter()
    //     .zip(op1.iter())
    //     .map(|(p0, p1)| (*p0 * *p1))
    //     .collect();
    op0.iter_mut()
        .zip(op1.iter())
        .for_each(|(p0, p1)| *p0 = *p0 * *p1);
    // for (op0i, op1i) in op0.iter_mut().zip(op1.iter()) {
    //     *op0i *= op1i;
    // }

    //Keep?
    let max: f64 = op0
        .iter()
        .max_by(|p0, p1| {
            p0.norm()
                .partial_cmp(&p1.norm())
                .unwrap_or(std::cmp::Ordering::Less)
        })
        .map(|p| p.norm())
        .unwrap_or(f64::NAN);
    if max.is_nan() || max == 0.0 {
        panic!("Could not normalize in fft domain.");
    }
    op0.iter_mut().for_each(|p| *p = *p / max);
}

fn ifft(mut data: Vec<Complex<f64>>, ifft: &Arc<dyn Fft<f64>>) -> Vec<f64> {
    ifft.process(&mut data);
    let res_temp: Vec<f64> = data.into_iter().map(|c| c.re).collect();
    let mut res: Vec<f64> = vec![0 as f64; res_temp.len()];
    let sz = res_temp.len();
    let n = res_temp.len() / 2;
    for (i, v) in res_temp[..n].into_iter().enumerate() {
        res[i + n] = *v / sz as f64;
    }
    for (i, v) in res_temp[n..].into_iter().enumerate() {
        res[i] = *v / sz as f64;
    }
    to_probabilities(&mut res);
    res
}

fn to_probabilities(data: &mut Vec<f64>) {
    let max = *{
        data.iter()
            .max_by(|p0, p1| p0.partial_cmp(p1).unwrap_or(std::cmp::Ordering::Less))
            .unwrap_or(&f64::NAN)
    };

    //let sum: f64 = data.iter().sum();
    if max == 0.0 || max.is_nan() {
        panic!(
            "No valid message encountered in to_probabilities.\nMsg: {:?}",
            data
        );
    }
    data.iter_mut().for_each(|p| *p /= max);
}

fn derive_from_inequality_greater<const ETA: usize>(
    dist_sum: Vec<f64>,
    value: i64,
    coeff: i64,
) -> CheckMsg<ETA> {
    let mut result = CheckMsg::new();
    //coeff*v+sum >=< value
    for v in -(ETA as i64) / 2..=(ETA as i64) / 2 {
        let vc = coeff * v;
        result[v] += pdf_ge(&dist_sum, value - vc);
    }
    result.normalize().expect("Failed to normalize");
    result
}

fn derive_from_inequality_smaller<const ETA: usize>(
    dist_sum: Vec<f64>,
    value: i64,
    coeff: i64,
) -> CheckMsg<ETA> {
    let mut result = CheckMsg::new();
    //coeff*v+sum >=< value
    for v in -(ETA as i64) / 2..=(ETA as i64) / 2 {
        let vc = coeff * v;
        result[v] += pdf_le(&dist_sum, value - vc);
        //println!("dist sum: {:?}", dist_sum.data.to_vec());
        //println!("coeff: {}, vc: {}, res: {}, value: {}", coeff, vc, result[*v], value);
    }
    result.normalize().expect("Failed to normalize");
    //panic!("d");
    result
}

fn derive_from_inequality_greater_prob<const ETA: usize>(
    dist_sum: Vec<f64>,
    value: i64,
    coeff: i64,
    prob_correct: f64,
) -> CheckMsg<ETA> {
    let mut result = CheckMsg::new();
    let prob_incorrect = 1.0 - prob_correct;
    //coeff*v+sum >=< value
    for v in -(ETA as i64) / 2..=(ETA as i64) / 2 {
        let vc = coeff * v;
        let p_le = pdf_le(&dist_sum, value - vc);
        let p_ge = pdf_ge(&dist_sum, value - vc);
        result[v] += prob_incorrect * p_le + prob_correct * p_ge;
    }
    result.normalize().expect("Failed to normalize");
    result
}

fn derive_from_inequality_smaller_prob<const ETA: usize>(
    dist_sum: Vec<f64>,
    value: i64,
    coeff: i64,
    prob_correct: f64,
) -> CheckMsg<ETA> {
    let mut result = CheckMsg::new();
    let prob_incorrect = 1.0 - prob_correct;
    //coeff*v+sum >=< value
    for v in -(ETA as i64) / 2..=(ETA as i64) / 2 {
        let vc = coeff * v;
        let p_le = pdf_le(&dist_sum, value - vc);
        let p_ge = pdf_ge(&dist_sum, value - vc);
        result[v] += prob_correct * p_le + prob_incorrect * p_ge;
    }
    result.normalize().expect("Failed to normalize");
    //panic!("d");
    result
}
