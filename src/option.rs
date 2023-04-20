use crate::constants::{CHARSET, ECHO, GA, GMCP, MCCP2, MSP, MSSP, MXP, SGA, TELOPT_EOR};

/// Represents all Telnet options supported by Nectar.
/// See `<https://tools.ietf.org/html/rfc854>` for more information.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum TelnetOption {
    Echo,
    GoAhead,
    SuppressGoAhead,
    EndOfRecord,
    Charset,
    MCCP2,
    GMCP,
    MSSP,
    MSP,
    MXP,
    Unknown(u8),
}

impl From<u8> for TelnetOption {
    fn from(byte: u8) -> Self {
        match byte {
            ECHO => TelnetOption::Echo,
            GA => TelnetOption::GoAhead,
            SGA => TelnetOption::SuppressGoAhead,
            TELOPT_EOR => TelnetOption::EndOfRecord,
            CHARSET => TelnetOption::Charset,
            MCCP2 => TelnetOption::MCCP2,
            GMCP => TelnetOption::GMCP,
            MSSP => TelnetOption::MSSP,
            MSP => TelnetOption::MSP,
            MXP => TelnetOption::MXP,
            _ => TelnetOption::Unknown(byte),
        }
    }
}

impl From<TelnetOption> for u8 {
    fn from(option: TelnetOption) -> Self {
        match option {
            TelnetOption::Echo => ECHO,
            TelnetOption::GoAhead => GA,
            TelnetOption::SuppressGoAhead => SGA,
            TelnetOption::EndOfRecord => TELOPT_EOR,
            TelnetOption::Charset => CHARSET,
            TelnetOption::MCCP2 => MCCP2,
            TelnetOption::GMCP => GMCP,
            TelnetOption::MSSP => MSSP,
            TelnetOption::MSP => MSP,
            TelnetOption::MXP => MXP,
            TelnetOption::Unknown(byte) => byte,
        }
    }
}
