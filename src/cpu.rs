use std::io::BufRead;

use crate::instruction::{Instruction, Location, Register};

enum Bits {
    High,
    Low,
    All,
}

#[derive(Default)]
pub struct Cpu<T> {
    /// 0: ax
    /// 1: bx
    /// 2: cx
    /// 3: dx
    /// 4: sp
    /// 5: bp
    /// 6: si
    /// 7: di
    /// 8: ss
    /// 9: ds
    /// 10: es
    registers: [u16; 11],
    instructions: T,
    sf: bool,
    zf: bool,
}

impl<T: BufRead> Cpu<T> {
    pub fn new(instructions: T) -> Self {
        Self {
            registers: [0; 11],
            instructions,
            sf: false,
            zf: false,
        }
    }
    pub fn execute_instruction(&mut self, instruction: Instruction) {
        match instruction {
            Instruction::Mov(src, dest) => self.execute_mov(src, dest),
            Instruction::Add(src, dest) => self.execute_add(src, dest),
            Instruction::Adc(_, _) => todo!(),
            Instruction::Sbb(_, _) => todo!(),
            Instruction::Sub(src, dest) => self.execute_sub(src, dest),
            Instruction::Cmp(src, dest) => self.execute_cmp(src, dest),
            Instruction::Jump(_, _) => todo!(),
            Instruction::Daa => todo!(),
            Instruction::Aaa => todo!(),
            Instruction::Inc(_, _) => todo!(),
            Instruction::Dec(_, _) => todo!(),
        }
    }
    fn get_location(&mut self, src: &Location, dest: &Location) -> (u16, &mut u16, Bits) {
        let val = match src {
            Location::Register(ref reg) => {
                let (reg, w) = self.decode_register(reg);

                match w {
                    Bits::High => *reg >> 8,
                    Bits::Low => *reg & 0xFF,
                    Bits::All => *reg,
                }
            }
            Location::Memory(_) => todo!(),
            Location::Immediate(val) => val.data as u16,
        };
        let (mov_to, w) = match dest {
            Location::Register(ref reg) => self.decode_register(reg),
            Location::Memory(_) => todo!(),
            Location::Immediate(_) => unimplemented!(),
        };
        (val, mov_to, w)
    }
    fn execute_mov(&mut self, src: Location, dest: Location) {
        let (val, mov_to, w) = self.get_location(&src, &dest);
        let val_to_mov = match w {
            Bits::High => (val << 8) | (*mov_to & 0xFF),
            Bits::Low => (val & 0xFF) | (*mov_to & 0xFF00),
            Bits::All => val,
        };

        println!("mov {}: {:#06x}->{:#06x}", dest, *mov_to, val_to_mov);
        *mov_to = val_to_mov;
    }
    fn execute_add(&mut self, src: Location, dest: Location) {
        let (val, to, w) = self.get_location(&src, &dest);
        let result = match w {
            Bits::High => (val << 8) + (*to & 0xFF),
            _ => val + *to,
        };
        *to = result;
        print!("add {}: {:#06x}->{:#06x} ", dest, *to, result);

        self.set_flags(result);
        self.print_flags();
    }
    fn execute_sub(&mut self, src: Location, dest: Location) {
        let (val, to, w) = self.get_location(&src, &dest);
        let result = match w {
            Bits::High => *to - (val << 8),
            _ => *to - val,
        };
        *to = result;
        print!("sub {}: {:#06x}->{:#06x} ", dest, *to, result);

        self.set_flags(result);
        self.print_flags();
    }
    fn execute_cmp(&mut self, src: Location, dest: Location) {
        let (val, to, w) = self.get_location(&src, &dest);
        let result = match w {
            Bits::High => *to - (val << 8),
            _ => *to - val,
        };
        print!("cmp {}: {:#06x}->{:#06x} ", dest, *to, result);

        self.set_flags(result);
        self.print_flags();
    }
    fn set_flags(&mut self, result: u16) {
        self.zf = result == 0;
        self.sf = (result & 0x8000) > 0;
    }
    fn decode_register(&mut self, reg: &Register) -> (&mut u16, Bits) {
        match *reg {
            Register::AL => (&mut self.registers[0], Bits::Low),
            Register::CL => (&mut self.registers[2], Bits::Low),
            Register::DL => (&mut self.registers[3], Bits::Low),
            Register::BL => (&mut self.registers[1], Bits::Low),
            Register::AH => (&mut self.registers[0], Bits::High),
            Register::CH => (&mut self.registers[2], Bits::High),
            Register::DH => (&mut self.registers[3], Bits::High),
            Register::BH => (&mut self.registers[1], Bits::High),
            Register::AX => (&mut self.registers[0], Bits::All),
            Register::CX => (&mut self.registers[2], Bits::All),
            Register::DX => (&mut self.registers[3], Bits::All),
            Register::BX => (&mut self.registers[1], Bits::All),
            Register::SP => (&mut self.registers[4], Bits::All),
            Register::BP => (&mut self.registers[5], Bits::All),
            Register::SI => (&mut self.registers[6], Bits::All),
            Register::DI => (&mut self.registers[7], Bits::All),
            Register::SS => (&mut self.registers[8], Bits::All),
            Register::DS => (&mut self.registers[9], Bits::All),
            Register::ES => (&mut self.registers[10], Bits::All),
        }
    }
    pub fn print_registers(&self) {
        println!("ax: {:#04x} ({})", self.registers[0], self.registers[0]);
        println!("bx: {:#04x} ({})", self.registers[1], self.registers[1]);
        println!("cx: {:#04x} ({})", self.registers[2], self.registers[2]);
        println!("dx: {:#04x} ({})", self.registers[3], self.registers[3]);
        println!("sp: {:#04x} ({})", self.registers[4], self.registers[4]);
        println!("bp: {:#04x} ({})", self.registers[5], self.registers[5]);
        println!("si: {:#04x} ({})", self.registers[6], self.registers[6]);
        println!("di: {:#04x} ({})", self.registers[7], self.registers[7]);
        println!("ss: {:#04x} ({})", self.registers[7], self.registers[8]);
        println!("ds: {:#04x} ({})", self.registers[7], self.registers[9]);
        println!("es: {:#04x} ({})", self.registers[7], self.registers[10]);
        self.print_flags();
    }
    pub fn print_flags(&self) {
        print!("flags: ");
        if self.zf {
            print!("Z");
        }
        if self.sf {
            print!("S");
        }
        println!();
    }
}
