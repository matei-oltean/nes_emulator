use crate::{cpu::CPU, ram::RAM};

#[derive(Debug)]
pub struct NES {
    cpu: CPU,
    ram: RAM,
}

const CYCLES_PER_FRAME: u64 = 29781;

impl NES {
    pub fn new(rom_file: &str) -> NES {
        let ram: RAM = RAM::from_file(rom_file);
        NES {
            cpu: CPU::from_ram(&ram),
            ram,
        }
    }

    pub fn run(&mut self) {
        let mut n_cycles: u64 = CYCLES_PER_FRAME;
        loop {
            n_cycles = self.cpu.execute_instructions(&mut self.ram, n_cycles) % CYCLES_PER_FRAME;
            // TODO render
        }
    }
}
