#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Opcode {
    Nop = 0x00,
    Read = 0x01,
    Write = 0x02,
    Reset = 0x03,
    Status = 0x04,
    Inference = 0x05,
    LoadWeights = 0x06,
    Shutdown = 0x07,
}

impl Opcode {
    pub fn from_byte(b: u8) -> Option<Self> {
        match b {
            0x00 => Some(Opcode::Nop),
            0x01 => Some(Opcode::Read),
            0x02 => Some(Opcode::Write),
            0x03 => Some(Opcode::Reset),
            0x04 => Some(Opcode::Status),
            0x05 => Some(Opcode::Inference),
            0x06 => Some(Opcode::LoadWeights),
            0x07 => Some(Opcode::Shutdown),
            _ => None,
        }
    }

    pub fn has_payload(&self) -> bool {
        matches!(self, Opcode::Write | Opcode::LoadWeights)
    }
}

impl std::fmt::Display for Opcode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Opcode::Nop => "nop",
            Opcode::Read => "read",
            Opcode::Write => "write",
            Opcode::Reset => "reset",
            Opcode::Status => "status",
            Opcode::Inference => "inference",
            Opcode::LoadWeights => "load_weights",
            Opcode::Shutdown => "shutdown",
        };
        write!(f, "{s}")
    }
}

const HEADER_SIZE: usize = 8;
const MAGIC: u8 = 0xA5;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Command {
    pub opcode: Opcode,
    pub addr: u32,
    pub seq: u16,
    pub payload: Vec<u8>,
}

impl Command {
    pub fn new(opcode: Opcode) -> Self {
        Self {
            opcode,
            addr: 0,
            seq: 0,
            payload: Vec::new(),
        }
    }

    pub fn with_addr(mut self, addr: u32) -> Self {
        self.addr = addr;
        self
    }

    pub fn with_seq(mut self, seq: u16) -> Self {
        self.seq = seq;
        self
    }

    pub fn with_payload(mut self, data: Vec<u8>) -> Self {
        self.payload = data;
        self
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(HEADER_SIZE + self.payload.len());
        buf.push(MAGIC);
        buf.push(self.opcode as u8);
        buf.extend_from_slice(&self.addr.to_le_bytes());
        buf.extend_from_slice(&self.seq.to_le_bytes());
        buf.extend_from_slice(&self.payload);
        buf
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseError {
    TooShort { got: usize, need: usize },
    BadMagic { got: u8 },
    UnknownOpcode { got: u8 },
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::TooShort { got, need } => {
                write!(f, "too short: {got}/{need}")
            }
            ParseError::BadMagic { got } => {
                write!(f, "bad magic: 0x{got:02X}")
            }
            ParseError::UnknownOpcode { got } => {
                write!(f, "unknown opcode: 0x{got:02X}")
            }
        }
    }
}

impl std::error::Error for ParseError {}

pub fn parse_command(data: &[u8]) -> Result<Command, ParseError> {
    if data.len() < HEADER_SIZE {
        return Err(ParseError::TooShort {
            got: data.len(),
            need: HEADER_SIZE,
        });
    }
    if data[0] != MAGIC {
        return Err(ParseError::BadMagic { got: data[0] });
    }
    let opcode = Opcode::from_byte(data[1]).ok_or(ParseError::UnknownOpcode { got: data[1] })?;
    let addr = u32::from_le_bytes([data[2], data[3], data[4], data[5]]);
    let seq = u16::from_le_bytes([data[6], data[7]]);
    let payload = data[HEADER_SIZE..].to_vec();
    Ok(Command {
        opcode,
        addr,
        seq,
        payload,
    })
}

#[derive(Debug, Clone)]
pub struct CommandParser {
    total_parsed: u64,
    total_errors: u64,
}

impl CommandParser {
    pub fn new() -> Self {
        Self {
            total_parsed: 0,
            total_errors: 0,
        }
    }

    pub fn parse(&mut self, data: &[u8]) -> Result<Command, ParseError> {
        match parse_command(data) {
            Ok(cmd) => {
                self.total_parsed += 1;
                Ok(cmd)
            }
            Err(e) => {
                self.total_errors += 1;
                Err(e)
            }
        }
    }

    pub fn total_parsed(&self) -> u64 {
        self.total_parsed
    }

    pub fn total_errors(&self) -> u64 {
        self.total_errors
    }
}

impl Default for CommandParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opcode_from_byte() {
        assert_eq!(Opcode::from_byte(0x01), Some(Opcode::Read));
        assert_eq!(Opcode::from_byte(0xFF), None);
    }

    #[test]
    fn opcode_display() {
        assert_eq!(Opcode::Read.to_string(), "read");
        assert_eq!(Opcode::LoadWeights.to_string(), "load_weights");
    }

    #[test]
    fn opcode_has_payload() {
        assert!(Opcode::Write.has_payload());
        assert!(!Opcode::Nop.has_payload());
    }

    #[test]
    fn encode_roundtrip() {
        let cmd = Command::new(Opcode::Write)
            .with_addr(0x100)
            .with_seq(42)
            .with_payload(vec![0xAA, 0xBB]);
        let encoded = cmd.encode();
        let decoded = parse_command(&encoded).unwrap();
        assert_eq!(decoded.opcode, Opcode::Write);
        assert_eq!(decoded.addr, 0x100);
        assert_eq!(decoded.seq, 42);
        assert_eq!(decoded.payload, vec![0xAA, 0xBB]);
    }

    #[test]
    fn encode_no_payload() {
        let cmd = Command::new(Opcode::Status).with_seq(1);
        assert_eq!(cmd.encode().len(), HEADER_SIZE);
    }

    #[test]
    fn parse_too_short() {
        let err = parse_command(&[MAGIC, 0x01]).unwrap_err();
        assert!(matches!(err, ParseError::TooShort { .. }));
    }

    #[test]
    fn parse_bad_magic() {
        let mut buf = vec![0x00; HEADER_SIZE];
        buf[0] = 0xFF;
        let err = parse_command(&buf).unwrap_err();
        assert!(matches!(err, ParseError::BadMagic { .. }));
    }

    #[test]
    fn parse_unknown_opcode() {
        let mut buf = vec![0x00; HEADER_SIZE];
        buf[0] = MAGIC;
        buf[1] = 0xFE;
        let err = parse_command(&buf).unwrap_err();
        assert!(matches!(err, ParseError::UnknownOpcode { .. }));
    }

    #[test]
    fn parser_tracks_stats() {
        let mut p = CommandParser::new();
        let ok_cmd = Command::new(Opcode::Nop);
        p.parse(&ok_cmd.encode()).unwrap();
        p.parse(&[0xFF]).unwrap_err();
        assert_eq!(p.total_parsed(), 1);
        assert_eq!(p.total_errors(), 1);
    }

    #[test]
    fn command_builder() {
        let cmd = Command::new(Opcode::Inference).with_addr(0x200).with_seq(99);
        assert_eq!(cmd.opcode, Opcode::Inference);
        assert_eq!(cmd.addr, 0x200);
        assert_eq!(cmd.seq, 99);
    }

    #[test]
    fn error_display() {
        assert!(ParseError::TooShort { got: 2, need: 8 }.to_string().contains("2/8"));
        assert!(ParseError::BadMagic { got: 0xFF }.to_string().contains("0xFF"));
    }
}
