// Echo a message back to the other side
pub const ECHO: u8 = 1;

// Go Ahead
pub const GA: u8 = 249;

// Suppress Go Ahead
pub const SGA: u8 = 3;

// Interpret As Command
pub const IAC: u8 = 255;

// Subnegotiation Begin
pub const SB: u8 = 250;

// Negotiate About Window Size <https://datatracker.ietf.org/doc/rfc1073/>
pub const NAWS: u8 = 31;

// Subnegotiation End
pub const SE: u8 = 240;

// Erase Line
pub const EL: u8 = 248;

// No Operation
pub const NOP: u8 = 241;

// No Operation
pub const NULL: u8 = 0;

// Carriage Return
pub const CR: u8 = 13;

// Line Feed
pub const LF: u8 = 10;

// Carriage Return + Line Feed
pub const CRLF: &[u8] = b"\r\n";

// Linemode - <https://datatracker.ietf.org/doc/html/rfc1116#section-2.1>
pub const LINEMODE: u8 = 34;

// Indicates the suboption code for the LINEMODE MODE. This is used to negotiate
// the mode of line processing between the Telnet client and server, allowing
// for a more flexible and efficient handling of line-oriented data.
//
// `<https://tools.ietf.org/search/rfc1116>` 2.2 LINEMODE suboption MODE
pub const MODE: u8 = 1;

/// Defines the suboption code for the LINEMODE SLC (Special Character)
/// function. This is used in Telnet communications to negotiate the handling of
/// special characters that control various terminal functions, such as
/// interrupt, flush, and suspend actions. The SLC suboption allows for the
/// configuration and manipulation of these special characters, enhancing the
/// control over the terminal behavior in a Telnet session.
pub const LINEMODE_SLC: u8 = 3;

/// Represents a mask used in LINEMODE negotiations to indicate the forwarding
/// of linemode options. When set, this mask signals that the client wishes to
/// forward linemode options to the server, thereby enabling the server to
/// configure linemode settings. This capability is part of the LINEMODE
/// suboptions negotiation, allowing for dynamic and flexible configuration of
/// line processing behaviors based on the client's requirements and
/// capabilities.
pub const LINEMODE_FORWARD_MASK: u8 = 2;

// When set, the client side of the connection should process all input lines,
// performing any editing function, and only send completed lines to the remote
// side. When unset, client side should not process any input from the user, and
// the server side should take care of all character processing that needs to be
// done.
pub const LINEMODE_EDIT: u8 = 1;

// When set, the client side should translate appropriate interrupts/signals to
// their Telnet equivalent. (These would be IP, BRK, ABORT, EOF, and SUSP). When
// unset, the client should pass interrupts/signals as their normal ASCII
// values.
pub const LINEMODE_TRAPSIG: u8 = 2;

// Indicates the desire to begin performing, or confirmation that you are now
// performing, the indicated option.
pub const WILL: u8 = 251;

// Indicates the refusal to perform, or continue performing, the indicated
// option.
pub const WONT: u8 = 252;

// Indicates the request that the other party perform, or confirmation that you
// are expecting the other party to perform, the indicated option.
pub const DO: u8 = 253;

// Indicates the demand that the other party stop performing, or confirmation
// that you are no longer expecting the other party to perform, the indicated
// option.
pub const DONT: u8 = 254;

// End of Record negotiation
pub const TELOPT_EOR: u8 = 25;

/// STATUS - Verify the current status of options -
/// <https://www.rfc-editor.org/rfc/rfc859.html>
pub const STATUS: u8 = 5;

/// TIMING MARK - Verify that requested information has been used -
/// <https://datatracker.ietf.org/doc/rfc860/>
pub const TIMING_MARK: u8 = 6;

/// Remote flow control - <https://datatracker.ietf.org/doc/rfc1372/>
pub const REMOTE_FLOW_CONTROL: u8 = 33;

// End of Record - <https://tintin.mudhalla.net/protocols/eor/>
pub const EOR: u8 = 239;

// Mud Server Status Protocol - <https://mudhalla.net/tintin/protocols/mssp/>
pub const MSSP: u8 = 70;

// Mud Client Compression Protocol (v2) -
// <https://www.gammon.com.au/mccp/protocol.html>
pub const MCCP2: u8 = 86;

// Mud Sound Protocol - <https://www.zuggsoft.com/zmud/msp.htm>
pub const MSP: u8 = 90;

// Mud eXtension Protocol - <https://www.zuggsoft.com/zmud/mxp.htm>
pub const MXP: u8 = 91;

// Generic Mud Communication Protocol - <https://www.gammon.com.au/gmcp>
pub const GMCP: u8 = 201;

// CHARSET - <https://tools.ietf.org/html/rfc2066>
pub const CHARSET: u8 = 42;

// CHARSET subnegotiation commands
pub const CHARSET_REQUEST: u8 = 1;
pub const CHARSET_ACCEPTED: u8 = 2;
pub const CHARSET_REJECTED: u8 = 3;
pub const CHARSET_TTABLE_IS: u8 = 4;
pub const CHARSET_TTABLE_REJECTED: u8 = 5;
pub const CHARSET_TTABLE_ACK: u8 = 6;
pub const CHARSET_TTABLE_NAK: u8 = 7;

/// Constants representing different levels and functionalities associated with
/// Telnet's Special Linemode Characters (SLC).

/// `SLC_DEFAULT`: Represents the default state of a linemode option. This level
/// indicates that the default action should be taken for a particular SLC
/// function. It is typically used when no specific action or value is assigned
/// to an SLC function.
pub const SLC_DEFAULT: u8 = 3;

/// `SLC_VALUE`: Signifies that a specific value is associated with an SLC
/// function. This level is used when a particular SLC function is configured
/// with a specific, non-default value that must be recognized and acted upon.
pub const SLC_VALUE: u8 = 2;

/// `SLC_CANTCHANGE`: Indicates that the current SLC function's setting cannot
/// be modified. This level is used for SLC functions that are essential for the
/// operation or for which the ability to change the setting would result in
/// undesired behavior.
pub const SLC_CANTCHANGE: u8 = 1;

/// `SLC_NOSUPPORT`: Represents the lack of support for a particular SLC
/// function. This level is used when a Telnet client or server does not
/// recognize or cannot implement the specific SLC function being queried or
/// set.
pub const SLC_NOSUPPORT: u8 = 0;

/// `SLC_LEVELBITS`: A mask used to isolate the level bits in an SLC function
/// definition. This constant is used in operations that require identifying the
/// specific level associated with an SLC function.
pub const SLC_LEVELBITS: u8 = 3;

/// `SLC_ACK`: A flag used to acknowledge the receipt or acceptance of an SLC
/// function setting. This acknowledgment can be part of a negotiation process
/// where one side proposes a setting and the other side acknowledges it.
pub const SLC_ACK: u8 = 128;

/// `SLC_FLUSHIN`: A flag indicating that all incoming data should be flushed
/// (i.e., discarded). This is used in situations where it is necessary to clear
/// the input buffer, such as when the mode of operation changes or in response
/// to certain error conditions.
pub const SLC_FLUSHIN: u8 = 64;

/// `SLC_FLUSHOUT`: Similar to `SLC_FLUSHIN`, but for outgoing data. This flag
/// indicates that all data queued for output should be flushed. This can be
/// useful in situations where an immediate response is needed, or when clearing
/// the output buffer is required to maintain the correct sequence of data or
/// commands.
pub const SLC_FLUSHOUT: u8 = 32;

/// Telnet Special Linemode Characters (SLC) Functions as Constants

// SLC Function Names
/// SLC_SYNCH: Synchronize
pub const SLC_SYNCH: u8 = 1;

/// SLC_BRK: Break
pub const SLC_BRK: u8 = 2;

/// SLC_IP: Interrupt Process
pub const SLC_IP: u8 = 3;

/// SLC_AO: Abort Output
pub const SLC_AO: u8 = 4;

/// SLC_AYT: Are You There
pub const SLC_AYT: u8 = 5;

/// SLC_EOR: End of Record
pub const SLC_EOR: u8 = 6;

/// SLC_ABORT: Abort
pub const SLC_ABORT: u8 = 7;

/// SLC_EOF: End of File
pub const SLC_EOF: u8 = 8;

/// SLC_SUSP: Suspend Process
pub const SLC_SUSP: u8 = 9;

/// SLC_EC: Erase Character
pub const SLC_EC: u8 = 10;

/// SLC_EL: Erase Line
pub const SLC_EL: u8 = 11;

/// SLC_EW: Erase Word
pub const SLC_EW: u8 = 12;

/// SLC_RP: Repaint
pub const SLC_RP: u8 = 13;

/// SLC_LNEXT: Literal Next
pub const SLC_LNEXT: u8 = 14;

/// SLC_XON: Resume Transmission
pub const SLC_XON: u8 = 15;

/// SLC_XOFF: Stop Transmission
pub const SLC_XOFF: u8 = 16;

/// SLC_FORW1: Forward Character
pub const SLC_FORW1: u8 = 17;

/// SLC_FORW2: Forward Line
pub const SLC_FORW2: u8 = 18;

/// SLC_MCL: Move Cursor Left
pub const SLC_MCL: u8 = 19;

/// SLC_MCR: Move Cursor Right
pub const SLC_MCR: u8 = 20;

/// SLC_MCWL: Move Cursor Word Left
pub const SLC_MCWL: u8 = 21;

/// SLC_MCWR: Move Cursor Word Right
pub const SLC_MCWR: u8 = 22;

/// SLC_MCUB: Move Cursor Up One Line
pub const SLC_MCUB: u8 = 23;

/// SLC_MCUF: Move Cursor Down One Line
pub const SLC_MCUF: u8 = 24;

/// SLC_LP: Local Print
pub const SLC_LP: u8 = 25;

/// SLC_XONC: XON Character
pub const SLC_XONC: u8 = 26;

/// SLC_XOFFC: XOFF Character
pub const SLC_XOFFC: u8 = 27;

/// SLC_EXIT: Exit
pub const SLC_EXIT: u8 = 28;

/// SLC_SUSPC: Suspend Current Process
pub const SLC_SUSPC: u8 = 29;

/// SLC_DSUSPC: Delayed Suspend Current Process
pub const SLC_DSUSPC: u8 = 30;

/// SLC_REPRINT: Reprint Unread Input
pub const SLC_REPRINT: u8 = 31;

/// SLC_ABORTC: Abort Output Character
pub const SLC_ABORTC: u8 = 32;

/// SLC_EOFCHAR: End of File Character
pub const SLC_EOFCHAR: u8 = 33;

/// SLC_SUSPCHAR: Suspend Process Character
pub const SLC_SUSPCHAR: u8 = 34;

/// SLC_BRKC: Break Character
pub const SLC_BRKC: u8 = 35;

/// SLC_EORC: End of Record Character
pub const SLC_EORC: u8 = 36;

/// RFC 1572: Telnet Environment Option
/// <https://datatracker.ietf.org/doc/html/rfc1572>
/// Negotiate About Environment Variables: Telnet option
pub const ENVIRON: u8 = 39;

// ENVIRON Option Sub-negotiation codes
/// Environment variable definition
pub const ENV_IS: u8 = 0;
/// Request to send environment variables
pub const ENV_SEND: u8 = 1;
/// Environment variable change notification
pub const ENV_INFO: u8 = 2;

// ENVIRON Variable Types
/// Environment variable
pub const ENV_VAR: u8 = 0;
/// Value of variable
pub const ENV_VALUE: u8 = 1;

// ENVIRON ESC Byte
/// Indicates the data following it should be inserted into the data stream as is.
pub const ENV_ESC: u8 = 2;
// ENVIRON User-defined variable
/// User-defined variable
pub const ENV_USERVAR: u8 = 3;

/// `ENV_USER`: Represents the "User" environment variable in accordance with RFC1572.
/// This is typically used to identify the user on the remote system.
pub const ENV_USER: &str = "USER";

/// `ENV_JOB`: Represents the "Job" or "Jobname" environment variable in accordance with RFC1572.
/// This is used to pass the id of the job (process, service) the user wants to use.
pub const ENV_JOB: &str = "JOB";

/// `ENV_ACCT`: Represents the "Acct" or "Account" environment variable in accordance with RFC1572.
/// This is used for the account id the client wishes to use.
pub const ENV_ACCT: &str = "ACCT";

/// `ENV_PRINTER`: Represents the "Printer" environment variable in accordance with RFC1572.
/// This is used to specify the default location for printer output
pub const ENV_PRINTER: &str = "PRINTER";

/// `ENV_SYSTEMTYPE`: Represents the "SystemType" environment variable in accordance with RFC1572.
/// This variable identifies the type of operating system of the client.
pub const ENV_SYSTEMTYPE: &str = "SYSTEMTYPE";

/// `ENV_DISPLAY`: Represents the "Display" environment variable in accordance with RFC1572.
/// This is used to convey information about the user's display environment, similar to
/// the DISPLAY environment variable in Unix-like systems.
pub const ENV_DISPLAY: &str = "DISPLAY";

/// Binary Transmission - <https://datatracker.ietf.org/doc/rfc856/>
/// In accordance with RFC856, this option specifies a way to
/// indicate binary data should be transmitted across the connection.
/// It allows the sender and receiver to negotiate and agree upon
/// the data transfer mode to use during a Telnet session.
pub const BINARY: u8 = 0;
