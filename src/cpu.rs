use crate::{bitfield::Bitfield, ram::RAM};

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

enum Register {
    A,
    X,
    Y,
}

enum StatusFlag {
    Carry = 0,
    Zero = 1,
    InterruptDisable = 2,
    DecimalMode = 3,
    Overflow = 6,
    Negative = 7,
}

#[derive(Debug)]
pub struct CPU {
    a: u8, // Accumulator
    x: u8,
    y: u8,
    pc: u16,     // Program Counter
    s: u8,       // Stack Pointer
    p: Bitfield, // Status register
}

impl CPU {
    pub fn from_ram(ram: &RAM) -> CPU {
        println!(
            "pc at {}",
            u16::from_le_bytes([ram.read(0xFFFC), ram.read(0xFFFD)])
        );
        CPU {
            a: 0,
            x: 0,
            y: 0,
            pc: u16::from_le_bytes([ram.read(0xFFFC), ram.read(0xFFFD)]),
            s: 0,
            p: Bitfield::new(0),
        }
    }

    fn read(&self, ram: &RAM, addr: u16) -> u8 {
        ram.read(addr)
    }

    fn read_next_byte(&mut self, ram: &RAM) -> u8 {
        let value: u8 = self.read(ram, self.pc);
        self.pc += 1;
        value
    }

    fn write(&mut self, ram: &mut RAM, addr: u16, data: u8) {
        ram.write(addr, data);
    }

    fn read_word_number(&mut self, ram: &RAM, addr: u16) -> u16 {
        u16::from_le_bytes([self.read(ram, addr), self.read(ram, addr + 1)])
    }

    fn read_next_word_number(&mut self, ram: &RAM) -> u16 {
        let res = self.read_word_number(ram, self.pc);
        self.pc += 2;
        res
    }

    fn is_crossing_page_boundary(addr1: u16, addr2: u16) -> bool {
        addr1 & 0xFF00 != addr2 & 0xFF00
    }

    fn bne(&mut self, ram: &RAM) -> u64 {
        let mut cycles: u64 = 2;
        let new_location: u16 = self.get_value(ram, AddressingMode::Relative);
        if !self.p.get_bit(StatusFlag::Zero as u8) {
            let page_boundary_crossed: bool = CPU::is_crossing_page_boundary(self.pc, new_location);
            self.pc = new_location;
            cycles += if page_boundary_crossed { 2 } else { 1 };
        }
        cycles
    }

    fn dex(&mut self) {
        println!("DEX");
        self.decrement_register(Register::X);
    }

    fn dey(&mut self) {
        println!("DEY");
        self.decrement_register(Register::Y);
    }

    fn decrement_register(&mut self, register: Register) {
        let reg: &mut u8 = match register {
            Register::A => &mut self.a,
            Register::X => &mut self.x,
            Register::Y => &mut self.y,
        };
        *reg = reg.wrapping_sub(1);
        if *reg == 0 {
            self.p.set_bit(StatusFlag::Zero as u8, true);
        }
        if *reg & (1 << 7) != 0 {
            self.p.set_bit(StatusFlag::Negative as u8, true);
        }
    }

    fn lda(&mut self, ram: &mut RAM, mode: AddressingMode) {
        let value: u8 = self.get_value(&ram, mode) as u8;
        println!("LDA #${:X}", value);
        if value == 0 {
            self.p.set_bit(StatusFlag::Zero as u8, true);
        }
        if value & (1 << 7) != 0 {
            self.p.set_bit(StatusFlag::Negative as u8, true);
        }
        self.a = value;
    }

    fn ldx(&mut self, ram: &mut RAM, mode: AddressingMode) {
        let loaded_value = self.load_into_register(ram, mode, Register::X);
        println!("LDX #${:X}", loaded_value);
    }

    fn ldy(&mut self, ram: &mut RAM, mode: AddressingMode) {
        let loaded_value = self.load_into_register(ram, mode, Register::Y);
        println!("LDY #${:X}", loaded_value);
    }

    fn load_into_register(
        &mut self,
        ram: &mut RAM,
        mode: AddressingMode,
        register: Register,
    ) -> u8 {
        let value: u8 = self.get_value(&ram, mode) as u8;
        if value == 0 {
            self.p.set_bit(StatusFlag::Zero as u8, true);
        }
        if value & (1 << 7) != 0 {
            self.p.set_bit(StatusFlag::Negative as u8, true);
        }
        match register {
            Register::A => self.a = value,
            Register::X => self.x = value,
            Register::Y => self.y = value,
        }
        value
    }

    fn sta(&mut self, ram: &mut RAM, mode: AddressingMode) {
        let addr: u16 = self.get_value(&ram, mode);
        println!("STA ${:X}", addr);
        self.write(ram, addr, self.a);
    }

    fn stx(&mut self, ram: &mut RAM, mode: AddressingMode) {
        let addr: u16 = self.get_value(&ram, mode);
        println!("STX ${:X}", addr);
        self.write(ram, addr, self.x);
    }

    fn txs(&mut self) {
        println!("TXS");
        self.s = self.x;
    }

    fn get_value(&mut self, ram: &RAM, mode: AddressingMode) -> u16 {
        match mode {
            AddressingMode::Accumulator => self.a as u16,
            AddressingMode::Absolute => {
                let addr: u16 = self.read_next_word_number(ram);
                self.read(ram, addr) as u16
            }
            AddressingMode::AbsoluteX => {
                let addr: u16 = self.read_next_word_number(ram)
                    + self.x as u16
                    + self.p.get_bit(StatusFlag::Carry as u8) as u16;
                self.read(ram, addr) as u16
            }
            AddressingMode::AbsoluteY => {
                let addr: u16 = self.read_next_word_number(ram)
                    + self.y as u16
                    + self.p.get_bit(StatusFlag::Carry as u8) as u16;
                self.read(ram, addr) as u16
            }
            AddressingMode::Immediate => {
                let value: u8 = self.read_next_byte(ram);
                value as u16
            }
            AddressingMode::Implied => 0,
            AddressingMode::Indirect => {
                let addr: u16 = self.read_next_word_number(ram);
                self.read_word_number(ram, addr)
            }
            AddressingMode::IndexedIndirect => {
                let addr: u8 = self.read_next_byte(ram);
                self.read_word_number(ram, (addr as u16 + self.x as u16) & 0xFF)
            }
            AddressingMode::IndirectIndexed => {
                let addr: u8 = self.read_next_byte(ram);
                let indirect_addr: u16 = self.read_word_number(ram, addr as u16);
                self.read(ram, indirect_addr + self.y as u16) as u16
            }
            AddressingMode::Relative => {
                // TODO: fix
                let offset: i8 = self.read_next_byte(ram) as i8;
                match i16::try_from(self.pc) {
                    Ok(pc) => (pc + offset as i16) as u16,
                    Err(_) => {
                        eprintln!("Program counter conversion failed");
                        std::process::exit(1);
                    }
                }
            }
            AddressingMode::ZeroPage => {
                let addr: u8 = self.read_next_byte(ram);
                self.read(ram, addr as u16) as u16
            }
            AddressingMode::ZeroPageX => {
                let addr: u8 = self.read_next_byte(ram);
                self.read(ram, (addr + self.x) as u16) as u16
            }
            AddressingMode::ZeroPageY => {
                let addr: u8 = self.read_next_byte(ram);
                self.read(ram, (addr + self.y) as u16) as u16
            }
        }
    }

    fn execute_next_instruction(&mut self, ram: &mut RAM) -> u64 {
        let opcode: u8 = self.read_next_byte(ram);
        match opcode {
            0x00 => {
                println!("BRK");
                std::process::exit(1);
            }
            0x78 => {
                println!("SEI");
                self.p.set_bit(StatusFlag::InterruptDisable as u8, true);
                2
            }
            0x81 => {
                self.sta(ram, AddressingMode::IndexedIndirect);
                6
            }
            0x85 => {
                self.sta(ram, AddressingMode::ZeroPage);
                3
            }
            0x86 => {
                self.stx(ram, AddressingMode::ZeroPage);
                3
            }
            0x88 => {
                self.dey();
                2
            }
            0x8D => {
                self.sta(ram, AddressingMode::Absolute);
                4
            }
            0x8E => {
                self.stx(ram, AddressingMode::Absolute);
                4
            }
            0x91 => {
                self.sta(ram, AddressingMode::IndirectIndexed);
                6
            }
            0x95 => {
                self.sta(ram, AddressingMode::ZeroPageX);
                4
            }
            0x96 => {
                self.stx(ram, AddressingMode::ZeroPageY);
                4
            }
            0x99 => {
                self.sta(ram, AddressingMode::AbsoluteY);
                5
            }
            0x9A => {
                self.txs();
                2
            }
            0x9D => {
                self.sta(ram, AddressingMode::AbsoluteX);
                5
            }
            0xA0 => {
                self.ldy(ram, AddressingMode::Immediate);
                2
            }
            0xA2 => {
                self.ldx(ram, AddressingMode::Immediate);
                2
            }
            0xA4 => {
                self.ldy(ram, AddressingMode::ZeroPage);
                3
            }
            0xA9 => {
                self.lda(ram, AddressingMode::Immediate);
                2
            }
            0xAC => {
                self.ldy(ram, AddressingMode::Absolute);
                4
            }
            0xB4 => {
                self.ldy(ram, AddressingMode::ZeroPageX);
                4
            }
            0xD0 => self.bne(ram),
            0xD8 => {
                println!("CLD");
                self.p.set_bit(StatusFlag::DecimalMode as u8, false);
                2
            }
            _ => {
                eprintln!("Unknown opcode: {:#X}", opcode);
                std::process::exit(1);
            }
        }
    }

    pub fn execute_instructions(&mut self, ram: &mut RAM, n_instructions: u64) -> u64 {
        let mut n_cycles: u64 = 0_u64;
        while n_cycles < n_instructions {
            n_cycles += self.execute_next_instruction(ram);
        }
        n_cycles
    }
}
