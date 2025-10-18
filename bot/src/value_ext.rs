use crate::line_follower_robot::devices::DeviceValue;

pub trait DeviceValueExt {
    fn get_bool(&self, index: usize) -> bool;
    fn get_u8(&self, index: usize) -> u8;
    fn get_u16(&self, index: usize) -> u16;
    fn get_u32(&self, index: usize) -> u32;
    fn get_i8(&self, index: usize) -> i8;
    fn get_i16(&self, index: usize) -> i16;
    fn get_i32(&self, index: usize) -> i32;
}

impl DeviceValueExt for DeviceValue {
    fn get_bool(&self, index: usize) -> bool {
        self.get_u8(index) != 0
    }

    fn get_u8(&self, index: usize) -> u8 {
        match index {
            0 => self.v0,
            1 => self.v1,
            2 => self.v2,
            3 => self.v3,
            4 => self.v4,
            5 => self.v5,
            6 => self.v6,
            7 => self.v7,
            _ => 0,
        }
    }

    fn get_u16(&self, index: usize) -> u16 {
        match index {
            0 => u16::from_le_bytes([self.v0, self.v1]),
            1 => u16::from_le_bytes([self.v2, self.v3]),
            2 => u16::from_le_bytes([self.v4, self.v5]),
            3 => u16::from_le_bytes([self.v6, self.v7]),
            _ => 0,
        }
    }

    fn get_u32(&self, index: usize) -> u32 {
        match index {
            0 => u32::from_le_bytes([self.v0, self.v1, self.v2, self.v3]),
            1 => u32::from_le_bytes([self.v4, self.v5, self.v6, self.v7]),
            _ => 0,
        }
    }

    fn get_i8(&self, index: usize) -> i8 {
        self.get_u8(index) as i8
    }

    fn get_i16(&self, index: usize) -> i16 {
        self.get_u16(index) as i16
    }

    fn get_i32(&self, index: usize) -> i32 {
        self.get_u32(index) as i32
    }
}
