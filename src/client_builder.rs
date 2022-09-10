use pyo3::{exceptions::PyValueError, prelude::*};
use std::sync::{Arc, Mutex};
use thiserror::Error;

use crate::client::Client;

#[pyclass]
pub struct ClientBuilder(Arc<Mutex<Option<matrix_sdk::ClientBuilder>>>);

#[derive(Error, Debug)]
#[error("Builder has been used")]
enum ClientBuilderError {
    #[error("Builder has been used")]
    UsedUp,
    Inner(matrix_sdk::ClientBuildError),
}

impl ClientBuilder {
    fn wrap(builder: matrix_sdk::ClientBuilder) -> Self {
        Self(Mutex::new(Some(builder)).into())
    }
}

impl From<ClientBuilderError> for PyErr {
    fn from(error: ClientBuilderError) -> Self {
        PyValueError::new_err(error.to_string())
    }
}

#[pymethods]
impl ClientBuilder {
    #[new]
    pub fn new() -> Self {
        Self::wrap(matrix_sdk::Client::builder())
    }

    pub fn homeserver_url(&self, homeserver_url: &str) -> PyResult<Self> {
        Ok(Self::wrap(
            self.0
                .lock()
                .unwrap()
                .take()
                .ok_or(ClientBuilderError::UsedUp)?
                .homeserver_url(homeserver_url),
        ))
    }

    pub fn store(&self, path: &str, passphrase: &str) -> PyResult<Self> {
        Ok(Self::wrap(
            self.0
                .lock()
                .unwrap()
                .take()
                .ok_or(ClientBuilderError::UsedUp)?
                .sled_store(path, Some(passphrase))
                .map_err(anyhow::Error::from)?,
        ))
    }

    pub fn proxy(&self, url: &str) -> PyResult<Self> {
        Ok(Self::wrap(
            self.0
                .lock()
                .unwrap()
                .take()
                .ok_or(ClientBuilderError::UsedUp)?
                .proxy(url),
        ))
    }

    pub fn disable_ssl_verification(&self) -> PyResult<Self> {
        Ok(Self::wrap(
            self.0
                .lock()
                .unwrap()
                .take()
                .ok_or(ClientBuilderError::UsedUp)?
                .disable_ssl_verification(),
        ))
    }

    pub fn build<'a>(&'a self, py: Python<'a>) -> PyResult<&PyAny> {
        let future = self
            .0
            .lock()
            .unwrap()
            .take()
            .ok_or(anyhow::anyhow!("Builder has been used"))?
            .build();

        pyo3_asyncio::tokio::future_into_py(py, async {
            let client = future.await.map_err(ClientBuilderError::Inner)?;
            let client = Client(client);
            Ok(client)
        })
    }
}
