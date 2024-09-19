use std::env;

use nes::NES;

mod bitfield;
mod cpu;
mod nes;
mod ram;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: cargo run -- <rom_file>");
        std::process::exit(1);
    }
    let mut nes: NES = NES::new(&args[1]);
    nes.run();
}
