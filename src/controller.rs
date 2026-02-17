#[derive(Debug)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

pub struct Controller {}

// Using information from https://github.com/user154lt/LEDDMX-00/blob/main/Dmx00Data.kt
impl Controller {
    pub fn power(on: bool) -> [u8; 9] {
        [
            0x7B,
            0xFF,
            0x04,
            if on { 0x03 } else { 0x02 },
            0xFF,
            0xFF,
            0xFF,
            0xFF,
            0xBF,
        ]
    }

    pub fn color(c: Color) -> [u8; 9] {
        [0x7B, 0xFF, 0x07, c.r, c.g, c.b, 0x00, 0xFF, 0xBF]
    }

    pub fn pattern(index: u8) -> [u8; 9] {
        let index = index.clamp(0, 210);
        [0x7B, 0xFF, 0x03, index, 0xFF, 0xFF, 0xFF, 0xFF, 0xBF]
    }

    pub fn mic(sensitivity: u8) -> [u8; 9] {
        [0x7B, 0xFF, 0x0B, sensitivity, 0x00, 0xFF, 0xFF, 0xBF, 0x00]
    }
}
