use bytes::{Bytes, BytesMut};

use crate::{
    constants::{
        ENV_ACCT, ENV_DISPLAY, ENV_ESC, ENV_INFO, ENV_IS, ENV_JOB, ENV_PRINTER, ENV_SEND,
        ENV_SYSTEMTYPE, ENV_USER, ENV_USERVAR, ENV_VALUE, ENV_VAR, IAC,
    },
    env::Escape::Unescaped,
    event::TelnetEvent,
    subnegotiation::SubnegotiationType,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EnvironmentOperation {
    /// `Is` variant is used to send the keys and values of environment variables
    Is(Vec<(EnvironmentKind, Option<Vec<u8>>)>),
    /// `Send` variant is used to request environment variables
    Send(Vec<EnvironmentKind>),
    /// `Info` variant is used to update the client about environment variable changes.
    Info(Vec<(EnvironmentKind, Option<Vec<u8>>)>),
    /// `Unknown` variant is for the operations that are not recognized. It takes u8 as a parameter.
    Unknown(u8, Bytes),
}

/// `EnvironmentKind` is an enumeration of the distinct types of environment.
/// An environment can either be well known or user defined.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EnvironmentKind {
    /// `WellKnown` variant is for environment that is known.
    WellKnown(Option<WellKnownVariable>),
    /// `UserDefined` variant is for environments that are defined by the user.
    UserDefined(Option<String>),
}

/// `WellKnownVariable` is an enumeration of all the well known
/// variables that can be utilized in an environment.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WellKnownVariable {
    /// `User` variant represents the username the client wishes to use for logging in.
    User,
    /// `Job` variant represents the job id that the user wants to use.
    Job,
    /// `Acct` variant represents the account id of the user.
    Acct,
    /// `Printer` variant represents the default location for printer output
    Printer,
    /// `SystemType` variant represents the type of operating system.
    SystemType,
    /// `Display` variant represents the location of the X display.
    Display,
    /// `Unknown` variant represents the variables that are not recognized. It takes a string as parameter.
    Unknown(String),
}

impl From<&str> for WellKnownVariable {
    fn from(value: &str) -> Self {
        match value {
            ENV_USER => WellKnownVariable::User,
            ENV_JOB => WellKnownVariable::Job,
            ENV_ACCT => WellKnownVariable::Acct,
            ENV_PRINTER => WellKnownVariable::Printer,
            ENV_SYSTEMTYPE => WellKnownVariable::SystemType,
            ENV_DISPLAY => WellKnownVariable::Display,
            _ => WellKnownVariable::Unknown(value.to_string()),
        }
    }
}

impl From<WellKnownVariable> for String {
    fn from(value: WellKnownVariable) -> Self {
        match value {
            WellKnownVariable::User => ENV_USER.to_string(),
            WellKnownVariable::Job => ENV_JOB.to_string(),
            WellKnownVariable::Acct => ENV_ACCT.to_string(),
            WellKnownVariable::Printer => ENV_PRINTER.to_string(),
            WellKnownVariable::SystemType => ENV_SYSTEMTYPE.to_string(),
            WellKnownVariable::Display => ENV_DISPLAY.to_string(),
            WellKnownVariable::Unknown(data) => data.clone(),
        }
    }
}

impl From<EnvironmentOperation> for u8 {
    fn from(value: EnvironmentOperation) -> Self {
        match value {
            EnvironmentOperation::Is(_) => ENV_IS,
            EnvironmentOperation::Send(_) => ENV_SEND,
            EnvironmentOperation::Info(_) => ENV_INFO,
            EnvironmentOperation::Unknown(b, _) => b,
        }
    }
}

impl From<u8> for EnvironmentOperation {
    fn from(value: u8) -> Self {
        match value {
            ENV_IS => EnvironmentOperation::Is(Vec::new()),
            ENV_SEND => EnvironmentOperation::Send(Vec::new()),
            ENV_INFO => EnvironmentOperation::Info(Vec::new()),
            _ => EnvironmentOperation::Unknown(value, Bytes::new()),
        }
    }
}

impl WellKnownVariable {
    pub fn encoded_size(&self) -> usize {
        self.as_str().len()
    }

    pub fn as_str(&self) -> &str {
        match self {
            WellKnownVariable::User => ENV_USER,
            WellKnownVariable::Job => ENV_JOB,
            WellKnownVariable::Acct => ENV_ACCT,
            WellKnownVariable::Printer => ENV_PRINTER,
            WellKnownVariable::SystemType => ENV_SYSTEMTYPE,
            WellKnownVariable::Display => ENV_DISPLAY,
            WellKnownVariable::Unknown(s) => s.as_str(),
        }
    }
}

impl EnvironmentKind {
    pub fn as_u8(&self) -> u8 {
        match self {
            EnvironmentKind::WellKnown(_) => ENV_VAR,
            EnvironmentKind::UserDefined(_) => ENV_USERVAR,
        }
    }

    pub fn is_wildcard(&self) -> bool {
        matches!(self, EnvironmentKind::WellKnown(None) | EnvironmentKind::UserDefined(None))
    }

    pub fn name(&self) -> Option<String> {
        match self {
            EnvironmentKind::WellKnown(s) => s.clone().map(|v| v.into()),
            EnvironmentKind::UserDefined(s) => s.clone(),
        }
    }

    pub fn encoded_size(&self) -> usize {
        match self {
            EnvironmentKind::WellKnown(None) => 1,
            EnvironmentKind::UserDefined(None) => 1,
            EnvironmentKind::WellKnown(Some(v)) => 1 + v.encoded_size(),
            EnvironmentKind::UserDefined(Some(v)) => 1 + v.len(),
        }
    }
}

pub fn encode_bytes(buf: &[u8]) -> Vec<u8> {
    buf.iter()
        .flat_map(|&b| match b {
            ENV_ESC | ENV_VAR | ENV_VALUE | ENV_USERVAR => vec![ENV_ESC, b].into_iter(),
            IAC => vec![IAC, IAC].into_iter(),
            _ => vec![b].into_iter(),
        })
        .collect::<Vec<u8>>()
}

pub fn encode_env_vars(vars: Vec<(EnvironmentKind, Option<Vec<u8>>)>, buffer: &mut BytesMut) {
    for (kind, name, value) in
        vars.iter().filter_map(|(k, v)| k.name().map(|name| (k.as_u8(), name, v)))
    {
        buffer.extend([kind]);
        let encoded_name = encode_bytes(name.as_bytes());

        buffer.extend(encoded_name);

        if let Some(value) = value {
            buffer.extend([ENV_VALUE]);
            let encoded_value = encode_bytes(value.as_slice());
            buffer.extend(encoded_value);
        }
    }
}

pub fn encode_env_op(op: EnvironmentOperation, buffer: &mut BytesMut) {
    match op {
        EnvironmentOperation::Is(vars) => {
            buffer.extend([ENV_IS]);
            encode_env_vars(vars, buffer);
        }
        EnvironmentOperation::Send(vars) => {
            buffer.extend([ENV_SEND]);
            for (kind, name) in vars.iter().filter_map(|k| k.name().map(|name| (k.as_u8(), name))) {
                buffer.extend([kind]);
                // TODO: Maybe we should strip UTF-8 here but we could also just leave that to the user...
                buffer.extend(name.as_bytes());
            }
        }
        EnvironmentOperation::Info(vars) => {
            buffer.extend([ENV_INFO]);
            encode_env_vars(vars, buffer);
        }
        EnvironmentOperation::Unknown(b, buf) => {
            buffer.reserve(1 + buf.len());
            buffer.extend([b]);
            buffer.extend(buf);
        }
    }
}

// The INFO / IS command use the same grammar
// I've outlined it as eBNF below. Please note that this ignores any escape sequences.
// IAC SB NEW-ENVIRON IS/INFO <variables> IAC SE
// <variables> ::= <var>*
// <kind> ::= "USERVAR" | "VAR"
// <var> ::= <kind> <name> <value>?
// <name> ::= ([0-9] | [a-z] | [A-Z])
// <value> ::= "VALUE" ([0-9] | [a-z] | [A-Z])*

// IAC SB NEW-ENVIRON IS type ... [ VALUE ... ] [ type ... [ VALUE ... ]
// [ ... ] ] IAC SE
//
//    The sender of this command is sending environment variables.  This
//    command is sent in response to a SEND request.  Only the side that
//    is WILL NEW-ENVIRON may send an IS command.  The "type"/VALUE
//    pairs must be returned in the same order as the SEND request
//    specified them, and there must be a response for each "type ..."
//    explicitly requested.  The "type" will be VAR or USERVAR.
//    Multiple environment variables may be sent.  The characters
//    following a "type" up to the next "type" or VALUE specify the
//    variable name.  The characters following a VALUE up to the next
//    "type" specify the value of the variable.  If a "type" is not
//    followed by a VALUE (e.g., by another VAR, USERVAR, or IAC SE)
//    then that variable is undefined.  If a VALUE is immediately
//    followed by a "type" or IAC, then the variable is defined, but has
//    no value.  If an IAC is contained between the IS and the IAC SE,
//    it must be sent as IAC IAC.  If a variable or a value contains a
//    VAR, it must be sent as ESC VAR.  If a variable or a value
//    contains a USERVAR, it must be sent as ESC USERVAR.  If a variable
//    or a value contains a VALUE, it must be sent as ESC VALUE.  If a
//    variable or a value contains an ESC, it must be sent as ESC ESC.

#[derive(Copy, Clone, Debug)]
enum Escape {
    Unescaped,
    Escaped(u8),
}

/// Decodes the provided environment variable name.
///
/// # Arguments
///
/// * `subvec: &[u8]` - A byte slice representing the encoded environment
///   variable name.
///
/// The function decodes the environment variable name, accounting for escape
/// sequences present in the encoding. The escape sequences considered are
/// ENV_ESC combined with ENV_VAR, ENV_USERVAR, ENV_VALUE and ENV_ESC as well
/// as IAC (Interpret as Command) escaped with IAC.
///
/// # Returns
///  
/// * `Option<(Vec<u8>, usize)>` - Returns an option containing a tuple. The
///   first element of the tuple is a vector of bytes representing the decoded
///   environment variable. The second element represents the count of bytes
///   parsed.
///  
/// If decoding fails due to invalid data, the function returns None.
///  
/// If the input byte slice subvec is empty, the function returns None.
///
/// If the input ends on an escape sequence, the function returns None.
pub fn decode_env_name(subvec: &[u8]) -> Option<(Vec<u8>, usize)> {
    if subvec.is_empty() {
        return None;
    }

    // Use a state machine to track escape sequences
    let mut escape = Unescaped;
    // We have to use a new Vec as we're potentially shrinking the input.
    let mut buf = Vec::new();

    for (i, b) in subvec.iter().enumerate() {
        match (*b, &escape) {
            // Start of an escape sequence
            (ENV_ESC, Unescaped) => {
                escape = Escape::Escaped(ENV_ESC);
            }

            // Valid escape sequences
            (ENV_VAR, Escape::Escaped(ENV_ESC))
            | (ENV_USERVAR, Escape::Escaped(ENV_ESC))
            | (ENV_VALUE, Escape::Escaped(ENV_ESC))
            | (ENV_ESC, Escape::Escaped(ENV_ESC)) => {
                buf.push(*b);
                escape = Unescaped;
            }

            // End of a name
            (ENV_VALUE, Unescaped) | (ENV_USERVAR, Unescaped) | (ENV_VAR, Unescaped) => {
                // Name is completely parsed, return name upto current byte
                return Some((buf, i));
            }

            // Start of IAC sequence
            (IAC, Unescaped) => {
                escape = Escape::Escaped(IAC);
            }

            // IAC sequence completed
            (IAC, Escape::Escaped(IAC)) => {
                buf.push(IAC); // IAC is added to the buffer
                escape = Unescaped; // Transition back to the Unescaped state
            }

            // Any other byte when Escaped is invalid
            (_, Escape::Escaped(_)) => {
                // Invalid data!
                return None;
            }

            // For unescaped byte
            (b, Unescaped) => {
                buf.push(b);
            }
        }
    }
    // If the input ends on an escape sequence, it's not valid
    match escape {
        Unescaped => Some((buf, subvec.len())), // Complete parsed name and its size is returned
        Escape::Escaped(_) => None,             // Parsing failed, None is returned
    }
}

/// Decodes the encoded environment variable value given as input.
///
/// # Parameters
///
/// * `subvec: &[u8]` - An encoded byte slice of environment variable value.
///
/// The decoding is done by checking for escape sequences like `ENV_ESC` with `ENV_VAR`,
/// `ENV_USERVAR`, `ENV_VALUE`, `ENV_ESC` and `IAC` (Interpret as Command) escaped with `IAC`.
///
/// # Returns
///
/// * `Option<(Vec<u8>, usize)>` - Returns an option of tuple consisting of two elements:
///     - `Vec<u8>` - The decoded environment variable value byte vector.
///     - `usize` - count of bytes parsed.
///        
/// This function returns `None` in these conditions:
///     - If decoding fails due to invalid data.
///     - If the last character visited in the function loop is part of an incomplete escape sequence.
pub fn decode_env_value(subvec: &[u8]) -> Option<(Vec<u8>, usize)> {
    if subvec.is_empty() {
        return Some((Vec::new(), 0));
    }

    // Use a state machine to track escape sequences
    let mut escape = Unescaped;
    // We have to use a new Vec as we're potentially shrinking the input.
    let mut buf = Vec::new();

    for (i, b) in subvec.iter().enumerate() {
        match (*b, &escape) {
            // Begins an escape sequence
            (ENV_ESC, Unescaped) => {
                escape = Escape::Escaped(ENV_ESC);
            }
            // Handles valid escape sequences by adding the escaped byte to
            // the buffer and returning to Unescaped state.
            (ENV_VAR, Escape::Escaped(ENV_ESC))
            | (ENV_USERVAR, Escape::Escaped(ENV_ESC))
            | (ENV_VALUE, Escape::Escaped(ENV_ESC))
            | (ENV_ESC, Escape::Escaped(ENV_ESC)) => {
                buf.push(*b);
                escape = Unescaped;
            }
            // For any of these bytes, we have finished parsing a value.
            (ENV_USERVAR, Unescaped) | (ENV_VAR, Unescaped) => {
                // We're done parsing here
                return Some((buf, i));
            }
            // Start of an IAC sequence
            (IAC, Unescaped) => {
                escape = Escape::Escaped(IAC);
            }
            // IAC sequence completed. We add the IAC byte to the buffer and transition back to the Unescaped state.
            (IAC, Escape::Escaped(IAC)) => {
                buf.push(IAC);
                escape = Unescaped;
            }
            // Invalid data or unresolved escape sequences result in returning None.
            (ENV_VALUE, Unescaped) | (_, Escape::Escaped(_)) => {
                // Invalid data!
                return None;
            }
            // Any unescaped byte is added to the buffer.
            (b, Unescaped) => {
                buf.push(b);
            }
        }
    }

    // Checks if the entire input was parsed. If the input ended in the middle of an escape sequence, returns None.
    match escape {
        Unescaped => Some((buf, subvec.len())),
        Escape::Escaped(_) => None,
    }
}

pub fn decode_env_var(subvec: &[u8]) -> Option<(String, Option<Vec<u8>>, usize)> {
    let (raw_name, mut size) = decode_env_name(subvec)?;

    if raw_name.is_empty() {
        return None;
    }

    let valuevec = &subvec[size..];

    let value = match valuevec.first().copied() {
        Some(ENV_VALUE) => {
            let (value, value_size) = decode_env_value(&valuevec[1..])?;
            size += 1 + value_size;
            Some(value)
        }
        None | Some(_) => None,
    };

    let name = String::from_utf8(raw_name).ok()?;

    Some((name, value, size))
}
pub fn decode_env_is(subvec: &[u8]) -> Option<Vec<(EnvironmentKind, Option<Vec<u8>>)>> {
    let mut index = 0;
    let mut buf = Vec::new();

    if subvec.is_empty() {
        return Some(buf);
    }

    while index < subvec.len() {
        match subvec[index] {
            ENV_USERVAR => {
                let (name, value, size) = decode_env_var(&subvec[index + 1..])?;
                buf.push((EnvironmentKind::UserDefined(Some(name)), value));
                index += size + 1;
            }
            ENV_VAR => {
                let (name, value, size) = decode_env_var(&subvec[index + 1..])?;
                buf.push((
                    EnvironmentKind::WellKnown(Some(WellKnownVariable::from(name.as_str()))),
                    value,
                ));
                index += size + 1;
            }
            _ => return None,
        }
    }

    Some(buf)
}

pub fn decode_env_send_var(kind: u8, name: &[u8]) -> Option<EnvironmentKind> {
    let inner = if name.is_empty() {
        None
    } else {
        let name = std::str::from_utf8(name).ok()?;
        Some(name)
    };

    match kind {
        ENV_USERVAR => {
            let name = inner.map(WellKnownVariable::from);
            Some(EnvironmentKind::WellKnown(name))
        }
        ENV_VAR => {
            let name = inner.map(|n| n.to_string());
            Some(EnvironmentKind::UserDefined(name))
        }
        _ => None,
    }
}

pub fn decode_env_send(subvec: &[u8]) -> Option<Vec<EnvironmentKind>> {
    // Create empty buffer to store decoded EnvironmentKinds
    let mut buf = Vec::new();

    // If subvec is empty, return the empty buffer
    if subvec.is_empty() {
        return Some(buf);
    }

    // Create a mutable reference to hold the name of the current EnvironmentKind
    let mut current_name = Vec::new();

    // Assign the kind of the current EnvironmentKind
    let mut current_kind = subvec[0];

    // Iterate through each byte in subvec (skipping the first)
    // We basically parse until we hit the next variable or the end of subvec
    for b in &subvec[1..] {
        match *b {
            // If the byte matches ENV_USERVAR or ENV_VAR...
            ENV_USERVAR | ENV_VAR => {
                // Decode current_name into EnvironmentKind and push it to the buffer
                buf.push(decode_env_send_var(current_kind, current_name.as_slice())?);
                // Update the current_kind
                current_kind = *b;
                // Clear the current_name vector for the next EnvironmentKind
                current_name.clear();
            }
            // For any other byte...
            _ => {
                // Push the byte to current_name
                current_name.push(*b);
            }
        }
    }

    // Also decode any trailing declarations
    if !current_name.is_empty() {
        buf.push(decode_env_send_var(current_kind, current_name.as_slice())?);
    }

    // Return the filled buffer
    Some(buf)
}

pub fn decode_env(subvec: &[u8]) -> Option<TelnetEvent> {
    // Return None if incoming byte slice is empty.
    if subvec.is_empty() {
        return None;
    }

    // Extract the operation from the first element of the slice,
    // and match it to the corresponding EnvironmentOperation.
    let op = match EnvironmentOperation::from(subvec[0]) {
        // For 'Is' operations, decode the environment variables.
        EnvironmentOperation::Is(_) => EnvironmentOperation::Is(decode_env_is(&subvec[1..])?),
        // For 'Send' operations, decode the environment variables to be sent.
        EnvironmentOperation::Send(_) => EnvironmentOperation::Send(decode_env_send(&subvec[1..])?),
        // For 'Info' operations, decode the environment variables in the information.
        // This is the same as with `EnvironmentOperation::Is`.
        EnvironmentOperation::Info(_) => EnvironmentOperation::Info(decode_env_is(&subvec[1..])?),
        // For 'Unknown' operations, store the unknown data.
        EnvironmentOperation::Unknown(id, _) => {
            EnvironmentOperation::Unknown(id, Bytes::from(subvec[1..].to_vec()))
        }
    };

    // Return a Telnet event with the processed operation data encapsulated within a SubnegotiationType enum.
    Some(TelnetEvent::Subnegotiate(SubnegotiationType::Environment(op)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_env_name_empty_input() {
        let input = &[];
        let decoded = decode_env_name(input);
        assert_eq!(decoded, None);
    }

    #[test]
    fn test_decode_env_name_unescaped_chars_only() {
        let input = b"abcxyz";
        let decoded = decode_env_name(input);
        assert_eq!(decoded, Some((vec![97, 98, 99, 120, 121, 122], 6)));
    }

    #[test]
    fn test_decode_env_name_esc_sequences() {
        let input = &[
            ENV_ESC,
            ENV_VAR,
            ENV_ESC,
            ENV_USERVAR,
            ENV_ESC,
            ENV_VALUE,
            ENV_ESC,
            ENV_ESC,
            IAC,
            IAC,
        ];
        let decoded = decode_env_name(input);
        assert_eq!(decoded, Some((vec![ENV_VAR, ENV_USERVAR, ENV_VALUE, ENV_ESC, IAC], 10)));
    }

    #[test]
    fn test_decode_env_name_non_escaped_special_chars() {
        let input = &[ENV_VAR, ENV_USERVAR, ENV_VALUE];
        let decoded = decode_env_name(input);
        // First match encountered should return
        assert_eq!(decoded, Some((vec![], 0)));
    }

    #[test]
    fn test_decode_env_name_invalid_data() {
        let input = &[ENV_ESC]; // insufficient data to construct an escape sequence
        let decoded = decode_env_name(input);
        assert_eq!(decoded, None);
    }

    #[test]
    fn test_decode_env_name_invalid_esc_seq() {
        let input = &[ENV_ESC, 99]; // 'c' does not form a valid escape sequence
        let decoded = decode_env_name(input);
        assert_eq!(decoded, None);
    }

    #[test]
    fn test_decode_env_value_empty_input() {
        let input = &[];
        let decoded = decode_env_value(input);
        assert_eq!(decoded, Some((Vec::new(), 0)));
    }

    #[test]
    fn test_decode_env_value_unescaped_chars_only() {
        let input = b"abcxyz";
        let decoded = decode_env_value(input);
        assert_eq!(decoded, Some((vec![97, 98, 99, 120, 121, 122], 6)));
    }

    #[test]
    fn test_decode_env_value_esc_sequences() {
        let input = &[
            ENV_ESC,
            ENV_VAR,
            ENV_ESC,
            ENV_USERVAR,
            ENV_ESC,
            ENV_VALUE,
            ENV_ESC,
            ENV_ESC,
            IAC,
            IAC,
        ];
        let decoded = decode_env_value(input);
        assert_eq!(decoded, Some((vec![ENV_VAR, ENV_USERVAR, ENV_VALUE, ENV_ESC, IAC], 10)));
    }

    #[test]
    fn test_decode_env_value_non_escaped_special_chars() {
        let input = &[ENV_VAR, ENV_USERVAR, ENV_VALUE];
        let decoded = decode_env_value(input);
        // First match encountered should return
        assert_eq!(decoded, Some((vec![], 0)));
    }

    #[test]
    fn test_decode_env_value_invalid_data() {
        let input = &[ENV_ESC]; // insufficient data to construct an escape sequence
        let decoded = decode_env_value(input);
        assert_eq!(decoded, None);
    }

    #[test]
    fn test_decode_env_value_invalid_esc_seq() {
        let input = &[ENV_ESC, 99]; // 'c' does not form a valid escape sequence
        let decoded = decode_env_value(input);
        assert_eq!(decoded, None);
    }

    #[test]
    fn test_decode_env_uservar() {
        let (name, value, size) =
            decode_env_var(b"USER\x01test\x03HOME\x03DISPLAY\x01:0.0").unwrap();

        assert_eq!(name, "USER");

        assert_eq!(value, Some(vec![116, 101, 115, 116]));
        assert_eq!(size, 9);
    }

    #[test]
    fn test_decode_env_vars() {
        let decoded = decode_env_is(b"\x00USER\x01test\x03HOME\x03DISPLAY\x01:0.0").unwrap();

        assert_eq!(decoded.len(), 3);

        let (kind, value) = &decoded[0];
        assert_eq!(value, &Some(vec![116, 101, 115, 116]));
        assert!(matches!(kind, EnvironmentKind::WellKnown(Some(WellKnownVariable::User))));
        let (kind, value) = &decoded[1];
        assert!(matches!(kind, EnvironmentKind::UserDefined(Some(_))));
        assert_eq!(kind.name().unwrap(), "HOME");
        assert!(value.is_none());
        let (kind, value) = &decoded[2];
        assert!(matches!(kind, EnvironmentKind::UserDefined(Some(_))));
        assert_eq!(kind.name().unwrap(), "DISPLAY");
        assert_eq!(value, &Some(vec![58, 48, 46, 48]));
    }

    #[test]
    fn test_decode_env_vars_invalid() {
        let decoded = decode_env_is(b"\x00USER\x01test\x03\x03DISPLAY\x01:0.0");
        assert!(decoded.is_none());

        let decoded = decode_env_is(b"\x00USER\x01te\x02st\x03HOME\x03DISPLAY\x01:0.0");
        assert!(decoded.is_none());
    }

    #[test]
    fn test_decode_env_vars_empty() {
        let decoded = decode_env_is(&[]).unwrap();
        assert!(decoded.is_empty());
    }

    #[test]
    fn test_encode_env_op_is() {
        let mut buffer = BytesMut::new();
        let env_var = EnvironmentKind::WellKnown(Some(WellKnownVariable::Job));
        let op = EnvironmentOperation::Is(vec![(env_var, Some(vec![1]))]); // Using Job as example
        encode_env_op(op, &mut buffer);
        assert_eq!(buffer[0], ENV_IS);
        assert_eq!(buffer[1], ENV_VAR);
        assert_eq!(buffer.last(), Some(&1));
    }

    #[test]
    fn test_encode_env_op_send() {
        let mut buffer = BytesMut::new();
        let env_var = EnvironmentKind::UserDefined(Some("VarExample".into()));
        let op = EnvironmentOperation::Send(vec![env_var]);
        encode_env_op(op, &mut buffer);
        assert_eq!(buffer[0], ENV_SEND);
        assert_eq!(buffer[1], ENV_USERVAR);
        assert_eq!(buffer[2..12], *b"VarExample");
    }

    #[test]
    fn test_encode_env_op_info() {
        let mut buffer = BytesMut::new();
        let env_var = EnvironmentKind::WellKnown(Some(WellKnownVariable::User));
        let op = EnvironmentOperation::Info(vec![(env_var, Some(vec![2, 3, 4]))]); // Using User as example with some example bytes for value
        encode_env_op(op, &mut buffer);
        assert_eq!(buffer[0], ENV_INFO);
        assert_eq!(buffer[1], ENV_VAR);
        assert_eq!(&buffer[2..6], b"USER");
        assert_eq!(buffer[6], ENV_VALUE);

        assert_eq!(buffer[7..12], [ENV_ESC, 2, ENV_ESC, 3, 4]);
    }

    #[test]
    fn test_encode_env_op_unknown() {
        let mut buffer = BytesMut::new();
        let buf = Bytes::from_static(b"unknown data");
        let op = EnvironmentOperation::Unknown(5, buf);
        encode_env_op(op, &mut buffer);
        assert_eq!(buffer[0], 5);
        assert_eq!(&buffer[1..], b"unknown data");
    }
}
