use std::{
    fmt::Display,
    io::{self, Read},
};

/// Logic for decoding 8086 instructions into assembly
/// User Manual: https://edge.edx.org/c4x/BITSPilani/EEE231/asset/8086_family_Users_Manual_1_.pdf

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

pub struct Codec<T> {
    source: T,
    buf: [u8; 1024],
    len: usize,
    cur_byte: usize,
}

impl<T: Read> Codec<T> {
    pub fn new(source: T) -> Self {
        Self {
            source,
            buf: [0u8; 1024],
            len: 0,
            cur_byte: 0,
        }
    }
    fn load_bytes(&mut self) -> Result<(), io::Error> {
        let read = self.source.read(&mut self.buf[..])?;

        self.cur_byte = 0;
        self.len = read;
        Ok(())
    }
    pub fn get_byte(&mut self) -> Option<u8> {
        if self.cur_byte >= self.len {
            self.load_bytes().ok()?;
        }
        if self.len == 0 {
            return None;
        }
        let byte = self.buf[self.cur_byte];
        self.cur_byte += 1;
        Some(byte)
    }
    pub fn load_two(&mut self) -> Option<(u8, u8)> {
        Some((self.get_byte()?, self.get_byte()?))
    }

    pub fn decode_all(self) -> Vec<Instruction> {
        self.into_iter().collect()
    }

    fn next_opcode(&mut self) -> Option<Instruction> {
        let (b1, b2) = self.load_two()?;
        // User Manual page 161
        let opcode = b1 >> 2;
        let d = (b1 & 0b10) >> 1;
        let w = b1 & 0b1;
        let md = b2 >> 6; // mod
        let reg = (b2 >> 3) & 0b111;
        let rm = b2 & 0b111; // r/m
        println!("{} {}", b1, b2);
        let instruction = match (opcode, md, w) {
            (0b100010, 0b11, w) => {
                let r1 = Register::new(reg, w);
                let r2 = Register::new(rm, w);
                let (src, dest) = if d == 1 { (r2, r1) } else { (r1, r2) };
                Instruction::MovRegisters { src, dest }
            }
            (0b100010, x, w) => {
                // no displacement
                let r1 = Register::new(reg, w);
                let r2 = Register::new(rm, w);
                let (src, dest) = if d == 1 { (r2, r1) } else { (r1, r2) };
                Instruction::MovRegisters { src, dest }
            }
            (0b100010, 0b01, 2) => {
                // 8-bit displacement
                let r1 = Register::new(reg, w);
                let r2 = Register::new(rm, w);
                let (src, dest) = if d == 1 { (r2, r1) } else { (r1, r2) };
                Instruction::MovRegisters { src, dest }
            }
            (0b100010, 0b00, 2) => {
                // 16-bit displacement
                let r1 = Register::new(reg, w);
                let r2 = Register::new(rm, w);
                let (src, dest) = if d == 1 { (r2, r1) } else { (r1, r2) };
                Instruction::MovRegisters { src, dest }
            }
            _ => unimplemented!(),
        };
        Some(instruction)
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
