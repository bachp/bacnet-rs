use crate::application::*;
use crate::{Decode, Encode};

use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::{FromPrimitive, ToPrimitive};
use std::convert::TryFrom;

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use bytes::BufMut;

use tracing::trace;

/// Network Layer PDU Message Priority (6.2.2)
#[derive(Copy, Clone, Debug, Eq, PartialEq, FromPrimitive, ToPrimitive)]
pub enum NPDUPriority {
    LifeSafety = 0b11,
    CriticalEquipment = 0b10,
    Urgent = 0b01,
    Normal = 0b00,
}

impl Into<u8> for NPDUPriority {
    fn into(self) -> u8 {
        match self {
            Self::LifeSafety => 0b11,
            Self::CriticalEquipment => 0b10,
            Self::Urgent => 0b01,
            Self::Normal => 0b00,
        }
    }
}

impl Default for NPDUPriority {
    fn default() -> Self {
        Self::Normal
    }
}

/// Network Layer PDU Message Type (6.2.4)
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NPDUMessage {
    WhoIsRouterToNetwork,          // = 0x00,
    IAmRouterToNetwork,            // = 0x01,
    ICouldBeRouterToNetwork,       // = 0x02,
    RejectMessageToNetwork,        // = 0x03,
    RouterBusyToNetwork,           // = 0x04,
    RouterAvailableToNetwork,      // = 0x05,
    InitializeRoutingTable,        // = 0x06,
    InitializeRoutingTableAck,     // = 0x07,
    EstablishConnectionToNetwork,  // = 0x08,
    DisconnectConnectionToNetwork, // = 0x09,
    ChallengeRequest,              // = 0x0A,
    SecurityPayload,               // = 0x0B,
    SecurityResponse,              // = 0x0C,
    RequestKeyUpdate,              // = 0x0D,
    UpdateKeySet,                  // = 0x0E,
    UpdateDistributionKey,         // = 0x0F,
    RequestMasterKey,              // = 0x10,
    SetMasterKey,                  // = 0x11,
    WhatIsNetworkNumber,           // = 0x12,
    NetworkNumberIs,               // = 0x13,
    Proprietary(u8),               // = 0x80 to 0xFF, Available for vendor proprietary messages
    Reserved(u8),                  // = 0x14 to 0x7F, Reserved for use by ASHRAE
}

impl TryFrom<u8> for NPDUMessage {
    type Error = String;

    fn try_from(v: u8) -> Result<Self, Self::Error> {
        match v {
            0x00 => Ok(Self::WhoIsRouterToNetwork),
            // TODO: Implement rest
            v if (v >= 0x80 && v <= 0xFF) => Ok(Self::Proprietary(v)),
            v => Err(format!("Unknown Message type: {}", v)),
        }
    }
}

impl Encode for NPDUMessage {
    fn encode<T: std::io::Write + Sized>(&self, writer: &mut T) -> std::io::Result<()> {
        unimplemented!();
    }

    fn len(&self) -> usize {
        unimplemented!();
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NPDUDest {
    net: u16,
    adr: Vec<u8>,
    hops: u8,
}

impl NPDUDest {
    pub fn new(net: u16, capacity: usize) -> Self {
        NPDUDest {
            net,
            adr: Vec::with_capacity(capacity),
            hops: 255,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct NPDUSource {
    net: u16,
    adr: Vec<u8>,
}

impl NPDUSource {
    pub fn new(net: u16, capacity: usize) -> Self {
        NPDUSource {
            net,
            adr: Vec::with_capacity(capacity),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NPDUContent<A: Encode = APDU, B: Encode = NPDUMessage> {
    APDU(A),
    Message(B),
}

impl<A: Encode, B: Encode> From<A> for NPDUContent<A, B> {
    fn from(apdu: A) -> Self {
        NPDUContent::APDU(apdu)
    }
}

impl<A: Encode, B: Encode> Encode for NPDUContent<A, B> {
    fn encode<T: std::io::Write + Sized>(&self, writer: &mut T) -> std::io::Result<()> {
        Ok(match self {
            Self::APDU(apdu) => apdu.encode(writer)?,
            Self::Message(msg) => msg.encode(writer)?,
        })
    }

    fn len(&self) -> usize {
        match self {
            Self::APDU(apdu) => apdu.len(),
            Self::Message(msg) => msg.len(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NPDU<A: Encode = APDU, B: Encode = NPDUMessage> {
    /// Protocol Version Number (6.2.1)
    pub version: u8,
    pub destination: Option<NPDUDest>,
    pub source: Option<NPDUSource>,
    pub data_expecting_reply: bool,
    pub priority: NPDUPriority,
    pub content: NPDUContent<A, B>,
}

impl<A: Encode, B: Encode> NPDU<A, B> {
    pub fn new<T: Into<NPDUContent<A, B>>>(
        content: T,
        destination: Option<NPDUDest>,
        source: Option<NPDUSource>,
        priority: NPDUPriority,
    ) -> Self {
        NPDU {
            version: 1,
            content: content.into(),
            destination,
            source,
            data_expecting_reply: false,
            priority,
        }
    }
}

impl<A: Encode, B: Encode> Encode for NPDU<A, B> {
    fn encode<T: std::io::Write + Sized>(&self, writer: &mut T) -> std::io::Result<()> {
        // NPCI
        writer.write_u8(self.version)?;

        let mut control: u8 = self.priority.into();
        if self.data_expecting_reply {
            control |= 1 << 2;
        }
        if self.source.is_some() {
            control |= 1 << 3;
        }
        if self.destination.is_some() {
            control |= 1 << 5;
        }
        if let NPDUContent::Message(_) = self.content {
            control |= 1 << 7;
        }
        writer.write_u8(control)?;
        if let Some(ref d) = self.destination {
            writer.write_u16::<BigEndian>(d.net)?;
            writer.write_u8(d.adr.len() as u8)?;
            writer.write(&d.adr)?;
        }
        if let Some(ref s) = self.source {
            writer.write_u16::<BigEndian>(s.net)?;
            writer.write_u8(s.adr.len() as u8)?;
            writer.write(&s.adr)?;
        }
        if let Some(ref d) = self.destination {
            writer.write_u8(d.hops)?;
        }

        // Content
        self.content.encode(writer)?;

        Ok(())
    }

    fn len(&self) -> usize {
        let mut l: usize = 0;
        l += 1; // Version
        l += 1; // Control
        l += self
            .destination
            .as_ref()
            .and_then(|d| Some(2 + 1 + d.adr.len() + 1))
            .unwrap_or(0) as usize; // DNET(2) + DLEN(1) + DADR(*) + HOPS(1)
        l += self
            .source
            .as_ref()
            .and_then(|s| Some(2 + 1 + s.adr.len()))
            .unwrap_or(0) as usize; // SNET(2) + SLEN(1) + SADR(*)
        l += self.content.len();
        l
    }
}

impl Decode for NPDU {
    fn decode<T: std::io::Read + Sized>(reader: &mut T) -> std::io::Result<Self> {
        let version = reader.read_u8()?;
        trace!("Version: {:02x}", version);
        // Read and parse the Network Layer Protocol Control Information (6.2.2)
        let control = reader.read_u8()?;
        trace!("Control: {:08b}", control);
        let priority = NPDUPriority::from_u8(control & 0b0000_00011).unwrap();
        let has_apdu = (control & 1 << 7) == 0;
        let has_dest = (control & 1 << 5) != 0;
        let has_source = (control & 1 << 3) != 0;
        let data_expecting_reply = (control & 1 << 2) != 0;

        let mut destination: Option<NPDUDest> = if has_dest {
            let net = reader.read_u16::<BigEndian>()?;
            let len = reader.read_u8()?;
            let mut dest = NPDUDest::new(net, len as usize);
            reader.read_exact(&mut dest.adr)?;
            Some(dest)
        } else {
            None
        };

        let source: Option<NPDUSource> = if has_source {
            let net = reader.read_u16::<BigEndian>()?;
            let len = reader.read_u8()?;
            let mut source = NPDUSource::new(net, len as usize);
            reader.read_exact(&mut source.adr)?;
            Some(source)
        } else {
            None
        };
        println!("{:?}", destination);
        if let Some(dest) = &mut destination {
            dest.hops = reader.read_u8()?;
        };

        let content = if has_apdu {
            APDU::decode(reader)?.into()
        } else {
            /*Ok(NPDUContentSlice::Message(NPDUMessage::try_from(
                self.slice[0],
            )?))*/
            unimplemented!();
        };

        Ok(Self {
            version,
            destination,
            source,
            data_expecting_reply,
            priority,
            content,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Decode, Encode};
    use bytes::{BufMut, BytesMut};

    use crate::tests::*;

    #[test]
    fn test_encode_npdu() {
        let content = NPDUContent::<Dummy, Dummy>::APDU(Dummy::default());
        let npdu = NPDU::<Dummy, Dummy>::new(content, None, None, NPDUPriority::Normal);

        let mut w = BytesMut::new().writer();
        npdu.encode(&mut w).expect("Write NPDU to buffer");
        assert_eq!(w.into_inner().to_vec(), vec![1, 0]);
    }

    #[test]
    fn test_encode_npdu_with_dest() {
        let content = NPDUContent::<Dummy, Dummy>::APDU(Dummy::default());
        let dest = NPDUDest {
            net: 0x126,
            adr: vec![0; 16],
            hops: 255,
        };
        let npdu = NPDU::<Dummy, Dummy>::new(content, Some(dest), None, NPDUPriority::Normal);

        let mut w = BytesMut::new().writer();
        npdu.encode(&mut w).expect("Write NPDU to buffer");
        assert_eq!(
            w.into_inner().to_vec(),
            vec![1, 32, 1, 38, 16, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 255]
        );
    }

    #[test]
    fn test_encode_npdu_with_source() {
        let content = NPDUContent::<Dummy, Dummy>::APDU(Dummy::default());
        let source = NPDUSource {
            net: 0x126,
            adr: vec![0; 16],
        };
        let npdu = NPDU::<Dummy, Dummy>::new(content, None, Some(source), NPDUPriority::Normal);

        let mut w = BytesMut::new().writer();
        npdu.encode(&mut w).expect("Write NPDU to buffer");
        assert_eq!(
            w.into_inner().to_vec(),
            vec![1, 8, 1, 38, 16, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
        );
    }

    #[test]
    fn test_encode_npdu_with_dest_and_source() {
        let content = NPDUContent::<Dummy, Dummy>::APDU(Dummy::default());
        let dest = NPDUDest {
            net: 0x126,
            adr: vec![0; 16],
            hops: 255,
        };
        let source = NPDUSource {
            net: 0x126,
            adr: vec![0; 16],
        };
        let npdu =
            NPDU::<Dummy, Dummy>::new(content, Some(dest), Some(source), NPDUPriority::Normal);

        let mut w = BytesMut::with_capacity(1024).writer();
        npdu.encode(&mut w).expect("Write NPDU to buffer");
        assert_eq!(
            w.into_inner().to_vec(),
            vec![
                1, 40, 1, 38, 16, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 38, 16, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 255
            ]
        );
    }
}
