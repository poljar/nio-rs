use std::time::Duration;

use pyo3::prelude::*;

#[pyclass]
#[derive(Clone, Debug)]
pub struct SyncSettings {
    pub timeout: Option<Duration>,
    #[pyo3(get, set)]
    pub token: Option<String>,
    #[pyo3(get, set)]
    pub full_state: bool,
}

impl Default for SyncSettings {
    fn default() -> Self {
        Self {
            timeout: Some(Duration::from_secs(30)),
            token: Default::default(),
            full_state: Default::default(),
        }
    }
}

#[pymethods]
impl SyncSettings {
    #[new]
    pub fn new() -> Self {
        Default::default()
    }

    #[getter]
    pub fn timeout(&self) -> u64 {
        self.timeout
            .as_ref()
            .map(|t| t.as_secs())
            .unwrap_or_default()
    }

    #[setter]
    pub fn set_timeout(&mut self, timeout: u64) {
        self.timeout = Some(Duration::from_secs(timeout))
    }
}

impl<'a> From<SyncSettings> for matrix_sdk::config::SyncSettings<'a> {
    fn from(s: SyncSettings) -> Self {
        let settings = matrix_sdk::config::SyncSettings::default().full_state(s.full_state);

        let settings = if let Some(timeout) = s.timeout {
            settings.timeout(timeout)
        } else {
            settings
        };

        let settings = if let Some(token) = s.token {
            settings.token(token)
        } else {
            settings
        };

        settings
    }
}
