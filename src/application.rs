use crate::{Decode, Encode};

use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::FromPrimitive;
use std::convert::TryFrom;

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use std::io::Write;

pub mod service;
pub use service::*;

use tracing::trace;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BACnetPDUSlice<'a> {
    ConfirmedRequest(ConfirmedRequestPDUSlice<'a>), // = 0x00; (2.1.2)
    UnconfirmedRequest(UnconfirmedRequestPDUSlice<'a>), // = 0x01; (2.1.3)
    SimpleACK,                                      // = 0x02; (2.1.4)
    ComplexACK,                                     // = 0x03; (2.1.5)
    SegmentACK,                                     // = 0x04; (2.1.6)
    Error,                                          // = 0x05; (2.1.7)
    Reject,                                         // = 0x06; (2.1.8)
    Abort,                                          // = 0x07; (2.1.9)
}

impl<'a> BACnetPDUSlice<'a> {
    pub fn from_slice(slice: &'a [u8]) -> Result<Self, String> {
        let type_ = slice[0] >> 4;
        trace!("PDU Slice: {:02x?}, Type: {}", slice, type_);
        match type_ {
            0x01 => Ok(Self::UnconfirmedRequest(
                UnconfirmedRequestPDUSlice::from_slice(&slice[1..])?,
            )),
            _ => Err(format!("Unsupported PDU type: {}", type_)),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConfirmedRequestPDUSlice<'a> {
    slice: &'a [u8],
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UnconfirmedRequestPDUSlice<'a> {
    slice: &'a [u8],
}

impl<'a> UnconfirmedRequestPDUSlice<'a> {
    ///Creates a slice containing an APDU.
    pub fn from_slice(slice: &'a [u8]) -> Result<UnconfirmedRequestPDUSlice<'a>, String> {
        // TODO: Add checks

        Ok(UnconfirmedRequestPDUSlice { slice: &slice[..] })
    }

    ///Returns the slice containing the APDU
    #[inline]
    pub fn slice(&self) -> &'a [u8] {
        self.slice
    }

    pub fn service(&self) -> Result<UnconfirmedService, String> {
        UnconfirmedService::from_slice(self.slice)
    }
}

/// A slice containing a Application layer Protocol Data Unit (6.2)
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct APDUSlice<'a> {
    slice: &'a [u8],
}

impl<'a> APDUSlice<'a> {
    ///Creates a slice containing an APDU.
    pub fn from_slice(slice: &'a [u8]) -> Result<APDUSlice<'a>, String> {
        // TODO: Add checks

        Ok(APDUSlice { slice: &slice[..] })
    }

    ///Returns the slice containing the APDU
    #[inline]
    pub fn slice(&self) -> &'a [u8] {
        self.slice
    }

    fn type_(&self) -> u8 {
        println!("{:02x?}", self.slice);
        self.slice[0] >> 4
    }

    pub fn content(&self) -> Result<BACnetPDUSlice, String> {
        BACnetPDUSlice::from_slice(self.slice)
    }

    pub fn service_slice(&self) -> &'a [u8] {
        &self.slice[2..]
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum APDUType {
    ConfirmedRequest,   // = 0x00; (2.1.2)
    UnconfirmedRequest, // = 0x01; (2.1.3)
    SimpleACK,          // = 0x02; (2.1.4)
    ComplexACK,         // = 0x03; (2.1.5)
    SegmentACK,         // = 0x04; (2.1.6)
    Error,              // = 0x05; (2.1.7)
    Reject,             // = 0x06; (2.1.8)
    Abort,              // = 0x07; (2.1.9)
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct APDU {
    apdu_type: u8,
    pub service_choice: u8,
    content: Vec<u8>,
}

impl APDU {
    pub fn new(apdu_type: u8, service_choice: u8, content: Vec<u8>) -> Self {
        Self {
            apdu_type,
            service_choice,
            content,
        }
    }
}

impl Encode for APDU {
    fn encode<T: std::io::Write + Sized>(&self, writer: &mut T) -> std::io::Result<()> {
        writer.write_u8(self.apdu_type << 4)?;
        writer.write_u8(self.service_choice)?;
        writer.write(&self.content)?;
        Ok(())
    }

    fn len(&self) -> usize {
        let mut l = 0;
        l += 1; // Type
        l += 1; // Service Choice
        l += self.content.len(); // Content
        l
    }
}

impl Decode for APDU {
    fn decode<T: std::io::Read + Sized>(reader: &mut T) -> std::io::Result<Self> {
        let apdu_type = reader.read_u8()? >> 4;
        let service_choice = reader.read_u8()?;
        let mut content = Vec::new(); // TODO: What capacity?
        reader.read_to_end(&mut content)?;
        trace!("APDU Type: {}", apdu_type);
        Ok(APDU::new(apdu_type, service_choice, content))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Decode, Encode};
    use bytes::{BufMut, BytesMut};
    use hex;

    use crate::tests::*;

    #[test]
    fn test_encode_apdu() {
        let content = vec![0, 0, 0];
        let apdu = APDU::new(1, 8, content);

        let mut w = BytesMut::new().writer();
        apdu.encode(&mut w).expect("Write APDU to buffer");
        assert_eq!(w.into_inner().to_vec(), vec![16, 8, 0, 0, 0]);
    }

    #[test]
    fn test_who_is() {
        let mut data = hex::decode("1008").unwrap();

        let apdu = APDU::decode(&mut std::io::Cursor::new(&mut data)).expect("Decode APDU");

        assert_eq!(apdu.apdu_type, 0x01);
        assert_eq!(apdu.service_choice, 0x08);

        let mut w = BytesMut::new().writer();
        apdu.encode(&mut w).expect("Write APDU to buffer");
        assert_eq!(w.into_inner().to_vec(), data);
    }

    #[test]
    fn test_i_am() {
        let mut data = hex::decode("1000c4020002572204009100210f").unwrap();

        let apdu = APDU::decode(&mut std::io::Cursor::new(&mut data)).expect("Decode APDU");

        assert_eq!(apdu.apdu_type, 0x01);
        assert_eq!(apdu.service_choice, 0x00);
        assert_eq!(
            apdu.content,
            vec![196, 2, 0, 2, 87, 34, 4, 0, 145, 0, 33, 15]
        );

        let mut w = BytesMut::new().writer();
        apdu.encode(&mut w).expect("Write APDU to buffer");
        assert_eq!(w.into_inner().to_vec(), data);
    }
}
