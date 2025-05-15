#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    Tun(#[from] tun::Error),
    
    #[error("{0}")]
    Io(#[from] std::io::Error),

    #[error("{0}")]
    Protocol(#[from] protocol::error::Error),
}

pub type Result<T> = std::result::Result<T, Error>;