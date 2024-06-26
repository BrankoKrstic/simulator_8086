use std::io::{BufRead, Seek, SeekFrom};

use crate::instruction::{Immediate, Instruction, JumpType, Location, Memory, Register};

/// Logic for decoding 8086 instructions into assembly
/// User Manual: https://edge.edx.org/c4x/BITSPilani/EEE231/asset/8086_family_Users_Manual_1_.pdf

pub struct Codec<T> {
    source: T,
}

impl<T: BufRead + Seek> Codec<T> {
    pub fn new(source: T) -> Self {
        Self { source }
    }

    pub fn get_byte(&mut self) -> Option<u8> {
        let mut buf = [0; 1];
        self.source.read_exact(&mut buf).ok()?;
        Some(buf[0])
    }
    pub fn load_two(&mut self) -> Option<(u8, u8)> {
        let mut buf = [0; 2];
        self.source.read_exact(&mut buf).ok()?;
        Some((buf[0], buf[1]))
    }

    pub fn decode_all(self) -> Vec<Instruction> {
        self.into_iter().collect()
    }

    pub fn jump(&mut self, bytes: i8) {
        let pos = SeekFrom::Current(bytes as i64);
        let new_pos = self.source.seek(pos).unwrap();
    }

    pub fn next_op(&mut self) -> Option<Instruction> {
        let b1 = self.get_byte()?;
        // User Manual page 161
        match b1 {
            0b01110100 => return Some(Instruction::Jump(JumpType::Je, self.get_byte()? as i8)),
            0b01111100 => return Some(Instruction::Jump(JumpType::Jl, self.get_byte()? as i8)),
            0b01111110 => return Some(Instruction::Jump(JumpType::Jle, self.get_byte()? as i8)),
            0b01110010 => return Some(Instruction::Jump(JumpType::Jb, self.get_byte()? as i8)),
            0b01110110 => return Some(Instruction::Jump(JumpType::Jbe, self.get_byte()? as i8)),
            0b01111010 => return Some(Instruction::Jump(JumpType::Jp, self.get_byte()? as i8)),
            0b01110000 => return Some(Instruction::Jump(JumpType::Jo, self.get_byte()? as i8)),
            0b01111000 => return Some(Instruction::Jump(JumpType::Js, self.get_byte()? as i8)),
            0b01110101 => return Some(Instruction::Jump(JumpType::Jne, self.get_byte()? as i8)),
            0b01111101 => return Some(Instruction::Jump(JumpType::Jnl, self.get_byte()? as i8)),
            0b01111111 => return Some(Instruction::Jump(JumpType::Jnle, self.get_byte()? as i8)),
            0b01110011 => return Some(Instruction::Jump(JumpType::Jnb, self.get_byte()? as i8)),
            0b01110111 => return Some(Instruction::Jump(JumpType::Jnbe, self.get_byte()? as i8)),
            0b01111011 => return Some(Instruction::Jump(JumpType::Jnp, self.get_byte()? as i8)),
            0b01110001 => return Some(Instruction::Jump(JumpType::Jno, self.get_byte()? as i8)),
            0b01111001 => return Some(Instruction::Jump(JumpType::Jns, self.get_byte()? as i8)),
            0b11100010 => return Some(Instruction::Jump(JumpType::Loop, self.get_byte()? as i8)),
            0b11100001 => {
                return Some(Instruction::Jump(
                    JumpType::Jnloopzs,
                    self.get_byte()? as i8,
                ))
            }
            0b11100000 => return Some(Instruction::Jump(JumpType::Loopnz, self.get_byte()? as i8)),
            0b11100011 => return Some(Instruction::Jump(JumpType::Jcxz, self.get_byte()? as i8)),
            0b00110111 => return Some(Instruction::Aaa),
            0b00100111 => return Some(Instruction::Daa),
            _ => {}
        }

        let prefix = b1 >> 4;

        let instruction = match prefix {
            0b1011 => self.decode_immediate_to_register(b1),
            0b1000 => {
                if b1 >> 2 == 0b100000 {
                    self.decode_arithmetic_immediate_to_register_memory(b1)
                } else {
                    self.decode_register_to_memory(b1)
                }
            }
            0b1100 => self.decode_immediate_to_register_memory(b1),
            0b1010 => self.decode_accumulator(b1),
            0b0000 | 0b0010 | 0b0011 => {
                if (b1 >> 2) & 1 == 1 {
                    self.decode_arithmetic_immediate_to_accumulator(b1)
                } else {
                    self.decode_arithmetic_register_memory(b1)
                }
            }
            0b0100 => {
                if (b1 >> 3) & 1 == 1 {
                    Instruction::Dec(Location::Register(Register::new(b1 & 0b111, 1)), None)
                } else {
                    Instruction::Inc(Location::Register(Register::new(b1 & 0b111, 1)), None)
                }
            }
            _ => unreachable!(),
        };

        Some(instruction)
    }
    fn generate_displacement_value(&mut self, w: u8) -> i16 {
        match w {
            1 => {
                let bytes = self.load_two().unwrap();
                ((bytes.1 as i16) << 8) + bytes.0 as i16
            }
            0 => self.get_byte().unwrap() as i8 as i16,
            _ => unreachable!(),
        }
    }
    fn decode_accumulator(&mut self, b1: u8) -> Instruction {
        let opcode = b1 >> 1;
        let w = b1 & 1;
        let displacement = self.generate_displacement_value(w);
        let memory = Location::Memory(Memory::new(None, None, displacement));
        let reg = Location::Register(if w == 1 { Register::AX } else { Register::AL });

        if opcode == 0b1010000 {
            Instruction::Mov(memory, reg)
        } else {
            Instruction::Mov(reg, memory)
        }
    }
    fn get_immediate_data(&mut self, w: u8) -> Immediate {
        let data = if w == 1 {
            let bytes = self.load_two().unwrap();
            ((bytes.1 as i16) << 8) + bytes.0 as i16
        } else {
            self.get_byte().unwrap() as i8 as i16
        };
        Immediate::new(data, None)
    }
    fn decode_immediate_to_register(&mut self, b1: u8) -> Instruction {
        let w = (b1 >> 3) & 1;
        let reg = Register::new(b1 & 0b111, w);
        let immediate = self.get_immediate_data(w);
        Instruction::Mov(Location::Immediate(immediate), Location::Register(reg))
    }

    fn decode_immediate_to_register_memory(&mut self, b1: u8) -> Instruction {
        let w = b1 & 1;

        let b2 = self.get_byte().unwrap();
        let md = b2 >> 6;
        let rm = b2 & 0b111;

        let memory = self.get_memory_location(rm, md);
        let mut immediate = self.get_immediate_data(w);
        immediate.set_w(Some(w));
        Instruction::Mov(Location::Immediate(immediate), Location::Memory(memory))
    }

    fn get_memory_location(&mut self, rm: u8, md: u8) -> Memory {
        let displacement = match (md, rm) {
            (0b10, _) | (0b00, 0b110) => {
                let bytes = self.load_two().unwrap();
                ((bytes.1 as i16) << 8) + bytes.0 as i16
            }
            (0b01, _) => self.get_byte().unwrap() as i8 as i16,
            _ => 0i16,
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
        Memory::new(right_reg1, right_reg2, displacement)
    }

    fn decode_register_to_memory_locations(&mut self, b1: u8) -> (Location, Location) {
        let b2 = self.get_byte().unwrap();

        let d = (b1 & 0b10) >> 1;
        let w = b1 & 0b1;
        let md = b2 >> 6; // mod
        let reg = (b2 >> 3) & 0b111;
        let rm = b2 & 0b111; // r/m

        match (md, w) {
            (0b11, w) => {
                let r1 = Register::new(reg, w);
                let r2 = Register::new(rm, w);
                let (src, dest) = if d == 1 { (r2, r1) } else { (r1, r2) };
                (Location::Register(src), Location::Register(dest))
            }
            (md, w) => {
                let r1 = Location::Register(Register::new(reg, w));

                let r2 = Location::Memory(self.get_memory_location(rm, md));
                let (src, dest) = if d == 1 { (r2, r1) } else { (r1, r2) };
                (src, dest)
            }
        }
    }
    fn decode_register_to_memory(&mut self, b1: u8) -> Instruction {
        let (l1, l2) = self.decode_register_to_memory_locations(b1);
        Instruction::Mov(l1, l2)
    }
    fn decode_arithmetic_register_memory(&mut self, b1: u8) -> Instruction {
        let (l1, l2) = self.decode_register_to_memory_locations(b1);
        let arithmetic_opcode = (b1 >> 3) & 0b111;
        match arithmetic_opcode {
            0b000 => Instruction::Add(l1, l2),
            0b101 => Instruction::Sub(l1, l2),
            0b111 => Instruction::Cmp(l1, l2),
            _ => unreachable!(),
        }
    }
    fn decode_arithmetic_immediate_to_register_memory(&mut self, b1: u8) -> Instruction {
        let w = if b1 & 0b11 == 0b01 { 1 } else { 0 };

        let b2 = self.get_byte().unwrap();

        let md = b2 >> 6;
        let rm = b2 & 0b111;

        let memory = match md {
            0b11 => {
                let r2 = Register::new(rm, b1 & 1);

                Location::Register(r2)
            }
            md => Location::Memory(self.get_memory_location(rm, md)),
        };

        let mut data = self.get_immediate_data(w);
        if md != 0b11 {
            data.set_w(Some(b1 & 1));
        }
        let immediate = Location::Immediate(data);

        let arithmetic_opcode = (b2 >> 3) & 0b111;

        match arithmetic_opcode {
            0b000 => Instruction::Add(immediate, memory),
            0b101 => Instruction::Sub(immediate, memory),
            0b111 => Instruction::Cmp(immediate, memory),
            _ => unreachable!(),
        }
    }

    fn decode_arithmetic_immediate_to_accumulator(&mut self, b1: u8) -> Instruction {
        let w = b1 & 1;
        let displacement = self.generate_displacement_value(w);
        let memory = Location::Memory(Memory::new(None, None, displacement));
        let reg = Location::Register(if w == 1 { Register::AX } else { Register::AL });
        let arithmetic_opcode = (b1 >> 3) & 0b111;

        match arithmetic_opcode {
            0b000 => Instruction::Add(memory, reg),
            0b101 => Instruction::Sub(memory, reg),
            0b111 => Instruction::Cmp(memory, reg),
            _ => unreachable!(),
        }
    }
}

impl<T: BufRead + Seek> IntoIterator for Codec<T> {
    type Item = Instruction;

    type IntoIter = InstructionIterator<T>;

    fn into_iter(self) -> Self::IntoIter {
        InstructionIterator { codec: self }
    }
}

pub struct InstructionIterator<T> {
    codec: Codec<T>,
}

impl<T: BufRead + Seek> Iterator for InstructionIterator<T> {
    type Item = Instruction;

    fn next(&mut self) -> Option<Self::Item> {
        self.codec.next_op()
    }
}
