pub mod buffer;
pub mod error;
pub mod fixint;
mod json;
pub mod protobuf;
pub mod varint;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::buffer::Reader;
    use crate::protobuf::{decode_protobuf_from, Map, ProtoData};
    use std::vec;

    #[test]
    fn test_encode_protobuf() {
        let mut pb = Map::new();
        pb.extend([
            (1, 2.into()),
            (2, "hello".into()),
            (
                3,
                vec![
                    ProtoData::String("hello".into()),
                    ProtoData::String("proto".into()),
                ]
                .into(),
            ),
            (4, vec![ProtoData::Varint(1), 2.into()].into()),
        ]);

        let data = pb.encode().unwrap();
        println!("{}", hex::encode(data.as_slice()));
    }

    #[test]
    fn test_decode_protobuf() {
        let pb = read_protobuf(
            "08d2fe061d424b1d002952bf0100000000003a0d3131343531343139313938313042047468697342026973420161420872657065617465644206737472696e674a10ec9962d372ce9be816be0b7fdea0127b5210292c88aab6b4386c5259b3db5bb3d83552106c67dcd74452b07dbab383af60213edc5210625b2652e828a520e2d3738fbc7623f952105c4627267501260e1635696f21df442352100e4f254dabe6afe218d47229213fce2e5210d61fea3ad5f6c7de29cb530974daebe85210bd7969536028618e616db54cc5bc17ab5210b82c3166710a0373d0fae7bfa5da6a405210aacb0a4e4eb2400cf3a9674cbd9fac30521056b3e167c737ac1d52770b789ba1e04a5210d99426dc60b67af15121a38816fa33c35210328fce15be2219c6cd950241d9db083652102db0c75a59f5527584b2924a32995f625210258d410b8fcf29b7c8405ae99560d43a5210eeafec9d7d49109a08d5d36f26e5f94852107aa31e3a68c0ea5cf197a9613a68de46",
        )
            .unwrap();

        println!("{}", pb);

        let mut expect_pb = Map::new();
        expect_pb.insert(1, 114514.into());
        expect_pb.insert(3, ProtoData::Fix32(1919810));
        expect_pb.insert(5, ProtoData::Fix64(114514));
        expect_pb.insert(7, "1145141919810".to_string().into());
        expect_pb.insert(
            8,
            ProtoData::Repeated(vec![
                "this".to_string().into(),
                "is".to_string().into(),
                "a".to_string().into(),
                "repeated".to_string().into(),
                "string".to_string().into(),
            ]),
        );
        expect_pb.insert(
            9,
            hex::decode("ec 99 62 d3 72 ce 9b e8 16 be 0b 7f de a0 12 7b".replace(" ", ""))
                .unwrap()
                .into(),
        );
        expect_pb.insert(
            10,
            ProtoData::Repeated(vec![
                hex::decode("29 2c 88 aa b6 b4 38 6c 52 59 b3 db 5b b3 d8 35".replace(" ", ""))
                    .unwrap()
                    .into(),
                hex::decode("6c 67 dc d7 44 52 b0 7d ba b3 83 af 60 21 3e dc".replace(" ", ""))
                    .unwrap()
                    .into(),
                hex::decode("62 5b 26 52 e8 28 a5 20 e2 d3 73 8f bc 76 23 f9".replace(" ", ""))
                    .unwrap()
                    .into(),
                hex::decode("5c 46 27 26 75 01 26 0e 16 35 69 6f 21 df 44 23".replace(" ", ""))
                    .unwrap()
                    .into(),
                hex::decode("0e 4f 25 4d ab e6 af e2 18 d4 72 29 21 3f ce 2e".replace(" ", ""))
                    .unwrap()
                    .into(),
                hex::decode("d6 1f ea 3a d5 f6 c7 de 29 cb 53 09 74 da eb e8".replace(" ", ""))
                    .unwrap()
                    .into(),
                hex::decode("bd 79 69 53 60 28 61 8e 61 6d b5 4c c5 bc 17 ab".replace(" ", ""))
                    .unwrap()
                    .into(),
                hex::decode("b8 2c 31 66 71 0a 03 73 d0 fa e7 bf a5 da 6a 40".replace(" ", ""))
                    .unwrap()
                    .into(),
                hex::decode("aa cb 0a 4e 4e b2 40 0c f3 a9 67 4c bd 9f ac 30".replace(" ", ""))
                    .unwrap()
                    .into(),
                hex::decode("56 b3 e1 67 c7 37 ac 1d 52 77 0b 78 9b a1 e0 4a".replace(" ", ""))
                    .unwrap()
                    .into(),
                hex::decode("d9 94 26 dc 60 b6 7a f1 51 21 a3 88 16 fa 33 c3".replace(" ", ""))
                    .unwrap()
                    .into(),
                hex::decode("32 8f ce 15 be 22 19 c6 cd 95 02 41 d9 db 08 36".replace(" ", ""))
                    .unwrap()
                    .into(),
                hex::decode("2d b0 c7 5a 59 f5 52 75 84 b2 92 4a 32 99 5f 62".replace(" ", ""))
                    .unwrap()
                    .into(),
                hex::decode("25 8d 41 0b 8f cf 29 b7 c8 40 5a e9 95 60 d4 3a".replace(" ", ""))
                    .unwrap()
                    .into(),
                hex::decode("ee af ec 9d 7d 49 10 9a 08 d5 d3 6f 26 e5 f9 48".replace(" ", ""))
                    .unwrap()
                    .into(),
                hex::decode("7a a3 1e 3a 68 c0 ea 5c f1 97 a9 61 3a 68 de 46".replace(" ", ""))
                    .unwrap()
                    .into(),
            ]),
        );

        assert_eq!(pb, expect_pb.into());
    }

    fn read_protobuf(hex_str: &str) -> anyhow::Result<ProtoData> {
        let bytes = hex::decode(hex_str.replace(" ", ""))?;
        decode_protobuf_from(&mut Reader::new(&bytes.as_slice()))
    }

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
