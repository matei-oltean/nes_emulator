use std::{
    fs::File,
    io::{Read, Seek, SeekFrom},
};

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

const CPU_ROM_START_ADDRESS: usize = 0x8000;
const PRG_PAGE_SIZE: usize = 0x4000;
const CHR_PAGE_SIZE: usize = 0x2000;

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
        let mut header: [u8; 16] = [0; 16];
        file.read_exact(&mut header).unwrap();
        if header[..4] != [0x4E, 0x45, 0x53, 0x1A] {
            panic!("Invalid NES file");
        }
        let prg_rom_size: usize;
        let _chr_rom_size: usize;
        let is_nes_2_0: bool = (header[7] & 0x0C) == 0x08;
        let has_trainer: bool = (header[6] & 0b00000100) != 0;
        if is_nes_2_0 {
            prg_rom_size =
                (((header[9] & 0b1111) as usize) << 8 | header[4] as usize) * PRG_PAGE_SIZE;
            _chr_rom_size =
                (((header[9] & 0b11110000) as usize) << 4 | header[5] as usize) * CHR_PAGE_SIZE;
        } else {
            prg_rom_size = header[4] as usize * PRG_PAGE_SIZE;
            _chr_rom_size = header[5] as usize * CHR_PAGE_SIZE;
        }
        // always skip trainer
        if has_trainer {
            file.seek(SeekFrom::Current(512)).unwrap();
        }
        // TODO handle more than 2 pages of PRG ROM
        file.read_exact(&mut ram.ram[CPU_ROM_START_ADDRESS..CPU_ROM_START_ADDRESS + prg_rom_size])
            .unwrap();
        if prg_rom_size == PRG_PAGE_SIZE {
            ram.ram.copy_within(
                CPU_ROM_START_ADDRESS..CPU_ROM_START_ADDRESS + PRG_PAGE_SIZE,
                0xC000,
            );
        }
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
