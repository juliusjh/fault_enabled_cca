use pyo3::prelude::*;

#[allow(unused)]
pub mod kyber;
pub use kyber::*;

#[pymodule]
fn python_kyber(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<Ciphertext>()?;
    m.add_class::<SecretKey>()?;
    m.add_class::<PublicKey>()?;
    m.add_class::<KyberSample>()?;
    m.add_class::<Poly>()?;
    m.add_class::<Polyvec>()?;
    m.add_class::<kyber::constants::KyberConstants>()?;
    Ok(())
}
