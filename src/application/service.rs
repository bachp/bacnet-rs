use crate::{Decode, Encode};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Service {}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum UnconfirmedService {
    IAm(IAm),                           // = 0;
    IHave,                              // = 1;
    UnconfirmedCovNotification,         // = 2;
    UnconfirmedEventNotification,       // = 3;
    UnconfirmedPrivateTransfer,         // = 4;
    UnconfirmedTextMessage,             // = 5;
    TimeSynchronization,                // = 6;
    WhoHas,                             // = 7;
    WhoIs(),                            // = 8;
    UtcTimeSynchronization,             // = 9;
    WriteGroup,                         // = 10;
    UnconfirmedCovNotificationMultiple, // = 11;
}

impl Decode for UnconfirmedService {
    fn decode<T: std::io::Read + Sized>(reader: &mut T) -> std::io::Result<Self> {
        // TODO: Add checks
        let type_ = reader.read_u8()?;

        match type_ {
            0x00 => Ok(Self::IAm(IAm::decode(reader)?)),
            0x08 => Ok(Self::WhoIs()),
            t => unimplemented!(),
        }
    }
}

impl Encode for UnconfirmedService {
    fn encode<T: std::io::Write + Sized>(&self, writer: &mut T) -> std::io::Result<()> {
        match self {
            Self::IAm(a) => a.encode(writer),
            Self::WhoIs() => Ok(()),
            _ => unimplemented!(),
        }
    }

    fn len(&self) -> usize {
        match self {
            Self::IAm(a) => a.len(),
            Self::WhoIs() => 0,
            _ => unimplemented!(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IAm {}

impl Decode for IAm {
    fn decode<T: std::io::Read + Sized>(_reader: &mut T) -> std::io::Result<Self> {
        Ok(Self {})
    }
}

impl Encode for IAm {
    fn encode<T: std::io::Write + Sized>(&self, writer: &mut T) -> std::io::Result<()> {
        let data = vec![196, 2, 0, 2, 87, 34, 4, 0, 145, 0, 33, 15];
        writer.write(&data)?;
        Ok(())
    }

    fn len(&self) -> usize {
        12
    }
}
