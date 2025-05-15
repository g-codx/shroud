#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    Bincode(#[from] bincode::Error),
    #[error("{0}")]
    InvalidKeyLength(#[from] aes_gcm::aes::cipher::InvalidLength),
    #[error("{0}")]
    AesGcm(#[from] aes_gcm::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
