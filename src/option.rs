use crate::constants::{ECHO, GA, SGA};

/// Represents all Telnet options supported by Nectar.
#[derive(Debug)]
pub enum TelnetOption {
    Echo,
    GoAhead,
    SupressGoAhead,
    Unknown(u8),
}

impl From<u8> for TelnetOption {
    fn from(byte: u8) -> Self {
        match byte {
            ECHO => TelnetOption::Echo,
            GA => TelnetOption::GoAhead,
            SGA => TelnetOption::SupressGoAhead,
            _ => TelnetOption::Unknown(byte),
        }
    }
}

impl From<TelnetOption> for u8 {
    fn from(option: TelnetOption) -> Self {
        match option {
            TelnetOption::Echo => ECHO,
            TelnetOption::GoAhead => GA,
            TelnetOption::SupressGoAhead => SGA,
            TelnetOption::Unknown(byte) => byte,
        }
    }
}
