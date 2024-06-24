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
pub struct Memory {
    reg1: Option<Register>,
    reg2: Option<Register>,
    displacement: u16,
}

impl Memory {
    fn new(reg1: Option<Register>, reg2: Option<Register>, displacement: u16) -> Self {
        Self {
            reg1,
            reg2,
            displacement,
        }
    }
}

impl Display for Memory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match (self.reg1.as_ref(), self.reg2.as_ref(), self.displacement) {
            (Some(reg1), Some(reg2), 0) => write!(f, "[{} + {}]", reg1, reg2),
            (Some(reg1), Some(reg2), x) => write!(f, "[{} + {} + {}]", reg1, reg2, x),
            (Some(reg1), None, 0) => write!(f, "[{}]", reg1),
            (Some(reg1), None, x) => write!(f, "[{} + {}]", reg1, x),
            (_, _, x) => {
                write!(f, "[{}]", x)
            }
        }
    }
}

#[derive(Debug)]
struct Immediate {
    data: u16,
}

impl Display for Immediate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.data)
    }
}

impl Immediate {
    fn new(data: u16) -> Self {
        Self { data }
    }
}

#[derive(Debug)]
pub enum Copy {
    Register(Register),
    Memory(Memory),
    Immediate(Immediate),
}

impl Display for Copy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Copy::Register(x) => write!(f, "{}", x),
            Copy::Memory(x) => write!(f, "{}", x),
            Copy::Immediate(x) => write!(f, "{}", x),
        }
    }
}

#[derive(Debug)]
pub enum Instruction {
    Mov(Copy, Copy),
}

impl Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Instruction::Mov(src, dest) => {
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
        let b1 = self.get_byte()?;
        // User Manual page 161
        let opcode = b1 >> 2;
        let prefix = opcode >> 2;
        println!("next");
        println!("{:#b} {:#b}", prefix, opcode);
        let instruction = match (prefix, opcode) {
            (0b1011, _) => self.decode_immediate_to_register(b1),
            (_, 0b100010) => self.decode_register_to_memory(b1),
            _ => unreachable!(),
        };

        Some(instruction)
    }
    fn decode_immediate_to_register(&mut self, b1: u8) -> Instruction {
        let w = (b1 >> 3) & 1;
        let reg = Register::new(b1 & 0b111, w);
        let data = if w == 1 {
            let bytes = self.load_two().unwrap();
            ((bytes.1 as u16) << 8) + bytes.0 as u16
        } else {
            self.get_byte().unwrap() as u16
        };
        Instruction::Mov(Copy::Immediate(Immediate::new(data)), Copy::Register(reg))
    }

    fn decode_register_to_memory(&mut self, b1: u8) -> Instruction {
        let b2 = self.get_byte().unwrap();

        let opcode = b1 >> 2;
        let d = (b1 & 0b10) >> 1;
        let w = b1 & 0b1;
        let md = b2 >> 6; // mod
        let reg = (b2 >> 3) & 0b111;
        let rm = b2 & 0b111; // r/m

        match (opcode, md, w) {
            (0b100010, 0b11, w) => {
                let r1 = Register::new(reg, w);
                let r2 = Register::new(rm, w);
                let (src, dest) = if d == 1 { (r2, r1) } else { (r1, r2) };
                Instruction::Mov(Copy::Register(src), Copy::Register(dest))
            }
            (0b100010, md, w) => {
                let r1 = Register::new(reg, w);

                let displacement = match (md, rm) {
                    (0b10, _) | (0b00, 0b110) => {
                        let bytes = self.load_two().unwrap();
                        ((bytes.1 as u16) << 8) + bytes.0 as u16
                    }
                    (0b01, _) => self.get_byte().unwrap() as u16,
                    _ => 0u16,
                };

                let (right_reg1, right_reg2) = match (rm, md) {
                    (0b110, 0b00) => (None, None),
                    (0b000, _) => (Some(Register::BX), Some(Register::SI)),
                    (0b001, _) => (Some(Register::BX), Some(Register::DI)),
                    (0b010, _) => (Some(Register::BP), Some(Register::SI)),
                    (0b011, _) => (Some(Register::BP), Some(Register::DI)),
                    (0b100, _) => (Some(Register::SI), None),
                    (0b101, _) => (Some(Register::DI), None),
                    (0b110, _) => (Some(Register::BP), None),
                    (0b111, _) => (Some(Register::BX), None),

                    _ => unreachable!(),
                };

                let r2 = Copy::Memory(Memory::new(right_reg1, right_reg2, displacement));
                let r1 = Copy::Register(r1);
                let (src, dest) = if d == 1 { (r2, r1) } else { (r1, r2) };
                Instruction::Mov(src, dest)
            }
            _ => unreachable!(),
        }
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
