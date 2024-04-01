use crate::constants::{SLC_ABORT, SLC_ABORTC, SLC_ACK, SLC_AO, SLC_AYT, SLC_BRK, SLC_BRKC, SLC_DSUSPC, SLC_EC, SLC_EL, SLC_EOF, SLC_EOFCHAR, SLC_EOR, SLC_EORC, SLC_EW, SLC_EXIT, SLC_FLUSHIN, SLC_FLUSHOUT, SLC_FORW1, SLC_FORW2, SLC_IP, SLC_LEVELBITS, SLC_LNEXT, SLC_LP, SLC_MCL, SLC_MCR, SLC_MCUB, SLC_MCUF, SLC_MCWL, SLC_MCWR, SLC_REPRINT, SLC_RP, SLC_SUSP, SLC_SUSPC, SLC_SUSPCHAR, SLC_SYNCH, SLC_XOFF, SLC_XOFFC, SLC_XON, SLC_XONC};

/// Represents the support level of Telnet's Special Linemode Characters (SLC).
/// This enum categorizes the possible states or capabilities associated with
/// a specific SLC function, reflecting its configurability and support status.
#[derive(Debug, PartialEq, Copy, Clone, Eq)]
pub enum Level {
    /// Indicates that the Telnet client or server does not support the specific SLC function.
    /// This level is used for SLC functions that are unrecognized or cannot be implemented.
    NoSupport,

    /// Signifies that the SLC function's current setting or value cannot be changed.
    /// This is typically used for essential SLC functions or those where changeability
    /// might result in undesired behavior or operation inconsistencies.
    CantChange,

    /// Denotes that the SLC function has a specific, assignable value that is not the default.
    /// This level is employed when a particular SLC function is set to a custom value,
    /// distinct from its standard or default setting.
    Value,

    /// Represents the default state or action for an SLC function, implying standard behavior.
    /// This level is selected when an SLC function is intended to operate according to
    /// its predefined or most common configuration.
    Default,
}


impl From<u8> for Level {
    fn from(value: u8) -> Self {
        match value & SLC_LEVELBITS {
            0 => Level::NoSupport,
            1 => Level::CantChange,
            2 => Level::Value,
            3 => Level::Default,
            _ => unreachable!("Level value out of range"), // Since we're masking with SLC_LEVELBITS, this should never happen
        }
    }
}

/// Represents a mapping between a Telnet Special Linemode Character (SLC) function
/// and its associated modifiers. This struct is used to define the behavior and
/// properties of specific SLC functions within a Telnet session, enabling detailed
/// control over their implementation and usage.
///
/// The `Dispatch` struct combines an SLC function, represented by the `SlcFunction` enum,
/// with a set of modifiers that further specify the function's behavior, encapsulated
/// in the `Modifiers` struct. This allows for a nuanced approach to handling SLC functions,
/// facilitating customized responses and actions based on the combination of function and modifiers.
#[derive(Debug, PartialEq, Copy, Clone, Eq)]
pub struct Dispatch {
    /// The SLC function being dispatched. This field specifies which of the defined SLC functions
    /// is being referenced or acted upon. Each SLC function has a specific role or action associated
    /// with it, such as interrupting the process, erasing characters, managing data flow, etc.
    ///
    /// The `SlcFunction` enum encompasses a comprehensive list of standard SLC functions as defined
    /// in the Telnet protocol, along with the capability to represent unknown or proprietary functions
    /// through the `Unknown(u8)` variant.
    pub function: SlcFunction,

    /// The set of modifiers associated with the SLC function. Modifiers provide additional context
    /// or instructions regarding how the SLC function should be processed or applied. For example,
    /// modifiers can indicate whether an SLC function's default behavior should be overridden, whether
    /// acknowledgments are required, or if specific data streams need to be flushed in conjunction with
    /// the function's invocation.
    ///
    /// The `Modifiers` struct captures this information in a structured form, making it straightforward
    /// to interpret and apply the modifiers in conjunction with the specified SLC function.
    pub modifiers: Modifiers,
}

impl From<(u8, u8)> for Dispatch {
    fn from((function, modifiers): (u8, u8)) -> Self {
        Self {
            function: function.into(),
            modifiers: modifiers.into()
        }
    }
}

impl From<u8> for Modifiers {
    fn from(value: u8) -> Self {
        Modifiers {
            level: Level::from(value),
            ack: value & SLC_ACK != 0,
            flush_in: value & SLC_FLUSHIN != 0,
            flush_out: value & SLC_FLUSHOUT != 0,
        }
    }
}


impl Into<(u8, u8)> for Dispatch {
    fn into(self) -> (u8, u8) {
        (self.function.into(), self.modifiers.into())
    }
}

/// Encapsulates the modifiers associated with a Telnet SLC function, including its
/// support level and additional operational flags. This struct provides a structured
/// representation of the configuration and capabilities related to SLC functions.
#[derive(Debug, PartialEq, Copy, Clone, Eq)]
pub struct Modifiers {
    /// Specifies the support and configurability level of the SLC function, as defined
    /// by the `Level` enum. This field determines how the SLC function can be manipulated
    /// or interpreted within the context of a Telnet session.
    pub level: Level,

    /// A flag indicating acknowledgment. When set to `true`, it signifies that the
    /// SLC function setting or value has been acknowledged or accepted, typically as part
    /// of a negotiation process between the Telnet client and server.
    pub ack: bool,

    /// A flag for flushing incoming data. When set to `true`, it implies that all queued
    /// incoming data should be discarded. This is often used to reset or clear the input
    /// buffer in response to certain commands or error conditions.
    pub flush_in: bool,

    /// Similar to `flush_in`, but for outgoing data. When this flag is `true`, it indicates
    /// that all data awaiting output should be flushed. This can be necessary to ensure
    /// immediate processing of urgent commands or to maintain data sequence integrity.
    pub flush_out: bool,
}


impl Into<u8> for Modifiers {
    fn into(self) -> u8 {
        let mut value: u8 = self.level.into();
        if self.ack {
            value |= SLC_ACK;
        }
        if self.flush_in {
            value |= SLC_FLUSHIN;
        }
        if self.flush_out {
            value |= SLC_FLUSHOUT;
        }
        value
    }
}

impl Into<u8> for Level {
    fn into(self) -> u8 {
        match self {
            Level::NoSupport => 0,
            Level::CantChange => 1,
            Level::Value => 2,
            Level::Default => 3,
        }
    }
}

/// Represents the Special Line Mode (SLC) functions in the Telnet protocol.
/// Each variant of this enum corresponds to a specific control function that
/// can be used within a Telnet session to control aspects like data flow, signal
/// transmission, and other auxiliary functions. The numerical values associated
/// with these functions are defined according to the Telnet specification.
#[repr(u8)]
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum SlcFunction {
    /// Synchronization: Used to indicate an urgent data stream in Telnet communications.
    Synch = SLC_SYNCH,

    /// Break: Indicates a break or interruption in the data stream.
    Brk = SLC_BRK,

    /// Interrupt Process: Allows the user to interrupt the process at the other end.
    Ip = SLC_IP,

    /// Abort Output: Used to clear the data remaining in the output buffer.
    Ao = SLC_AO,

    /// Are You There: Sends a signal to check if the system at the other end is still responsive.
    Ayt = SLC_AYT,

    /// End of Record: Marks the end of a record in the data stream.
    Eor = SLC_EOR,

    /// Abort: Used to signal an abort action.
    Abort = SLC_ABORT,

    /// End of File: Indicates the end of a file transmission.
    Eof = SLC_EOF,

    /// Suspend: Temporarily suspends the process at the other end.
    Susp = SLC_SUSP,

    /// Erase Character: Used to erase the last character in the current line.
    Ec = SLC_EC,

    /// Erase Line: Clears the entire current line.
    El = SLC_EL,

    /// Erase Word: Erases the last word in the current line.
    Ew = SLC_EW,

    /// Reprint Line: Reprints the current line.
    Rp = SLC_RP,

    /// Literal Next: Indicates the next character should be treated as literal input.
    Lnext = SLC_LNEXT,

    /// Resume Transmission: Signals to resume the data transmission if it was paused.
    Xon = SLC_XON,

    /// Pause Transmission: Instructs to pause the data transmission.
    Xoff = SLC_XOFF,

    /// Forward Character: Moves the cursor forward by one character.
    Forw1 = SLC_FORW1,

    /// Forward Line: Moves the cursor forward by one line.
    Forw2 = SLC_FORW2,

    /// Miscellaneous control functions follow, each with a specific role within the Telnet SLC framework.
    /// These functions may control cursor movement, line editing, and other terminal behaviors.
    Mcl = SLC_MCL,
    Mcr = SLC_MCR,
    Mcwl = SLC_MCWL,
    Mcwr = SLC_MCWR,
    Mcub = SLC_MCUB,
    Mcuf = SLC_MCUF,

    /// Local Print: Triggers the local print function.
    Lp = SLC_LP,

    /// XON Character: The character used to resume transmission.
    Xonc = SLC_XONC,

    /// XOFF Character: The character used to pause transmission.
    Xoffc = SLC_XOFFC,

    /// Exit: Used to signal an exit action.
    Exit = SLC_EXIT,

    /// Suspend Current: Suspends the current process.
    Suspc = SLC_SUSPC,

    /// Delayed Suspend: Suspends the current process with a delay.
    Dsuspc = SLC_DSUSPC,

    /// Reprint: Reprints the entire current line.
    Reprint = SLC_REPRINT,

    /// Abort Current: Aborts the current process.
    Abortc = SLC_ABORTC,

    /// EOF Character: The character signifying the end of a file.
    Eofchar = SLC_EOFCHAR,

    /// Suspend Character: The character used to signal a suspend action.
    Suspchar = SLC_SUSPCHAR,

    /// Break Character: The character used to signal a break condition.
    Brkc = SLC_BRKC,

    /// EOR Character: The character indicating the end of a record.
    Eorc = SLC_EORC,

    /// Represents any SLC functions that are not predefined in this enum.
    /// This variant allows for flexibility in handling unknown or proprietary SLC functions.
    Unknown(u8),
    // Additional SLC functions can be added here as needed.
}


impl From<u8> for SlcFunction {
    fn from(value: u8) -> Self {
        match value {
            SLC_SYNCH => SlcFunction::Synch,
            SLC_BRK => SlcFunction::Brk,
            SLC_IP => SlcFunction::Ip,
            SLC_AO => SlcFunction::Ao,
            SLC_AYT => SlcFunction::Ayt,
            SLC_EOR => SlcFunction::Eor,
            SLC_ABORT => SlcFunction::Abort,
            SLC_EOF => SlcFunction::Eof,
            SLC_SUSP => SlcFunction::Susp,
            SLC_EC => SlcFunction::Ec,
            SLC_EL => SlcFunction::El,
            SLC_EW => SlcFunction::Ew,
            SLC_RP => SlcFunction::Rp,
            SLC_LNEXT => SlcFunction::Lnext,
            SLC_XON => SlcFunction::Xon,
            SLC_XOFF => SlcFunction::Xoff,
            SLC_FORW1 => SlcFunction::Forw1,
            SLC_FORW2 => SlcFunction::Forw2,
            SLC_MCL => SlcFunction::Mcl,
            SLC_MCR => SlcFunction::Mcr,
            SLC_MCWL => SlcFunction::Mcwl,
            SLC_MCWR => SlcFunction::Mcwr,
            SLC_MCUB => SlcFunction::Mcub,
            SLC_MCUF => SlcFunction::Mcuf,
            SLC_LP => SlcFunction::Lp,
            SLC_XONC => SlcFunction::Xonc,
            SLC_XOFFC => SlcFunction::Xoffc,
            SLC_EXIT => SlcFunction::Exit,
            SLC_SUSPC => SlcFunction::Suspc,
            SLC_DSUSPC => SlcFunction::Dsuspc,
            SLC_REPRINT => SlcFunction::Reprint,
            SLC_ABORTC => SlcFunction::Abortc,
            SLC_EOFCHAR => SlcFunction::Eofchar,
            SLC_SUSPCHAR => SlcFunction::Suspchar,
            SLC_BRKC => SlcFunction::Brkc,
            SLC_EORC => SlcFunction::Eorc,
            // Add additional SLC functions as needed...
            _ => SlcFunction::Unknown(value), // Gracefully handle unknown or unsupported SLC function codes
        }
    }
}

impl Into<u8> for SlcFunction {
    fn into(self) -> u8 {
        match self {
            SlcFunction::Synch => SLC_SYNCH,
            SlcFunction::Brk => SLC_BRK,
            SlcFunction::Ip => SLC_IP,
            SlcFunction::Ao => SLC_AO,
            SlcFunction::Ayt => SLC_AYT,
            SlcFunction::Eor => SLC_EOR,
            SlcFunction::Abort => SLC_ABORT,
            SlcFunction::Eof => SLC_EOF,
            SlcFunction::Susp => SLC_SUSP,
            SlcFunction::Ec => SLC_EC,
            SlcFunction::El => SLC_EL,
            SlcFunction::Ew => SLC_EW,
            SlcFunction::Rp => SLC_RP,
            SlcFunction::Lnext => SLC_LNEXT,
            SlcFunction::Xon => SLC_XON,
            SlcFunction::Xoff => SLC_XOFF,
            SlcFunction::Forw1 => SLC_FORW1,
            SlcFunction::Forw2 => SLC_FORW2,
            SlcFunction::Mcl => SLC_MCL,
            SlcFunction::Mcr => SLC_MCR,
            SlcFunction::Mcwl => SLC_MCWL,
            SlcFunction::Mcwr => SLC_MCWR,
            SlcFunction::Mcub => SLC_MCUB,
            SlcFunction::Mcuf => SLC_MCUF,
            SlcFunction::Lp => SLC_LP,
            SlcFunction::Xonc => SLC_XONC,
            SlcFunction::Xoffc => SLC_XOFFC,
            SlcFunction::Exit => SLC_EXIT,
            SlcFunction::Suspc => SLC_SUSPC,
            SlcFunction::Dsuspc => SLC_DSUSPC,
            SlcFunction::Reprint => SLC_REPRINT,
            SlcFunction::Abortc => SLC_ABORTC,
            SlcFunction::Eofchar => SLC_EOFCHAR,
            SlcFunction::Suspchar => SLC_SUSPCHAR,
            SlcFunction::Brkc => SLC_BRKC,
            SlcFunction::Eorc => SLC_EORC,
            SlcFunction::Unknown(value) => value,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_slc_function() {
        let input = SLC_SYNCH;  // Use a constant that represents a known SLC function
        let expected = SlcFunction::Synch;
        let result = SlcFunction::from(input);
        assert_eq!(result, expected, "Failed to parse SLC_SYNCH into SlcFunction::Synch");
    }

    #[test]
    fn test_modifiers_from_byte() {
        let input = SLC_ACK | SLC_FLUSHIN;  // Example combining two modifier flags
        let result = Modifiers::from(input);
        assert!(result.ack && result.flush_in, "Modifiers did not correctly interpret ACK and FLUSHIN flags");
    }

}