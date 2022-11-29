use matrix_sdk::ruma::RoomId;
use pyo3::prelude::*;
use std::{collections::HashMap, fs::File, io::Write, path::PathBuf};
use vodozemac::megolm::ExportedSessionKey;

use matrix_sdk_crypto::{
    decrypt_room_key_export as rust_decrypt, encrypt_room_key_export as rust_encrypt,
    olm::{Curve25519PublicKey, ExportedRoomKey},
    types::SigningKeys,
};

#[pyclass]
#[derive(Debug, Clone)]
pub struct RoomKey {
    #[pyo3(get)]
    pub algorithm: String,
    #[pyo3(get)]
    pub room_id: String,
    #[pyo3(get)]
    pub sender_key: String,
    #[pyo3(get)]
    pub session_id: String,
    #[pyo3(get)]
    pub session_key: String,
    #[pyo3(get)]
    pub sender_claimed_keys: HashMap<String, String>,
    #[pyo3(get)]
    pub forwarding_curve25519_key_chain: Vec<String>,
}

impl From<ExportedRoomKey> for RoomKey {
    fn from(k: ExportedRoomKey) -> Self {
        Self {
            algorithm: k.algorithm.to_string(),
            room_id: k.room_id.to_string(),
            sender_key: k.sender_key.to_base64(),
            session_id: k.session_id,
            session_key: k.session_key.to_base64(),
            forwarding_curve25519_key_chain: k
                .forwarding_curve25519_key_chain
                .into_iter()
                .map(|k| k.to_base64())
                .collect(),
            sender_claimed_keys: k
                .sender_claimed_keys
                .into_iter()
                .map(|(k, v)| (k.to_string(), v.to_base64()))
                .collect(),
        }
    }
}

impl TryInto<ExportedRoomKey> for RoomKey {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<ExportedRoomKey, Self::Error> {
        Ok(ExportedRoomKey {
            algorithm: self.algorithm.into(),
            room_id: RoomId::parse(self.room_id)?,
            sender_key: Curve25519PublicKey::from_base64(&self.sender_key)?,
            session_id: self.session_id,
            session_key: ExportedSessionKey::from_base64(&self.session_key)?,
            sender_claimed_keys: SigningKeys::new(),
            forwarding_curve25519_key_chain: vec![],
        })
    }
}

#[pyfunction]
pub fn decrypt_key_export(path: &str, passphrase: &str) -> anyhow::Result<Vec<RoomKey>> {
    let path = PathBuf::from(path);
    let file = File::open(path)?;

    let keys = rust_decrypt(file, passphrase)?;

    Ok(keys.into_iter().map(Into::into).collect())
}

#[pyfunction]
pub fn encrypt_key_export(keys: Vec<RoomKey>, path: &str, passphrase: &str) -> anyhow::Result<()> {
    let mut file = File::create(path)?;

    let keys: Vec<_> = keys
        .into_iter()
        .filter_map(|k| TryInto::try_into(k).ok())
        .collect();

    let out = rust_encrypt(&keys, passphrase, 10)?;

    file.write_all(out.as_bytes())?;

    Ok(())
}
