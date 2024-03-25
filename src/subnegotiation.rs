use bytes::Bytes;
use crate::constants::{LINEMODE_FORWARD_MASK, LINEMODE_SLC, MODE};
use crate::linemode::SlcFunction;

use crate::option::TelnetOption;

/// Represents all Telnet subnegotiation events supported by Nectar.
/// See `<https://tools.ietf.org/html/rfc854>` for more information.
#[derive(Debug, PartialEq, Eq)]
pub enum SubnegotiationType {
    WindowSize(u16, u16),
    CharsetRequest(Vec<Bytes>),
    CharsetAccepted(Bytes),
    CharsetRejected,
    CharsetTTableRejected,
    LineMode(LineModeOption),
    Unknown(TelnetOption, Bytes),
}


#[derive(Debug, PartialEq, Eq)]
pub enum LineModeOption {
    Mode(u8),
    SLC(Vec<(SlcFunction, u8, char)>),
    ForwardMask,
    Unknown(u8)
}

impl From<u8> for LineModeOption {
    fn from(value: u8) -> Self {
        match value {
            MODE => LineModeOption::Mode(0),
            LINEMODE_SLC => LineModeOption::SLC(Vec::new()),
            LINEMODE_FORWARD_MASK => LineModeOption::ForwardMask,
            _ => LineModeOption::Unknown(value)
        }
    }
}

impl Into<u8> for LineModeOption {
    fn into(self) -> u8 {
        match self {
            LineModeOption::Mode(_) => MODE,
            LineModeOption::SLC(_) => LINEMODE_SLC,
            LineModeOption::ForwardMask => LINEMODE_FORWARD_MASK,
            LineModeOption::Unknown(value) => value,

        }
    }
}


impl SubnegotiationType {
    /// Returns the length (in bytes) of the subnegotiation data.
    /// This _does not_ include the IAC SB and IAC SE bytes, _nor_ the single
    /// byte that represents the option.
    pub fn len(&self) -> usize {
        match self {
            SubnegotiationType::WindowSize(_, _) => 4,
            SubnegotiationType::CharsetRequest(vec) => {
                // 1 separator per charset, as the list starts with one.
                let mut len = vec.len();

                for bytes in vec {
                    len += bytes.len();
                }
                // add one more for the subnegotation sub-option (i.e. CHARSET_REQUEST)
                len + 1
            }
            SubnegotiationType::CharsetAccepted(charset) => {
                // add one more for the subnegotation sub-option (i.e. CHARSET_ACCEPTED)
                charset.len() + 1
            }
            SubnegotiationType::CharsetRejected => 1,
            SubnegotiationType::CharsetTTableRejected => 1,
            SubnegotiationType::LineMode(mode) => {
                match mode {
                    LineModeOption::SLC(triples) => {
                        /// Mode byte plus length of triples
                        return triples.len() * 3 + 1;
                    },
                    LineModeOption::Mode(_) => 2,
                    _ => unimplemented!()
                }
            },
            SubnegotiationType::Unknown(_, bytes) => bytes.len(),
        }
    }

    /// Returns true if the subnegotiation data has a length (in bytes) of 0.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
