use std::env;
use std::process::exit;
use crate::pl0_vm::PL0VM;

const VERSION: &str = env!("CARGO_PKG_VERSION");

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
        } else if arg == "--help" || arg == "-h" {
            println!("\
Usage: pl0_vm_rs [flags] <filename>
Flags:
  -a, --analyze\tOutput bytecode analysis information. (doesn't run the program)
  -d, --debug\tOutput debug information while running the program. (outputs operations being run, with additional information)
  -h, --help\tDisplay this message and exit.

pl0_vm_rs v{VERSION}");
            exit(0);
        } else {
            filename = Some(arg);
        }
    });

    if filename.is_none() {
        println!("View usage information with: {} --help", args[0]);
        return;
    }

    let pl0vm = PL0VM::from_file(debug, filename.unwrap()).expect("Failed loading PL0VM");

    if analyze_only {
        pl0vm.print_analysis();
    } else {
        pl0vm.execute();
    }
}
