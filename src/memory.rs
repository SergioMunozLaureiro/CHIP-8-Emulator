
use std::fs::File;
use std::io::Read;
const FONTS: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0,
    0x20, 0x60, 0x20, 0x20, 0x70,
    0xF0, 0x10, 0xF0, 0x80, 0xF0,
    0xF0, 0x10, 0xF0, 0x10, 0xF0,
    0x90, 0x90, 0xF0, 0x10, 0x10,
    0xF0, 0x80, 0xF0, 0x10, 0xF0,
    0xF0, 0x80, 0xF0, 0x90, 0xF0,
    0xF0, 0x10, 0x20, 0x40, 0x40,
    0xF0, 0x90, 0xF0, 0x90, 0xF0,
    0xF0, 0x90, 0xF0, 0x10, 0xF0,
    0xF0, 0x90, 0xF0, 0x90, 0x90,
    0xE0, 0x90, 0xE0, 0x90, 0xE0,
    0xF0, 0x80, 0x80, 0x80, 0xF0,
    0xE0, 0x90, 0x90, 0x90, 0xE0,
    0xF0, 0x80, 0xF0, 0x80, 0xF0,
    0xF0, 0x80, 0xF0, 0x80, 0x80,
];

pub struct Memory {
    ram: [u8; 4096],
}

impl Memory {
    pub fn new() -> Self {
        let mut memory = Self {
            ram: [0; 4096],
        };

        for i in 0..80 {
            memory.ram[i] = FONTS[i];
        }

        memory
    }

    pub fn read(&self, adress: u16) -> u8 {
        self.ram[adress as usize]
    }

    pub fn write(&mut self, adress: u16, value: u8) {
        self.ram[adress as usize] = value;
    }

    pub fn load_rom(&mut self, filename: &str) -> std::io::Result<()> { 
    let mut file = File::open(filename)?;
    let mut buffer = Vec::new();

    file.read_to_end(&mut buffer)?;

    for (i, &byte) in buffer.iter().enumerate() {
        if 512 + i < 4096 {
            self.ram[512 + i] = byte;
        }
    }

    Ok(())
}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_las_fuentes_se_cargan_bien() {
        let memoria = Memory::new();
        assert_eq!(memoria.read(0), 0xF0);
        assert_eq!(memoria.read(80), 0);
    }

    #[test]
    fn test_escribir_y_leer_en_ram() {
        let mut memoria = Memory::new();
        memoria.write(512, 150);
        assert_eq!(memoria.read(512), 150);
        assert_eq!(memoria.read(513), 0);
    }
}