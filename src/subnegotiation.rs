use bytes::Bytes;

use crate::constants::{LINEMODE_FORWARD_MASK, LINEMODE_SLC, MODE};
use crate::linemode::{Dispatch, ForwardMaskOption};
use crate::option::TelnetOption;

/// Represents all Telnet subnegotiation events supported by Nectar.
#[derive(Debug, PartialEq, Eq)]
pub enum SubnegotiationType {
    /// A subnegotiation for the window size, where the first value is the width
    /// and the second value is the height. The values are in characters.
    WindowSize(u16, u16),
    /// Indicates an intent to begin CHARSET subnegotiation. This can only be
    /// sent after receiving a DO CHARSET after sending a WILL CHARSET (in any
    /// order).
    CharsetRequest(Vec<Bytes>),
    /// Indicates that the receiver has accepted the charset request.
    CharsetAccepted(Bytes),
    /// Indicates that the receiver acknowledges the charset request, but will
    /// not use any of the requested characters.
    CharsetRejected,
    /// Indicates that the receiver acknowledges a TTABLE-IS message, but is
    /// unable to handle it. This will terminate subnegotiation.
    CharsetTTableRejected,
    LineMode(LineModeOption),
    /// A subnegotiation for an unknown option.
    Unknown(TelnetOption, Bytes),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum LineModeOption {
    Mode(u8),
    SLC(Vec<(Dispatch, char)>),
    ForwardMask(ForwardMaskOption),
    Unknown(u8, Bytes),
}

impl From<u8> for LineModeOption {
    fn from(value: u8) -> Self {
        match value {
            MODE => LineModeOption::Mode(0),
            LINEMODE_SLC => LineModeOption::SLC(Vec::new()),
            LINEMODE_FORWARD_MASK => LineModeOption::ForwardMask(ForwardMaskOption::Unknown(0)),
            _ => LineModeOption::Unknown(value, Bytes::new()),
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
                // add one more for the subnegotation sub-option (i.e.
                // CHARSET_REQUEST)
                len + 1
            }
            SubnegotiationType::CharsetAccepted(charset) => {
                // add one more for the subnegotation sub-option (i.e.
                // CHARSET_ACCEPTED)
                charset.len() + 1
            }
            SubnegotiationType::CharsetRejected => 1,
            SubnegotiationType::CharsetTTableRejected => 1,
            SubnegotiationType::LineMode(mode) => {
                match mode {
                    LineModeOption::SLC(triples) => {
                        // Mode byte plus length of triples
                        triples.len() * 3 + 1
                    }
                    LineModeOption::Mode(_) => 2,
                    LineModeOption::ForwardMask(ForwardMaskOption::Do(_)) => 2 + 16,
                    LineModeOption::ForwardMask(_) => 2,
                    LineModeOption::Unknown(_, data) => 1 + data.len(),
                }
            }
            SubnegotiationType::Unknown(_, bytes) => bytes.len(),
        }
    }

    /// Returns true if the subnegotiation data has a length (in bytes) of 0.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
