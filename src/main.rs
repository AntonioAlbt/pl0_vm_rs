use std::env;
use crate::pl0_vm::PL0VM;

mod pl0_vm;
mod opcodes;

fn main() {
    let mut analyze_only = false;
    let mut debug = false;
    let mut filename: Option<&str> = None;
    let args: Vec<String> = env::args().collect();

    args.iter().skip(1).for_each(|arg| {
        if arg == "--analyze" || arg == "-a" {
            analyze_only = true;
        } else if arg == "--debug" || arg == "-d" {
            debug = true;
        } else {
            filename = Some(arg);
        }
    });

    if filename.is_none() {
        println!("Usage: {} [-a|--analyze] [-d|--debug] <filename>", args[0]);
        return;
    }

    let pl0vm = PL0VM::from_file(debug, filename.unwrap()).expect("Failed loading PL0VM");

    if analyze_only {
        pl0vm.print_analysis();
    } else {
        pl0vm.execute();
    }
}
