pub mod ingest;
pub mod project;
pub mod rehydrate;
pub mod symbolize;
pub mod vault;

pub const FORMAT_VERSION: &str = "cpx-v1";

pub const EXIT_CODE_SUCCESS: i32 = 0;
pub const EXIT_CODE_GENERAL_ERROR: i32 = 1;
pub const EXIT_CODE_SAFETY_FAILURE: i32 = 2;
pub const EXIT_CODE_VAULT_ERROR: i32 = 3;
pub const EXIT_CODE_INPUT_ERROR: i32 = 4;
pub const EXIT_CODE_FORMAT_MISMATCH: i32 = 5;
