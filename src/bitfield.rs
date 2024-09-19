#[derive(Debug)]
pub struct Bitfield {
    value: u8,
}

impl Bitfield {
    pub fn new(value: u8) -> Bitfield {
        Bitfield { value }
    }

    pub fn get_bit(&self, bit: u8) -> bool {
        (self.value & (1 << bit)) != 0
    }

    pub fn set_bit(&mut self, bit: u8, value: bool) {
        if value {
            self.value |= 1 << bit;
        } else {
            self.value &= !(1 << bit);
        }
    }
}
