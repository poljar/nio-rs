use pyo3::prelude::*;

mod client;
mod client_builder;
mod encryption;
mod sync_settings;

#[pymodule]
fn nio_rust(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<client_builder::ClientBuilder>()?;
    m.add_class::<client::Client>()?;
    m.add_class::<sync_settings::SyncSettings>()?;
    m.add_class::<encryption::RoomKey>()?;
    m.add_function(wrap_pyfunction!(encryption::decrypt_key_export, m)?)?;
    m.add_function(wrap_pyfunction!(encryption::encrypt_key_export, m)?)?;

    Ok(())
}
