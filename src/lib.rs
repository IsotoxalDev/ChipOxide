use log::info;
use std::{io::Error, thread::sleep, time::Duration};

pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 32;
pub const KEYBOARD_SIZE: usize = 16;

const MEM_SIZE: usize = 4096;
const REGISTER_SIZE: usize = 16;
const COUNTER_START: usize = 0x200;
const INSTRUCTION_SIZE: usize = 2;
const FONT_SIZE: u16 = 5;
const VF: usize = 0xF;

const FONT_DATA: [[u8; 5]; 16] = [
    [0xF0, 0x90, 0x90, 0x90, 0xF0], // 0
    [0x20, 0x60, 0x20, 0x20, 0x70], // 1
    [0xF0, 0x10, 0xF0, 0x80, 0xF0], // 2
    [0xF0, 0x10, 0xF0, 0x10, 0xF0], // 3
    [0x90, 0x90, 0xF0, 0x10, 0x10], // 4
    [0xF0, 0x80, 0xF0, 0x10, 0xF0], // 5
    [0xF0, 0x80, 0xF0, 0x90, 0xF0], // 6
    [0xF0, 0x10, 0x20, 0x40, 0x40], // 7
    [0xF0, 0x90, 0xF0, 0x90, 0xF0], // 8
    [0xF0, 0x90, 0xF0, 0x10, 0xF0], // 9
    [0xF0, 0x90, 0xF0, 0x90, 0x90], // A
    [0xE0, 0x90, 0xE0, 0x90, 0xE0], // B
    [0xF0, 0x80, 0x80, 0x80, 0xF0], // C
    [0xE0, 0x90, 0x90, 0x90, 0xE0], // D
    [0xF0, 0x80, 0xF0, 0x80, 0xF0], // E
    [0xF0, 0x80, 0xF0, 0x80, 0x80], // F
];

mod config;
mod instruction;
mod io;
mod opcodes;

pub use config::ChipConfig;
pub use io::ChipIO;

use instruction::Instruction;

/// The ChipOxide Struct
pub struct ChipOxide<'a, I: ChipIO> {
    memory: [u8; MEM_SIZE],
    screen: [[bool; SCREEN_HEIGHT]; SCREEN_WIDTH],
    stack: Vec<u16>,
    register: [u8; REGISTER_SIZE],
    timer: (u8, u8), // Delay Timer, Sound Timer
    keyboard: [bool; KEYBOARD_SIZE],
    counter: usize,
    index: u16,
    io: &'a mut I,
    config: &'a ChipConfig,
}

impl<'a, I> ChipOxide<'a, I>
where
    I: ChipIO,
{
    // Create an empty shell.
    fn empty(io: &'a mut I, config: &'a ChipConfig) -> Self {
        Self {
            memory: [0; MEM_SIZE],
            screen: [[false; SCREEN_HEIGHT]; SCREEN_WIDTH],
            stack: vec![],
            register: [0; REGISTER_SIZE],
            timer: (0, 0),
            keyboard: [false; KEYBOARD_SIZE],
            counter: 0,
            index: 0,
            io,
            config,
        }
    }

    // Load and put a program in loop.
    pub fn start(program: &[u8], io: &'a mut I, config: &'a ChipConfig) -> Result<(), Error> {
        let mut chip8 = Self::empty(io, config);

        for font in FONT_DATA {
            for byte in font {
                chip8.memory[chip8.counter] = byte;
                chip8.counter += 1;
            }
        }

        chip8.counter = COUNTER_START;
        for byte in program {
            chip8.memory[chip8.counter] = *byte;
            chip8.counter += 1;
        }
        chip8.counter = COUNTER_START;

        info!("Starting Chip Oxide");

        // Mailoop.
        loop {
            sleep(Duration::from_millis(
                ((1.0 / chip8.config.timer_hz as f64) * 1000.0) as u64,
            ));
            chip8.update_timer()?;
            for _ in 0..chip8.config.opcodes_per_cycle {
                if let Some((key, state)) = chip8.io.get_key()? {
                    chip8.keyboard[key] = state;
                }
                let inst = chip8.fetch_instruction()?;
                chip8.execute_instruction(inst)?;
            }
        }
    }

    // Update the delay timer and the sound timer.
    fn update_timer(&mut self) -> Result<(), Error> {
        if self.timer.0 != 0 {
            self.timer.0 -= 1;
        }
        if self.timer.1 != 0 {
            self.timer.1 -= 1;
            if self.timer.1 == 0 {
                self.io.end_beep()?
            }
        }
        Ok(())
    }

    // Fetch the instruction from memory.
    fn fetch_instruction(&mut self) -> Result<Instruction, Error> {
        self.counter += INSTRUCTION_SIZE;
        Instruction::try_from(
            (self.memory[self.counter - 1] as u16) | ((self.memory[self.counter - 2] as u16) << 8),
        )
    }
}
