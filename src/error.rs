use thiserror::Error;

#[derive(Debug, Error)]
pub enum DecodeError {
    #[error("overflow 64bit")]
    OverFlow64Bit,
    #[error("overflow 32bit")]
    OverFlow32Bit,
}
