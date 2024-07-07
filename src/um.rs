#![allow(dead_code)]

use std::{ops::{Index, IndexMut, Div, BitAnd, Not}, usize, rc::Rc};

use rustc_hash::FxHashMap;


pub type Plate = u32;

pub trait IOInterface {
    fn request_input(&mut self) -> u8;
    fn request_output(&mut self, ch: u8);
}

pub struct UniversalMachine<'a> {
    pub registers: Registers,
    pub ip: usize,
    pub next_array_id: Plate,
    pub arrays: FxHashMap<Plate, Rc<[Plate]>>,
    pub is_halted: bool,
    pub io: &'a mut dyn IOInterface,
}

#[derive(Default)]
pub struct Registers {
    pub regs: [Plate; 8],
}

impl Index<RegId> for Registers {
    type Output = Plate;

    fn index(&self, index: RegId) -> &Self::Output {
        &self.regs[index as usize]
    }
}

impl IndexMut<RegId> for Registers {
    fn index_mut(&mut self, index: RegId) -> &mut Self::Output {
        &mut self.regs[index as usize]
    }
}

impl <'a> UniversalMachine<'a> {
    fn plate_from_bytes(bytes: &[u8]) -> Option<Plate> {
        let bytes = bytes.try_into().ok()?;
        return Some(Plate::from_be_bytes(bytes))
    }  
    
    pub fn new(program: &[u8],
               io: &'a mut dyn IOInterface) -> Option<Self> {
        let program_array: Vec<Plate> = program.chunks(4)
            .map(UniversalMachine::plate_from_bytes)
            .collect::<Option<Vec<Plate>>>()?;
        let registers = Registers::default();
        let mut arrays = FxHashMap::default();
        arrays.insert(0, program_array.into());
        Some(UniversalMachine {
            registers,
            ip: 0,
            next_array_id: 1,
            arrays,
            io,
            is_halted: false,
        })
    }
    
    pub fn run(&mut self, max_steps: Option<u32>) {
        let mut step = 0;
        let max_steps = max_steps.unwrap_or(u32::max_value());
        while !self.is_halted && step < max_steps {
            let command = Command::decode(self.arrays[&0][self.ip]);
            step += 1;
            self.perform_command(&command);
            match &command {
                Command::LoadProg { .. } => {},
                _ => {
                    self.ip += 1;
                }
            }
        }
    } 
    
    fn perform_command(&mut self, command: &Command) {
        match *command {
            Command::CondMove { dst, src, cnd } => {
                if self.registers[cnd] != 0  {
                    self.registers[dst] = self.registers[src]
                }
            },
            Command::ArrLoad { dst, arr, offset } => {
                let arr = self.registers[arr];
                let offset = self.registers[offset] as usize;
                self.registers[dst] = self.arrays[&arr][offset];
            },
            Command::ArrStore { src, arr, offset } => {
                let arr = self.registers[arr];
//                if arr == 0 {
//                    eprintln!("Modifying program: {:?}", command);
//                }
                let offset = self.registers[offset] as usize;
                let v = self.arrays.get_mut(&arr).unwrap();
                match Rc::get_mut(v) {
                    Some(vm) => {
                        vm[offset] = self.registers[src];
                    },
                    None => {
                        let mut new_v = v.to_vec();
                        new_v[offset] = self.registers[src];
                        *v = new_v.into();
                    },
                }
            },
            Command::Add { dst, op1, op2 } => {
                let op1 = self.registers[op1];
                let op2 = self.registers[op2];
                self.registers[dst] = op1.wrapping_add(op2);
            },
            Command::Mul { dst, op1, op2 } => {
                let op1 = self.registers[op1];
                let op2 = self.registers[op2];
                self.registers[dst] = op1.wrapping_mul(op2);
            },
            Command::Div { dst, op1, op2 } => {
                let op1 = self.registers[op1];
                let op2 = self.registers[op2];
                self.registers[dst] = op1.div(op2);
            },
            Command::NotAnd { dst, op1, op2 } => {
                let op1 = self.registers[op1];
                let op2 = self.registers[op2];
                self.registers[dst] = op1.bitand(op2).not();
            },
            Command::Halt => {
                self.is_halted = true;
            },
            Command::Alloc { dst, size } => {
                let next_id = self.next_array_id;
                self.next_array_id += 1;
                let size = self.registers[size] as usize;
                self.arrays.insert(next_id, vec![0; size].into());
                self.registers[dst] = next_id;
            },
            Command::Free { arr } => {
                let arr = self.registers[arr];
                self.arrays.remove(&arr);
            },
            Command::Output { src } => {
                let src = self.registers[src];
                self.io.request_output(src as u8);
            },
            Command::Input { dst } => {
                self.registers[dst] = self.io.request_input() as Plate;
            },
            Command::LoadProg { arr, offset } => {
                let arr = self.registers[arr];
                let offset = self.registers[offset] as usize;
                self.arrays.insert(0, self.arrays[&arr].clone());
                self.ip = offset;
            },
            Command::StoreConst { dst, val } => {
                self.registers[dst] = val;
            },
        }
    }
}

type RegId = u8;

#[derive(Debug)]
enum Command {
    CondMove {
        dst: RegId,
        src: RegId,
        cnd: RegId,
    },
    ArrLoad {
        dst: RegId,
        arr: RegId,
        offset: RegId,
    },
    ArrStore {
        src: RegId,
        arr: RegId,
        offset: RegId,
    },
    Add {
        dst: RegId,
        op1: RegId,
        op2: RegId,
    },
    Mul {
        dst: RegId,
        op1: RegId,
        op2: RegId
    },
    Div {
        dst: RegId,
        op1: RegId,
        op2: RegId,
    },
    NotAnd {
        dst: RegId,
        op1: RegId,
        op2: RegId,
    },
    Halt,
    Alloc {
        dst: RegId,
        size: RegId,
    },
    Free {
        arr: RegId,
    },
    Output {
        src: RegId,
    },
    Input {
        dst: RegId,
    },
    LoadProg {
        arr: RegId,
        offset: RegId
    },
    StoreConst {
        dst: RegId,
        val: Plate,
    }
}

impl Command {
    fn decode_registers_standard(p: Plate) -> (RegId, RegId, RegId) {
        let c = (p & 0b111) as RegId;
        let b = ((p >> 3) & 0b111) as RegId;
        let a = ((p >> 6) & 0b111) as RegId;
        (a, b, c)
    }
    
    fn decode_special(p: Plate) -> (RegId, Plate) {
        let a = ((p >> 25) & 0b111) as RegId;
        let v = p & 0b1111111111111111111111111;
        (a, v)
    }
    
    fn decode_command_id(p: Plate) -> u8 {
        ((p >> 28) & 0b1111) as u8
    }
    
    pub fn decode(p: Plate) -> Command {
        match Command::decode_command_id(p) {
            0  => {
                let (a, b, c) = Command::decode_registers_standard(p);
                Command::CondMove { src: b, dst: a, cnd: c }
            },
            1 => {
                let (a, b, c) = Command::decode_registers_standard(p);
                Command::ArrLoad { dst: a, arr: b, offset: c }
            }
            2 => {
                let (a, b, c) = Command::decode_registers_standard(p);
                Command::ArrStore { src: c, arr: a, offset: b }
            }
            3 => {
                let (a, b, c) = Command::decode_registers_standard(p);
                Command::Add { dst: a, op1: b, op2: c }
            }
            4 => {
                let (a, b, c) = Command::decode_registers_standard(p);
                Command::Mul { dst: a, op1: b, op2: c }
            }
            5 => {
                let (a, b, c) = Command::decode_registers_standard(p);
                Command::Div { dst: a, op1: b, op2: c }
            }
            6 => {
                let (a, b, c) = Command::decode_registers_standard(p);
                Command::NotAnd { dst: a, op1: b, op2: c }
            }
            7 => {
                Command::Halt
            }
            8 => {
                let (_a, b, c) = Command::decode_registers_standard(p);
                Command::Alloc { dst: b, size: c }
            }
            9 => {
                let (_a, _b, c) = Command::decode_registers_standard(p);
                Command::Free { arr: c }
            }
            10 => {
                let (_a, _b, c) = Command::decode_registers_standard(p);
                Command::Output { src: c }
            }
            11 => {
                let (_a, _b, c) = Command::decode_registers_standard(p);
                Command::Input { dst: c }
            }
            12 => {
                let (_a, b, c) = Command::decode_registers_standard(p);
                Command::LoadProg { arr: b, offset: c }
            }
            13 => {
                let (a, v) = Command::decode_special(p);
                Command::StoreConst { dst: a, val: v }
            }

            _ => {
                unreachable!()
            }
        }
    }
}