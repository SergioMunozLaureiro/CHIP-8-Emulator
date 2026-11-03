mod memory;
mod cpu;

use memory::Memory;
use cpu::CPU;
use std::thread;
use std::time::{Duration, Instant};

fn main() {
    let mut memory = Memory::new();
    let mut cpu = CPU::new();

    // Cargamos la ROM desde tu nueva carpeta
    let rom_path = "c8games/pong.ch8";
    match memory.load_rom(rom_path) {
        Ok(_) => println!("¡ROM '{}' cargada con éxito!", rom_path),
        Err(e) => {
            println!("Aviso: No se pudo cargar '{}' ({}).", rom_path, e);
            println!("El emulador arrancará vacío.");
        }
    }

    println!("¡Emulador Chip-8 encendido! Pulsa Ctrl+C para salir.");

    let mut ultimo_tick_temporizador = Instant::now();
    let intervalo_temporizador = Duration::from_millis(16); // 60Hz

    loop {
        // 1. La CPU ejecuta una instrucción (va rápido)
        cpu.tick(&mut memory);

        // 2. Comprobamos el cronómetro de 60Hz
        if ultimo_tick_temporizador.elapsed() >= intervalo_temporizador {
            cpu.tick_timers(); // Actualiza los temporizadores
            ultimo_tick_temporizador = Instant::now(); // Resetea el cronómetro
        }

        // 3. Pausa mínima para que el bucle no sature el 100% de tu ordenador moderno
        thread::sleep(Duration::from_millis(2));
    }
}