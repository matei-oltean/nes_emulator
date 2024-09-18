use std::{fs::File, io::Read};

// Memory map:
// 0x0000 - 0x07FF: 2KB internal RAM
// 0x0800 - 0x0FFF: Mirrors of 0x0000 - 0x07FF
// 0x1000 - 0x17FF: Mirrors of 0x0000 - 0x07FF
// 0x1800 - 0x1FFF: Mirrors of 0x0000 - 0x07FF
// 0x2000 - 0x2007: PPU registers
// 0x2008 - 0x3FFF: Mirrors of 0x2000 - 0x2007 (repeats every 8 bytes)
// 0x4000 - 0x4017: APU and I/O registers
// 0x4018 - 0x401F: APU and I/O functionality that is normally disabled
// 0x4020 - 0xFFFF: Cartridge space: PRG ROM, PRG RAM, and mapper registers

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

    fn get_ram_address(addr: u16) -> usize {
        match addr {
            0x0800..=0x1FFF => (addr % 0x0800) as usize,
            0x2008..=0x3FFF => 0x2000 + (addr % 0x0008) as usize,
            _ => addr as usize,
        }
    }

    pub fn read(&self, addr: u16) -> u8 {
        self.ram[RAM::get_ram_address(addr)]
    }

    pub fn write(&mut self, addr: u16, data: u8) {
        self.ram[RAM::get_ram_address(addr)] = data;
    }
}