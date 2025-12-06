use crate::opcodes::OpCode;

type Data = i16;
const DATA_SIZE: usize = size_of::<Data>();
const ARG_SIZE: usize = 2;
const HEX_ARG_SIZE: usize = ARG_SIZE * 2;

struct Procedure {
    // byte position of procedure in program
    start_pos: usize,
    // starts with space for variables
    frame_ptr: usize,
}

pub struct PL0VM {
    program: Vec<u8>,
    stack: Vec<u8>,
    procedures: Vec<Procedure>,
}

impl PL0VM {
    pub fn new() -> PL0VM { PL0VM {
        program: vec![],
        stack: vec![],
        procedures: vec![],
    } }

    pub fn from_file(filename: &str) -> Result<PL0VM, std::io::Error> {
        let mut pl0vm = PL0VM::new();
        match pl0vm.load_from_file(filename) {
            Ok(_) => Ok(pl0vm),
            Err(e) => Err(e),
        }
    }

    pub fn load_from_file(&mut self, filename: &str) -> Result<bool, std::io::Error> {
        match std::fs::read(filename) {
            Ok(bytes) => { self.program = bytes; Ok(true) },
            Err(err) => { Err(err) },
        }
    }

    fn read_arg(&self, offset: usize) -> i16 {
        i16::from_le_bytes(self.program[offset..(offset + ARG_SIZE)].try_into().expect("Invalid byte count?!"))
    }

    pub fn debug_print(&self) {
        let mut pc = 4;
        println!("Procedure count: {}", self.read_arg(0));
        print!("Arch: ");
        match self.read_arg(ARG_SIZE) {
            2 => println!("16 bit"),
            4 => println!("32 bit"),
            8 => println!("64 bit"),
            _ => (),
        }

        let print_arg = |pc: &mut usize, last: bool| {
            print!("{:0HEX_ARG_SIZE$X}{}", self.read_arg(*pc), if last { "" } else { ", " });
            *pc += ARG_SIZE;
        };

        while pc < self.program.len() {
            let byte = self.program[pc];
            let op = OpCode::try_from(byte).expect("Unknown opcode");
            print!("{:04X}: {:02X} {:<20} ", pc - 4, byte, op);
            pc += 1;
            match op {
                OpCode::PushValueLocalVar | OpCode::PushValueMainVar
                    | OpCode::PushAddressLocalVar | OpCode::PushAddressMainVar
                    | OpCode::CallProc => {
                    print_arg(&mut pc, true);
                },
                OpCode::Jump | OpCode::JumpIfFalse => {
                    let arg = self.read_arg(pc);
                    print!("{}{:0HEX_ARG_SIZE$X}", if arg < 0 { "-" } else { "" }, arg.abs());
                    pc += ARG_SIZE;
                },
                OpCode::PushValueGlobalVar | OpCode::PushAddressGlobalVar => {
                    print_arg(&mut pc, false);
                    print_arg(&mut pc, true);
                },
                OpCode::EntryProc => {
                    print_arg(&mut pc, false);
                    print_arg(&mut pc, false);
                    print_arg(&mut pc, true);
                    print!(" <<< Procedure start");
                }
                OpCode::PutString => {
                    let strb: Vec<_> = self.program.iter().skip(pc).take_while(|&&b| b != 0).map(|b| *b).collect();
                    pc += strb.len() + 1;
                    let str = String::from_utf8(strb).expect("Invalid UTF-8");
                    print!("\"{str}\"");
                }
                _ => {},
            }

            println!();
        }
    }

    fn load_procedures(&mut self) {
        let mut i = 0;
        let mut proc_bytes_remaining = 0;
        while i < self.program.len() {
            let byte = self.program[i];
            if proc_bytes_remaining == 0 && byte == OpCode::EntryProc.into() {
                i += 1;
                proc_bytes_remaining = self.read_arg(i);
            }
        }
    }

    pub fn execute(&mut self) {
        self.load_procedures();
    }
}
