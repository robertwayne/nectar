#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]

// RFC 854 `<https://tools.ietf.org/html/rfc854>`
//
// Originally based off of https://github.com/jtenner/telnet_codec, which has
// been archived.

use std::mem;

use bytes::{Buf, BufMut, Bytes, BytesMut};
use tokio_util::codec::{Decoder, Encoder};

use crate::{
    constants::{
        CHARSET, CHARSET_ACCEPTED, CHARSET_REJECTED, CHARSET_REQUEST, CHARSET_TTABLE_REJECTED, DO,
        DONT, ENVIRON, IAC, LINEMODE, LINEMODE_FORWARD_MASK, LINEMODE_SLC, MODE, NAWS, NOP, SB, SE,
        WILL, WONT,
    },
    env::{decode_env, encode_env_op},
    error::TelnetError,
    event::TelnetEvent,
    linemode::ForwardMaskOption,
    option::TelnetOption,
    subnegotiation::{LineModeOption, SubnegotiationType},
};

/// Various byte or byte sequences used in the Telnet protocol.
pub mod constants;
/// Telnet environment options
pub mod env;
/// Codec and Io errors that may occur while processing Telnet events.
pub mod error;
/// Top-level Telnet events, such as Message, Do, Will, and Subnegotiation.
pub mod event;
/// Telnet linemode options
pub mod linemode;
/// Telnet options such as `Echo`, `GoAhead`, and `SuppressGoAhead`.
pub mod option;
/// Telnet subnegotiation options.
pub mod subnegotiation;

type Result<T> = std::result::Result<T, TelnetError>;

/// Implements a Tokio codec for the Telnet protocol, along with MUD-specific
/// extension protocols such as GMCP.
///
/// You should never have to interact with this directly.
#[derive(Debug)]
pub struct TelnetCodec {
    /// Whether or not the client has enabled the Suppress Go Ahead option.
    pub sga: bool,
    pub max_buffer_length: usize,
    pub buffer: Vec<u8>,
    /// If this field is set to false, nectar will generate an event for each
    /// character instead of each message
    pub message_mode: bool,
    /// Attempt to parse unicode when received
    #[cfg(feature = "unicode")]
    pub unicode: bool,
}

impl TelnetCodec {
    #[must_use]
    pub fn new(max_buffer_length: usize) -> Self {
        TelnetCodec {
            sga: false,
            max_buffer_length,
            buffer: Vec::new(),
            message_mode: true,
            #[cfg(feature = "unicode")]
            unicode: false,
        }
    }
}

impl Decoder for TelnetCodec {
    type Item = TelnetEvent;
    type Error = TelnetError;

    fn decode(&mut self, buffer: &mut BytesMut) -> Result<Option<Self::Item>> {
        let mut byte_index = 0;

        if self.sga && !self.buffer.is_empty() {
            let buf = mem::take(&mut self.buffer);
            let result = String::from_utf8_lossy(&buf[..]);

            return Ok(Some(TelnetEvent::Message(result.to_string())));
        }

        if buffer.is_empty() {
            return Ok(None);
        }

        if self.sga {
            return Ok(decode_suppress_go_ahead(&mut byte_index, buffer));
        }

        Ok(decode_bytes(self, &mut byte_index, buffer))
    }
}

impl Encoder<TelnetEvent> for TelnetCodec {
    type Error = TelnetError;

    fn encode(&mut self, event: TelnetEvent, buffer: &mut BytesMut) -> Result<()> {
        match event {
            TelnetEvent::Do(option) => encode_negotiate(DO, option, buffer),
            TelnetEvent::Dont(option) => encode_negotiate(DONT, option, buffer),
            TelnetEvent::Will(option) => encode_negotiate(WILL, option, buffer),
            TelnetEvent::Wont(option) => encode_negotiate(WONT, option, buffer),
            TelnetEvent::Subnegotiate(sb_type) => encode_sb(sb_type, buffer),
            TelnetEvent::Message(msg) => encode_message(msg, buffer),
            TelnetEvent::RawMessage(msg) => encode_raw_message(msg, buffer),
            _ => {}
        }

        Ok(())
    }
}

#[cfg(feature = "unicode")]
fn decode_utf8(byte_index: usize, buffer: &mut BytesMut, start: u8) -> Option<TelnetEvent> {
    let length = match start {
        0xC2..=0xDF => 2,
        0xE0..=0xEF => 3,
        // In theory this should never happen...
        0xF0..=0xF4 => 4,
        _ => 1,
    };

    if length == 1 {
        buffer.advance(byte_index + 1);
        Some(TelnetEvent::Unicode(start as char))
    } else {
        if let Ok(s) = std::str::from_utf8(&buffer[byte_index..byte_index + length]) {
            if s.chars().count() != 1 {
                // Something weird happened here...
                // Maybe we should disconnect / fail here instead

                buffer.advance(byte_index + length);
                return Some(TelnetEvent::Nop);
            }

            // We can unwrap here since we checked it above.
            let c = s.chars().next().unwrap();

            buffer.advance(byte_index + length);
            return Some(TelnetEvent::Unicode(c));
        }

        // We were unable to parse the unicode...
        // Discard the input and act like nothing happened!

        buffer.advance(byte_index + length);
        Some(TelnetEvent::Nop)
    }
}

fn decode_negotiate(byte_index: usize, buffer: &mut BytesMut, option: u8) -> Option<TelnetEvent> {
    if byte_index + 2 >= buffer.len() {
        return None;
    }

    let byte = buffer[byte_index + 2];
    buffer.advance(byte_index + 3);
    match option {
        WILL => Some(TelnetEvent::Will(byte.into())),
        WONT => Some(TelnetEvent::Wont(byte.into())),
        DO => Some(TelnetEvent::Do(byte.into())),
        DONT => Some(TelnetEvent::Dont(byte.into())),
        _ => None,
    }
}

fn decode_suppress_go_ahead(byte_index: &mut usize, buffer: &mut BytesMut) -> Option<TelnetEvent> {
    match buffer[0] {
        IAC => {
            if 1 >= buffer.len() {
                return None;
            }

            match buffer[*byte_index + 1] {
                IAC => {
                    buffer.advance(2);
                    Some(TelnetEvent::Character(IAC))
                }
                _ => None,
            }
        }
        _ => None,
    }
}

fn decode_negotiate_about_window_size(subvec: &[u8]) -> Option<TelnetEvent> {
    match subvec.len() {
        4 => {
            let result = SubnegotiationType::WindowSize(
                (u16::from(subvec[0]) << 8) | u16::from(subvec[1]),
                (u16::from(subvec[2]) << 8) | u16::from(subvec[3]),
            );
            Some(TelnetEvent::Subnegotiate(result))
        }
        _ => None,
    }
}

fn decode_linemode(subvec: &[u8]) -> Option<TelnetEvent> {
    if subvec.is_empty() {
        return None;
    }

    let suboption = match subvec[0] {
        WILL | WONT | DO | DONT => LineModeOption::ForwardMask(ForwardMaskOption::from(subvec[0])),
        _ => LineModeOption::from(subvec[0]),
    };

    match suboption {
        LineModeOption::SLC(_) => {
            let slc_data = &subvec[1..];

            // (function, flag, value)
            let slc_triples = slc_data
                .chunks_exact(3)
                .map(|chunk| ((chunk[0], chunk[1]).into(), chunk[2] as char))
                .collect();
            Some(TelnetEvent::Subnegotiate(SubnegotiationType::LineMode(LineModeOption::SLC(
                slc_triples,
            ))))
        }
        LineModeOption::ForwardMask(_) => {
            let data = &subvec[2..];
            let option = match subvec[0] {
                DO => ForwardMaskOption::Do(data.to_vec()),
                byte => ForwardMaskOption::from(byte),
            };

            Some(TelnetEvent::Subnegotiate(SubnegotiationType::LineMode(
                LineModeOption::ForwardMask(option),
            )))
        }
        LineModeOption::Mode(_) => {
            let mode = subvec[1];

            Some(TelnetEvent::Subnegotiate(SubnegotiationType::LineMode(LineModeOption::Mode(
                mode,
            ))))
        }
        LineModeOption::Unknown(_, _) => {
            let data = &subvec[1..];
            Some(TelnetEvent::Subnegotiate(SubnegotiationType::LineMode(LineModeOption::Unknown(
                subvec[0],
                Bytes::from(data.to_vec()),
            ))))
        }
    }
}

fn decode_charset(subvec: &[u8]) -> Option<TelnetEvent> {
    if subvec.is_empty() {
        return None;
    }

    match subvec[0] {
        CHARSET_REQUEST => {
            if subvec.len() == 1 {
                return None;
            }

            let separator = subvec[1];
            let charsets: Vec<_> =
                subvec[2..].split(|&x| x == separator).map(|x| Bytes::from(x.to_vec())).collect();

            if charsets.is_empty() {
                return None;
            }

            let result = SubnegotiationType::CharsetRequest(charsets);
            Some(TelnetEvent::Subnegotiate(result))
        }
        CHARSET_ACCEPTED => {
            let result = SubnegotiationType::CharsetAccepted(Bytes::from(subvec[1..].to_vec()));
            Some(TelnetEvent::Subnegotiate(result))
        }
        CHARSET_REJECTED => {
            let result = SubnegotiationType::CharsetRejected;
            Some(TelnetEvent::Subnegotiate(result))
        }
        CHARSET_TTABLE_REJECTED => {
            let result = SubnegotiationType::CharsetTTableRejected;
            Some(TelnetEvent::Subnegotiate(result))
        }
        _ => None,
    }
}

fn decode_unknown(option: u8, subvec: Vec<u8>) -> TelnetEvent {
    TelnetEvent::Subnegotiate(SubnegotiationType::Unknown(option.into(), Bytes::from(subvec)))
}

fn decode_next_byte(codec: &mut TelnetCodec, buffer_size: &mut usize, byte: u8) {
    if buffer_size < &mut codec.max_buffer_length {
        codec.buffer.push(byte);
        *buffer_size += 1;
    }
}

fn decode_subnegotiation_end(
    invalid: bool,
    buffer: &mut BytesMut,
    subvec: Vec<u8>,
    option: u8,
) -> Option<TelnetEvent> {
    if invalid {
        None
    } else {
        let opt = match option {
            NAWS => decode_negotiate_about_window_size(&subvec),
            CHARSET => decode_charset(&subvec),
            LINEMODE => decode_linemode(&subvec),
            ENVIRON => decode_env(&subvec),
            _ => Some(decode_unknown(option, subvec)),
        };

        if let Some(event) = &opt {
            buffer.advance(event.len());
        }

        opt
    }
}

fn decode_bytes(
    codec: &mut TelnetCodec,
    byte_index: &mut usize,
    buffer: &mut BytesMut,
) -> Option<TelnetEvent> {
    let mut codec_buffer_size = codec.buffer.len();

    loop {
        if *byte_index >= buffer.len() {
            return None;
        }

        // Handle matches against the first byte in the buffer.
        match buffer[*byte_index] {
            IAC => {
                if *byte_index + 1 >= buffer.len() {
                    return None;
                }

                // Handle matches against the second byte in the buffer.
                match buffer[*byte_index + 1] {
                    IAC => {
                        if codec.buffer.len() < codec.max_buffer_length {
                            codec.buffer.push(IAC);
                            codec_buffer_size += 1;
                        }

                        *byte_index += 1;
                    }
                    DO => return decode_negotiate(*byte_index, buffer, DO),
                    DONT => return decode_negotiate(*byte_index, buffer, DONT),
                    WILL => return decode_negotiate(*byte_index, buffer, WILL),
                    WONT => return decode_negotiate(*byte_index, buffer, WONT),
                    SB => {
                        if *byte_index + 2 >= buffer.len() {
                            buffer.advance(*byte_index + 2);
                            return None;
                        }

                        let start = *byte_index;
                        let opt = buffer[*byte_index + 2];

                        *byte_index += 3;

                        let mut subvec: Vec<u8> = Vec::new();
                        let mut invalid = false;

                        loop {
                            if *byte_index > buffer.len() {
                                buffer.advance(start);
                                return None;
                            }

                            // Handle matches against the third byte in the
                            // buffer. This is for subnegotiation.
                            match buffer[*byte_index] {
                                IAC => {
                                    if *byte_index + 1 > buffer.len() {
                                        return None;
                                    }

                                    // Handle matches against the fourth byte in
                                    // the buffer. This is the final byte in the
                                    // buffer.
                                    match buffer[*byte_index + 1] {
                                        SE => {
                                            return decode_subnegotiation_end(
                                                invalid, buffer, subvec, opt,
                                            )
                                        }
                                        IAC => subvec.push(IAC),
                                        _ => invalid = true,
                                    }

                                    *byte_index += 1;
                                }
                                _ => subvec.push(buffer[*byte_index]),
                            }

                            *byte_index += 1;
                        }
                    }
                    NOP => *byte_index += 1,
                    _ => {}
                }
            }
            b'\n' => {
                let mut codec_buffer = mem::take(&mut codec.buffer);
                if codec_buffer.ends_with(&[b'\r']) {
                    codec_buffer.pop();
                    buffer.advance(*byte_index + 1);

                    let result = String::from_utf8_lossy(&codec_buffer[..]);
                    return Some(TelnetEvent::Message(result.to_string()));
                }

                decode_next_byte(codec, &mut codec_buffer_size, buffer[*byte_index]);
            }
            #[cfg(not(feature = "unicode"))]
            c if !codec.message_mode => {
                let mut codec_buffer = mem::take(&mut codec.buffer);
                codec_buffer.pop();
                buffer.advance(*byte_index + 1);
                return Some(TelnetEvent::Character(c));
            }

            #[cfg(feature = "unicode")]
            c if !codec.message_mode => {
                // Unicode support is compiled in but not enabled,
                // so just pass characters on as they are

                if !codec.unicode {
                    let mut codec_buffer = mem::take(&mut codec.buffer);
                    codec_buffer.pop();
                    buffer.advance(*byte_index + 1);
                    return Some(TelnetEvent::Character(c));
                }

                return decode_utf8(*byte_index, buffer, c);
            }
            _ => decode_next_byte(codec, &mut codec_buffer_size, buffer[*byte_index]),
        };

        *byte_index += 1;
    }
}

fn encode_negotiate(opt: u8, subopt: TelnetOption, buf: &mut BytesMut) {
    buf.reserve(3);
    buf.put_u8(IAC);

    match opt {
        DO => buf.put_u8(DO),
        DONT => buf.put_u8(DONT),
        WILL => buf.put_u8(WILL),
        WONT => buf.put_u8(WONT),
        _ => unreachable!(),
    }

    buf.put_u8(subopt.into());
}

fn encode_sb(sb: SubnegotiationType, buffer: &mut BytesMut) {
    match sb {
        SubnegotiationType::WindowSize(width, height) => {
            buffer.reserve(9);
            buffer.extend([IAC, SB, NAWS]);
            buffer.put_u16(width);
            buffer.put_u16(height);
            buffer.extend([IAC, SE]);
        }
        SubnegotiationType::CharsetRequest(charsets) => {
            let charset_lens = charsets.iter().map(|c| c.len()).sum::<usize>();
            let spaces = charsets.len().saturating_sub(1);

            buffer.reserve(7 + charset_lens + spaces);
            let sep = b' ';
            buffer.extend([IAC, SB, CHARSET, CHARSET_REQUEST, sep]);

            for (i, charset) in charsets.iter().enumerate() {
                buffer.extend(charset);
                if i < charsets.len() - 1 {
                    buffer.put_u8(sep);
                }
            }

            buffer.extend([IAC, SE]);
        }
        SubnegotiationType::CharsetAccepted(charset) => {
            buffer.reserve(6 + charset.len());
            buffer.extend([IAC, SB, CHARSET, CHARSET_ACCEPTED]);
            buffer.extend(charset);
            buffer.extend([IAC, SE]);
        }
        SubnegotiationType::CharsetRejected => {
            buffer.reserve(6);
            buffer.extend([IAC, SB, CHARSET, CHARSET_REJECTED, IAC, SE]);
        }
        SubnegotiationType::CharsetTTableRejected => {
            buffer.reserve(6);
            buffer.extend([IAC, SB, CHARSET, CHARSET_TTABLE_REJECTED, IAC, SE]);
        }
        SubnegotiationType::Environment(op) => {
            buffer.extend([IAC, SB, ENVIRON]);
            encode_env_op(op, buffer);
            buffer.extend([IAC, SE]);
        }
        SubnegotiationType::Unknown(option, bytes) => {
            let mut bytes_buffer_size = bytes.len() + 5;

            for byte in &bytes {
                if *byte == IAC {
                    bytes_buffer_size += 1;
                }
            }

            buffer.reserve(bytes_buffer_size);

            // IAC SUB OPTION
            buffer.extend([IAC, SB, option.into()]);

            // Write to the buffer
            for byte in &bytes {
                if *byte == IAC {
                    buffer.extend([IAC, IAC]);
                } else {
                    buffer.put_u8(*byte);
                }
            }

            // IAC SUBNEGOTIATION END
            buffer.extend([IAC, SE]);
        }
        SubnegotiationType::LineMode(mode) => match mode {
            LineModeOption::Mode(value) => {
                buffer.reserve(7);
                buffer.extend([IAC, SB, LINEMODE, MODE, value, IAC, SE]);
            }
            LineModeOption::SLC(values) => {
                // 4: Subnegotiation begin values.len() * 3: each entry
                // symbolizes a triple of bytes:
                // - Function
                // - Modifiers (acknowledgement, urgency etc.)
                // - Character 2: Subnegotiation end

                buffer.reserve(6 + values.len() * 3);
                buffer.extend([IAC, SB, LINEMODE, LINEMODE_SLC]);

                for &(dispatch, char) in &values {
                    let (first, second) = dispatch.into();
                    buffer.extend([first, second, char as u8]);
                }

                buffer.extend([IAC, SE]);
            }
            LineModeOption::ForwardMask(ForwardMaskOption::Do(data)) => {
                // Note: this needs to be 32 bytes in binary mode
                buffer.reserve(7 + 16);

                buffer.extend([IAC, SB, LINEMODE, DO, LINEMODE_FORWARD_MASK]);

                let iter = data.into_iter().take(16);
                let zeros = std::iter::repeat(0).take(16 - iter.len());

                buffer.extend(iter.chain(zeros));
                buffer.extend([IAC, SE]);
            }
            LineModeOption::ForwardMask(option) => {
                buffer.reserve(7);
                buffer.extend([IAC, SB, LINEMODE, option.into(), LINEMODE_FORWARD_MASK, IAC, SE]);
            }
            LineModeOption::Unknown(option, data) => {
                buffer.reserve(7 + data.len());
                buffer.extend([IAC, SB, LINEMODE, option]);
                buffer.extend(data);
                buffer.extend([IAC, SE]);
            }
        },
    }
}

fn encode_raw_message(message: String, buffer: &mut BytesMut) {
    let bytes = Bytes::from(message);
    let mut bytes_buffer_size = bytes.len();

    for byte in &bytes {
        if *byte == IAC {
            bytes_buffer_size += 1;
        }
    }

    buffer.reserve(bytes_buffer_size);

    for byte in &bytes {
        if *byte == IAC {
            buffer.extend([IAC, IAC]);
        }
        buffer.put_u8(*byte);
    }
}

fn encode_message(message: String, buffer: &mut BytesMut) {
    encode_raw_message(message, buffer);

    if !buffer.ends_with(b"\r\n") {
        buffer.reserve(2);
        buffer.extend([b'\r', b'\n']);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() -> (TelnetCodec, BytesMut) {
        let codec = TelnetCodec::new(16);
        let buffer = BytesMut::new();
        (codec, buffer)
    }

    mod test_decode {
        use super::*;

        #[test]
        fn test_sga_true() {
            let (mut codec, mut buffer) = setup();
            codec.sga = true;

            // when both the codec's internal buffer, and the input buffer are
            // empty, there's nothing going on.
            assert!(codec.decode(&mut buffer).unwrap().is_none());

            // when the codec's internal buffer is not empty, clear it out and
            // send it as a message
            codec.buffer.extend([b'h', b'i', b'y', b'a', b' ', 0xf0, 0x9f, 0x98, 0x81]);
            assert_eq!(
                codec.decode(&mut buffer).unwrap().unwrap(),
                TelnetEvent::Message("hiya 😁".to_string())
            );
            assert!(codec.buffer.is_empty());

            // when the codec's internal buffer is empty, and the input buffer
            // has data, decode as a SuppressGoAhead
            buffer.extend([IAC]);
            assert!(codec.decode(&mut buffer).unwrap().is_none());
            assert!(codec.buffer.is_empty());
            assert_eq!(buffer.as_ref(), &[IAC]);
            buffer.extend([IAC]); // Add a second, as two are interpreted as a single IAC
            assert_eq!(codec.decode(&mut buffer).unwrap().unwrap(), TelnetEvent::Character(IAC));
            assert!(codec.buffer.is_empty());
            assert!(buffer.is_empty());

            // Ignore IAC followed by non-IAC
            buffer.extend([IAC, WILL]);
            assert!(codec.decode(&mut buffer).unwrap().is_none());
            assert!(codec.buffer.is_empty());
            assert_eq!(buffer.as_ref(), &[IAC, WILL]);

            // Ignore non-IAC followed by IAC
            buffer.extend([WILL, IAC]);
            assert!(codec.decode(&mut buffer).unwrap().is_none());
            assert!(codec.buffer.is_empty());
            assert_eq!(buffer.as_ref(), &[IAC, WILL, WILL, IAC]); // previous stuff is still there
        }

        mod test_sga_false {
            use super::*;

            #[test]
            fn test_buffer_starts_with_newline() {
                let (mut codec, mut buffer) = setup();

                codec.buffer.extend([b'c', b'o', b'o', b'l', b'!', b'\r']);
                buffer.extend([b'\n', b'y', b'e', b's']);

                // when the newline completes a \r\n sequence, send the contents
                // of the codec's internal buffer as a message
                assert_eq!(
                    codec.decode(&mut buffer).unwrap().unwrap(),
                    TelnetEvent::Message("cool!".to_string())
                );
                assert!(codec.buffer.is_empty());
                assert_eq!(buffer.as_ref(), &[b'y', b'e', b's']);

                // When the character does not complete a \r\n sequence, and is
                // not IAC, append it to the codec's internal buffer, but do not
                // remove it from the input buffer.
                assert_eq!(codec.decode(&mut buffer).unwrap(), None);
                assert_eq!(&codec.buffer, &[b'y', b'e', b's']);
                assert_eq!(buffer.as_ref(), &[b'y', b'e', b's']);
            }

            #[test]
            fn test_overflow() {
                let (mut codec, mut buffer) = setup();

                buffer.extend([b'a'; 10]);
                buffer.extend([b'z'; 10]);

                assert!(codec.decode(&mut buffer).unwrap().is_none());

                assert_eq!(&codec.buffer[..=9], &[b'a'; 10]);
                assert_eq!(&codec.buffer[10..], &[b'z'; 6]);

                assert_eq!(&buffer[..=9], &[b'a'; 10]);
                assert_eq!(&buffer[10..], &[b'z'; 10]);
            }

            mod test_iac {
                use crate::constants::ECHO;

                use super::*;

                #[test]
                fn test_double_iac() {
                    let (mut codec, mut buffer) = setup();

                    // a doubled IAC on the wire is interpreted as a single byte
                    // of data
                    buffer.extend([IAC, IAC]);
                    assert_eq!(codec.decode(&mut buffer).unwrap(), None);
                    assert_eq!(&codec.buffer, &[IAC]);
                    assert_eq!(buffer.as_ref(), &[IAC, IAC]);
                }

                #[test]
                fn test_do() {
                    let (mut codec, mut buffer) = setup();

                    buffer.extend([IAC, DO, ECHO]);
                    assert_eq!(
                        codec.decode(&mut buffer).unwrap().unwrap(),
                        TelnetEvent::Do(TelnetOption::Echo)
                    );
                    assert!(codec.buffer.is_empty());
                    assert!(buffer.is_empty());
                }

                #[test]
                fn test_dont() {
                    let (mut codec, mut buffer) = setup();

                    buffer.extend([IAC, DONT, ECHO]);
                    assert_eq!(
                        codec.decode(&mut buffer).unwrap().unwrap(),
                        TelnetEvent::Dont(TelnetOption::Echo)
                    );
                    assert!(codec.buffer.is_empty());
                    assert!(buffer.is_empty());
                }

                #[test]
                fn test_will() {
                    let (mut codec, mut buffer) = setup();

                    buffer.extend([IAC, WILL, ECHO]);
                    assert_eq!(
                        codec.decode(&mut buffer).unwrap().unwrap(),
                        TelnetEvent::Will(TelnetOption::Echo)
                    );
                    assert!(codec.buffer.is_empty());
                    assert!(buffer.is_empty());
                }

                #[test]
                fn test_wont() {
                    let (mut codec, mut buffer) = setup();

                    buffer.extend([IAC, WONT, ECHO]);
                    assert_eq!(
                        codec.decode(&mut buffer).unwrap().unwrap(),
                        TelnetEvent::Wont(TelnetOption::Echo)
                    );
                    assert!(codec.buffer.is_empty());
                    assert!(buffer.is_empty());
                }

                #[test]
                fn test_nop() {
                    let (mut codec, mut buffer) = setup();

                    buffer.extend([IAC, NOP]);
                    assert_eq!(codec.decode(&mut buffer).unwrap(), None);
                    assert!(codec.buffer.is_empty());
                    assert_eq!(buffer.as_ref(), &[IAC, NOP]);
                }

                #[test]
                fn test_sb_naws() {
                    let (mut codec, mut buffer) = setup();

                    buffer.extend([IAC, SB, NAWS, 0x00, 0x50, 0x00, 0x50, IAC, SE]);
                    assert_eq!(
                        codec.decode(&mut buffer).unwrap().unwrap(),
                        TelnetEvent::Subnegotiate(SubnegotiationType::WindowSize(80, 80))
                    );
                    assert!(codec.buffer.is_empty());
                    assert!(buffer.is_empty());
                }

                #[test]
                fn test_sb_charset_request() {
                    let (mut codec, mut buffer) = setup();

                    buffer.extend([IAC, SB, CHARSET, CHARSET_REQUEST, b' ']);
                    buffer.extend("UTF-8".bytes());
                    buffer.put_u8(b' ');
                    buffer.extend("US-ASCII".bytes());
                    buffer.extend([IAC, SE]);

                    assert_eq!(
                        codec.decode(&mut buffer).unwrap().unwrap(),
                        TelnetEvent::Subnegotiate(SubnegotiationType::CharsetRequest(vec![
                            Bytes::from("UTF-8"),
                            Bytes::from("US-ASCII")
                        ]))
                    );
                    assert!(codec.buffer.is_empty());
                    assert!(buffer.is_empty());
                }

                #[test]
                fn test_sb_charset_accepted() {
                    let (mut codec, mut buffer) = setup();

                    buffer.extend([IAC, SB, CHARSET, CHARSET_ACCEPTED]);
                    buffer.extend("UTF-8".bytes());
                    buffer.extend([IAC, SE]);

                    assert_eq!(
                        codec.decode(&mut buffer).unwrap().unwrap(),
                        TelnetEvent::Subnegotiate(SubnegotiationType::CharsetAccepted(
                            Bytes::from("UTF-8")
                        ))
                    );
                    assert!(codec.buffer.is_empty());
                    assert!(buffer.is_empty());
                }

                #[test]
                fn test_sb_charset_rejected() {
                    let (mut codec, mut buffer) = setup();

                    buffer.extend([IAC, SB, CHARSET, CHARSET_REJECTED, IAC, SE]);

                    assert_eq!(
                        codec.decode(&mut buffer).unwrap().unwrap(),
                        TelnetEvent::Subnegotiate(SubnegotiationType::CharsetRejected)
                    );
                    assert!(codec.buffer.is_empty());
                    assert!(buffer.is_empty());
                }

                #[test]
                fn test_sb_charset_ttable_rejected() {
                    let (mut codec, mut buffer) = setup();

                    buffer.extend([IAC, SB, CHARSET, CHARSET_TTABLE_REJECTED, IAC, SE]);

                    assert_eq!(
                        codec.decode(&mut buffer).unwrap().unwrap(),
                        TelnetEvent::Subnegotiate(SubnegotiationType::CharsetTTableRejected)
                    );
                    assert!(codec.buffer.is_empty());
                    assert!(buffer.is_empty());
                }
            }
        }
    }

    mod test_encode {
        use crate::{
            constants::{ECHO, LINEMODE_EDIT, SLC_ABORT, SLC_BRK, SLC_SYNCH},
            linemode::{Dispatch, SlcFunction},
        };

        use super::*;

        #[test]
        fn test_message() {
            let (mut codec, mut buffer) = setup();
            codec.encode(TelnetEvent::Message("hiya 😁".to_string()), &mut buffer).unwrap();
            assert_eq!(buffer.as_ref(), b"hiya \xF0\x9F\x98\x81\r\n");

            let (mut codec, mut buffer) = setup();
            let msg = "this message is larger than the max buffer length".to_string();
            assert!(msg.len() > codec.max_buffer_length);
            codec.encode(TelnetEvent::Message(msg), &mut buffer).unwrap();
            assert_eq!(buffer.as_ref(), b"this message is larger than the max buffer length\r\n");
        }

        #[test]
        #[cfg(feature = "unicode")]
        fn test_unicode() {
            let (mut codec, mut buffer) = setup();
            codec.message_mode = false;
            codec.unicode = true;
            codec.sga = false;

            buffer.extend(b"\xC3\xA4");

            let result = codec.decode(&mut buffer);

            assert!(matches!(result, Ok(Some(TelnetEvent::Unicode('ä')))));
        }

        #[test]
        fn test_raw_message() {
            let (mut codec, mut buffer) = setup();
            codec.encode(TelnetEvent::RawMessage("hiya 😁".to_string()), &mut buffer).unwrap();
            assert_eq!(buffer.as_ref(), b"hiya \xF0\x9F\x98\x81");
        }

        #[test]
        fn test_do() {
            let (mut codec, mut buffer) = setup();
            codec.encode(TelnetEvent::Do(TelnetOption::Echo), &mut buffer).unwrap();
            assert_eq!(buffer.as_ref(), &[IAC, DO, ECHO]);
        }

        #[test]
        fn test_dont() {
            let (mut codec, mut buffer) = setup();
            codec.encode(TelnetEvent::Dont(TelnetOption::Echo), &mut buffer).unwrap();
            assert_eq!(buffer.as_ref(), &[IAC, DONT, ECHO]);
        }

        #[test]
        fn test_will() {
            let (mut codec, mut buffer) = setup();
            codec.encode(TelnetEvent::Will(TelnetOption::Echo), &mut buffer).unwrap();
            assert_eq!(buffer.as_ref(), &[IAC, WILL, ECHO]);
        }

        #[test]
        fn test_wont() {
            let (mut codec, mut buffer) = setup();
            codec.encode(TelnetEvent::Wont(TelnetOption::Echo), &mut buffer).unwrap();
            assert_eq!(buffer.as_ref(), &[IAC, WONT, ECHO]);
        }

        #[test]
        fn test_sb_naws() {
            let (mut codec, mut buffer) = setup();
            codec
                .encode(
                    TelnetEvent::Subnegotiate(SubnegotiationType::WindowSize(80, 80)),
                    &mut buffer,
                )
                .unwrap();
            assert_eq!(buffer.as_ref(), &[IAC, SB, NAWS, 0x00, 0x50, 0x00, 0x50, IAC, SE]);
        }

        #[test]
        fn test_sb_charset_request() {
            let (mut codec, mut buffer) = setup();
            codec
                .encode(
                    TelnetEvent::Subnegotiate(SubnegotiationType::CharsetRequest(vec![
                        Bytes::from("UTF-8"),
                        Bytes::from("US-ASCII"),
                    ])),
                    &mut buffer,
                )
                .unwrap();
            assert_eq!(&buffer.as_ref()[0..=4], &[IAC, SB, CHARSET, CHARSET_REQUEST, b' ']);
            assert_eq!(&buffer.as_ref()[5..], b"UTF-8 US-ASCII\xFF\xF0" as &[u8]);
        }

        #[test]
        fn test_sb_charset_accepted() {
            let (mut codec, mut buffer) = setup();
            codec
                .encode(
                    TelnetEvent::Subnegotiate(SubnegotiationType::CharsetAccepted(Bytes::from(
                        "UTF-8",
                    ))),
                    &mut buffer,
                )
                .unwrap();
            assert_eq!(&buffer.as_ref()[0..=3], &[IAC, SB, CHARSET, CHARSET_ACCEPTED]);
            assert_eq!(&buffer.as_ref()[4..], b"UTF-8\xFF\xF0" as &[u8]);
        }

        #[test]
        fn test_sb_charset_rejected() {
            let (mut codec, mut buffer) = setup();
            codec
                .encode(TelnetEvent::Subnegotiate(SubnegotiationType::CharsetRejected), &mut buffer)
                .unwrap();
            assert_eq!(buffer.as_ref(), &[IAC, SB, CHARSET, CHARSET_REJECTED, IAC, SE]);
        }

        #[test]
        fn test_sb_charset_ttable_rejected() {
            let (mut codec, mut buffer) = setup();
            codec
                .encode(
                    TelnetEvent::Subnegotiate(SubnegotiationType::CharsetTTableRejected),
                    &mut buffer,
                )
                .unwrap();
            assert_eq!(buffer.as_ref(), &[IAC, SB, CHARSET, CHARSET_TTABLE_REJECTED, IAC, SE]);
        }

        #[test]
        fn test_sb_linemode_mode_encode() {
            let (mut codec, mut buffer) = setup();
            codec
                .encode(
                    TelnetEvent::Subnegotiate(SubnegotiationType::LineMode(LineModeOption::Mode(
                        LINEMODE_EDIT,
                    ))),
                    &mut buffer,
                )
                .unwrap();

            assert_eq!(buffer.as_ref(), &[IAC, SB, LINEMODE, MODE, LINEMODE_EDIT, IAC, SE]);
        }

        #[test]
        fn test_sb_linemode_mode_decode() {
            let (mut codec, mut buffer) = setup();
            buffer.extend([IAC, SB, LINEMODE, MODE, LINEMODE_EDIT, IAC, SE]);
            let event = codec.decode(&mut buffer).unwrap().unwrap();
            match event {
                TelnetEvent::Subnegotiate(SubnegotiationType::LineMode(LineModeOption::Mode(
                    mode,
                ))) => {
                    assert_eq!(mode, LINEMODE_EDIT);
                }
                _ => panic!("Bad decode!"),
            };
        }

        #[test]
        fn test_sb_linemode_slc_encode() {
            let (mut codec, mut buffer) = setup();
            let triples = [
                (Dispatch::from((SLC_ABORT, 0)), '0'),
                (Dispatch::from((SLC_SYNCH, 0)), '1'),
                (Dispatch::from((SLC_BRK, 0)), '2'),
            ];

            codec
                .encode(
                    TelnetEvent::Subnegotiate(SubnegotiationType::LineMode(LineModeOption::SLC(
                        triples.to_vec(),
                    ))),
                    &mut buffer,
                )
                .unwrap();

            assert_eq!(
                buffer.as_ref(),
                &[
                    IAC,
                    SB,
                    LINEMODE,
                    LINEMODE_SLC,
                    SLC_ABORT,
                    0,
                    b'0',
                    SLC_SYNCH,
                    0,
                    b'1',
                    SLC_BRK,
                    0,
                    b'2',
                    IAC,
                    SE
                ]
            )
        }

        #[test]
        fn test_sb_linemode_unk_decode() {
            let (mut codec, mut buffer) = setup();

            buffer.extend([IAC, SB, LINEMODE, 123, 1, 2, 3, 4, 5, 6, IAC, SE]);

            let event = codec.decode(&mut buffer).unwrap().unwrap();

            match event {
                TelnetEvent::Subnegotiate(SubnegotiationType::LineMode(
                    LineModeOption::Unknown(123, data),
                )) => {
                    assert_eq!(data.as_ref(), &[1, 2, 3, 4, 5, 6]);
                }
                _ => panic!("Bad decode!"),
            }
        }

        #[test]
        fn test_sb_linemode_unk_encode() {
            let (mut codec, mut buffer) = setup();

            codec
                .encode(
                    TelnetEvent::Subnegotiate(SubnegotiationType::LineMode(
                        LineModeOption::Unknown(123, [1, 2, 3, 4, 5, 6].to_vec().into()),
                    )),
                    &mut buffer,
                )
                .unwrap();

            assert_eq!(buffer.as_ref(), &[IAC, SB, LINEMODE, 123, 1, 2, 3, 4, 5, 6, IAC, SE]);
        }

        #[test]
        fn test_sb_linemode_slc_decode() {
            let (mut codec, mut buffer) = setup();

            buffer.extend([
                IAC,
                SB,
                LINEMODE,
                LINEMODE_SLC,
                SLC_ABORT,
                0,
                b'0',
                SLC_SYNCH,
                0,
                b'1',
                SLC_BRK,
                0,
                b'2',
                IAC,
                SE,
            ]);

            let event = codec.decode(&mut buffer).unwrap().unwrap();

            match event {
                TelnetEvent::Subnegotiate(SubnegotiationType::LineMode(LineModeOption::SLC(
                    triples,
                ))) => {
                    assert_eq!(triples.len(), 3);

                    const CHARS: [char; 3] = ['0', '1', '2'];
                    const FUNCS: [u8; 3] = [SLC_ABORT, SLC_SYNCH, SLC_BRK];

                    for (index, &(dispatch, char)) in triples.iter().enumerate() {
                        assert_eq!(dispatch.function, SlcFunction::from(FUNCS[index]));
                        assert_eq!(char, CHARS[index]);
                    }
                }
                _ => panic!("Bad decode!"),
            }
        }

        #[test]
        fn test_sb_linemode_fmask_decode() {
            let (mut codec, mut buffer) = setup();
            buffer.extend([
                IAC,
                SB,
                LINEMODE,
                DO,
                LINEMODE_FORWARD_MASK,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                123,
                IAC,
                SE,
            ]);

            let event = codec.decode(&mut buffer).unwrap().unwrap();

            match event {
                TelnetEvent::Subnegotiate(SubnegotiationType::LineMode(
                    LineModeOption::ForwardMask(ForwardMaskOption::Do(data)),
                )) => {
                    assert_eq!(data.len(), 16);
                    assert_eq!(data[15], 123)
                }
                _ => panic!("Bad decode!"),
            }
        }

        #[test]
        fn test_sb_linemode_fmask_encode() {
            let (mut codec, mut buffer) = setup();
            codec
                .encode(
                    TelnetEvent::Subnegotiate(SubnegotiationType::LineMode(
                        LineModeOption::ForwardMask(ForwardMaskOption::Do(Vec::with_capacity(16))),
                    )),
                    &mut buffer,
                )
                .unwrap();

            assert_eq!(
                buffer.as_ref(),
                &[
                    IAC,
                    SB,
                    LINEMODE,
                    DO,
                    LINEMODE_FORWARD_MASK,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    IAC,
                    SE
                ]
            )
        }
    }
}
