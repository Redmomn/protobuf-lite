use crate::protobuf::WireType;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DecodeError {
    #[error("overflow 64bit")]
    OverFlow64Bit,

    #[error("overflow 32bit")]
    OverFlow32Bit,

    #[error("Unknown wire type: {0}")]
    UnknownWireType(u64),

    #[error("deprecated wire type: {0}")]
    DeprecatedWireType(WireType),

    #[error("unexpected EOF")]
    UnexpectedEof,

    #[error("EOF")]
    EOF,
}
