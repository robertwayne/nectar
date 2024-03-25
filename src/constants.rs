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

// Linemode subnegotiation options -
// <https://datatracker.ietf.org/doc/html/rfc1184#section-2>
pub const MODE: u8 = 1; //

/// TODO: Document this
pub const LINEMODE_SLC: u8 = 3;
pub const LINEMODE_FORWARD_MASK: u8 = 2;

// When set, the client side of the connection should process all input lines,
// performing any editing function, and only send completed lines to the remote
// side. When unset, client side should not process any input from the user,
// and the server side should take care of all character processing that needs
// to be done.
pub const LINEMODE_EDIT: u8 = 1;

// When set, the client side should translate appropriate interrupts/signals to
// their Telnet equivalent. (These would be IP, BRK, ABORT, EOF, and SUSP).
// When unset, the client should pass interrupts/signals as their normal ASCII
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

/// STATUS - Verify the current status of options - https://www.rfc-editor.org/rfc/rfc859.html
pub const STATUS: u8 = 5;

/// TIMING MARK - Verify that requested information has been used - https://datatracker.ietf.org/doc/rfc860/
pub const TIMING_MARK: u8 = 6;

/// Remote flow control - https://datatracker.ietf.org/doc/rfc1372/
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
