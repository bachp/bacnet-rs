pub mod application;
pub mod encoding;
pub mod network;
pub mod transport;

pub trait Decode<S: Decode = Self> {
    fn decode<T: std::io::Read + Sized>(reader: &mut T) -> std::io::Result<S>;

    fn decode_slice(slice: &[u8]) -> std::io::Result<S> {
        let mut reader = std::io::Cursor::new(slice);
        S::decode(&mut reader)
    }
}

pub trait Encode {
    fn encode<T: std::io::Write + Sized>(&self, writer: &mut T) -> std::io::Result<()>;

    fn encode_vec(&self) -> std::io::Result<Vec<u8>> {
        let mut v = Vec::with_capacity(self.len());
        self.encode(&mut v)?;
        Ok(v)
    }

    fn len(&self) -> usize;
}

#[cfg(test)]
mod tests {
    use crate::{Decode, Encode};

    #[derive(Clone, Debug, Eq, PartialEq, Default)]
    pub struct Dummy {}

    impl Encode for Dummy {
        fn encode<T: std::io::Write + Sized>(&self, _writer: &mut T) -> std::io::Result<()> {
            Ok(())
        }

        fn len(&self) -> usize {
            0
        }
    }

    impl Decode for Dummy {
        fn decode<T: std::io::Read + Sized>(_reader: &mut T) -> std::io::Result<Self> {
            Ok(Self {})
        }
    }

    /*#[test]
    fn test_asn1_decode() {
        use serde::{Serialize, Deserialize};
        use std::option::Option;

        let plain = Option::None;
        let serialized = picky_asn1_der::to_vec(&plain).unwrap();
        println!("{:?}", serialized);
        //let deserialized: Option::None = picky_asn1_der::from_bytes(&serialized).unwrap();
    }*/
}
