/// Implements BACnet/IP (Annex J)
use crate::network::*;
use crate::{Decode, Encode};

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

const BACNETIP: u8 = 0x81;

pub trait AsU8 {
    fn as_u8(&self) -> u8;
}

/// BACnet Virtual Link Control Function
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BVLCFunction {
    OriginalBroadcastNPDU(NPDU),
    OriginalUnicastNPDU(NPDU),
}

impl AsU8 for BVLCFunction {
    fn as_u8(&self) -> u8 {
        match self {
            Self::OriginalBroadcastNPDU(_) => 0x0b,
            Self::OriginalUnicastNPDU(_) => 0x0a,
        }
    }
}

impl Encode for BVLCFunction {
    fn encode<T: std::io::Write + Sized>(&self, writer: &mut T) -> std::io::Result<()> {
        match self {
            Self::OriginalBroadcastNPDU(n) | Self::OriginalUnicastNPDU(n) => n.encode(writer)?,
        }
        Ok(())
    }

    fn len(&self) -> usize {
        match self {
            Self::OriginalBroadcastNPDU(n) | Self::OriginalUnicastNPDU(n) => n.len(),
        }
    }
}

/// A Struct containing a BACnet Virtual Link Control (Annex J).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BVLC<F = BVLCFunction> {
    bvlc_type: u8,
    pub function: F,
}

impl<F> BVLC<F> {
    pub fn new(function: F) -> Self {
        Self {
            bvlc_type: BACNETIP, // BACnet/IP (Annex J)
            function: function,
        }
    }

    pub fn set_function(&mut self, function: F) {
        self.function = function;
    }
}

impl<F: Encode + AsU8> Encode for BVLC<F> {
    fn encode<T: std::io::Write + Sized>(&self, writer: &mut T) -> std::io::Result<()> {
        writer.write_u8(self.bvlc_type)?;
        writer.write_u8(self.function.as_u8())?;
        writer.write_u16::<BigEndian>(self.len() as u16)?;
        self.function.encode(writer)?;
        Ok(())
    }

    fn len(&self) -> usize {
        let mut l: usize = 0;
        l += 1; // Type
        l += 1; // Function
        l += 2; // Content Length
        l += self.function.len(); // Function Content
        l
    }
}

impl Decode for BVLC {
    fn decode<T: std::io::Read + Sized>(reader: &mut T) -> std::io::Result<Self> {
        let bvlc_type = reader.read_u8()?;
        if bvlc_type != BACNETIP {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("BVLC type not supported: {}", bvlc_type),
            ));
        }
        let function = reader.read_u8()?;
        let lenght = reader.read_u16::<BigEndian>()?; // TODO: Check lenght
        let function = match function {
            0x0b => {
                let npdu = NPDU::decode(reader)?;
                Ok(BVLCFunction::OriginalBroadcastNPDU(npdu))
            }
            0x0a => {
                let npdu = NPDU::decode(reader)?;
                Ok(BVLCFunction::OriginalUnicastNPDU(npdu))
            }
            t => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("BVLC Function not supported: {}", t),
            )),
        };
        Ok(Self::new(function?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Decode, Encode};
    use bytes::{BufMut, BytesMut};
    use hex;

    use crate::tests::*;

    impl AsU8 for Dummy {
        fn as_u8(&self) -> u8 {
            0x00
        }
    }

    #[test]
    fn test_encode_bvlc() {
        let bvlc = BVLC::<Dummy>::new(Dummy::default());

        let mut w = BytesMut::new().writer();
        bvlc.encode(&mut w).expect("Write BVLC to buffer");
        assert_eq!(w.into_inner().to_vec(), vec![129, 0, 0, 4]);
    }

    #[test]
    fn test_decode_invalid_bvlc_type() {
        let data = hex::decode("00000000").unwrap();
        let err = BVLC::decode(&mut std::io::Cursor::new(&data)).unwrap_err();

        assert_eq!(err.kind(), std::io::ErrorKind::InvalidData);
        assert_eq!(
            err.into_inner().unwrap().to_string(),
            "BVLC type not supported: 0".to_string()
        );
    }
}
