#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Service {}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum UnconfirmedService<'a> {
    IAm(IAmSlice<'a>),                  // = 0;
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

impl<'a> UnconfirmedService<'a> {
    ///Creates a slice containing an APDU.
    pub fn from_slice(slice: &'a [u8]) -> Result<UnconfirmedService, String> {
        // TODO: Add checks
        let type_ = slice[0];

        match type_ {
            0x00 => Ok(Self::IAm(IAmSlice::from_slice(&slice[1..]).unwrap())),
            0x08 => Ok(Self::WhoIs()),
            t => Err(format!("Unknown Service type: {}", t)),
        }
    }
}

/// A slice containing a Application layer Protocol Data Unit (6.2)
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IAmSlice<'a> {
    slice: &'a [u8],
}

impl<'a> IAmSlice<'a> {
    ///Creates a slice containing an APDU.
    pub fn from_slice(slice: &'a [u8]) -> Result<IAmSlice<'a>, String> {
        // TODO: Add checks

        Ok(IAmSlice { slice: &slice[..] })
    }
}
