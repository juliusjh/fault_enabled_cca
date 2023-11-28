// mod bin_tree;
mod check_graph;
mod check_msg;
mod check_nodes;

use crate::check_graph::{test_fft_2, test_fft_3};
use pyo3::prelude::*;
use pyo3::wrap_pyfunction;

#[pymodule]
#[cfg(feature = "kyber1024")]
fn check_bp1024(_: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<check_graph::CheckGraph>()?;
    m.add_function(wrap_pyfunction!(test_fft_2, m)?).unwrap();
    m.add_function(wrap_pyfunction!(test_fft_3, m)?).unwrap();
    m.add_class::<check_graph::PyCmpOperator>()?;
    Ok(())
}

#[pymodule]
#[cfg(feature = "kyber768")]
fn check_bp768(_: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<check_graph::CheckGraph>()?;
    m.add_function(wrap_pyfunction!(test_fft_2, m)?).unwrap();
    m.add_function(wrap_pyfunction!(test_fft_3, m)?).unwrap();
    m.add_class::<check_graph::PyCmpOperator>()?;
    Ok(())
}

#[pymodule]
#[cfg(feature = "kyber512")]
fn check_bp512(_: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<check_graph::CheckGraph>()?;
    m.add_function(wrap_pyfunction!(test_fft_2, m)?).unwrap();
    m.add_function(wrap_pyfunction!(test_fft_3, m)?).unwrap();
    m.add_class::<check_graph::PyCmpOperator>()?;
    Ok(())
}
