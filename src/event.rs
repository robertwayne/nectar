use crate::{
    constants::{DO, DONT, GA, NOP, SB, WILL, WONT},
    option::TelnetOption,
    subnegotiation::SubnegotiationType,
};

/// Represents all Telnet events supported by Nectar.
/// See `<https://tools.ietf.org/html/rfc854>` for more information.
#[derive(Debug)]
pub enum TelnetEvent {
    Character(u8),
    Message(String),
    Do(TelnetOption),
    Will(TelnetOption),
    Dont(TelnetOption),
    Wont(TelnetOption),
    Subnegotiate(SubnegotiationType),
    GoAhead,
    Nop,
}

impl TelnetEvent {
    /// How many bytes does this event take up?
    pub fn len(&self) -> usize {
        match self {
            TelnetEvent::Message(message) => message.len(),
            TelnetEvent::Subnegotiate(subnegotiation) => {
                // the 5 is made up of the IAC SB, IAC SE, and the single byte option
                5 + subnegotiation.len()
            }
            _ => 5,
        }
    }
}

impl From<TelnetEvent> for u8 {
    fn from(event: TelnetEvent) -> Self {
        match event {
            TelnetEvent::Message(_) => 0x00,
            TelnetEvent::Do(_) => DO,
            TelnetEvent::Will(_) => WILL,
            TelnetEvent::Dont(_) => DONT,
            TelnetEvent::Wont(_) => WONT,
            TelnetEvent::Subnegotiate(_) => SB,
            TelnetEvent::Character(byte) => byte,
            TelnetEvent::GoAhead => GA,
            TelnetEvent::Nop => NOP,
        }
    }
}
