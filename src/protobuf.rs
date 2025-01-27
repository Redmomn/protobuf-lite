use crate::buffer::Reader;
use crate::error::DecodeError;
use crate::fixint::{read_fix32, read_fix64};
use crate::json;
use crate::varint::read_uvarint;
use anyhow::Result;
use std::collections::BTreeMap;
use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};
use std::str;

#[derive(Debug, Eq, PartialEq, Clone)]
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

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub enum ProtoData {
    Varint(u64),
    Fix64(i64),
    Fix32(i32),
    Bytes(Vec<u8>),
    String(String),
    Repeated(Vec<ProtoData>),
    Message(Map<u64, ProtoData>),
}

impl Display for ProtoData {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ProtoData::Varint(v) => {
                write!(f, "{}", v)
            }
            ProtoData::Fix64(v) => {
                write!(f, "{}", v)
            }
            ProtoData::Fix32(v) => {
                write!(f, "{}", v)
            }
            ProtoData::Bytes(v) => {
                write!(f, "\"{}\"", hex::encode(v))
            }
            ProtoData::String(v) => {
                write!(f, "\"{}\"", json::escape_string(v))
            }
            ProtoData::Repeated(v) => {
                write!(f, "[")?;
                for (i, item) in v.iter().enumerate() {
                    if i != 0 && i != v.len() {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", item)?;
                }
                write!(f, "]")
            }
            ProtoData::Message(v) => {
                write!(f, "{{")?;
                for (i, (key, value)) in v.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "\"{}\": {}", key, value)?;
                }
                write!(f, "}}")
            }
        }
    }
}

type MapImpl<K, V> = BTreeMap<K, V>;

#[derive(Debug)]
pub struct Map<K, V> {
    map: MapImpl<K, V>,
}

impl Default for Map<u64, ProtoData> {
    fn default() -> Self {
        Map {
            map: BTreeMap::new(),
        }
    }
}

impl Clone for Map<u64, ProtoData> {
    #[inline]
    fn clone(&self) -> Self {
        Map {
            map: self.map.clone(),
        }
    }

    #[inline]
    fn clone_from(&mut self, source: &Self) {
        self.map.clone_from(&source.map);
    }
}

impl PartialEq for Map<u64, ProtoData> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.map.eq(&other.map)
    }
}

impl Eq for Map<u64, ProtoData> {}

impl Hash for Map<u64, ProtoData> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.map.hash(state);
    }
}

impl Map<u64, ProtoData> {
    pub fn new() -> Self {
        Map {
            map: MapImpl::new(),
        }
    }

    pub fn clear(&mut self) {
        self.map.clear();
    }

    pub fn len(&self) -> usize {
        self.map.len()
    }

    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    pub fn contains_key(&self, key: u64) -> bool {
        self.map.contains_key(&key)
    }

    pub fn get(&self, key: u64) -> Option<&ProtoData> {
        self.map.get(&key)
    }

    pub fn get_mut(&mut self, key: u64) -> Option<&mut ProtoData> {
        self.map.get_mut(&key)
    }

    pub fn insert(&mut self, key: u64, message: ProtoData) -> Option<ProtoData> {
        self.map.insert(key, message)
    }

    pub fn remove(&mut self, key: u64) -> Option<ProtoData> {
        self.map.remove(&key)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&u64, &ProtoData)> {
        self.map.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&u64, &mut ProtoData)> {
        self.map.iter_mut()
    }

    pub fn keys(&self) -> impl Iterator<Item = &u64> {
        self.map.keys()
    }

    pub fn values(&self) -> impl Iterator<Item = &ProtoData> {
        self.map.values()
    }

    pub fn values_mut(&mut self) -> impl Iterator<Item = &mut ProtoData> {
        self.map.values_mut()
    }
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

pub fn read_tag<T>(buf: &mut Reader<T>) -> Result<(u64, WireType)>
where
    T: AsRef<[u8]>,
{
    let tag = read_uvarint(buf)?;
    Ok((tag >> 3, WireType::try_from(tag & 0x07)?))
}

pub fn read_length_delimited<T>(buf: &mut Reader<T>) -> Result<Vec<ProtoData>>
where
    T: AsRef<[u8]>,
{
    let mut result: Vec<ProtoData> = Vec::new();
    let len = read_uvarint(buf)?;

    let mut data_buf = Reader::new(buf.read_bytes(len as usize)?);

    // 优先protobuf
    loop {
        match decode_protobuf(&mut data_buf) {
            Ok(v) => match v {
                ProtoData::Message(msg) => {
                    if msg.len() > 0 {
                        result.push(ProtoData::Message(msg));
                        return Ok(result);
                    }
                }
                _ => {}
            },
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
            result.push(ProtoData::String(v.to_string()));
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
    result.push(ProtoData::Bytes(Vec::from(data_buf.read_all_bytes()?)));

    Ok(result)
}

pub fn decode_protobuf_hex(data: &str) -> Result<ProtoData> {
    decode_protobuf(&mut Reader::new(
        hex::decode(data.replace(" ", ""))?.as_slice(),
    ))
}

pub fn decode_protobuf<T>(buf: &mut Reader<T>) -> Result<ProtoData>
where
    T: AsRef<[u8]>,
{
    let mut parsed_data = Map::default();
    loop {
        match read_tag(buf) {
            Ok((field, wire_type)) => {
                let data = match wire_type {
                    WireType::VARINT => ProtoData::Varint(read_uvarint(buf)?),
                    WireType::I64 => ProtoData::Fix64(read_fix64(buf)?),
                    WireType::I32 => ProtoData::Fix32(read_fix32(buf)?),
                    WireType::LEN => {
                        let mut list = read_length_delimited(buf)?;
                        match list.len() {
                            0 => {
                                return Err(DecodeError::Error.into());
                            }
                            1 => list.remove(0),
                            _ => ProtoData::Repeated(list),
                        }
                    }
                    x => return Err(DecodeError::DeprecatedWireType(x).into()),
                };
                if parsed_data.contains_key(field) {
                    match parsed_data.get(field) {
                        None => {}
                        Some(ProtoData::Repeated(_)) => {
                            if let Some(ProtoData::Repeated(list)) = parsed_data.get_mut(field) {
                                list.push(data);
                            }
                        }
                        Some(v) => {
                            parsed_data.insert(field, ProtoData::Repeated(vec![v.clone(), data]));
                        }
                    };
                } else {
                    parsed_data.insert(field, data);
                }
            }
            Err(err) => match err.downcast_ref::<DecodeError>() {
                Some(DecodeError::EOF) => break,
                _ => return Err(err),
            },
        }
    }
    Ok(ProtoData::Message(parsed_data))
}
