use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Tera error")]
    TeraError(#[from] tera::Error),
    #[error("Format error")]
    FormatError(String),
    #[error("Unknown error")]
    Unknown,
}