use crate::{
    constants::{DO, DONT, GA, NOP, SB, WILL, WONT},
    option::TelnetOption,
    subnegotiation::SubnegotiationType,
};

/// Represents all Telnet events supported by Nectar.
/// See `<https://tools.ietf.org/html/rfc854>` for more information.
#[derive(Debug, PartialEq, Eq)]
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
    /// Returns the length (in bytes) of the event.
    pub fn len(&self) -> usize {
        match self {
            TelnetEvent::Message(message) => message.len(),
            TelnetEvent::Subnegotiate(subnegotiation) => {
                // the 5 is made up of the IAC SB, IAC SE, and the single byte
                // option
                5 + subnegotiation.len()
            }
            TelnetEvent::Character(_) => 1,
            TelnetEvent::Do(_)
            | TelnetEvent::Will(_)
            | TelnetEvent::Dont(_)
            | TelnetEvent::Wont(_) => 6,
            _ => 5,
        }
    }

    /// Returns true if the event has a length (in bytes) of 0.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
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
