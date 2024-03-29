use matrix_sdk::LoopCtrl;
use pyo3::prelude::*;
use pythonize::pythonize;

use crate::sync_settings::SyncSettings;

#[pyclass]
pub struct Client(pub matrix_sdk::Client);

#[pymethods]
impl Client {
    #[getter]
    pub fn user_id(&self) -> Option<&str> {
        self.0.user_id().map(|u| u.as_str())
    }

    #[getter]
    pub fn device_id(&self) -> Option<&str> {
        self.0.device_id().map(|u| u.as_str())
    }

    pub fn sync_token<'a>(&'a self, py: Python<'a>) -> PyResult<&PyAny> {
        let client = self.0.clone();

        pyo3_asyncio::tokio::future_into_py(py, async move { Ok(client.sync_token().await) })
    }

    pub fn homeserver_ulr<'a>(&'a self, py: Python<'a>) -> PyResult<&PyAny> {
        let client = self.0.clone();

        pyo3_asyncio::tokio::future_into_py(py, async move {
            let homeserver = client.homeserver().await;
            Ok(homeserver.to_string())
        })
    }

    pub fn login<'a>(
        &'a self,
        py: Python<'a>,
        user_id: String,
        password: String,
        device_id: Option<String>,
        device_display_name: Option<String>,
    ) -> PyResult<&PyAny> {
        let client = self.0.clone();

        pyo3_asyncio::tokio::future_into_py(py, async move {
            let builder = client.login_username(&user_id, &password);

            let builder = if let Some(device_id) = &device_id {
                builder.device_id(device_id)
            } else {
                builder
            };

            let builder = if let Some(display_name) = &device_display_name {
                builder.initial_device_display_name(display_name)
            } else {
                builder
            };

            builder.send().await.map_err(anyhow::Error::from)?;

            Ok(())
        })
    }

    pub fn sync_once<'a>(
        &'a self,
        py: Python<'a>,
        sync_settings: SyncSettings,
    ) -> PyResult<&PyAny> {
        let client = self.0.clone();

        pyo3_asyncio::tokio::future_into_py(py, async move {
            let response = client
                .sync_once(sync_settings.into())
                .await
                .map_err(anyhow::Error::from)?;

            Python::with_gil(|py| -> PyResult<_> { Ok(pythonize(py, &response)?) })
        })
    }

    #[args(sync_settings = "SyncSettings::default()", callbac = "None")]
    pub fn sync<'a>(
        &'a self,
        py: Python<'a>,
        sync_settings: SyncSettings,
        callback: Option<PyObject>,
    ) -> PyResult<&PyAny> {
        let client = self.0.clone();

        if let Some(callback) = callback {
            pyo3_asyncio::tokio::future_into_py(py, async move {
                client
                    .sync_with_callback(sync_settings.into(), {
                        let callback = callback.clone();
                        move |response| {
                            let callback = callback.clone();

                            async move {
                                let future = Python::with_gil(|py| -> PyResult<_> {
                                    let response = pythonize(py, &response)?;

                                    let coroutine = callback.call(py, (response,), None)?;
                                    let coroutine = coroutine.as_ref(py);

                                    let future = pyo3_asyncio::tokio::into_future(coroutine)?;

                                    Ok(future)
                                });

                                // TODO log some of those errors or perhaps raise an exception.
                                if let Ok(f) = future {
                                    if let Ok(_) = f.await {
                                        LoopCtrl::Continue
                                    } else {
                                        LoopCtrl::Break
                                    }
                                } else {
                                    LoopCtrl::Break
                                }
                            }
                        }
                    })
                    .await
                    .expect("Sync stopped");

                Ok(())
            })
        } else {
            pyo3_asyncio::tokio::future_into_py(py, async move {
                client
                    .sync(sync_settings.into())
                    .await
                    .expect("Sync stopped");
                Ok(())
            })
        }
    }
}
