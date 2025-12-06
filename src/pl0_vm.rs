use crate::opcodes::OpCode;

type Data = i32;
const DATA_SIZE: usize = size_of::<Data>();
const ARG_SIZE: usize = 2;
const HEX_ARG_SIZE: usize = ARG_SIZE * 2;

#[derive(Debug)]
struct Procedure {
    // byte position of procedure in program
    start_pos: usize,
    // starts with space for variables
    frame_ptr: usize,
}

pub struct PL0VM {
    program: Vec<u8>,
}

impl PL0VM {
    pub fn new() -> PL0VM { PL0VM {
        program: vec![],
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
    fn read_data(&self, offset: usize) -> Data {
        Data::from_le_bytes(self.program[offset..(offset + DATA_SIZE)].try_into().expect("Invalid byte count?!"))
    }

    pub fn debug_print(&self) {
        let mut pc = 4;
        let mut procedure_count = self.read_arg(0);
        println!("Procedure count: {}", procedure_count);
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

        let mut rem_bytes = 0;
        loop {
            let byte = self.program[pc];
            let opc = pc;
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
                    rem_bytes = self.read_arg(pc);
                    print!("{:0HEX_ARG_SIZE$X}, ", rem_bytes);
                    pc += ARG_SIZE;
                    print_arg(&mut pc, false);
                    print_arg(&mut pc, true);
                    print!(" <<< Procedure start");
                    procedure_count -= 1;
                }
                OpCode::PutString => {
                    let strb: Vec<_> = self.program.iter().skip(pc).take_while(|&&b| b != 0).map(|b| *b).collect();
                    pc += strb.len() + 1;
                    let str = String::from_utf8(strb).expect("Invalid UTF-8");
                    print!("\"{str}\"");
                }
                _ => {},
            }
            rem_bytes -= (pc - opc) as i16;

            println!();

            if rem_bytes <= 0 && procedure_count == 0 { break; }
        }
        (0..((self.program.len() - pc) / DATA_SIZE)).map(|i| self.read_data(pc + DATA_SIZE * i)).enumerate().for_each(|(i, constant)| {
            let ds2 = DATA_SIZE * 2;
            println!("Constant {:04}: 0x{:0ds2$X} = {}", i, constant, constant);
        });
    }

    fn load_data(&self) -> (Vec<Procedure>, Vec<Data>) {
        let mut procedure_count = self.read_arg(0);
        let mut procedures = Vec::with_capacity(procedure_count as usize);
        procedures.resize_with(procedures.capacity(), || None);
        let mut pc = 4;

        let mut rem_bytes = 0;
        loop {
            let byte = self.program[pc];
            let opc = pc;
            pc += 1;
            if rem_bytes == 0 && byte == OpCode::EntryProc.into() {
                rem_bytes = self.read_arg(pc);
                pc += ARG_SIZE;
                let proc_id = self.read_arg(pc) as usize;
                pc += ARG_SIZE * 2;
                procedures[proc_id] = Some(Procedure {
                    start_pos: pc - 1 - ARG_SIZE * 3,
                    frame_ptr: 0,
                });
                procedure_count -= 1;
            }
            rem_bytes -= (pc - opc) as i16;

            if rem_bytes <= 0 && procedure_count == 0 { break; }
        }
        (
            procedures.into_iter().map(|procedure| procedure.unwrap()).collect(),
            (0..((self.program.len() - pc) / DATA_SIZE)).map(|i| self.read_data(pc + DATA_SIZE * i)).collect(),
        )
    }

    pub fn execute(&self) {
        let (procedures, constants) = self.load_data();

        procedures.iter().for_each(|procedure| println!("{:?}", procedure));
        constants.iter().enumerate().for_each(|(i, constant)| println!("const {i}: {:?}", constant));

        let mut pc = procedures[0].start_pos;

        loop {
            let byte = self.program[pc];

            match OpCode::try_from(byte).expect("Unknown opcode") {
                OpCode::EntryProc => {},
                OpCode::ReturnProc => {
                    break;
                },
                _ => (),
            }
            pc += 1;
        }
    }
}
