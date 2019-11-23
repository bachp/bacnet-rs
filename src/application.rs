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
pub struct APDU {
    apdu_type: u8,
    service_choice: u8,
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
        // TODO: parse content
        let content = vec![];
        trace!("APDU Type: {}", apdu_type);
        Ok(APDU::new(apdu_type, service_choice, content))
    }
}
