use nom::number::streaming::{be_f64, be_i16, be_i24, be_u16, be_u24, be_u32, be_u8};
use nom::{Err, IResult, Needed};
use std::io::Cursor;

use crate::encoding::{ApplicationTag, ContextTag, LengthValueType, Tag, TagNumber};

pub fn parse_bacnet_tag<'a>(input: &'a [u8]) -> IResult<&'a [u8], Tag> {
    let mut cur = Cursor::new(input);
    let first_byte = cur.get_u8();
    let tag_number = (first_byte & 0b1111_0_000) >> 4;

    // 20.2.1.2 Tag Number
    let tag_number = match tag_number {
        t @ 0..=14 => t,
        15..=255 => cur.get_u8(),
    };

    // 20.2.1.1 Class
    let class = (first_byte & 0b0000_1_000) != 0;
    let tag_number = match class {
        false => TagNumber::Application(ApplicationTag::from(tag_number)),
        true => TagNumber::Context(ContextTag::from(tag_number)),
    };

    // 20.2.1.3 Length/Value/Type
    let lvt = first_byte & 0b0000_0_111;
    let lvt = match lvt {
        l if std::matches!(tag_number, TagNumber::Application(ApplicationTag::Boolean)) => {
            LengthValueType::Value(l)
        }
        l if l < 0b101 => LengthValueType::Length(l as u32),
        0b101 => {
            let extended = cur.get_u8();
            let length = match extended {
                l @ 0..=253 => l as u32,
                254 => cur.get_u16() as u32,
                255 => cur.get_u32(),
            };
            LengthValueType::Length(length)
        }
        0b110 => LengthValueType::Opening,
        0b111 => LengthValueType::Closing,
        _ => unreachable!("Length is only 3 bits"),
    };

    let data_start = cur.position() as usize;
    let mut data_end = data_start;

    if let LengthValueType::Length(l) = lvt {
        data_end += l as usize;
    }

    // TODO: Throw a proper error if slice is not long enough, currently it just panicks which is still safe

    let data = &input[data_start..data_end];
    let output = &input[data_end..];

    let tag = Tag {
        tag_number,
        lvt,
        data,
    };

    Ok((output, tag))
}

use bytes::{Buf, BufMut};

pub fn decode_buf<'a>(buf: &'a [u8]) -> Result<(u8, bool, u32, &'a [u8]), String> {
    let mut cur = Cursor::new(buf);

    let first_byte = cur.get_u8();
    let tag_number = (first_byte & 0b1111_0_000) >> 4;

    // 20.2.1.2 Tag Number
    let tag_number = match tag_number {
        t @ 0..=14 => t,
        15..=255 => cur.get_u8(),
    };

    // 20.2.1.1 Class
    let class = (first_byte & 0b0000_1_000) != 0;

    // 20.2.1.3 Length/Value/Type
    let length = first_byte & 0b0000_0_111;
    let length: u32 = if length < 0b101 {
        length as u32
    } else {
        let extended = cur.get_u8();
        match extended {
            l @ 0..=253 => l as u32,
            254 => cur.get_u16() as u32,
            255 => cur.get_u32(),
        }
    };

    // Offset where the data starts,
    // depends on how length is encoded
    let offset = cur.position() as usize;

    // TODO: Throw a proper error if slice is not long enough, currently it just panicks which is still safe
    let data = &buf[offset..offset + (length as usize)];

    Ok((tag_number, class, length, data))
}

pub fn encode_buf(tag_number: u8, class: bool, length: u32) -> Result<Vec<u8>, String> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::BufMut;
    use bytes::{Bytes, BytesMut};
    use hex;
    use std::matches;

    #[test]
    fn test_parse_application_tag_null() {
        let input: &[u8] = &[0b0000_0_000];
        let (_, tag) = parse_bacnet_tag(input).unwrap();
        assert!(matches!(
            tag.tag_number,
            TagNumber::Application(ApplicationTag::Null)
        ));
        assert!(matches!(tag.lvt, LengthValueType::Length(0)));
        assert!(tag.data.is_empty());
    }

    #[test]
    fn test_parse_application_tag_boolean_false() {
        let input: &[u8] = &[0b0001_0_000];
        let (_, tag) = parse_bacnet_tag(input).unwrap();
        assert!(matches!(
            tag.tag_number,
            TagNumber::Application(ApplicationTag::Boolean)
        ));
        assert!(matches!(tag.lvt, LengthValueType::Value(0)));
        assert!(tag.data.is_empty());
    }

    #[test]
    fn test_parse_application_tag_boolean_true() {
        let input: &[u8] = &[0b0001_0_001];
        let (_, tag) = parse_bacnet_tag(input).unwrap();
        assert!(matches!(
            tag.tag_number,
            TagNumber::Application(ApplicationTag::Boolean)
        ));
        assert!(matches!(tag.lvt, LengthValueType::Value(1)));
        assert!(tag.data.is_empty());
    }

    #[test]
    fn test_parse_context_tag_boolean_false() {
        let input: &[u8] = &[0b0001_1_001, 0b0000_0000];
        let (_, tag) = parse_bacnet_tag(input).unwrap();
        assert!(matches!(
            tag.tag_number,
            TagNumber::Context(ContextTag::Other(1))
        ));
        assert!(matches!(tag.lvt, LengthValueType::Length(1)));
        assert_eq!(tag.data, &[0b0000_0000]);
    }

    #[test]
    fn test_parse_context_tag_boolean_true() {
        let input: &[u8] = &[0b0001_1_001, 0b0000_0001];
        let (_, tag) = parse_bacnet_tag(input).unwrap();
        assert!(matches!(
            tag.tag_number,
            TagNumber::Context(ContextTag::Other(1))
        ));
        assert!(matches!(tag.lvt, LengthValueType::Length(1)));
        assert_eq!(tag.data, &[0b0000_0001]);
    }

    #[test]
    fn test_parse_application_tag_unsigned_integer_72() {
        let input: &[u8] = &[0x21, 0x48];
        let (_, tag) = parse_bacnet_tag(input).unwrap();
        assert!(matches!(
            tag.tag_number,
            TagNumber::Application(ApplicationTag::UnsignedInteger)
        ));
        assert!(matches!(tag.lvt, LengthValueType::Length(1)));
        assert_eq!(tag.data, &[72]);
    }

    #[test]
    fn test_parse_application_tag_signed_integer_72() {
        let input: &[u8] = &[0x31, 0x48];
        let (_, tag) = parse_bacnet_tag(input).unwrap();
        assert!(matches!(
            tag.tag_number,
            TagNumber::Application(ApplicationTag::SignedInteger)
        ));
        assert!(matches!(tag.lvt, LengthValueType::Length(1)));
        assert_eq!(tag.data, &[72]);
    }

    #[test]
    fn test_parse_application_tag_real_72_0() {
        let input: &[u8] = &[0x44, 0x42, 0x90, 0x00, 0x00];
        let (_, tag) = parse_bacnet_tag(input).unwrap();
        assert!(matches!(
            tag.tag_number,
            TagNumber::Application(ApplicationTag::Real)
        ));
        assert!(matches!(tag.lvt, LengthValueType::Length(4)));
        assert_eq!(tag.data, &[0x42, 0x90, 0x00, 0x00]);
    }

    #[test]
    fn test_parse_application_tag_double_72_0() {
        let input: &[u8] = &[0x55, 0x08, 0x40, 0x52, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
        let (_, tag) = parse_bacnet_tag(input).unwrap();
        assert!(matches!(
            tag.tag_number,
            TagNumber::Application(ApplicationTag::Double)
        ));
        assert!(matches!(tag.lvt, LengthValueType::Length(8)));
        assert_eq!(tag.data, &[0x40, 0x52, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);
    }

    #[test]
    fn test_parse_application_tag_octet_string_example() {
        let input: &[u8] = &[0x63, 0x12, 0x34, 0xFF];
        let (_, tag) = parse_bacnet_tag(input).unwrap();
        assert!(matches!(
            tag.tag_number,
            TagNumber::Application(ApplicationTag::OctetString)
        ));
        assert!(matches!(tag.lvt, LengthValueType::Length(3)));
        assert_eq!(tag.data, &[0x12, 0x34, 0xFF]);
    }

    #[test]
    fn test_parse_application_tag_character_string_utf8() {
        let mut input = BytesMut::from(&[0x75, 0x19, 0x00][..]);
        input.extend_from_slice(
            &hex::decode("546869732069732061204241436E657420737472696E6721").unwrap(),
        );
        let (_, tag) = parse_bacnet_tag(&input).unwrap();
        assert!(matches!(
            tag.tag_number,
            TagNumber::Application(ApplicationTag::CharacterString)
        ));
        assert!(matches!(tag.lvt, LengthValueType::Length(25)));
        let mut ref_data = BytesMut::from(&[0x00][..]);
        ref_data.extend_from_slice("This is a BACnet string!".as_bytes());
        assert_eq!(tag.data, ref_data);
    }

    #[test]
    fn test_parse_application_tag_character_string_utf8_non_ascii() {
        let mut input = BytesMut::from(&[0x75, 0x0A, 0x00][..]);
        input.extend_from_slice(&hex::decode("4672616EC3A7616973").unwrap());
        let (_, tag) = parse_bacnet_tag(&input).unwrap();
        assert!(matches!(
            tag.tag_number,
            TagNumber::Application(ApplicationTag::CharacterString)
        ));
        assert!(matches!(tag.lvt, LengthValueType::Length(10)));
        let mut ref_data = BytesMut::from(&[0x00][..]);
        ref_data.extend_from_slice("Français".as_bytes());
        assert_eq!(tag.data, ref_data);
    }

    #[test]
    fn test_parse_application_tag_character_string_dbcs() {
        let mut input = BytesMut::from(&[0x75, 0x1B][..]);
        input.extend_from_slice(
            &hex::decode("010352546869732069732061204241436E657420737472696E6721").unwrap(),
        );
        let (_, tag) = parse_bacnet_tag(&input).unwrap();
        assert!(matches!(
            tag.tag_number,
            TagNumber::Application(ApplicationTag::CharacterString)
        ));
        assert!(matches!(tag.lvt, LengthValueType::Length(27)));
        let ref_data =
            &hex::decode("010352546869732069732061204241436E657420737472696E6721").unwrap();
        assert_eq!(tag.data, ref_data);
    }

    #[test]
    fn test_parse_application_tag_character_string_ucs2() {
        let mut input = BytesMut::from(&[0x75, 0x31][..]);
        input.extend_from_slice(&hex::decode("040054006800690073002000690073002000610020004200410043006E0065007400200073007400720069006E00670021").unwrap());
        let (_, tag) = parse_bacnet_tag(&input).unwrap();
        assert!(matches!(
            tag.tag_number,
            TagNumber::Application(ApplicationTag::CharacterString)
        ));
        assert!(matches!(tag.lvt, LengthValueType::Length(49)));
        let ref_data = &hex::decode("040054006800690073002000690073002000610020004200410043006E0065007400200073007400720069006E00670021").unwrap();
        assert_eq!(tag.data, ref_data);
    }

    #[test]
    fn test_parse_application_tag_bit_string() {
        let input: &[u8] = &[0x82, 0x03, 0xA8];
        let (_, tag) = parse_bacnet_tag(input).unwrap();
        assert!(matches!(
            tag.tag_number,
            TagNumber::Application(ApplicationTag::BitString)
        ));
        assert!(matches!(tag.lvt, LengthValueType::Length(2)));
        assert_eq!(tag.data, &[0x03, 0xA8]);
    }

    #[test]
    fn test_parse_application_tag_enumerated_analog_input() {
        let input: &[u8] = &[0x91, 0x00];
        let (_, tag) = parse_bacnet_tag(input).unwrap();
        assert!(matches!(
            tag.tag_number,
            TagNumber::Application(ApplicationTag::Enumerated)
        ));
        assert!(matches!(tag.lvt, LengthValueType::Length(1)));
        assert_eq!(tag.data, &[0x00]);
    }

    #[test]
    fn test_parse_application_tag_date_specific_value() {
        let input: &[u8] = &[0xA4, 0x5B, 0x01, 0x18, 0x04];
        let (_, tag) = parse_bacnet_tag(input).unwrap();
        assert!(matches!(
            tag.tag_number,
            TagNumber::Application(ApplicationTag::Date)
        ));
        assert!(matches!(tag.lvt, LengthValueType::Length(4)));
        assert_eq!(tag.data, &[0x5B, 0x01, 0x18, 0x04]);
    }

    #[test]
    fn test_parse_application_tag_date_pattern() {
        let input: &[u8] = &[0xA4, 0x5B, 0xFF, 0x18, 0xFF];
        let (_, tag) = parse_bacnet_tag(input).unwrap();
        assert!(matches!(
            tag.tag_number,
            TagNumber::Application(ApplicationTag::Date)
        ));
        assert!(matches!(tag.lvt, LengthValueType::Length(4)));
        assert_eq!(tag.data, &[0x5B, 0xFF, 0x18, 0xFF]);
    }

    #[test]
    fn test_parse_application_tag_time_specific_value() {
        let input: &[u8] = &[0xB4, 0x11, 0x23, 0x2D, 0x11];
        let (_, tag) = parse_bacnet_tag(input).unwrap();
        assert!(matches!(
            tag.tag_number,
            TagNumber::Application(ApplicationTag::Time)
        ));
        assert!(matches!(tag.lvt, LengthValueType::Length(4)));
        assert_eq!(tag.data, &[0x11, 0x23, 0x2D, 0x11]);
    }

    #[test]
    fn test_parse_application_bacnet_object_identifier() {
        let input: &[u8] = &[0xC4, 0x00, 0xC0, 0x00, 0x0F];
        let (_, tag) = parse_bacnet_tag(input).unwrap();
        assert!(matches!(
            tag.tag_number,
            TagNumber::Application(ApplicationTag::BACnetObjectIdentifier)
        ));
        assert!(matches!(tag.lvt, LengthValueType::Length(4)));
        assert_eq!(tag.data, &[0x00, 0xC0, 0x00, 0x0F]);
    }

    #[test]
    fn test_decode_application_tag_number_0() {
        let (_, tag) = parse_bacnet_tag(&[0b0000_0_000]).unwrap();
        assert!(matches!(
            tag.tag_number,
            TagNumber::Application(ApplicationTag::Null)
        ));
    }

    #[test]
    fn test_decode_context_tag_number_0() {
        let (_, tag) = parse_bacnet_tag(&[0b0000_1_000]).unwrap();
        assert!(matches!(
            tag.tag_number,
            TagNumber::Context(ContextTag::Other(0))
        ));
    }

    #[test]
    fn test_decode_application_tag_number_14() {
        let (_, tag) = parse_bacnet_tag(&[0b1110_0_000]).unwrap();
        assert!(matches!(
            tag.tag_number,
            TagNumber::Application(ApplicationTag::Reserved(14))
        ));
    }

    #[test]
    fn test_decode_application_tag_number_15() {
        let (_, tag) = parse_bacnet_tag(&[0b1111_0_000, 15]).unwrap();
        assert!(matches!(
            tag.tag_number,
            TagNumber::Application(ApplicationTag::Reserved(15))
        ));
    }

    #[test]
    fn test_decode_application_tag_number_254() {
        let (_, tag) = parse_bacnet_tag(&[0b1111_0_000, 254]).unwrap();
        assert!(matches!(
            tag.tag_number,
            TagNumber::Application(ApplicationTag::Other(254))
        ));
    }

    #[test]
    fn test_decode_application_reserved_tag_number_255() {
        let (_, tag) = parse_bacnet_tag(&[0b1111_0_000, 255]).unwrap();
        assert!(matches!(
            tag.tag_number,
            TagNumber::Application(ApplicationTag::Other(255))
        ));
    }

    #[test]
    fn test_decode_length_0() {
        let (_, tag) = parse_bacnet_tag(&[0b0000_0_000]).unwrap();
        assert!(matches!(tag.lvt, LengthValueType::Length(0)));
    }

    #[test]
    fn test_decode_length_4() {
        let (_, tag) = parse_bacnet_tag(&[0b0000_0_100, 0, 0, 0, 0]).unwrap();
        assert!(matches!(tag.lvt, LengthValueType::Length(4)));
    }

    #[test]
    fn test_decode_length_5() {
        let mut input = BytesMut::from(&[0b0000_0_101, 5][..]);
        input.extend_from_slice(&[0u8; 5][..]);
        let (_, tag) = parse_bacnet_tag(&input).unwrap();
        assert!(matches!(tag.lvt, LengthValueType::Length(5)));
    }

    #[test]
    fn test_decode_length_253() {
        let mut input = BytesMut::from(&[0b0000_0_101, 253][..]);
        input.extend_from_slice(&[0u8; 253][..]);
        let (_, tag) = parse_bacnet_tag(&input).unwrap();
        assert!(matches!(tag.lvt, LengthValueType::Length(253)));
    }

    #[test]
    fn test_decode_length_254() {
        let mut input = BytesMut::from(&[0b0000_0_101, 254, 0, 254][..]);
        input.extend_from_slice(&[0u8; 254][..]);
        let (_, tag) = parse_bacnet_tag(&input).unwrap();
        assert!(matches!(tag.lvt, LengthValueType::Length(254)));
    }

    #[test]
    fn test_decode_length_65535() {
        let mut input = BytesMut::from(&[0b0000_0_101, 254, 255, 255][..]);
        input.extend_from_slice(&[0u8; 65535][..]);
        let (_, tag) = parse_bacnet_tag(&input).unwrap();
        assert!(matches!(tag.lvt, LengthValueType::Length(65535)));
    }

    #[test]
    fn test_length_65536() {
        let mut input = BytesMut::from(&[0b0000_0_101, 255, 0, 1, 0, 0][..]);
        input.extend_from_slice(&[0u8; 65536][..]);
        let (_, tag) = parse_bacnet_tag(&input).unwrap();
        assert!(matches!(tag.lvt, LengthValueType::Length(65536)));
    }

    /* TODO: These tests require to much memory! Find a better way to test them
    #[test]
    fn test_decode_length_u32max_minus_1() {
        let mut input = BytesMut::from(&[0b0000_0_101, 255, 255, 255, 255, 254][..]);
        input.extend_from_slice(&[0u8; (std::u32::MAX - 1) as usize][..]);
        let (_, tag) = parse_bacnet_tag(&input).unwrap();
        if let LengthValueType::Length(l) = tag.lvt {
            assert_eq!(l, std::u32::MAX - 1);
        } else {
            panic!("Not a LengthValueType::Length");
        };
    }

    #[test]
    fn test_reserved_length_u32max() {
        let mut input = BytesMut::from(&[0b0000_0_101, 255, 255, 255, 255, 255][..]);
        input.extend_from_slice(&[0u8; std::u32::MAX as usize][..]);
        let (_, tag) = parse_bacnet_tag(&input).unwrap();
        assert!(matches!(tag.lvt, LengthValueType::Length(std::u32::MAX)));
    }*/
}
