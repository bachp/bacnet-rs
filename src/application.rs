use crate::{Decode, Encode};

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

pub mod service;
pub use service::*;

use tracing::trace;

/// BACnetPDU variants (Chapter 21)
///
/// ```asn.1
/// BACnetPDU ::= CHOICE {
///     confirmed-request-pdu
///     unconfirmed-request-pdu
///     simple-ack-pdu
///     complex-ack-pdu
///     segment-ack-pdu
///     error-pdu
///     reject-pdu
///     abort-pdu
///     }
/// ```
///
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BACnetPDU {
    ConfirmedRequest,   // = 0x00;
    UnconfirmedRequest, // = 0x01;
    SimpleACK,          // = 0x02;
    ComplexACK,         // = 0x03;
    SegmentACK,         // = 0x04;
    Error,              // = 0x05;
    Reject,             // = 0x06;
    Abort,              // = 0x07;
}

impl BACnetPDU {
    fn as_u8(&self) -> u8 {
        match self {
            Self::ConfirmedRequest => 0,
            Self::UnconfirmedRequest => 1,
            Self::SimpleACK => 2,
            Self::ComplexACK => 3,
            Self::SegmentACK => 4,
            Self::Error => 5,
            Self::Reject => 6,
            Self::Abort => 7,
        }
    }
}

/// BACnet-Unconfirmed-Request-PDU struct (Chapter 21)
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BACnetUnconfirmedRequestPDU {}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct APDU {
    apdu_type: u8,
    pub service_choice: u8,
    user_data: Vec<u8>,
}

impl APDU {
    pub fn new(apdu_type: u8, service_choice: u8, user_data: Vec<u8>) -> Self {
        Self {
            apdu_type,
            service_choice,
            user_data,
        }
    }
}

impl Encode for APDU {
    fn encode<T: std::io::Write + Sized>(&self, writer: &mut T) -> std::io::Result<()> {
        writer.write_u8(self.apdu_type << 4)?;
        writer.write_u8(self.service_choice)?;
        writer.write(&self.user_data)?;
        Ok(())
    }

    fn len(&self) -> usize {
        let mut l = 0;
        l += 1; // Type
        l += 1; // Service Choice
        l += self.user_data.len(); // Content
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
            apdu.user_data,
            vec![196, 2, 0, 2, 87, 34, 4, 0, 145, 0, 33, 15]
        );

        let mut w = BytesMut::new().writer();
        apdu.encode(&mut w).expect("Write APDU to buffer");
        assert_eq!(w.into_inner().to_vec(), data);
    }
}
