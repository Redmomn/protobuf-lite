use crate::buffer::Reader;
use crate::error::DecodeError;
use crate::error::EncodeError::DataError;
use crate::fixint::{read_fix32, read_fix64, write_fix32, write_fix64};
use crate::json;
use crate::varint::{read_uvarint, write_uvarint};
use anyhow::Result;
use std::collections::btree_map::Entry;
use std::collections::BTreeMap;
use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};
use std::io::Write;
use std::mem::discriminant;
use std::ops::{Deref, DerefMut};
use std::str;

#[repr(u8)]
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

impl ProtoData {
    pub fn wire_type(&self) -> WireType {
        match self {
            ProtoData::Varint(_) => WireType::VARINT,
            ProtoData::Fix64(_) => WireType::I64,
            ProtoData::Fix32(_) => WireType::I32,
            _ => WireType::LEN,
        }
    }

    pub fn encode_to<T>(&self, field: u64, buf: &mut T) -> Result<()>
    where
        T: Write,
    {
        match self {
            ProtoData::Varint(v) => {
                write_uvarint((field << 3) | (self.wire_type() as u64), buf)?;
                write_uvarint(*v, buf)?;
            }
            ProtoData::Fix64(v) => {
                write_uvarint((field << 3) | (self.wire_type() as u64), buf)?;
                write_fix64(*v, buf)?;
            }
            ProtoData::Fix32(v) => {
                write_uvarint((field << 3) | (self.wire_type() as u64), buf)?;
                write_fix32(*v, buf)?
            }
            ProtoData::Bytes(v) => {
                write_uvarint((field << 3) | (self.wire_type() as u64), buf)?;
                write_uvarint(v.len() as u64, buf)?;
                buf.write_all(v.as_slice())?;
            }
            ProtoData::String(v) => {
                write_uvarint((field << 3) | (self.wire_type() as u64), buf)?;
                write_uvarint(v.len() as u64, buf)?;
                buf.write_all(v.as_bytes())?;
            }
            ProtoData::Repeated(v) => {
                let mut typ = None;
                if v.is_empty() {
                    return Ok(());
                }
                match v[0].wire_type() {
                    WireType::LEN => {}
                    _ => {
                        write_uvarint((field << 3) | (self.wire_type() as u64), buf)?;
                    }
                }
                for i in v {
                    let disc = discriminant(i);
                    if let Some(existing) = typ {
                        if existing != disc {
                            return Err(DataError.into());
                        }
                    } else {
                        typ = Some(disc);
                    }

                    if matches!(*i, ProtoData::Repeated(_)) {
                        return Err(DataError.into());
                    }
                    i.encode_repeated_to(field, buf)?
                }
            }
            ProtoData::Message(v) => v.encode_to(buf)?,
        }
        Ok(())
    }

    pub fn encode_repeated_to<T>(&self, field: u64, buf: &mut T) -> Result<()>
    where
        T: Write,
    {
        match self {
            ProtoData::Varint(v) => {
                write_uvarint(*v, buf)?;
            }
            ProtoData::Fix64(v) => {
                write_fix64(*v, buf)?;
            }
            ProtoData::Fix32(v) => write_fix32(*v, buf)?,
            ProtoData::Bytes(v) => {
                write_uvarint((field << 3) | (self.wire_type() as u64), buf)?;
                write_uvarint(v.len() as u64, buf)?;
                buf.write_all(v.as_slice())?;
            }
            ProtoData::String(v) => {
                write_uvarint((field << 3) | (self.wire_type() as u64), buf)?;
                write_uvarint(v.len() as u64, buf)?;
                buf.write_all(v.as_bytes())?;
            }
            ProtoData::Repeated(_) => {}
            ProtoData::Message(v) => v.encode_to(buf)?,
        }
        Ok(())
    }
}

macro_rules! impl_from {
    ($($t:ty => $variant:ident),*) => {
        $(
            impl From<$t> for ProtoData {
                fn from(v: $t) -> Self {
                    Self::$variant(v)
                }
            }
        )*
    };
}

macro_rules! impl_from_varint {
    ($($t:ty),*) => {
        $(
            impl From<$t> for ProtoData {
                fn from(v: $t) -> Self {
                    Self::Varint(v as u64)
                }
            }
        )*
    };
}

impl_from_varint!(u8, u16, u32, u64, i8, i16, i32, i64);

impl From<bool> for ProtoData {
    fn from(v: bool) -> Self {
        match v {
            true => Self::Varint(1),
            false => Self::Varint(0),
        }
    }
}

impl From<f32> for ProtoData {
    fn from(v: f32) -> Self {
        Self::Fix32(i32::from_le_bytes(v.to_le_bytes()))
    }
}

impl From<f64> for ProtoData {
    fn from(v: f64) -> Self {
        Self::Fix64(i64::from_le_bytes(v.to_le_bytes()))
    }
}

impl From<&str> for ProtoData {
    fn from(v: &str) -> Self {
        Self::String(v.to_string())
    }
}

impl_from!(Vec<u8> => Bytes, String => String, Vec<ProtoData> => Repeated, Map<u64, ProtoData> => Message);

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

#[derive(Debug)]
pub struct Map<K, V> {
    map: BTreeMap<K, V>,
}

impl Default for Map<u64, ProtoData> {
    fn default() -> Self {
        Map {
            map: BTreeMap::new(),
        }
    }
}

impl Deref for Map<u64, ProtoData> {
    type Target = BTreeMap<u64, ProtoData>;

    fn deref(&self) -> &Self::Target {
        &self.map
    }
}

impl DerefMut for Map<u64, ProtoData> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.map
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
            map: BTreeMap::new(),
        }
    }

    pub fn encode_to<T>(&self, buf: &mut T) -> Result<()>
    where
        T: Write,
    {
        for (&key, value) in self.iter() {
            value.encode_to(key, buf)?
        }
        Ok(())
    }

    pub fn encode(&self) -> Result<Vec<u8>> {
        let mut buf = Vec::new();
        self.encode_to(&mut buf)?;
        Ok(buf)
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
    if len == 0 {
        result.push(ProtoData::Message(Map::new()));
        return Ok(result);
    }

    let mut data_buf = Reader::new(buf.read_bytes(len as usize)?);

    // 优先protobuf
    loop {
        match decode_protobuf_from(&mut data_buf) {
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
    decode_protobuf_from(&mut Reader::new(
        hex::decode(data.replace(" ", ""))?.as_slice(),
    ))
}

pub fn decode_protobuf<T>(data: T) -> Result<ProtoData>
where
    T: AsRef<[u8]>,
{
    decode_protobuf_from(&mut Reader::new(data.as_ref()))
}

pub fn decode_protobuf_from<T>(buf: &mut Reader<T>) -> Result<ProtoData>
where
    T: AsRef<[u8]>,
{
    let mut parsed_data = Map::default();
    loop {
        match read_tag(buf) {
            Ok((field, wire_type)) => {
                let data = match wire_type {
                    WireType::VARINT => {
                        ProtoData::Varint(read_uvarint(buf).map_err(|_| DecodeError::Error)?)
                    }
                    WireType::I64 => {
                        ProtoData::Fix64(read_fix64(buf).map_err(|_| DecodeError::Error)?)
                    }
                    WireType::I32 => {
                        ProtoData::Fix32(read_fix32(buf).map_err(|_| DecodeError::Error)?)
                    }
                    WireType::LEN => {
                        let mut list =
                            read_length_delimited(buf).map_err(|_| DecodeError::Error)?;
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

                match parsed_data.entry(field) {
                    Entry::Occupied(mut entry) => match entry.get_mut() {
                        ProtoData::Repeated(list) => list.push(data),
                        existing => {
                            *existing = ProtoData::Repeated(vec![existing.clone(), data]);
                        }
                    },
                    Entry::Vacant(entry) => {
                        entry.insert(data);
                    }
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
