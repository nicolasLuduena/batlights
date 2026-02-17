#[derive(Debug)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

pub trait Controller {
    fn set_power(&mut self, on: bool) -> Result<(), String>;
    fn set_color(&mut self, color: Color) -> Result<(), String>;
    fn set_pattern(&mut self, index: u8) -> Result<(), String>;
    fn set_mic(&mut self, sensitivity: u8) -> Result<(), String>;
}

pub struct MockController {}

impl MockController {
    pub fn new() -> MockController {
        MockController {}
    }
}

impl Controller for MockController {
    fn set_power(&mut self, on: bool) -> Result<(), String> {
        println!("Setting power: {on}");
        Ok(())
    }

    fn set_color(&mut self, c: Color) -> Result<(), String> {
        println!("Setting color to #{:02X}{:02X}{:02X}", c.r, c.g, c.b);
        Ok(())
    }

    fn set_pattern(&mut self, index: u8) -> Result<(), String> {
        println!("Setting pattern to {index:02X}");
        Ok(())
    }

    fn set_mic(&mut self, sensitivity: u8) -> Result<(), String> {
        println!("Setting mic sensitivity to {sensitivity:02X}");
        Ok(())
    }
}
