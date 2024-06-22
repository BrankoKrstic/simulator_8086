use std::{fmt::Display, io::Read};

#[derive(Debug)]
pub enum Register {
    AL,
    CL,
    DL,
    BL,
    AH,
    CH,
    DH,
    BH,
    AX,
    CX,
    DX,
    BX,
    SP,
    BP,
    SI,
    DI,
}

impl Display for Register {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let display = match self {
            Register::AL => "al",
            Register::CL => "cl",
            Register::DL => "dl",
            Register::BL => "bl",
            Register::AH => "ah",
            Register::CH => "ch",
            Register::DH => "dh",
            Register::BH => "bh",
            Register::AX => "ax",
            Register::CX => "cx",
            Register::DX => "dx",
            Register::BX => "bx",
            Register::SP => "sp",
            Register::BP => "bp",
            Register::SI => "si",
            Register::DI => "di",
        };
        write!(f, "{}", display)
    }
}

impl Register {
    fn new(opcode: u8, w: u8) -> Self {
        use Register::*;
        match (opcode, w) {
            (0b000, 0) => AL,
            (0b000, 1) => AX,
            (0b001, 0) => CL,
            (0b001, 1) => CX,
            (0b010, 0) => DL,
            (0b010, 1) => DX,
            (0b011, 0) => BL,
            (0b011, 1) => BX,
            (0b100, 0) => AH,
            (0b100, 1) => SP,
            (0b101, 0) => CH,
            (0b101, 1) => BP,
            (0b110, 0) => DH,
            (0b110, 1) => SI,
            (0b111, 0) => BH,
            (0b111, 1) => DI,
            _ => panic!("Invalid register or w pattern!"),
        }
    }
}

#[derive(Debug)]
pub enum Instruction {
    MovRegisters { src: Register, dest: Register },
}

impl Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Instruction::MovRegisters { src, dest } => {
                write!(f, "mov {}, {}", dest, src)
            }
        }
    }
}

fn decode_bytes(b1: u8, b2: u8) -> Instruction {
    let opcode = b1 >> 2;
    let d = (b1 & 0b10) >> 1;
    let w = b1 & 0b1;
    let md = b2 >> 6; // mod
    let reg = (b2 >> 3) & 0b111;
    let rm = b2 & 0b111; // r/m

    match (opcode, md) {
        (0b100010, 0b11) => {
            let r1 = Register::new(reg, w);
            let r2 = Register::new(rm, w);
            let (src, dest) = if d == 1 { (r2, r1) } else { (r1, r2) };
            Instruction::MovRegisters { src, dest }
        }
        _ => unimplemented!(),
    }
}

pub struct Codec<T> {
    source: T,
    buf: Vec<u8>,
}

impl<T: Read> Codec<T> {
    pub fn new(source: T) -> Self {
        Self {
            source,
            buf: vec![0u8; 2],
        }
    }
    pub fn next_opcode(&mut self) -> Option<Instruction> {
        self.source.read_exact(&mut self.buf[0..2]).ok()?;
        Some(decode_bytes(self.buf[0], self.buf[1]))
    }

    pub fn decode_all(self) -> Vec<Instruction> {
        self.into_iter().collect()
    }
}

impl<T: Read> IntoIterator for Codec<T> {
    type Item = Instruction;

    type IntoIter = InstructionIterator<T>;

    fn into_iter(self) -> Self::IntoIter {
        InstructionIterator { codec: self }
    }
}

pub struct InstructionIterator<T> {
    codec: Codec<T>,
}

impl<T: Read> Iterator for InstructionIterator<T> {
    type Item = Instruction;

    fn next(&mut self) -> Option<Self::Item> {
        self.codec.next_opcode()
    }
}
