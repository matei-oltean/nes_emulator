use crate::{bitfield::Bitfield, ram::RAM};

#[derive(Debug)]
enum AddressingMode {
    Accumulator,
    Absolute,
    AbsoluteX,
    AbsoluteY,
    Immediate,
    // Implied is a placeholder for instructions that don't require an operand,
    Indirect,
    IndexedIndirect, // (Indirect, X)
    IndirectIndexed, // (Indirect), Y
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
            "pc at {:X}",
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

    fn print_instruction(op_name: &str, mode: &AddressingMode, value: u16) {
        match mode {
            AddressingMode::Accumulator => println!("{} A", op_name),
            AddressingMode::Absolute => println!("{} ${:04X}", op_name, value),
            AddressingMode::AbsoluteX => println!("{} ${:04X},X", op_name, value),
            AddressingMode::AbsoluteY => println!("{} ${:04X},Y", op_name, value),
            AddressingMode::Immediate => println!("{} #${:02X}", op_name, value),
            // AddressingMode::Implied => println!("{}", op_name),
            AddressingMode::Indirect => println!("{} (${:02X})", op_name, value),
            AddressingMode::IndexedIndirect => println!("{} (${:02X},X)", op_name, value),
            AddressingMode::IndirectIndexed => println!("{} (${:02X}),Y", op_name, value),
            AddressingMode::Relative | AddressingMode::ZeroPage => {
                println!("{} ${:02X}", op_name, value)
            }
            AddressingMode::ZeroPageX => println!("{} ${:02X},X", op_name, value),
            AddressingMode::ZeroPageY => println!("{} ${:02X},Y", op_name, value),
        };
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

    fn bcc(&mut self, ram: &RAM) -> u64 {
        self.branch_if_comparison(ram, !self.p.get_bit(StatusFlag::Carry as u8), "BCC")
    }

    fn bcs(&mut self, ram: &RAM) -> u64 {
        self.branch_if_comparison(ram, self.p.get_bit(StatusFlag::Carry as u8), "BCS")
    }

    fn beq(&mut self, ram: &RAM) -> u64 {
        self.branch_if_comparison(ram, self.p.get_bit(StatusFlag::Zero as u8), "BEQ")
    }

    fn bit(&mut self, ram: &RAM, mode: &AddressingMode) {
        let value = self.get_value(ram, mode).0 as u8;
        let result: u8 = self.a & value;
        self.p.set_bit(StatusFlag::Zero as u8, result == 0);
        self.p
            .set_bit(StatusFlag::Overflow as u8, value & (1 << 6) != 0);
        self.p
            .set_bit(StatusFlag::Negative as u8, value & (1 << 7) != 0);
    }

    fn bmi(&mut self, ram: &RAM) -> u64 {
        self.branch_if_comparison(ram, self.p.get_bit(StatusFlag::Negative as u8), "BMI")
    }

    fn bne(&mut self, ram: &RAM) -> u64 {
        self.branch_if_comparison(ram, !self.p.get_bit(StatusFlag::Zero as u8), "BNE")
    }

    fn bpl(&mut self, ram: &RAM) -> u64 {
        self.branch_if_comparison(ram, !self.p.get_bit(StatusFlag::Negative as u8), "BPL")
    }

    fn branch_if_comparison(&mut self, ram: &RAM, condition: bool, op_name: &str) -> u64 {
        let mut cycles: u64 = 2;
        let (new_location, page_boundary_crossed) = self.get_value(ram, &AddressingMode::Relative);
        println!(
            "{} ${:02X}",
            op_name,
            (new_location as i32 - self.pc as i32) as u8
        );
        if condition {
            self.pc = new_location;
            cycles += if page_boundary_crossed { 2 } else { 1 };
        }
        cycles
    }

    fn dec(&mut self, ram: &mut RAM, mode: &AddressingMode) {
        let addr: u16 = self.get_value(ram, mode).0;
        Self::print_instruction("DEC", mode, addr);
        let value: u8 = self.read(ram, addr).wrapping_sub(1);
        self.write(ram, addr, value);
        self.p.set_bit(StatusFlag::Zero as u8, value == 0);
        self.p
            .set_bit(StatusFlag::Negative as u8, value & (1 << 7) != 0);
    }

    fn decrement_register(name: &str, p: &mut Bitfield, reg: &mut u8) -> u64 {
        println!("{}", name);
        *reg = reg.wrapping_sub(1);
        p.set_bit(StatusFlag::Zero as u8, *reg == 0);
        p
            .set_bit(StatusFlag::Negative as u8, *reg & (1 << 7) != 0);
        2
    }

    fn jmp(&mut self, ram: &RAM, mode: &AddressingMode) {
        let (addr, _) = self.get_value(ram, mode);
        Self::print_instruction("JMP", mode, addr);
        self.pc = addr;
    }

    fn inc(&mut self, ram: &mut RAM, mode: &AddressingMode) {
        let addr: u16 = self.get_value(ram, mode).0;
        Self::print_instruction("INC", mode, addr);
        let value: u8 = self.read(ram, addr).wrapping_add(1);
        self.write(ram, addr, value);
        self.p.set_bit(StatusFlag::Zero as u8, value == 0);
        self.p
            .set_bit(StatusFlag::Negative as u8, value & (1 << 7) != 0);
    }

    fn increment_register(name: &str, p: &mut Bitfield, reg: &mut u8) -> u64 {
        println!("{}", name);
        *reg = reg.wrapping_add(1);
        p.set_bit(StatusFlag::Zero as u8, *reg == 0);
        p
            .set_bit(StatusFlag::Negative as u8, *reg & (1 << 7) != 0);
        2
    }

    fn lda(&mut self, ram: &mut RAM, mode: &AddressingMode) -> u64 {
        let (value, cycles) = self.load_into_register(ram, mode, Register::A);
        Self::print_instruction("LDA", mode, value as u16);
        cycles
    }

    fn ldx(&mut self, ram: &mut RAM, mode: &AddressingMode) -> u64 {
        let (value, cycles) = self.load_into_register(ram, mode, Register::X);
        Self::print_instruction("LDX", mode, value as u16);
        cycles
    }

    fn ldy(&mut self, ram: &mut RAM, mode: &AddressingMode) -> u64 {
        let (value, cycles) = self.load_into_register(ram, mode, Register::Y);
        Self::print_instruction("LDY", mode, value as u16);
        cycles
    }

    fn load_into_register(
        &mut self,
        ram: &mut RAM,
        mode: &AddressingMode,
        register: Register,
    ) -> (u8, u64) {
        let result = self.get_value(ram, mode);
        let value = result.0 as u8;
        let cycles = match mode {
            AddressingMode::Immediate => 2,
            AddressingMode::ZeroPage => 3,
            AddressingMode::ZeroPageX
            | AddressingMode::ZeroPageY
            | AddressingMode::Absolute
            | AddressingMode::AbsoluteX
            | AddressingMode::AbsoluteY => 4 + result.1 as u64,
            AddressingMode::IndexedIndirect => 6,
            AddressingMode::IndirectIndexed => 5 + result.1 as u64,
            _ => 0,
        };
        self.p.set_bit(StatusFlag::Zero as u8, value == 0);
        self.p
            .set_bit(StatusFlag::Negative as u8, value & (1 << 7) != 0);
        match register {
            Register::A => self.a = value,
            Register::X => self.x = value,
            Register::Y => self.y = value,
        }
        (value, cycles)
    }

    fn sta(&mut self, ram: &mut RAM, mode: &AddressingMode) {
        let addr: u16 = self.get_value(ram, mode).0;
        Self::print_instruction("STA", mode, addr);
        self.write(ram, addr, self.a);
    }

    fn stx(&mut self, ram: &mut RAM, mode: &AddressingMode) {
        let addr: u16 = self.get_value(ram, mode).0;
        Self::print_instruction("STX", mode, addr);
        self.write(ram, addr, self.x);
    }

    fn sty(&mut self, ram: &mut RAM, mode: &AddressingMode) {
        let addr: u16 = self.get_value(ram, mode).0;
        Self::print_instruction("STY", mode, addr);
        self.write(ram, addr, self.y);
    }

    fn transfer_accumulator_to(name: &str, p: &mut Bitfield, src: u8, dest: &mut u8) -> u64 {
        println!("{}", name);
        *dest = src;
        p.set_bit(StatusFlag::Zero as u8, src == 0);
        p.set_bit(StatusFlag::Negative as u8, src & (1 << 7) != 0);
        2
    }

    fn get_value(&mut self, ram: &RAM, mode: &AddressingMode) -> (u16, bool) {
        match mode {
            AddressingMode::Accumulator => (self.a as u16, false),
            AddressingMode::Absolute => {
                let addr: u16 = self.read_next_word_number(ram);
                (self.read(ram, addr) as u16, false)
            }
            AddressingMode::AbsoluteX => {
                let addr: u16 = self.read_next_word_number(ram)
                    + self.x as u16
                    + self.p.get_bit(StatusFlag::Carry as u8) as u16;
                (
                    self.read(ram, addr) as u16,
                    CPU::is_crossing_page_boundary(addr, addr - self.x as u16),
                )
            }
            AddressingMode::AbsoluteY => {
                let addr: u16 = self.read_next_word_number(ram)
                    + self.y as u16
                    + self.p.get_bit(StatusFlag::Carry as u8) as u16;
                (
                    self.read(ram, addr) as u16,
                    CPU::is_crossing_page_boundary(addr, addr - self.y as u16),
                )
            }
            AddressingMode::Immediate => {
                let value: u8 = self.read_next_byte(ram);
                (value as u16, false)
            }
            // AddressingMode::Implied => (0, false),
            AddressingMode::Indirect => {
                let addr: u16 = self.read_next_word_number(ram);
                (self.read_word_number(ram, addr), false)
            }
            AddressingMode::IndexedIndirect => {
                let addr: u8 = self.read_next_byte(ram);
                (
                    self.read_word_number(ram, (addr as u16 + self.x as u16) & 0xFF),
                    false,
                )
            }
            AddressingMode::IndirectIndexed => {
                let addr: u8 = self.read_next_byte(ram);
                let indirect_addr: u16 = self.read_word_number(ram, addr as u16);
                let new_location: u16 = indirect_addr + self.y as u16;
                (
                    self.read(ram, new_location) as u16,
                    CPU::is_crossing_page_boundary(indirect_addr, new_location),
                )
            }
            AddressingMode::Relative => {
                let offset: i8 = self.read_next_byte(ram) as i8;
                let pc: i32 = self.pc as i32;
                let new_location: u16 = (pc + offset as i32) as u16;
                (
                    new_location,
                    CPU::is_crossing_page_boundary(self.pc, new_location),
                )
            }
            AddressingMode::ZeroPage => {
                let addr: u8 = self.read_next_byte(ram);
                (self.read(ram, addr as u16) as u16, false)
            }
            AddressingMode::ZeroPageX => {
                let addr: u8 = self.read_next_byte(ram);
                (self.read(ram, (addr + self.x) as u16) as u16, false)
            }
            AddressingMode::ZeroPageY => {
                let addr: u8 = self.read_next_byte(ram);
                (self.read(ram, (addr + self.y) as u16) as u16, false)
            }
        }
    }

    fn execute_next_instruction(&mut self, ram: &mut RAM) -> u64 {
        let opcode: u8 = self.read_next_byte(ram);
        match opcode {
            0x00 => {
                println!("BRK");
                std::process::exit(0);
            }
            0x10 => self.bpl(ram),
            0x24 => {
                self.bit(ram, &AddressingMode::ZeroPage);
                3
            }
            0x2C => {
                self.bit(ram, &AddressingMode::Absolute);
                4
            }
            0x4C => {
                self.jmp(ram, &AddressingMode::Absolute);
                3
            }
            0x30 => self.bmi(ram),
            0x6C => {
                self.jmp(ram, &AddressingMode::Indirect);
                5
            }
            0x78 => {
                println!("SEI");
                self.p.set_bit(StatusFlag::InterruptDisable as u8, true);
                2
            }
            0x81 => {
                self.sta(ram, &AddressingMode::IndexedIndirect);
                6
            }
            0x84 => {
                self.sty(ram, &AddressingMode::ZeroPage);
                3
            }
            0x85 => {
                self.sta(ram, &AddressingMode::ZeroPage);
                3
            }
            0x86 => {
                self.stx(ram, &AddressingMode::ZeroPage);
                3
            }
            0x88 => Self::decrement_register("DEY", &mut self.p, &mut self.y),
            0x8A => Self::transfer_accumulator_to("TXA", &mut self.p, self.a, &mut self.x),
            0x8C => {
                self.sty(ram, &AddressingMode::Absolute);
                4
            }
            0x8D => {
                self.sta(ram, &AddressingMode::Absolute);
                4
            }
            0x8E => {
                self.stx(ram, &AddressingMode::Absolute);
                4
            }
            0x90 => self.bcc(ram),
            0x94 => {
                self.sty(ram, &AddressingMode::ZeroPageX);
                4
            }
            0x91 => {
                self.sta(ram, &AddressingMode::IndirectIndexed);
                6
            }
            0x95 => {
                self.sta(ram, &AddressingMode::ZeroPageX);
                4
            }
            0x96 => {
                self.stx(ram, &AddressingMode::ZeroPageY);
                4
            }
            0x98 => Self::transfer_accumulator_to("TYA", &mut self.p, self.a, &mut self.y),
            0x99 => {
                self.sta(ram, &AddressingMode::AbsoluteY);
                5
            }
            0x9A => Self::transfer_accumulator_to("TXS", &mut self.p, self.a, &mut self.s),
            0x9D => {
                self.sta(ram, &AddressingMode::AbsoluteX);
                5
            }
            0xA0 => self.ldy(ram, &AddressingMode::Immediate),
            0xA1 => self.lda(ram, &AddressingMode::IndexedIndirect),
            0xA2 => self.ldx(ram, &AddressingMode::Immediate),
            0xA4 => self.ldy(ram, &AddressingMode::ZeroPage),
            0xA5 => self.lda(ram, &AddressingMode::ZeroPage),
            0xA6 => self.ldx(ram, &AddressingMode::ZeroPage),
            0xA8 => Self::transfer_accumulator_to("TAY", &mut self.p, self.a, &mut self.y),
            0xA9 => self.lda(ram, &AddressingMode::Immediate),
            0xAA => Self::transfer_accumulator_to("TAX", &mut self.p, self.a, &mut self.x),
            0xAC => self.ldy(ram, &AddressingMode::Absolute),
            0xAD => self.lda(ram, &AddressingMode::Absolute),
            0xAE => self.ldx(ram, &AddressingMode::Absolute),
            0xB0 => self.bcs(ram),
            0xB1 => self.lda(ram, &AddressingMode::IndirectIndexed),
            0xB4 => self.ldy(ram, &AddressingMode::ZeroPageX),
            0xB5 => self.lda(ram, &AddressingMode::ZeroPageX),
            0xB9 => self.lda(ram, &AddressingMode::AbsoluteY),
            0xBA => Self::transfer_accumulator_to("TSX", &mut self.p, self.a, &mut self.x),
            0xBC => self.ldy(ram, &AddressingMode::AbsoluteX),
            0xBD => self.lda(ram, &AddressingMode::AbsoluteX),
            0xBE => self.ldx(ram, &AddressingMode::AbsoluteY),
            0xB6 => self.ldx(ram, &AddressingMode::ZeroPageY),
            0xC6 => {
                self.dec(ram, &AddressingMode::ZeroPage);
                5
            }
            0xC8 => Self::increment_register("INY", &mut self.p, &mut self.y),
            0xCA => Self::decrement_register("DEX", &mut self.p, &mut self.x),
            0xCE => {
                self.dec(ram, &AddressingMode::Absolute);
                6
            }
            0xD0 => self.bne(ram),
            0xD6 => {
                self.dec(ram, &AddressingMode::ZeroPageX);
                6
            }
            0xD8 => {
                println!("CLD");
                self.p.set_bit(StatusFlag::DecimalMode as u8, false);
                2
            }
            0xDE => {
                self.dec(ram, &AddressingMode::AbsoluteX);
                7
            }
            0xE6 => {
                self.inc(ram, &AddressingMode::ZeroPage);
                5
            }
            0xE8 => Self::increment_register("INX", &mut self.p, &mut self.x),
            0xEA => {
                println!("NOP");
                2
            }
            0xEE => {
                self.inc(ram, &AddressingMode::Absolute);
                6
            }
            0xF0 => self.beq(ram),
            0xF6 => {
                self.inc(ram, &AddressingMode::ZeroPageX);
                6
            }
            0xFE => {
                self.inc(ram, &AddressingMode::AbsoluteX);
                7
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
