use pyo3::prelude::*;

mod client;
mod client_builder;

#[pymodule]
fn nio_rust(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<client_builder::ClientBuilder>()?;
    m.add_class::<client::Client>()?;

    Ok(())
}
