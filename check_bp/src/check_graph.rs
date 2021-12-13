use crate::bin_var_node::{BinVariableNode, CtrlMsg, CtrlMsgA};
use crate::check_msg::CheckMsg;
use crate::check_node::{CheckNode, CmpOperator};
use belief_propagation::{BPError, BPGraph, Msg, BPResult, Probability};
use pyo3::prelude::*;
use pyo3::{create_exception, PyResult};
use rustfft::{num_complex::Complex, FftPlanner};
use std::collections::HashMap;
use std::convert::TryInto;
use crossbeam;

const N: usize = 1024;
#[cfg(feature = "kyber1024")]
const K: usize = 2048;
#[cfg(feature = "kyber768")]
const K: usize = 1536;
#[cfg(feature = "kyber512")]
const K: usize = 1024;

#[cfg(feature = "kyber1024")]
const ETA: usize = 5;
#[cfg(feature = "kyber768")]
const ETA: usize = 5;
#[cfg(feature = "kyber512")]
const ETA: usize = 7;

create_exception!(check_bp, PyCheckGraphError, pyo3::exceptions::PyException);

#[derive(Debug)]
pub struct CheckGraphError {
    desc: String,
}

impl CheckGraphError {
    pub fn new(desc: String) -> Self {
        CheckGraphError { desc: desc }
    }
    pub fn from_bp(err: BPError) -> Self {
        CheckGraphError {
            desc: err.to_string(),
        }
    }
}

impl std::fmt::Display for CheckGraphError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "An error occured in belief_propagation: {}", self.desc)
    }
}

impl std::error::Error for CheckGraphError {}

impl std::convert::From<BPError> for CheckGraphError {
    fn from(err: BPError) -> CheckGraphError {
        CheckGraphError::from_bp(err)
    }
}

impl std::convert::From<CheckGraphError> for PyErr {
    fn from(err: CheckGraphError) -> PyErr {
        PyCheckGraphError::new_err(err.to_string())
    }
}

#[pyclass]
pub struct CheckGraph {
    g: BPGraph<i16, CheckMsg<ETA>, CtrlMsg, CtrlMsgA>,
    var_nodes: usize,
}

#[pymethods]
impl CheckGraph {
    #[new]
    fn new() -> Self {
        let mut g = BPGraph::new();
        g.set_normalize(false);
        Self {
            g,
            var_nodes: 0,
        }
    }

    fn set_fixed(&mut self, node_index: usize, value: i16) {
        self.g
            .send_control_message(node_index, CtrlMsg::SetFixed(value))
            .expect("Node not found.");
    }

    fn get_fixed(&mut self, node_index: usize) -> bool {
        let response: CtrlMsgA = self
            .g
            .send_control_message(node_index, CtrlMsg::GetFixed)
            .expect("Node not found.");
        match response {
            CtrlMsgA::Fixed(v) => v,
            CtrlMsgA::None => panic!("Node did not return valid fixed state."),
        }
    }

    fn set_check_validity(&mut self, value: bool) {
        self.g.set_check_validity(value);
    }
    fn add_var_nodes(&mut self, prior: HashMap<i16, f64>) -> PyResult<()> {
        let mut prior_msg = CheckMsg::new(); //TODO:Ineffcient
        for (v, p) in prior {
            prior_msg[v] = p;
        }
        for i in 0..K {
            let mut n = BinVariableNode::new();
            prior_msg
                .normalize()
                .map_err(|e| CheckGraphError::from_bp(e))?;
            n.set_prior(&prior_msg)
                .map_err(|e| CheckGraphError::from_bp(e))?;
            self.g.add_node(i.to_string(), Box::new(n));
            self.var_nodes += 1;
        }
        Ok(())
    }
    fn add_var_node(&mut self, name: String, prior: HashMap<i16, f64>) -> PyResult<usize> {
        let mut prior_msg = CheckMsg::new(); //TODO:Ineffcient
        for (v, p) in prior {
            prior_msg[v] = p;
        }
        let mut n = BinVariableNode::new();
        prior_msg
            .normalize()
            .map_err(|e| CheckGraphError::from_bp(e))?;
        n.set_prior(&prior_msg)
            .map_err(|e| CheckGraphError::from_bp(e))?;
        let idx = self.g.add_node(name, Box::new(n));
        self.var_nodes += 1;
        Ok(idx)
    }
    fn add_equation(
        &mut self,
        name: String,
        coefficients: Vec<i16>,
        value: i16,
        is_smaller: bool,
        is_equal: bool
    ) -> PyResult<usize> {
        if self.var_nodes != K {
            panic!("Wrong number of variables.");
        }
        if coefficients.len() != self.var_nodes {
            panic!(
                "Wrong number of coefficients (should be {} but is {}).",
                self.var_nodes,
                coefficients.len()
            );
        }
        let op = if is_smaller {
            if is_equal {
                CmpOperator::SmallerEq
            }
            else {
                CmpOperator::Smaller
            }
        } else {
            if is_equal {
                CmpOperator::GreaterEq
            }
            else {
                CmpOperator::Greater
            }
        };
        let check_node: CheckNode<K, ETA> =
            CheckNode::new(coefficients.try_into().unwrap(), value, op, N);
        let idx = self.g.add_node(name, Box::new(check_node));
        for n in 0..self.var_nodes {
            self.g
                .add_edge(n, idx)
                .map_err(|e| CheckGraphError::from_bp(e))?;
        }
        Ok(idx)
    }
    fn ini(&mut self) -> PyResult<()> {
        self.g
            .initialize()
            .map_err(|e| CheckGraphError::from_bp(e))?;
        Ok(())
    }
    fn propagate(&mut self, steps: usize, threads: u32) -> PyResult<()> {
        if threads <= 0 {
            return Err(PyErr::from(CheckGraphError::new(
                "Cannot work with less than 1 thread.".to_owned(),
            )));
        } else if threads == 1 {
            self.g
                .propagate(steps)
                .map_err(|e| CheckGraphError::from_bp(e))?;
        } else {
            self.g
                .propagate_threaded(steps, threads)
                .map_err(|e| CheckGraphError::from_bp(e))?;
        }
        Ok(())
    }
    fn get_results(
        &self,
        thread_count: usize,
    ) -> PyResult<HashMap<usize, Option<(HashMap<i16, Probability>, f64)>>> {
        let res = fetch_results_parallel(&self.g, (0..self.var_nodes).collect(), thread_count)
            .map_err(|e| CheckGraphError::from_bp(e))?;
        Ok(res)
    }
    fn get_result(&self, node: usize) -> PyResult<HashMap<i16, f64>> {
        let res = self
            .g
            .get_result(node)
            .map_err(|e| CheckGraphError::from_bp(e))?
            .ok_or(CheckGraphError::new(
                "Node did not return a result.".to_owned(),
            ))?;
        let sum: f64 = res.values().sum();
        //println!("Sum: {}", sum);
        //println!("Res: {:?}\n", res);
        Ok(res.into_iter().map(|(v, p)| (v, p / sum)).collect())
    }
}

fn fetch_results_parallel(
    g: &BPGraph<i16, CheckMsg<ETA>, CtrlMsg, CtrlMsgA>,
    nodes: Vec<usize>,
    thread_count: usize,
) -> BPResult<HashMap<usize, Option<(HashMap<i16, Probability>, f64)>>> {
    crossbeam::scope(
        |scope| -> BPResult<HashMap<usize, Option<(HashMap<i16, Probability>, f64)>>> {
            let nodes_per_thread = nodes.len() / thread_count;
            let mut results = HashMap::new();
            let mut handles = Vec::new();
            for nodes_list in nodes.chunks(nodes_per_thread) {
                handles.push(scope.spawn(
                    move |_| -> Vec<(usize, BPResult<Option<(HashMap<i16, Probability>, f64)>>)> {
                        let mut tr_results = Vec::new();
                        for node in nodes_list {
                            let res = g.get_result(*node).map(|res| match res {
                                Some(mut r) => {
                                    let sum: f64 = r.values().sum();
                                    r.iter_mut().for_each(|(_, p)| *p /= sum);
                                    let ent = calc_entropy(&r);
                                    if ent.is_nan() {
                                        println!("{:?}", r);
                                        panic!("");
                                    }
                                    Some((r, ent))
                                }
                                None => panic!("Node {} did not return a result.", node),
                            });
                            tr_results.push((*node, res));
                        }
                        tr_results
                    },
                ));
            }
            for h in handles {
                let tr = h.join().expect("Joining threads failed in get_results.");
                for (node, res) in tr.into_iter() {
                    results.insert(node, res?);
                }
            }
            Ok(results)
        },
    )
    .expect("Scoped threading failed.")
}

fn calc_entropy(probs: &HashMap<i16, Probability>) -> f64 {
    -probs
        .iter()
        .map(|(_, p)| {
            let r = p * p.log2();
            if r.is_nan() {
                0.0
            } else {
                r
            }
        })
        .sum::<f64>()
}

#[pyfunction]
pub fn test_fft_2(op0: Vec<f64>, op1: Vec<f64>) -> (Vec<f64>, Vec<f64>) {
    let mut res_conv = vec![0.0; op0.len() + op1.len()];
    for (v0, p0) in op0.iter().enumerate() {
        for (v1, p1) in op1.iter().enumerate() {
            res_conv[v0 + v1] += p0 * p1;
        }
    }
    //[-256, .., 0, .., 256]
    let n = op0.len() + op1.len();
    let mut planner = FftPlanner::new();
    let fft = planner.plan_fft_forward(n);
    let ifft = planner.plan_fft_inverse(n);
    let mut op0_fft_in = vec![Complex { re: 0.0, im: 0.0 }; n];
    let mut op1_fft_in = vec![Complex { re: 0.0, im: 0.0 }; n];
    for i in 0..n / 2 {
        let idx = if i < op0.len() / 2 {
            n + (i - op0.len() / 2)
        } else {
            i - op0.len() / 2
        };
        op0_fft_in[idx] = Complex {
            re: op0[i],
            im: 0.0,
        };
        op1_fft_in[idx] = Complex {
            re: op1[i],
            im: 0.0,
        };
    }
    fft.process(&mut op0_fft_in);
    fft.process(&mut op1_fft_in);
    let mut res_fft: Vec<Complex<f64>> = op0_fft_in
        .iter()
        .zip(op1_fft_in.iter())
        .map(|(p0, p1)| p0 * p1 / n as f64)
        .collect();
    ifft.process(&mut res_fft);
    let res_temp: Vec<f64> = res_fft.iter().map(|c| c.re).collect();
    let mut res: Vec<f64> = vec![0.0; res_temp.len()];
    for (i, p) in res_temp[..n / 2].into_iter().enumerate() {
        res[i + n / 2] = *p;
    }
    for (i, p) in res_temp[n / 2..].into_iter().enumerate() {
        res[i] = *p;
    }
    (res_conv, res)
}

#[pyfunction]
pub fn test_fft_3(op0: Vec<f64>, op1: Vec<f64>, op2: Vec<f64>) -> (Vec<f64>, Vec<f64>) {
    let mut res_conv = vec![0.0; op0.len() + op1.len() + op2.len()];
    let mut res_temp = vec![0.0; op0.len() + op1.len()];
    for (v0, p0) in op0.iter().enumerate() {
        for (v1, p1) in op1.iter().enumerate() {
            res_temp[v0 + v1] += p0 * p1;
        }
    }
    for (vp, pp) in res_temp.iter().enumerate() {
        for (v2, p2) in op2.iter().enumerate() {
            res_conv[vp + v2] += pp * p2;
        }
    }
    let n = 3 * op0.len();
    let mut planner = FftPlanner::new();
    let fft = planner.plan_fft_forward(n);
    let ifft = planner.plan_fft_inverse(n);
    let mut op0_fft_in = vec![Complex { re: 0.0, im: 0.0 }; n];
    let mut op1_fft_in = vec![Complex { re: 0.0, im: 0.0 }; n];
    let mut op2_fft_in = vec![Complex { re: 0.0, im: 0.0 }; n];
    for i in 0..op0.len() {
        let idx = if i < op0.len() / 2 {
            n + (i - op0.len() / 2)
        } else {
            i - op0.len() / 2
        };
        op0_fft_in[idx] = Complex {
            re: op0[i],
            im: 0.0,
        };
        op1_fft_in[idx] = Complex {
            re: op1[i],
            im: 0.0,
        };
        op2_fft_in[idx] = Complex {
            re: op2[i],
            im: 0.0,
        };
    }
    fft.process(&mut op0_fft_in);
    fft.process(&mut op1_fft_in);
    let res_fft: Vec<Complex<f64>> = op0_fft_in
        .iter()
        .zip(op1_fft_in.iter())
        .map(|(p0, p1)| p0 * p1)
        .collect();
    //ifft.process(&mut res_fft);

    //println!("{:?}\n", res_fft.iter().map(|p| p.re).collect::<Vec<f64>>());
    //println!("{:?}\n", op2_fft_in);
    //println!("{:?}\n\n", res_temp);
    //fft.process(&mut res_fft);
    //println!("FFT: {:?}\n\n", res_fft);
    //res_fft.iter_mut().for_each(|p| *p/=n as f64);
    fft.process(&mut op2_fft_in);
    //println!("FFT: {:?}\n\n", op2_fft_in);
    //op2_fft_in.iter_mut().for_each(|p| *p/=(n as f64).sqrt());

    let mut res_fft2: Vec<Complex<f64>> = res_fft
        .iter()
        .zip(op2_fft_in.iter())
        .map(|(p0, p1)| p0 * p1)
        .collect();

    //println!("FFT mult: {:?}\n\n", res_fft2);
    ifft.process(&mut res_fft2);
    //println!("IFFT mult: {:?}\n\n", res_fft2);

    let res_temp: Vec<f64> = res_fft2.iter().map(|c| c.re).collect();
    let mut res: Vec<f64> = vec![0.0; res_temp.len()];
    for (i, p) in res_temp[..n / 2].into_iter().enumerate() {
        res[i + n / 2] = *p / n as f64;
    }
    for (i, p) in res_temp[n / 2..].into_iter().enumerate() {
        res[i] = *p / n as f64;
    }
    (res_conv, res)
}
