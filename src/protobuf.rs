#[derive(Debug)]
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
