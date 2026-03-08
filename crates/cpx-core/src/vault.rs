use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VaultHandle {
    pub case_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum VaultError {
    #[error("vault support is not implemented yet")]
    NotYetImplemented,
}

pub fn open(_case_id: &str) -> Result<VaultHandle, VaultError> {
    Err(VaultError::NotYetImplemented)
}

