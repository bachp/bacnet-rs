mod parse;

use nom::IResult;

pub struct Tag<'a> {
    tag_number: TagNumber,
    lvt: LengthValueType,
    data: &'a [u8],
}
pub enum TagNumber {
    Application(ApplicationTag),
    Context(ContextTag),
}
pub enum LengthValueType {
    Length(u32),
    Value(u8),
    Opening,
    Closing,
}

pub enum ApplicationTag {
    Null,                   //= 0,
    Boolean,                //= 1,
    UnsignedInteger,        //= 2,
    SignedInteger,          //= 3, // (2's complement notation)
    Real,                   //= 4, // (ANSI/IEEE-754 floating point)
    Double,                 //= 5, // (ANSI/IEEE-754 double precision floating point)
    OctetString,            //= 6,
    CharacterString,        //= 7,
    BitString,              //= 8,
    Enumerated,             //= 9,
    Date,                   //= 10,
    Time,                   //= 11,
    BACnetObjectIdentifier, //= 12,
    Reserved(u8),           //= 13, 14, 15 // Reserved for ASHRAE
    Other(u8),
}

impl From<u8> for ApplicationTag {
    fn from(tag_number: u8) -> Self {
        match tag_number {
            0 => ApplicationTag::Null,
            1 => ApplicationTag::Boolean,
            2 => ApplicationTag::UnsignedInteger,
            3 => ApplicationTag::SignedInteger,
            4 => ApplicationTag::Real,
            5 => ApplicationTag::Double,
            6 => ApplicationTag::OctetString,
            7 => ApplicationTag::CharacterString,
            8 => ApplicationTag::BitString,
            9 => ApplicationTag::Enumerated,
            10 => ApplicationTag::Date,
            11 => ApplicationTag::Time,
            12 => ApplicationTag::BACnetObjectIdentifier,
            t @ 13..=15 => ApplicationTag::Reserved(t),
            t => ApplicationTag::Other(t),
        }
    }
}

impl Into<u8> for ApplicationTag {
    fn into(self) -> u8 {
        match self {
            ApplicationTag::Null => 0,
            ApplicationTag::Boolean => 1,
            ApplicationTag::UnsignedInteger => 2,
            ApplicationTag::SignedInteger => 3,
            ApplicationTag::Real => 4,
            ApplicationTag::Double => 5,
            ApplicationTag::OctetString => 6,
            ApplicationTag::CharacterString => 7,
            ApplicationTag::BitString => 8,
            ApplicationTag::Enumerated => 9,
            ApplicationTag::Date => 10,
            ApplicationTag::Time => 11,
            ApplicationTag::BACnetObjectIdentifier => 12,
            ApplicationTag::Reserved(t) => t,
            ApplicationTag::Other(t) => t,
        }
    }
}

pub enum ContextTag {
    Other(u8),
}

impl From<u8> for ContextTag {
    fn from(tag_number: u8) -> Self {
        match tag_number {
            t => ContextTag::Other(t),
        }
    }
}

impl Into<u8> for ContextTag {
    fn into(self) -> u8 {
        match self {
            ContextTag::Other(t) => t,
        }
    }
}
