use crate::buffer::Reader;
use crate::error::DecodeError;
use crate::fixint::{read_fix32, read_fix64};
use crate::varint::read_uvarint;
use anyhow::Result;
use std::fmt::{Display, Formatter};
use std::str;

#[derive(Debug, PartialEq, Clone)]
pub enum WireType {
    VARINT = 0, // int32, int64, uint32, uint64, sint32, sint64, bool, enum
    I64 = 1,    // fixed64, sfixed64, double
    LEN = 2,    // string, bytes, embedded messages, packed repeated fields
    #[deprecated]
    SGROUP = 3, // group start (deprecated)
    #[deprecated]
    EGROUP = 4, // group end (deprecated)
    I32 = 5,    // fixed32, sfixed32, float
}

impl Display for WireType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        #[allow(deprecated)]
        match self {
            WireType::VARINT => {
                write!(f, "varint")
            }
            WireType::I64 => {
                write!(f, "64-bit")
            }
            WireType::LEN => {
                write!(f, "length-delimited")
            }
            WireType::SGROUP => {
                write!(f, "start group")
            }
            WireType::EGROUP => {
                write!(f, "end group")
            }
            WireType::I32 => {
                write!(f, "32-bit")
            }
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum DataType {
    Varint(u64),
    Fix64(i64),
    Fix32(i32),
    Bytes(Vec<u8>),
    String(String),
    Repeated(Vec<DataType>),
    Message(Vec<ProtoData>),
}

#[derive(Debug, PartialEq, Clone)]
pub struct ProtoData {
    pub field: u64,
    pub wire_type: WireType,
    pub data: DataType,
}

impl TryFrom<u64> for WireType {
    type Error = DecodeError;

    #[allow(deprecated)]
    fn try_from(wire_type: u64) -> Result<Self, Self::Error> {
        match wire_type {
            0 => Ok(WireType::VARINT),
            1 => Ok(WireType::I64),
            2 => Ok(WireType::LEN),
            3 => Ok(WireType::SGROUP),
            4 => Ok(WireType::EGROUP),
            5 => Ok(WireType::I32),
            _ => Err(DecodeError::UnknownWireType(wire_type)),
        }
    }
}

pub fn read_tag(buf: &mut Reader) -> Result<(u64, WireType)> {
    let tag = read_uvarint(buf)?;
    Ok((tag >> 3, WireType::try_from(tag & 0x07)?))
}

pub fn read_length_delimited(buf: &mut Reader) -> Result<Vec<DataType>> {
    let mut result: Vec<DataType> = Vec::new();
    let len = read_uvarint(buf)?;

    let mut data_buf = Reader::new(buf.read_bytes(len as usize)?);

    // 优先protobuf
    loop {
        match decode_protobuf(&mut data_buf) {
            Ok(v) => result.push(DataType::Message(v)),
            Err(err) => match err.downcast_ref::<DecodeError>() {
                Some(DecodeError::EOF) => return Ok(result),
                _ => {
                    result.clear();
                    data_buf.reset();
                    break;
                }
            },
        };
    }

    // 转为str 可能会把varint也转换成str
    match str::from_utf8(data_buf.read_all_bytes()?) {
        Ok(v) => {
            result.push(DataType::String(v.to_string()));
            return Ok(result);
        }
        _ => {
            result.clear();
            data_buf.reset();
        }
    }

    // 转为varint数组
    // loop {
    //     match read_uvarint(&mut data_buf) {
    //         Ok(v) => result.push(DataType::Varint(v)),
    //         Err(err) => {
    //             if let Some(DecodeError::EOF) = err.downcast_ref::<DecodeError>() {
    //                 return Ok(result);
    //             } else {
    //                 result.clear();
    //                 data_buf.reset();
    //                 break;
    //             }
    //         }
    //     }
    // }
    result.push(DataType::Bytes(Vec::from(data_buf.read_all_bytes()?)));

    Ok(result)
}

pub fn decode_protobuf(buf: &mut Reader) -> Result<Vec<ProtoData>> {
    let mut parsed_data = Vec::<ProtoData>::new();
    loop {
        match read_tag(buf) {
            Ok((field, wire_type)) => {
                let data = match wire_type {
                    WireType::VARINT => DataType::Varint(read_uvarint(buf)?),
                    WireType::I64 => DataType::Fix64(read_fix64(buf)?),
                    WireType::I32 => DataType::Fix32(read_fix32(buf)?),
                    WireType::LEN => {
                        let mut list = read_length_delimited(buf)?;
                        if list.len() > 1 {
                            DataType::Repeated(list)
                        } else {
                            list.remove(0)
                        }
                    }
                    x => return Err(DecodeError::DeprecatedWireType(x).into()),
                };
                parsed_data.push(ProtoData {
                    field,
                    wire_type,
                    data,
                });
            }
            Err(err) => match err.downcast_ref::<DecodeError>() {
                Some(DecodeError::EOF) => break,
                _ => return Err(err),
            },
        }
    }
    Ok(parsed_data)
}
