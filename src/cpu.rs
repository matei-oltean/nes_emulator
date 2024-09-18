use std::{fs::File, io::Read};

#[derive(Debug)]
pub struct CPU {
    a: u8,
    x: u8,
    y: u8,
    pc: u16,
    s: u8,
    p: u8,
    ram: [u8; 0x800],
}

impl CPU {
    pub fn new() -> CPU {
        CPU {
            a: 0,
            x: 0,
            y: 0,
            pc: 0,
            s: 0,
            p: 0,
            ram: [0; 0x800],
        }
    }

    pub fn from_file(file_path: &str) -> CPU {
        let mut cpu: CPU = CPU::new();
        let mut file: File = File::open(file_path).unwrap();
        file.read(&mut cpu.ram).unwrap();
        cpu
    }

    pub fn read(&self, addr: u16) -> u8 {
        self.ram[addr as usize]
    }

    pub fn write(&mut self, addr: u16, data: u8) {
        self.ram[addr as usize] = data;
    }
}
