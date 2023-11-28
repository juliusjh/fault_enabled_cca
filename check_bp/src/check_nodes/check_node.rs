use super::*;
use crate::check_msg::CheckMsg;
use belief_propagation::{BPError, BPResult, NodeFunction, NodeIndex};
use rustfft::{num_complex::Complex, Fft};
use std::sync::Arc;

pub struct CheckNode<const K: usize, const ETA: usize> {
    length_llo_distribution: usize,
    coeffs: [i64; K],
    op: CmpOperator,
    fft: Arc<dyn Fft<f64>>,
    ifft: Arc<dyn Fft<f64>>,
    connections: Vec<usize>,
    prob_correct: Option<f64>,
}

fn compute_llo_products<T: Clone+Default>(leafs: Vec<T>, mult: fn(&mut T, &T)) -> Vec<T> {
    assert!(leafs.len() >= 2);
    let n = leafs.len();
    let mut result = Vec::with_capacity(n);
    //Dummy placeholder, hack to avoid moving all
    //elements later
    result.push(T::default());

    let mut acc = leafs[0].clone();
    //forward pass
    for leaf in &leafs[1..] {
        result.push(acc.clone());
        mult(&mut acc, leaf);
    }
    //backward pass
    let mut acc = leafs[n-1].clone();
    for idx in (1..n-2).rev() {
        mult(&mut result[idx], &acc);
        mult(&mut acc, &leafs[idx+1]);
    }
    result[0] = acc;
    result
}

impl<const K: usize, const ETA: usize> CheckNode<K, ETA> {
    pub fn new(coeffs: [i64; K], op: CmpOperator, n: usize, prob_correct: Option<f64>) -> Self {
        let mut planner = FftPlanner::new();
        let fft = planner.plan_fft_forward(n);
        let ifft = planner.plan_fft_inverse(n);
        Self {
            length_llo_distribution: n,
            coeffs,
            op,
            connections: Vec::with_capacity(K),
            fft,
            ifft,
            prob_correct,
        }
    }
    fn node_function_normal(
        &self,
        inbox: Vec<(NodeIndex, CheckMsg<ETA>)>,
    ) -> BPResult<Vec<(NodeIndex, CheckMsg<ETA>)>> {
        //println!("{:?}", inbox[0].1.data.to_vec());
        let leafs: Vec<Vec<Complex<f64>>> = inbox
            .iter()
            .map(|(node_index, msg)| {
                msg.mult_and_transform(self.coeffs[*node_index], self.length_llo_distribution, &self.fft)
                    .iter()
                    .map(|p| p / (self.length_llo_distribution as f64).sqrt())
                    .collect()
            })
            .collect();

        assert!(inbox.iter().all(|x| (*x
            .1
            .data
            .iter()
            .max_by(|x, y| *x.partial_cmp(y).as_ref().unwrap())
            .unwrap()
            - 1.0)
            < 0.001));

        let products: Vec<Vec<Complex<f64>>> = compute_llo_products(leafs, multiply_pointwise);
        // let products: Vec<Vec<Complex<f64>>> =
        //     BinMultTree::new(leafs, multiply_pointwise).calculate();

        let partials: Vec<Vec<f64>> = products
            .into_iter()
            .map(|prd| ifft(prd, &self.ifft))
            .collect();
        let res: Vec<(NodeIndex, CheckMsg<ETA>)> = if let Some(p) = self.prob_correct {
            match self.op {
                CmpOperator::GreaterEq(value) => partials
                    .into_iter()
                    .zip(inbox.into_iter())
                    .map(|(dist_sum, ib)| {
                        (
                            ib.0,
                            derive_from_inequality_greater_prob(
                                dist_sum,
                                value,
                                self.coeffs[ib.0],
                                p,
                            ),
                        )
                    })
                    .collect(),

                CmpOperator::SmallerEq(value) => partials
                    .into_iter()
                    .zip(inbox.into_iter())
                    .map(|(dist_sum, ib)| {
                        (
                            ib.0,
                            derive_from_inequality_smaller_prob(
                                dist_sum,
                                value,
                                self.coeffs[ib.0],
                                p,
                            ),
                        )
                    })
                    .collect(),
            }
        } else {
            match self.op {
                CmpOperator::GreaterEq(value) => partials
                    .into_iter()
                    .zip(inbox.into_iter())
                    .map(|(dist_sum, ib)| {
                        (
                            ib.0,
                            derive_from_inequality_greater(dist_sum, value, self.coeffs[ib.0]),
                        )
                    })
                    .collect(),

                CmpOperator::SmallerEq(value) => partials
                    .into_iter()
                    .zip(inbox.into_iter())
                    .map(|(dist_sum, ib)| {
                        (
                            ib.0,
                            derive_from_inequality_smaller(dist_sum, value, self.coeffs[ib.0]),
                        )
                    })
                    .collect(),
            }
        };
        Ok(res)
    }
}

impl<const K: usize, const ETA: usize> NodeFunction<i64, CheckMsg<ETA>>
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
