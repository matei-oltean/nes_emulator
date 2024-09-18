use std::{fs::File, io::Read};

#[derive(Debug)]
enum AddressingMode {
    Accumulator,
    Absolute,
    AbsoluteX,
    AbsoluteY,
    Immediate,
    Implied,
    Indirect,
    IndexedIndirect,
    IndirectIndexed,
    Relative,
    ZeroPage,
    ZeroPageX,
    ZeroPageY,
}

#[derive(Debug)]
pub struct CPU {
    a: u8,
    x: u8,
    y: u8,
    pc: u16,
    s: u8,
    p: u8,
    ram: [u8; 0x10000],
}

impl CPU {
    pub fn new() -> CPU {
        CPU {
            a: 0, // Accumulator
            x: 0,
            y: 0,
            pc: 0, // Program Counter
            s: 0,  // Stack Pointer
            p: 0,  // Status register
            ram: [0; 0x10000],
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

    fn read_next_byte(&mut self) -> u8 {
        let value: u8 = self.read(self.pc);
        self.pc += 1;
        value
    }

    pub fn write(&mut self, addr: u16, data: u8) {
        self.ram[addr as usize] = data;
    }

    fn get_cary(&self) -> u8 {
        self.p & 0b0000_0001
    }

    fn read_word_number(&mut self, addr: u16) -> u16 {
        u16::from_le_bytes([self.read(addr), self.read(addr + 1)])
    }

    fn read_next_word_number(&mut self) -> u16 {
        let res = self.read_word_number(self.pc);
        self.pc += 2;
        res
    }

    fn get_value(&mut self, mode: AddressingMode) -> u16 {
        match mode {
            AddressingMode::Accumulator => self.a as u16,
            AddressingMode::Absolute => {
                let addr: u16 = self.read_next_word_number();
                self.read(addr) as u16
            }
            AddressingMode::AbsoluteX => {
                let addr: u16 =
                    self.read_next_word_number() + self.x as u16 + self.get_cary() as u16;
                self.read(addr) as u16
            }
            AddressingMode::AbsoluteY => {
                let addr: u16 =
                    self.read_next_word_number() + self.y as u16 + self.get_cary() as u16;
                self.read(addr) as u16
            }
            AddressingMode::Immediate => {
                let value: u8 = self.read_next_byte();
                value as u16
            }
            AddressingMode::Implied => 0,
            AddressingMode::Indirect => {
                let addr: u16 = self.read_next_word_number();
                let indirect_addr: u16 = self.read_word_number(addr);
                indirect_addr
            }
            AddressingMode::IndexedIndirect => {
                let addr: u8 = self.read_next_byte();
                self.read_word_number((addr as u16 + self.x as u16) & 0xFF)
            }
            AddressingMode::IndirectIndexed => {
                let addr: u8 = self.read_next_byte();
                let indirect_addr: u16 = self.read_word_number(addr as u16);
                self.read(indirect_addr + self.y as u16) as u16
            }
            AddressingMode::Relative => {
                let offset: i8 = self.read_next_byte() as i8;
                match i16::try_from(self.pc) {
                    Ok(pc) => (pc + offset as i16) as u16,
                    Err(_) => {
                        eprintln!("Program counter conversion failed");
                        std::process::exit(1);
                    }
                }
            }
            AddressingMode::ZeroPage => {
                let addr: u8 = self.read_next_byte();
                self.read(addr as u16) as u16
            }
            AddressingMode::ZeroPageX => {
                let addr: u8 = self.read_next_byte();
                self.read((addr + self.x) as u16) as u16
            }
            AddressingMode::ZeroPageY => {
                let addr: u8 = self.read_next_byte();
                self.read((addr + self.y) as u16) as u16
            }
            _ => {
                eprintln!("Unknown addressing mode: {:?}", mode);
                std::process::exit(1);
            }
        }
    }

    pub fn execute_next_intruction(&mut self) {
        let opcode: u8 = self.read_next_byte();
        match opcode {
            0x00 => {
                println!("BRK");
            }
            _ => {
                eprintln!("Unknown opcode: {:#X}", opcode);
                std::process::exit(1);
            }
        }
    }
}
