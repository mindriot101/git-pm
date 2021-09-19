use thiserror::Error;

#[derive(Error, Debug)]
pub enum PmError {
    #[error("index already exists")]
    IndexExists,
}
