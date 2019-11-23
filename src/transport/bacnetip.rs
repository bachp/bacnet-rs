/// Implements BACnet/IP (Annex J)
use crate::application::*;
use crate::network::*;
use crate::{Decode, Encode};
use std::convert::{Into, TryFrom, TryInto};

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use std::io::Write;

fn read_be_u16(input: &[u8]) -> u16 {
    let int_bytes = &input[0..std::mem::size_of::<u16>()];
    u16::from_be_bytes(int_bytes.try_into().unwrap())
}

pub trait AsU8 {
    fn as_u8(&self) -> u8;
}

/// A slice containing a BACnet Virtual Link Control (Annex J).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BVLCSlice<'a> {
    slice: &'a [u8],
}

/// BACnet Virtual Link Control Function
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BVLCSliceFunction<'a> {
    OriginalBroadcastNPDU(NPDUSlice<'a>),
    OriginalUnicastNPDU(NPDUSlice<'a>),
}

impl<'a> BVLCSlice<'a> {
    /// Creates a slice containing an BVLC.
    pub fn from_slice(slice: &'a [u8]) -> Result<BVLCSlice<'a>, String> {
        // TODO: Add checks

        let type_ = slice[0];
        if type_ != 0x81 {
            return Err(format!(
                "Only BACnet/IP (0x81) is currently supported, got: {}",
                type_
            ));
        }

        let length = read_be_u16(&slice[2..]) as usize;
        if slice.len() < length {
            return Err(format!(
                "Buffer doesn't contain enough data, contains: {}, requires: {}",
                slice.len(),
                length,
            ));
        }

        Ok(BVLCSlice {
            slice: &slice[..length],
        })
    }

    /// Returns the slice containing the BACnet Virtual Link Control
    #[inline]
    pub fn slice(&self) -> &'a [u8] {
        self.slice
    }

    /// Read the "Type" field of the BACnet Virtual Link Control.
    pub fn type_(&self) -> u8 {
        self.slice[0]
    }

    ///Read the "Function" field of the BACnet Virtual Link Control.
    pub fn function(&self) -> Result<BVLCSliceFunction, String> {
        match self.slice[1] {
            0x0b => {
                let npdu = NPDUSlice::try_from(&self.slice[4..self.length().into()])?;
                Ok(BVLCSliceFunction::OriginalBroadcastNPDU(npdu))
            }
            0x0a => {
                let npdu = NPDUSlice::try_from(&self.slice[4..self.length().into()])?;
                Ok(BVLCSliceFunction::OriginalUnicastNPDU(npdu))
            }
            t => Err(format!("Unknown type: {}", t)),
        }
    }

    ///Read the "Length" field of the BACnet Virtual Link Control.
    pub fn length(&self) -> u16 {
        read_be_u16(&self.slice[2..])
    }
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
            bvlc_type: 0x81, // BACnet/IP (Annex J)
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
        let function = reader.read_u8()?;
        let lenght = reader.read_u16::<BigEndian>()?;
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
}
