use crate::{
    constants::{DO, DONT, GA, NOP, SB, WILL, WONT},
    option::TelnetOption,
    subnegotiation::SubnegotiationType,
};

/// Represents all Telnet events supported by Nectar.
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
