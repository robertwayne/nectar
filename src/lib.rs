#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]

// RFC 854 https://tools.ietf.org/html/rfc854
//
// Originally based off of https://github.com/jtenner/telnet_codec, which has
// been archived.

/// Various byte or byte sequences used in the Telnet protocol.
pub mod constants;
/// Codec and Io errors that may occur while processing Telnet events.
pub mod error;
/// Top-level Telnet events, such as Message, Do, Will, and Subnegotiation.
pub mod event;
/// Telnet options such as Echo, GoAhead, and SupressGoAhead.
pub mod option;
/// Telnet subnegotiation options.
pub mod subnegotiation;

use std::mem;

use bytes::{Buf, BufMut, Bytes, BytesMut};
use tokio_util::codec::{Decoder, Encoder};

use crate::{
    constants::{DO, DONT, IAC, NAWS, NOP, SB, SE, WILL, WONT},
    error::TelnetError,
    event::TelnetEvent,
    option::TelnetOption,
    subnegotiation::SubnegotiationType,
};

type Result<T> = std::result::Result<T, TelnetError>;

/// Implements a Tokio codec for the Telnet protocol, along with MUD-specific
/// extension protocols such as GMCP. You should never have to interact with
/// this directly.
#[derive(Debug)]
pub struct TelnetCodec {
    pub sga: bool,
    max_buffer_length: usize,
    buffer: Vec<u8>,
}

impl TelnetCodec {
    #[must_use]
    pub fn new(max_buffer_length: usize) -> Self {
        TelnetCodec { sga: false, max_buffer_length, buffer: Vec::new() }
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
            _ => {}
        }

        Ok(())
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
    let _ = buffer.split_at(2);

    if invalid {
        None
    } else {
        match option {
            NAWS => decode_negotiate_about_window_size(&subvec),
            _ => Some(decode_unknown(option, subvec)),
        }
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
    }
}

fn encode_message(message: String, buffer: &mut BytesMut) {
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

    if !buffer.ends_with(b"\r\n") {
        buffer.reserve(2);
        buffer.extend([b'\r', b'\n']);
    }
}
