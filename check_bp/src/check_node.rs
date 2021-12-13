use crate::bin_tree::BinMultTree;
use crate::bin_var_node::{CtrlMsg, CtrlMsgA};
use crate::check_msg::CheckMsg;
use belief_propagation::{BPError, BPResult, NodeFunction, NodeIndex, Msg};
use rustfft::{num_complex::Complex, Fft, FftPlanner};
use std::sync::Arc;

pub enum CmpOperator {
    SmallerEq,
    GreaterEq,
    Smaller,
    Greater,
}

pub struct CheckNode<const K: usize, const ETA: usize> {
    n: usize,
    coeffs: [i16; K],
    value: i16,
    op: CmpOperator,
    fft: Arc<dyn Fft<f64>>,
    ifft: Arc<dyn Fft<f64>>,
    connections: Vec<usize>,
}

fn pdf_le(data: &Vec<f64>, value: i16) -> f64 {
    data[..=(value + data.len() as i16 / 2) as usize]
        .iter()
        .sum()
}
fn pdf_ge(data: &Vec<f64>, value: i16) -> f64 {
    data[(value + data.len() as i16 / 2) as usize..]
        .iter()
        .sum()
}

fn pdf_l(data: &Vec<f64>, value: i16) -> f64 {
    data[..(value + data.len() as i16 / 2) as usize]
        .iter()
        .sum()
}
fn pdf_g(data: &Vec<f64>, value: i16) -> f64 {
    data[(1+value + data.len() as i16 / 2) as usize..]
        .iter()
        .sum()
}

fn derive_from_inequality_greater_eq<const ETA: usize>(
    dist_sum: Vec<f64>,
    value: i16,
    coeff: i16,
) -> CheckMsg<ETA> {
    let mut result = CheckMsg::new();
    //coeff*v+sum >=< value
    for v in -(ETA as i16) / 2..=(ETA as i16) / 2 {
        let vc = coeff * v;
        result[v] += pdf_ge(&dist_sum, value - vc);
    }
    result.normalize().expect("Failed to normalize");
    result
}

fn derive_from_inequality_smaller_eq<const ETA: usize>(
    dist_sum: Vec<f64>,
    value: i16,
    coeff: i16,
) -> CheckMsg<ETA> {
    let mut result = CheckMsg::new();
    //coeff*v+sum >=< value
    for v in -(ETA as i16) / 2..=(ETA as i16) / 2 {
        let vc = coeff * v;
        result[v] += pdf_le(&dist_sum, value - vc);
    }
    result.normalize().expect("Failed to normalize");
    result
}

fn derive_from_inequality_greater<const ETA: usize>(
    dist_sum: Vec<f64>,
    value: i16,
    coeff: i16,
) -> CheckMsg<ETA> {
    let mut result = CheckMsg::new();
    //coeff*v+sum >=< value
    for v in -(ETA as i16) / 2..=(ETA as i16) / 2 {
        let vc = coeff * v;
        result[v] += pdf_g(&dist_sum, value - vc);
    }
    result.normalize().expect("Failed to normalize");
    result
}

fn derive_from_inequality_smaller<const ETA: usize>(
    dist_sum: Vec<f64>,
    value: i16,
    coeff: i16,
) -> CheckMsg<ETA> {
    let mut result = CheckMsg::new();
    //coeff*v+sum >=< value
    for v in -(ETA as i16) / 2..=(ETA as i16) / 2 {
        let vc = coeff * v;
        result[v] += pdf_l(&dist_sum, value - vc);
    }
    result.normalize().expect("Failed to normalize");
    result
}

fn multiply_pointwise(op0: &Vec<Complex<f64>>, op1: &Vec<Complex<f64>>) -> Vec<Complex<f64>> {
    let prod: Vec<Complex<f64>> = op0
        .iter()
        .zip(op1.iter())
        .map(|(p0, p1)| (*p0 * *p1))
        .collect();

    let max: f64 = prod
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
    prod.into_iter().map(|p| p / max).collect()
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

    if max == 0.0 || max.is_nan() {
        panic!("No valid message encountered in to_probabilities.");
    }
    data.iter_mut().for_each(|p| *p /= max);
}

impl<const K: usize, const ETA: usize> CheckNode<K, ETA> {
    pub fn new(coeffs: [i16; K], value: i16, op: CmpOperator, n: usize) -> Self {
        let mut planner = FftPlanner::new();
        let fft = planner.plan_fft_forward(n);
        let ifft = planner.plan_fft_inverse(n);
        Self {
            n: n,
            coeffs: coeffs,
            value: value,
            op: op,
            connections: Vec::with_capacity(K),
            fft,
            ifft,
        }
    }
    fn node_function_normal(
        &self,
        inbox: Vec<(NodeIndex, CheckMsg<ETA>)>,
    ) -> BPResult<Vec<(NodeIndex, CheckMsg<ETA>)>> {

        let leafs: Vec<Vec<Complex<f64>>> = inbox
            .iter()
            .map(|(node_index, msg)| {
                msg.mult_and_transform(self.coeffs[*node_index], self.n, &self.fft)
                    .iter()
                    .map(|p| p / (self.n as f64).sqrt())
                    .collect()
            })
            .collect();

        let products: Vec<Vec<Complex<f64>>> =
            BinMultTree::new(leafs, multiply_pointwise).calculate();

        let partials: Vec<Vec<f64>> = products
            .into_iter()
            .map(|prd| ifft(prd, &self.ifft))
            .collect();

        let res: Vec<(NodeIndex, CheckMsg<ETA>)> = match self.op {
            CmpOperator::GreaterEq => partials
                .into_iter()
                .zip(inbox.into_iter())
                .map(|(dist_sum, ib)| {
                    (
                        ib.0,
                        derive_from_inequality_greater_eq(dist_sum, self.value, self.coeffs[ib.0]),
                    )
                })
                .collect(),

            CmpOperator::SmallerEq => partials
                .into_iter()
                .zip(inbox.into_iter())
                .map(|(dist_sum, ib)| {
                    (
                        ib.0,
                        derive_from_inequality_smaller_eq(dist_sum, self.value, self.coeffs[ib.0]),
                    )
                })
                .collect(),

            CmpOperator::Greater => partials
                .into_iter()
                .zip(inbox.into_iter())
                .map(|(dist_sum, ib)| {
                    (
                        ib.0,
                        derive_from_inequality_greater(dist_sum, self.value, self.coeffs[ib.0]),
                    )
                })
                .collect(),

            CmpOperator::Smaller => partials
                .into_iter()
                .zip(inbox.into_iter())
                .map(|(dist_sum, ib)| {
                    (
                        ib.0,
                        derive_from_inequality_smaller(dist_sum, self.value, self.coeffs[ib.0]),
                    )
                })
                .collect(),
        };
        Ok(res)
    }
}

impl<const K: usize, const ETA: usize> NodeFunction<i16, CheckMsg<ETA>, CtrlMsg, CtrlMsgA>
    for CheckNode<K, ETA>
{
    fn node_function(
        &mut self,
        inbox: Vec<(NodeIndex, CheckMsg<ETA>)>,
    ) -> BPResult<Vec<(NodeIndex, CheckMsg<ETA>)>> {
        self.node_function_normal(inbox)
    }

    fn number_inputs(&self) -> Option<usize> {
        Some(K)
    }

    fn is_factor(&self) -> bool {
        true
    }
    fn get_prior(&self) -> Option<CheckMsg<ETA>> {
        None
    }
    fn initialize(&mut self, connections: Vec<NodeIndex>) -> BPResult<()> {
        //TODO: Ensure connections are sorted
        if connections.len() != K {
            Err(BPError::new(
                "CheckNode::initialize".to_owned(),
                format!(
                    "Wrong number ({}) of connections given ({}).",
                    K,
                    connections.len()
                ),
            ))
        } else {
            self.connections = connections;
            Ok(())
        }
    }
    fn reset(&mut self) -> BPResult<()> {
        Ok(())
    }

    fn is_ready(
        &self,
        recv_from: &Vec<(NodeIndex, CheckMsg<ETA>)>,
        _current_step: usize,
    ) -> BPResult<bool> {
        Ok(recv_from.len() == self.connections.len())
    }
}
