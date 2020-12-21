/// Implements BACnet/SC (Annex YY)
use crate::network::*;
use crate::{Decode, Encode};

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

const BACNETSC: u8 = 0x81;
