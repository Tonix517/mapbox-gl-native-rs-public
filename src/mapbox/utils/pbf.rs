use std::mem::transmute;

// TODO: due to arbitrary type-casting in this file, overflow may happen with very large values.

pub struct Pbf {
    pub data: Vec<u8>,
    pub inx: usize,
    pub value: u32,
    pub tag: u32,
}

impl Pbf {
    pub fn new(data: Vec<u8>) -> Self {
        Pbf {
            data,
            inx: 0,
            value: 0,
            tag: 0,
        }
    }

    fn get_data_len(&self) -> usize {
        self.data.len()
    }

    fn get_current_byte(&self) -> u8 {
        self.data[self.inx]
    }

    pub fn has_next(&self) -> bool {
        self.inx < self.get_data_len()
    }

    pub fn skip_bytes(&mut self, bytes: u32) {
        if self.inx + bytes as usize > self.get_data_len() {
            panic!("skipBytes beyond the end of the data!");
        }
        self.inx += bytes as usize;
    }

    pub fn boolean(&mut self) -> bool {
        let ret = self.get_current_byte() != 0;
        self.skip_bytes(1);
        ret
    }

    pub fn varint32(&mut self) -> u32 {
        let mut byte = 0x80;
        let mut result: u32 = 0;
        let mut bitpos: u32 = 0;

        while (bitpos < 70) && ((byte & 0x80) != 0) {
            if !self.has_next() {
                panic!("unterminated varint exception");
            }
            byte = self.get_current_byte();
            result |= ((byte & 0x7F) as u32) << bitpos;

            self.inx += 1;
            bitpos += 7;
        }

        if bitpos == 70 && (byte & 0x80) != 0 {
            panic!("throw varint_too_long_exception");
        }

        result
    }

    // TODO: to find some Trait can perfectly represent u32 and u64
    pub fn varint64(&mut self) -> u64 {
        let mut byte = 0x80;
        let mut result: u64 = 0;
        let mut bitpos: u32 = 0;

        while (bitpos < 70) && ((byte & 0x80) != 0) {
            if !self.has_next() {
                panic!("unterminated varint exception");
            }
            byte = self.get_current_byte();
            result |= ((byte & 0x7F) as u64) << bitpos;

            self.inx += 1;
            bitpos += 7;
        }

        if bitpos == 70 && (byte & 0x80) != 0 {
            panic!("throw varint_too_long_exception");
        }

        result
    }

    pub fn svarint32(&mut self) -> i32 {
        let n = self.varint32();
        let tmp = -((n & 1) as i32);
        let tmp = unsafe { transmute::<i32, u32>(tmp) };
        unsafe { transmute::<u32, i32>((n >> 1) ^ tmp) }
    }

    pub fn svarint64(&mut self) -> i64 {
        let n = self.varint64();
        let tmp = -((n & 1) as i64);
        let tmp = unsafe { transmute::<i64, u64>(tmp) };
        unsafe { transmute::<u64, i64>((n >> 1) ^ tmp) }
    }

    pub fn next(&mut self) -> bool {
        if self.has_next() {
            self.value = self.varint32();
            self.tag = self.value >> 3;
            true
        } else {
            false
        }
    }

    /*
    pub fn next_with_tag(&mut self, requested_tag: u32) -> bool {
        while self.next() {
            if self.tag == requested_tag {
                return true;
            } else {
                self.skip();
            }
        }
        false
    }
    */

    pub fn skip(&mut self) {
        self.skip_value(self.value);
    }

    pub fn skip_value(&mut self, val: u32) {
        match val & 0x7 {
            0 => {
                self.varint32();
            }
            1 => {
                self.skip_bytes(8);
            }
            2 => {
                let vint = self.varint32();
                self.skip_bytes(vint);
            }
            5 => {
                self.skip_bytes(4);
            }
            _ => {
                panic!("unknown field type exception");
            }
        }
    }

    pub fn fixed32(&mut self) -> f32 {
        let mut result: u32 = 0;

        for i in 0..4 {
            result |= (self.get_current_byte() as u32) << (32 - (i + 1) * 8);
            self.skip_bytes(1);
        }

        unsafe { transmute::<u32, f32>(result) }
    }

    pub fn fixed64(&mut self) -> f64 {
        let mut result: u64 = 0;

        for i in 0..8 {
            result |= (self.get_current_byte() as u64) << (64 - (i + 1) * 8);
            self.skip_bytes(1);
        }

        unsafe { transmute::<u64, f64>(result) }
    }

    pub fn string(&mut self) -> String {
        let bytes = self.varint32() as usize;
        let chunk = (&self.data[self.inx..self.inx + bytes]).to_vec();

        self.skip_bytes(bytes as u32);
        unsafe { String::from_utf8_unchecked(chunk) }
    }

    pub fn message(&mut self) -> Self {
        let bytes = self.varint32() as usize;
        let chunk = (&self.data[self.inx..self.inx + bytes]).to_vec();

        self.skip_bytes(bytes as u32);
        Pbf::new(chunk)
    }
}
