use crate::PersistenceResult;
use sha2::{Digest, Sha256};

pub(crate) fn sha256_json<T: serde::Serialize + ?Sized>(value: &T) -> PersistenceResult<String> {
    let bytes = serde_json::to_vec(value)?;
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    Ok(hex::encode(hasher.finalize()))
}
