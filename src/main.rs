use std::env;

use cpu::CPU;

mod cpu;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: cargo run -- <rom_file>");
        std::process::exit(1);
    }
    let cpu: CPU = CPU::from_file(&args[1]);
    println!("{:?}", cpu);
}
