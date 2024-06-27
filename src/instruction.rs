use std::fmt::Display;

static mut LABEL_COUNTER: usize = 0;

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
    SS,
    DS,
    ES,
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
            Register::SS => "ss",
            Register::DS => "ds",
            Register::ES => "es",
        };
        write!(f, "{}", display)
    }
}

impl Register {
    pub fn new(opcode: u8, w: u8) -> Self {
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
    pub(crate) reg1: Option<Register>,
    pub(crate) reg2: Option<Register>,
    pub(crate) displacement: i16,
}

impl Memory {
    pub fn new(reg1: Option<Register>, reg2: Option<Register>, displacement: i16) -> Self {
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
            (Some(reg1), Some(reg2), x) => {
                if x > 0 {
                    write!(f, "[{} + {} + {}]", reg1, reg2, x)
                } else {
                    write!(f, "[{} + {} - {}]", reg1, reg2, x.abs())
                }
            }
            (Some(reg1), None, 0) => write!(f, "[{}]", reg1),
            (Some(reg1), None, x) => {
                if x > 0 {
                    write!(f, "[{} + {}]", reg1, x)
                } else {
                    write!(f, "[{} - {}]", reg1, x.abs())
                }
            }
            (_, _, x) => {
                write!(f, "[{}]", x)
            }
        }
    }
}

#[derive(Debug)]
pub struct Immediate {
    pub data: i16,
    pub w: Option<u8>,
}

impl Display for Immediate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(w) = &self.w {
            if *w == 1 {
                write!(f, "word {}", self.data)
            } else {
                write!(f, "byte {}", self.data)
            }
        } else {
            write!(f, "{}", self.data)
        }
    }
}

impl Immediate {
    pub fn new(data: i16, w: Option<u8>) -> Self {
        Self { data, w }
    }
    pub fn set_w(&mut self, w: Option<u8>) {
        self.w = w;
    }
}

#[derive(Debug)]
pub enum Location {
    Register(Register),
    Memory(Memory),
    Immediate(Immediate),
}

impl Display for Location {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Location::Register(x) => write!(f, "{}", x),
            Location::Memory(x) => write!(f, "{}", x),
            Location::Immediate(x) => write!(f, "{}", x),
        }
    }
}

#[derive(Debug)]
pub enum Instruction {
    Mov(Location, Location),
    Add(Location, Location),
    Adc(Location, Location),
    Sbb(Location, Location),
    Sub(Location, Location),
    Cmp(Location, Location),
    Jump(JumpType, i8),
    Daa,
    Aaa,
    Inc(Location, Option<u8>),
    Dec(Location, Option<u8>),
}

impl Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Instruction::Mov(src, dest) => {
                write!(f, "mov {}, {}", dest, src)
            }
            Instruction::Add(src, dest) => write!(f, "add {}, {}", dest, src),
            Instruction::Adc(src, dest) => write!(f, "adc {}, {}", dest, src),
            Instruction::Sbb(src, dest) => write!(f, "sbb {}, {}", dest, src),
            Instruction::Sub(src, dest) => write!(f, "sub {}, {}", dest, src),
            Instruction::Cmp(src, dest) => write!(f, "cmp {}, {}", dest, src),
            Instruction::Jump(instruction, disp) => write!(
                f,
                "{} label_{} ; {}",
                instruction,
                {
                    unsafe {
                        LABEL_COUNTER += 1;
                        LABEL_COUNTER
                    }
                },
                disp
            ),
            Instruction::Daa => write!(f, "daa"),
            Instruction::Aaa => write!(f, "aaa"),
            Instruction::Inc(dest, amount) => {
                if let Some(amount) = amount {
                    write!(f, "inc {}, {}", dest, amount)
                } else {
                    write!(f, "inc {}", dest)
                }
            }
            Instruction::Dec(dest, amount) => {
                if let Some(amount) = amount {
                    write!(f, "dec {}, {}", dest, amount)
                } else {
                    write!(f, "dec {}", dest)
                }
            }
        }
    }
}

#[derive(Debug)]
pub enum JumpType {
    Je,
    Jl,
    Jle,
    Jb,
    Jbe,
    Jp,
    Jo,
    Js,
    Jne,
    Jnl,
    Jnle,
    Jnb,
    Jnbe,
    Jnp,
    Jno,
    Jns,
    Loop,
    Jnloopzs,
    Loopnz,
    Jcxz,
}

impl Display for JumpType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let out = match self {
            JumpType::Je => "je",
            JumpType::Jl => "jl",
            JumpType::Jle => "jle",
            JumpType::Jb => "jb",
            JumpType::Jbe => "jbe",
            JumpType::Jp => "jp",
            JumpType::Jo => "jo",
            JumpType::Js => "js",
            JumpType::Jne => "jne",
            JumpType::Jnl => "jnl",
            JumpType::Jnle => "jnle",
            JumpType::Jnb => "jnb",
            JumpType::Jnbe => "jnbe",
            JumpType::Jnp => "jnp",
            JumpType::Jno => "jno",
            JumpType::Jns => "jns",
            JumpType::Loop => "loop",
            JumpType::Jnloopzs => "jnloopzs",
            JumpType::Loopnz => "loopnz",
            JumpType::Jcxz => "jcxz",
        };
        write!(f, "{}", out)
    }
}
