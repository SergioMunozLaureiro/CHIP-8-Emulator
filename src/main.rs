mod memory;
mod cpu;
mod display;

use memory::Memory;
use cpu::CPU;
use display::Display;

use rodio::{OutputStream, Sink, source::SineWave, Source};
use std::env;

fn main() {
    // 1. Capturar la ruta de la ROM desde los argumentos de la consola (o usar Pong por defecto)
    let args: Vec<String> = env::args().collect();
    let rom_path = if args.len() > 1 {
        &args[1]
    } else {
        "c8games/pong.ch8"
    };

    // 2. Inicializar los componentes principales
    let mut memory = Memory::new();
    let mut cpu = CPU::new();
    let mut display = Display::new();

    // 3. Inicializar el sistema de audio con rodio
    let (_stream, stream_handle) = OutputStream::try_default().unwrap_or_else(|e| {
        panic!("No se pudo detectar un dispositivo de salida de audio: {}", e);
    });
    let sink = Sink::try_new(&stream_handle).unwrap();

    // Generamos un tono senoidal continuo de 440 Hz (Nota La / Beep estándar)
    let tone = SineWave::new(440.0).repeat_infinite();
    sink.append(tone);
    sink.pause(); // Arrancamos en silencio

    // 4. Cargar la ROM en memoria
    match memory.load_rom(rom_path) {
        Ok(_) => println!("¡ROM '{}' cargada con éxito!", rom_path),
        Err(e) => {
            println!("Aviso: No se pudo cargar '{}' ({}).", rom_path, e);
            println!("El emulador arrancará vacío.");
        }
    }

    println!("¡Emulador Chip-8 encendido! Cierra la ventana o pulsa ESC para salir.");

    // 5. Bucle gráfico y de audio principal
    while display.is_open() {
        let keypad = display.get_keypad();

        for _ in 0..10 {
            cpu.tick(&mut memory, &mut display, &keypad);
        }

        cpu.tick_timers();

        // Control del zumbador según el temporizador de la CPU
        if cpu.sound_timer > 0 {
            sink.play();
        } else {
            sink.pause();
        }

        display.draw();
    }
}