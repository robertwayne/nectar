use crate::{
    constants::{DO, DONT, GA, NOP, SB, WILL, WONT},
    option::TelnetOption,
    subnegotiation::SubnegotiationType,
};

/// Represents message types supported by Nectar.
#[derive(Debug, PartialEq, Eq)]
pub enum TelnetEvent {
    /// A single byte character.
    Character(u8),
    /// A message that guarantees it ends with `\r\n`.
    Message(String),
    /// A message that does not guarantee it ends with `\r\n`. Allows for
    /// sending messages without enforced newlines. Used for outgoing messages
    /// only.
    RawMessage(String),
    /// A message requesting the other side to perform an option.
    Do(TelnetOption),
    /// A message indicating an intent to perform an option.
    Will(TelnetOption),
    /// A message telling the other side to discontinue using an option, or that
    /// it is no longer expected.
    Dont(TelnetOption),
    /// A message refusing to perform an option.
    Wont(TelnetOption),
    /// A message indicating that a subnegotiation is beginning.
    Subnegotiate(SubnegotiationType),
    /// A message indicating that the data stream should resume.
    GoAhead,
    /// No operation.
    Nop,
}

impl TelnetEvent {
    /// Returns the length (in bytes) of the event.
    pub fn len(&self) -> usize {
        match self {
            TelnetEvent::Message(message) => message.len(),
            TelnetEvent::RawMessage(message) => message.len(),
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
            TelnetEvent::Message(_) | TelnetEvent::RawMessage(_) => 0x00,
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
