use std::{fs::File, io::Read};

#[derive(Debug)]
pub struct RAM {
    ram: [u8; 0x10000],
}

impl RAM {
    pub fn new() -> RAM {
        RAM { ram: [0; 0x10000] }
    }

    pub fn from_file(file_path: &str) -> RAM {
        let mut ram: RAM = RAM::new();
        let mut file: File = File::open(file_path).unwrap();
        file.read(&mut ram.ram).unwrap();
        ram
    }

    pub fn read(&self, addr: u16) -> u8 {
        self.ram[addr as usize]
    }

    pub fn write(&mut self, addr: u16, data: u8) {
        self.ram[addr as usize] = data;
    }
}
