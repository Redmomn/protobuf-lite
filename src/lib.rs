pub mod buffer;
pub mod error;
pub mod fixint;
pub mod protobuf;
pub mod varint;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixint() {
        let nums: Vec<i64> = vec![-100, -10, 0, 10, 100];
        for &num in &nums {
            let data = fixint::encode_fix64(num);
            assert_eq!(
                num,
                fixint::read_fix64(&mut Reader::new(data.as_slice())).unwrap()
            )
        }

        for &num in &nums {
            let data = fixint::encode_fix32(num as i32);
            assert_eq!(
                num as i32,
                fixint::read_fix32(&mut Reader::new(data.as_slice())).unwrap()
            )
        }
    }

    #[test]
    fn uvarint() {
        let nums: Vec<u64> = vec![0, 10, 1000, 10000, 123456, 128, 256, 512];
        for num in nums {
            let data = varint::encode_uvarint(num);
            assert_eq!(
                num,
                varint::read_uvarint(&mut Reader::new(data.as_slice())).unwrap()
            );
        }

        let data: Vec<u8> = vec![192, 196, 7];
        assert_eq!(
            123456,
            varint::read_uvarint(&mut Reader::new(data.as_slice())).unwrap()
        );

        let data: Vec<u8> = vec![192, 196, 7, 1, 1, 1, 1, 1, 1, 1, 1];
        assert_eq!(
            123456,
            varint::read_uvarint(&mut Reader::new(data.as_slice())).unwrap()
        );
    }

    #[test]
    fn varint() {
        let nums: Vec<i64> = vec![
            0, 10, 1000, 10000, 128, 256, 512, -10, -1000, -10000, -128, -256, -512,
        ];
        for num in nums {
            let data = varint::encode_varint(num);
            assert_eq!(
                num,
                varint::read_varint(&mut Reader::new(data.as_slice())).unwrap()
            );
        }
    }
}
