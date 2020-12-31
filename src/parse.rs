use bytes::{Buf, BufMut};
use std::io::Cursor;

pub enum Tag<'a> {
    Application(ApplicationTag, u32, &'a [u8]),
    Context,
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
    Reserved,               //= 13, 14, 15 // Reserved for ASHRAE
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
            13..=15 => ApplicationTag::Reserved,
            t => ApplicationTag::Other(t),
        }
    }
}

impl<'a> Tag<'a> {
    fn decode_buf(buf: &'a [u8]) -> Result<(u8, bool, u32, u32), String> {
        let mut buf = Cursor::new(buf);

        let first_byte = buf.get_u8();
        let tag_number = (first_byte & 0b1111_0_000) >> 4;

        let tag_number = match tag_number {
            t @ 0..=14 => t,
            15..=255 => buf.get_u8(),
        };

        let length = first_byte & 0b0000_0_111;
        let class = (first_byte & 0b0000_1_000) != 0;

        let length: u32 = if length < 0b101 {
            length as u32
        } else {
            let extended = buf.get_u8();
            match extended {
                l @ 0..=253 => l as u32,
                254 => buf.get_u16() as u32,
                255 => buf.get_u32(),
            }
        };

        let offset = buf.position() as u32;

        Ok((tag_number, class, length, offset))
    }

    /*pub fn decode(buf: &[u8]) -> Result<Tag, String> {
        let (tag_number, class, length) = Tag::decode_buf(buf)?;

        let tag = if !class {
            Tag::Application(ApplicationTag::from(tag_number, length, buf[]))
        } else {
            unimplemented!("Properly handle context specific");
            //Tag::Context(tag_number);
        }

        Ok(tag)
    }*/

    fn encode_buf(tag_number: u8, class: bool, length: u32) -> Result<Vec<u8>, String> {
        let mut buf: Vec<u8> = vec![0x00]; // Initial tag set to zero so we can do bitwise or

        match tag_number {
            t @ 0..=14 => {
                buf[0] |= t << 4;
            }
            t @ 15..=255 => {
                buf[0] |= 0b1111 << 4;
                buf.put_u8(t);
            }
        };

        if class {
            buf[0] |= 0b0000_1_000;
        }

        match length {
            l @ 0..=4 => {
                buf[0] |= l as u8;
            }
            l @ 5..=253 => {
                buf[0] |= 0b101;
                buf.put_u8(l as u8);
            }
            l @ 254..=65535 => {
                buf[0] |= 0b101;
                buf.put_u8(254);
                buf.put_u16(l as u16);
            }
            l @ 65536..=core::u32::MAX => {
                buf[0] |= 0b101;
                buf.put_u8(255);
                buf.put_u32(l as u32);
            }
        }
        Ok(buf)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_application_tag_number_0() {
        let buf = Tag::encode_buf(0, false, 0).unwrap();
        assert_eq!(buf, &[0b0000_0_000])
    }

    #[test]
    fn test_encode_context_tag_number_0() {
        let buf = Tag::encode_buf(0, true, 0).unwrap();
        assert_eq!(buf, &[0b0000_1_000])
    }

    #[test]
    fn test_decode_application_tag_number_0() {
        let tag = Tag::decode_buf(&[0b0000_0_000]).unwrap();
        assert_eq!(tag, (0, false, 0, 1))
    }

    #[test]
    fn test_decode_context_tag_number_0() {
        let tag = Tag::decode_buf(&[0b0000_1_000]).unwrap();
        assert_eq!(tag, (0, true, 0, 1))
    }

    #[test]
    fn test_encode_application_tag_number_14() {
        let buf = Tag::encode_buf(14, false, 0).unwrap();
        assert_eq!(buf, &[0b1110_0_000])
    }

    #[test]
    fn test_decode_application_tag_number_14() {
        let tag = Tag::decode_buf(&[0b1110_0_000]).unwrap();
        assert_eq!(tag, (14, false, 0, 1))
    }

    #[test]
    fn test_encode_application_tag_number_15() {
        let buf = Tag::encode_buf(15, false, 0).unwrap();
        assert_eq!(buf, &[0b1111_0_000, 15])
    }

    #[test]
    fn test_decode_application_tag_number_15() {
        let tag = Tag::decode_buf(&[0b1111_0_000, 15]).unwrap();
        assert_eq!(tag, (15, false, 0, 2))
    }

    #[test]
    fn test_encode_application_tag_number_254() {
        let buf = Tag::encode_buf(254, false, 0).unwrap();
        assert_eq!(buf, &[0b1111_0_000, 254])
    }

    #[test]
    fn test_decode_application_tag_number_254() {
        let tag = Tag::decode_buf(&[0b1111_0_000, 254]).unwrap();
        assert_eq!(tag, (254, false, 0, 2))
    }

    #[test]
    fn test_encode_application_reserved_tag_number_255() {
        let buf = Tag::encode_buf(255, false, 0).unwrap();
        assert_eq!(buf, &[0b1111_0_000, 255])
    }

    #[test]
    fn test_decode_application_reserved_tag_number_255() {
        let tag = Tag::decode_buf(&[0b1111_0_000, 255]).unwrap();
        assert_eq!(tag, (255, false, 0, 2))
    }

    #[test]
    fn test_encode_length_0() {
        let buf = Tag::encode_buf(0, false, 0).unwrap();
        assert_eq!(buf, &[0b0000_0_000])
    }

    #[test]
    fn test_decode_length_0() {
        let tag = Tag::decode_buf(&[0b0000_0_000]).unwrap();
        assert_eq!(tag, (0, false, 0, 1))
    }

    #[test]
    fn test_encode_length_4() {
        let buf = Tag::encode_buf(0, false, 4).unwrap();
        assert_eq!(buf, &[0b0000_0_100])
    }

    #[test]
    fn test_decode_length_4() {
        let tag = Tag::decode_buf(&[0b0000_0_100]).unwrap();
        assert_eq!(tag, (0, false, 4, 1))
    }

    #[test]
    fn test_length_5() {
        let buf = Tag::encode_buf(0, false, 5).unwrap();
        assert_eq!(buf, &[0b0000_0_101, 5])
    }

    #[test]
    fn test_length_253() {
        let buf = Tag::encode_buf(0, false, 253).unwrap();
        assert_eq!(buf, &[0b0000_0_101, 253])
    }

    #[test]
    fn test_length_254() {
        let buf = Tag::encode_buf(0, false, 254).unwrap();
        assert_eq!(buf, &[0b0000_0_101, 254, 0, 254])
    }

    #[test]
    fn test_length_65535() {
        let buf = Tag::encode_buf(0, false, 65535).unwrap();
        assert_eq!(buf, &[0b0000_0_101, 254, 255, 255])
    }

    #[test]
    fn test_length_65536() {
        let buf = Tag::encode_buf(0, false, 65536).unwrap();
        assert_eq!(buf, &[0b0000_0_101, 255, 0, 1, 0, 0])
    }

    #[test]
    fn test_length_u32max_minus_1() {
        let buf = Tag::encode_buf(0, false, std::u32::MAX - 1).unwrap();
        assert_eq!(buf, &[0b0000_0_101, 255, 255, 255, 255, 254])
    }

    #[test]
    fn test_reserved_length_u32max() {
        let buf = Tag::encode_buf(0, false, std::u32::MAX).unwrap();
        assert_eq!(buf, &[0b0000_0_101, 255, 255, 255, 255, 255])
    }
}
