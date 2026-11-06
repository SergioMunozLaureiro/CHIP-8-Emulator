use crate::memory::Memory;
use crate::display::{Display, WIDTH, HEIGHT};
pub struct CPU {
    pub v: [u8; 16],
    pub i: u16,
    pub pc: u16,
    pub sp: u8,
    pub stack: [u16; 16],
    pub delay_timer: u8,
    pub sound_timer: u8,
}

impl CPU {
    pub fn new() -> Self {
        Self {
            v: [0; 16],
            i: 0,
            pc: 0x200,
            sp: 0,
            stack: [0; 16],
            delay_timer: 0,
            sound_timer: 0,
      
        }

    }
    pub fn reset(&mut self) {
        self.v = [0; 16];
        self.i = 0;
        self.pc = 0x200;
        self.sp = 0;
        self.stack = [0; 16];
        self.delay_timer = 0;
        self.sound_timer = 0;
    }
    fn push(&mut self, val: u16) {
                    self.stack[self.sp as usize] = val;
                    self.sp += 1;
                }
    fn pop(&mut self) -> u16 {
                    self.sp -= 1;
                    self.stack[self.sp as usize]
                }
    pub fn fetch(&mut self, memory: &Memory) -> u16 {
        let high_byte = memory.read(self.pc) as u16;//EL CHIP-8 TIENE INSTRUCCIONES DE 2 BYTES, POR ESO SE LEEN 2 BYTES Y SE COMBINAN EN UN SOLO OPCODE
        let low_byte = memory.read(self.pc + 1) as u16;
        let opcode = (high_byte << 8) | low_byte;
        self.pc += 2;
        opcode
    }
    // Esta función ejecutará un ciclo completo de la CPU
pub fn tick(&mut self, memory: &mut crate::memory::Memory,display: &mut Display,keypad: &[bool; 16]) {
    if self.pc as usize >= 4094 {
        return;
    }
    // 1. FETCH: Leemos la instrucción de la memoria (pasamos la memoria prestada)
    let opcode = self.fetch(memory);

    // 2. DECODE: Diseccionamos el opcode en sus posibles partes
    let c   = (opcode & 0xF000) >> 12; // Categoría (1º nibble)
    let x   = ((opcode & 0x0F00) >> 8) as usize; // Registro X (2º nibble). Lo convertimos a usize para usarlo como índice de array v[x]
    let y   = ((opcode & 0x00F0) >> 4) as usize; // Registro Y (3º nibble)
    let d   = (opcode & 0x000F) as u8; // Último dígito (4º nibble)
    
    // Algunas instrucciones en lugar de registros usan números directos más grandes:
    let nn  = (opcode & 0x00FF) as u8; // Últimos 2 dígitos (8 bits)
    let nnn = opcode & 0x0FFF;         // Últimos 3 dígitos (12 bits, normalmente una dirección de memoria)

    // 3. EXECUTE: Buscamos qué instrucción coincide con nuestros recortes
    match (c, x, y, d) {
        (0, 0, 0, 0) => {},
        // 00E0: Limpiar la pantalla
        (0, 0, 0xE, 0) => {
            display.clear();
        }

        // 00EE: Return. Volver de una subrutina.
        (0, 0, 0xE, 0xE) => {
            self.sp -= 1;
            self.pc = self.stack[self.sp as usize];
        }

        // 1NNN: Jump. Saltar a la dirección NNN.
        (1, _, _, _) => self.pc = nnn,

        // 2NNN: Call. Llamar a una subrutina en NNN.
        (2, _, _, _) => {
            self.stack[self.sp as usize] = self.pc;
            self.sp += 1;
            self.pc = nnn;
        }

        // 3XNN: Skip si Vx == NN
        (3, _, _, _) => if self.v[x] == nn { self.pc += 2; },

        // 4XNN: Skip si Vx != NN
        (4, _, _, _) => if self.v[x] != nn { self.pc += 2; },

        // 5XY0: Skip si Vx == Vy
        (5, _, _, 0) => if self.v[x] == self.v[y] { self.pc += 2; },

        // 6XNN: Set. Vx = NN
        (6, _, _, _) => self.v[x] = nn,

        // 7XNN: Add. Vx += NN (sin afectar al flag de desbordamiento)
        (7, _, _, _) => self.v[x] = self.v[x].wrapping_add(nn),

        // 8XY_: Operaciones Aritméticas y Lógicas (ALU)
        (8, _, _, _) => match d {
            0 => self.v[x] = self.v[y],
            1 => self.v[x] |= self.v[y], // OR binario
            2 => self.v[x] &= self.v[y], // AND binario
            3 => self.v[x] ^= self.v[y], // XOR binario
            
            4 => { // SUMA con detección de desbordamiento (Carry)
                let (res, overflow) = self.v[x].overflowing_add(self.v[y]);
                self.v[x] = res;
                self.v[0xF] = if overflow { 1 } else { 0 };
            },
            
            5 => { // RESTA (Vx - Vy) con flag de no-préstamo (Borrow)
                let (res, overflow) = self.v[x].overflowing_sub(self.v[y]);
                self.v[x] = res;
                self.v[0xF] = if overflow { 0 } else { 1 };
            },
            
            6 => { // SHIFT RIGHT. Desplaza bits a la derecha. VF guarda el bit perdido.
                let bit_perdido = self.v[x] & 1;
                self.v[x] >>= 1;
                self.v[0xF] = bit_perdido;
            },
            
            7 => { // RESTA INVERSA (Vy - Vx)
                let (res, overflow) = self.v[y].overflowing_sub(self.v[x]);
                self.v[x] = res;
                self.v[0xF] = if overflow { 0 } else { 1 };
            },
            
            0xE => { // SHIFT LEFT. Desplaza bits a la izquierda. VF guarda el bit perdido.
                let bit_perdido = (self.v[x] >> 7) & 1;
                self.v[x] <<= 1;
                self.v[0xF] = bit_perdido;
            },
            
            _ => println!("ALU Opcode desconocido: {:04X}", opcode),
        },

        // 9XY0: Skip si Vx != Vy
        (9, _, _, 0) => if self.v[x] != self.v[y] { self.pc += 2; },

        // ANNN: Set I. Registro I = NNN
        (0xA, _, _, _) => self.i = nnn,

        // BNNN: Jump offset. PC = V0 + NNN
        (0xB, _, _, _) => self.pc = (self.v[0] as u16) + nnn,

        // CXNN: Random. Vx = Aleatorio AND NN
        (0xC, _, _, _) => {
            // TODO: Generar número aleatorio
            // let num_aleatorio = rand::random::<u8>();
            // self.v[x] = num_aleatorio & nn;
        }

        // DXYN: Draw. Dibuja un sprite en pantalla
        (0xD, _, _, _) => {
            let x_pos = (self.v[x] as usize) % WIDTH;
            let y_pos = (self.v[y] as usize) % HEIGHT;
            let height = d as usize;

            // 1. Inicialmente no hay colisión (bandera VF = 0)
            self.v[0xF] = 0;

            // 2. Iteramos fila a fila (N bytes de alto)
            for row in 0..height {
                // Leemos el byte del sprite desde la memoria RAM en la dirección I + row
                let sprite_byte = memory.read(self.i + row as u16);

                // 3. Un byte contiene 8 píxeles (bits) horizontales
                for col in 0..8 {
                    // Extraemos el bit de la posición 'col' (de izquierda a derecha)
                    let sprite_pixel = (sprite_byte >> (7 - col)) & 1;

                    // Solo nos importan los bits encendidos (1)
                    if sprite_pixel == 1 {
                        let px = (x_pos + col) % WIDTH;
                        let py = (y_pos + row) % HEIGHT;
                        let index = py * WIDTH + px;

                        // Si el píxel en pantalla ya estaba encendido (true), hay colisión
                        if display.vram[index] {
                            self.v[0xF] = 1;
                        }

                        // Operación XOR: Invertimos el estado del píxel
                        display.vram[index] ^= true;
                    }
                }
            }
        }

        // EX__: Operaciones de Teclado
        (0xE, _, 9, 0xE) => {
            let key = self.v[x] as usize;
            if key < 16 && keypad[key] {
                self.pc += 2;
            }
        },

        // EXA1: Skip si la tecla con el valor guardado en Vx NO está pulsada
        (0xE, _, 0xA, 1) => {
            let key = self.v[x] as usize;
            if key < 16 && !keypad[key] {
                self.pc += 2;
            }
        },

        // FX__: Operaciones de Memoria, Timers y Varios
        (0xF, _, _, _) => match (y, d) {
            
            // FX07: Vx = Delay Timer
            (0, 7) => self.v[x] = self.delay_timer,
            
            // FX0A: Esperar a que se pulse una tecla (bloquea la ejecución)
            // FX0A: Esperar a que se pulse una tecla (Bloquea la CPU hasta recibir input)
            (0, 0xA) => {
                let mut pressed = false;
                for i in 0..16 {
                    if keypad[i] {
                        self.v[x] = i as u8;
                        pressed = true;
                        break;
                    }
                }
                // Si ninguna tecla fue pulsada, rebobinamos el PC 2 bytes para repetirlo en el siguiente ciclo
                if !pressed {
                    self.pc -= 2;
                }
            },
            
            
            // FX15: Delay Timer = Vx
            (1, 5) => self.delay_timer = self.v[x],
            
            // FX18: Sound Timer = Vx
            (1, 8) => self.sound_timer = self.v[x],
            
            // FX1E: I += Vx
            (1, 0xE) => self.i = self.i.wrapping_add(self.v[x] as u16),
            
            // FX29: I = Dirección de memoria del sprite del carácter en Vx
            (2, 9) => {
                // Las fuentes empiezan en la posición 0 de la memoria, y cada una ocupa 5 bytes
                self.i = (self.v[x] as u16) * 5; 
            },
            
            // FX33: Guardar representación BCD (Decimal Codificado en Binario) de Vx en memoria
            (3, 3) => {
                let valor = self.v[x];
                memory.write(self.i, valor / 100);             // Centenas
                memory.write(self.i + 1, (valor / 10) % 10);   // Decenas
                memory.write(self.i + 2, valor % 10);          // Unidades
            },
            
            // FX55: Volcar registros V0 a Vx en la memoria a partir de I
            (5, 5) => {
                for idx in 0..=x {
                    memory.write(self.i + (idx as u16), self.v[idx]);
                }
            },
            
            // FX65: Cargar registros V0 a Vx desde la memoria a partir de I
            (6, 5) => {
                for idx in 0..=x {
                    self.v[idx] = memory.read(self.i + (idx as u16));
                }
            },
            
            _ => println!("FX Opcode desconocido: {:04X}", opcode),
        },

        _ => println!("Instrucción totalmente desconocida: {:04X}", opcode),
    }
}
pub fn tick_timers(&mut self) {
    if self.delay_timer > 0 {
        self.delay_timer -= 1;
    }
    
    if self.sound_timer > 0 {
        if self.sound_timer == 1 {
            // Aquí en el futuro le diremos al ordenador que emita un pitido
            // println!("¡BEEP!");
        }
        self.sound_timer -= 1;
    }
}
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_cpu_initialization(){
        let cpu=CPU::new();
        assert_eq!(cpu.pc,0x200);
        assert_eq!(cpu.sp,0);
        assert_eq!(cpu.i,0);
        assert_eq!(cpu.v,[0;16]);
        assert_eq!(cpu.stack,[0;16]);
        assert_eq!(cpu.delay_timer,0);
        assert_eq!(cpu.sound_timer,0);
    }
    #[test]
fn test_fetch_instruccion() {
    let mut cpu = CPU::new();
    
    // Necesitamos la memoria para que la CPU tenga de dónde leer
    let mut memoria = crate::memory::Memory::new();

    // Simulamos que hay un juego cargado escribiendo 2 bytes manualmente en la posición 512 (0x200)
    // Vamos a meter la instrucción 0xA2F0 fragmentada en dos
    memoria.write(0x200, 0xA2); 
    memoria.write(0x201, 0xF0); 

    // ¡La CPU hace su magia!
    let opcode = cpu.fetch(&memoria);

    // 1. Comprobamos que la CPU pegó los trozos correctamente
    assert_eq!(opcode, 0xA2F0);

    // 2. Comprobamos que el Program Counter avanzó de 0x200 a 0x202
    assert_eq!(cpu.pc, 0x202);
}
}
