pub mod application;
pub mod network;
pub mod transport;

pub trait Decode<S = Self> {
    fn decode<T: std::io::Read + Sized>(reader: &mut T) -> std::io::Result<S>;
}

pub trait Encode {
    fn encode<T: std::io::Write + Sized>(&self, writer: &mut T) -> std::io::Result<()>;
    fn len(&self) -> usize;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Decode, Encode};
    use bytes::{BufMut, BytesMut};
    use hex;

    #[derive(Clone, Debug, Eq, PartialEq, Default)]
    pub struct Dummy {}

    impl Encode for Dummy {
        fn encode<T: std::io::Write + Sized>(&self, writer: &mut T) -> std::io::Result<()> {
            Ok(())
        }

        fn len(&self) -> usize {
            0
        }
    }

    impl Decode for Dummy {
        fn decode<T: std::io::Read + Sized>(reader: &mut T) -> std::io::Result<Self> {
            Ok(Self {})
        }
    }
}
