use minifb::{Scale, Window, WindowOptions};
use minifb::Key;
// La resolución oficial del Chip-8
pub const WIDTH: usize = 64;
pub const HEIGHT: usize = 32;

pub struct Display {
    window: Window,
    // VRAM: Nuestra matriz interna de píxeles (true = blanco, false = negro)
    pub vram: [bool; WIDTH * HEIGHT],
}

impl Display {
    pub fn new() -> Self {
        // Creamos la ventana con minifb
        let mut window = Window::new(
            "Emulador Chip-8",
            WIDTH,
            HEIGHT,
            WindowOptions {
                scale: Scale::X16, // Escalamos x16 porque 64x32 en un monitor moderno sería minúsculo
                ..WindowOptions::default()
            },
        ).unwrap_or_else(|e| {
            panic!("No se pudo abrir la ventana gráfica: {}", e);
        });

        // Limitamos la tasa de refresco visual a ~60 FPS
        window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));

        Self {
            window,
            vram: [false; WIDTH * HEIGHT],
        }
    }

    // Método para renderizar nuestra matriz bool en la ventana gráfica
    pub fn draw(&mut self) {
        // minifb no entiende de 'trues' y 'falses', necesita colores hex (0xRRGGBB)
        let mut color_buffer = [0u32; WIDTH * HEIGHT];

        for (i, &pixel) in self.vram.iter().enumerate() {
            // Si el píxel está encendido, lo pintamos blanco. Si no, negro.
            color_buffer[i] = if pixel { 0xFFFFFF } else { 0x000000 };
        }

        // Volcamos los colores en la ventana
        self.window.update_with_buffer(&color_buffer, WIDTH, HEIGHT).unwrap();
    }

    // Comprueba si el usuario ha cerrado la ventana o pulsado Escape
    pub fn is_open(&self) -> bool {
        self.window.is_open() && !self.window.is_key_down(minifb::Key::Escape)
    }
    
    // Método rápido para borrar toda la pantalla
    pub fn clear(&mut self) {
        self.vram = [false; WIDTH * HEIGHT];
    }
    pub fn get_keypad(&self) -> [bool; 16] {
        let mut keys = [false; 16];

        keys[0x1] = self.window.is_key_down(Key::Key1);
        keys[0x2] = self.window.is_key_down(Key::Key2);
        keys[0x3] = self.window.is_key_down(Key::Key3);
        keys[0xC] = self.window.is_key_down(Key::Key4);

        keys[0x4] = self.window.is_key_down(Key::Q);
        keys[0x5] = self.window.is_key_down(Key::W);
        keys[0x6] = self.window.is_key_down(Key::E);
        keys[0xD] = self.window.is_key_down(Key::R);

        keys[0x7] = self.window.is_key_down(Key::A);
        keys[0x8] = self.window.is_key_down(Key::S);
        keys[0x9] = self.window.is_key_down(Key::D);
        keys[0xE] = self.window.is_key_down(Key::F);

        keys[0xA] = self.window.is_key_down(Key::Z);
        keys[0x0] = self.window.is_key_down(Key::X);
        keys[0xB] = self.window.is_key_down(Key::C);
        keys[0xF] = self.window.is_key_down(Key::V);

        keys
    }
}