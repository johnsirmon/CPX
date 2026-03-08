use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RehydrateRequest {
    pub projection_response: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum RehydrateError {
    #[error("rehydration is not implemented yet")]
    NotYetImplemented,
}

pub fn rehydrate(_request: &RehydrateRequest) -> Result<String, RehydrateError> {
    Err(RehydrateError::NotYetImplemented)
}

