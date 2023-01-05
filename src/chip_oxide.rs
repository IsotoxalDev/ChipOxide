pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 32;
pub const KEYBOARD_SIZE: usize = 16;

const MEM_SIZE: usize = 4096;
const REGISTER_SIZE: usize = 16;
const COUNTER_START: usize = 0x200;
const INSTRUCTION_SIZE: usize = 2;
const FONT_SIZE: u16 = 5;
const VF: usize = 0xF;

use std::io::{Error, ErrorKind};
use std::thread::sleep;
use std::time::Duration;

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

use log::info;

/// Trait for IO.
pub trait ChipIO {
    /// Update the screen
    fn update_screen(
        &mut self,
        screen: &[[bool; SCREEN_HEIGHT]; SCREEN_WIDTH],
    ) -> Result<(), Error>;

    /// Toggle Sound
    fn start_beep(&mut self) -> Result<(), Error>;
    fn end_beep(&mut self) -> Result<(), Error>;

    /// Get keyboard State
    fn get_key(&mut self) -> Result<Option<(usize, bool)>, Error>;
}

pub struct ChipConfig {
    opcodes_per_cycle: usize,
    timer_hz: u8,
    legacy: bool,
}

impl ChipConfig {
    pub fn default(legacy: bool) -> Self {
        Self {
            opcodes_per_cycle: 8,
            timer_hz: 60,
            legacy,
        }
    }
}

#[derive(Debug)]
enum Instruction {
    Clear,
    Return,
    Jump(u16),
    SubRoutine(u16),
    SkipED(u8, u8),       // Equal to Data
    SkipNED(u8, u8),      // Not Equal to Data
    SkipER(u8, u8),       // Equal to Register
    SetRegisterD(u8, u8), // From Data
    AddRegisterD(u8, u8), // From Data
    SetRegisterR(u8, u8), // From Register
    BinaryOR(u8, u8),
    BinaryAND(u8, u8),
    LogicalXOR(u8, u8),
    AddRegisterR(u8, u8), // From Data
    SubtractXY(u8, u8),   // X to Y
    ShiftRight(u8, u8),   // Y to X
    SubtractYX(u8, u8),
    ShiftLeft(u8, u8),
    SkipNER(u8, u8), // Not Equal to Register
    SetIndex(u16),
    OffsetJump(u8, u16),
    Random(u8, u8),
    Draw(u8, u8, u8),
    KeyPressed(u8),
    KeyReleased(u8),
    GetDelay(u8),
    KeyWait(u8),
    SetDelay(u8),
    SetSound(u8),
    AddIndex(u8),
    GetFont(u8),
    AsDecimal(u8),
    Save(u8),
    Load(u8),
}

/// Decode the instruction and take out usefull data
impl TryFrom<u16> for Instruction {
    type Error = Error;

    fn try_from(value: u16) -> Result<Self, Error> {
        let inst = ((value & 0b1111000000000000) >> 12) as u8;
        let r0 = ((value & 0b0000111100000000) >> 8) as u8;
        let r1 = ((value & 0b0000000011110000) >> 4) as u8;
        let n = (value & 0b0000000000001111) as u8;
        let nn = (value & 0b0000000011111111) as u8;
        let nnn = value & 0b0000111111111111;

        match (inst, r0, r1, n) {
            (0, 0, 0xE, 0) => Ok(Instruction::Clear),
            (0, 0, 0xE, 0xE) => Ok(Instruction::Return),
            (1, _, _, _) => Ok(Instruction::Jump(nnn)),
            (2, _, _, _) => Ok(Instruction::SubRoutine(nnn)),
            (3, _, _, _) => Ok(Instruction::SkipED(r0, nn)),
            (4, _, _, _) => Ok(Instruction::SkipNED(r0, nn)),
            (5, _, _, 0) => Ok(Instruction::SkipER(r0, r1)),
            (6, _, _, _) => Ok(Instruction::SetRegisterD(r0, nn)),
            (7, _, _, _) => Ok(Instruction::AddRegisterD(r0, nn)),
            (8, _, _, 0) => Ok(Instruction::SetRegisterR(r0, r1)),
            (8, _, _, 1) => Ok(Instruction::BinaryOR(r0, r1)),
            (8, _, _, 2) => Ok(Instruction::BinaryAND(r0, r1)),
            (8, _, _, 3) => Ok(Instruction::LogicalXOR(r0, r1)),
            (8, _, _, 4) => Ok(Instruction::AddRegisterR(r0, r1)),
            (8, _, _, 5) => Ok(Instruction::SubtractXY(r0, r1)),
            (8, _, _, 6) => Ok(Instruction::ShiftRight(r0, r1)),
            (8, _, _, 7) => Ok(Instruction::SubtractYX(r0, r1)),
            (8, _, _, 0xE) => Ok(Instruction::ShiftLeft(r0, r1)),
            (9, _, _, 0) => Ok(Instruction::SkipNER(r0, r1)),
            (0xA, _, _, _) => Ok(Instruction::SetIndex(nnn)),
            (0xB, _, _, _) => Ok(Instruction::OffsetJump(r0, nnn)),
            (0xC, _, _, _) => Ok(Instruction::Random(r0, nn)),
            (0xD, _, _, _) => Ok(Instruction::Draw(r0, r1 as u8, n)),
            (0xE, _, 9, 0xE) => Ok(Instruction::KeyPressed(r0)),
            (0xE, _, 0xA, 1) => Ok(Instruction::KeyReleased(r0)),
            (0xF, _, 0, 7) => Ok(Instruction::GetDelay(r0)),
            (0xF, _, 0, 0xA) => Ok(Instruction::KeyWait(r0)),
            (0xF, _, 1, 5) => Ok(Instruction::SetDelay(r0)),
            (0xF, _, 1, 8) => Ok(Instruction::SetSound(r0)),
            (0xF, _, 1, 0xE) => Ok(Instruction::AddIndex(r0)),
            (0xF, _, 2, 9) => Ok(Instruction::GetFont(r0)),
            (0xF, _, 3, 3) => Ok(Instruction::AsDecimal(r0)),
            (0xF, _, 5, 5) => Ok(Instruction::Save(r0)),
            (0xF, _, 6, 5) => Ok(Instruction::Load(r0)),
            _ => Err(Error::new(
                ErrorKind::Other,
                format!("Invalid or Unimplemented Instruction: {:016x}", value),
            )),
        }
    }
}

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

    // Execute the instructions.
    fn execute_instruction(&mut self, inst: Instruction) -> Result<(), Error> {
        info!("Instruction: {:?}", inst);
        match inst {
            Instruction::Clear => self.clear_screen(),
            Instruction::Return => self.return_subroutine(),
            Instruction::Jump(addr) => self.jump(addr),
            Instruction::SubRoutine(addr) => self.subroutine(addr),
            Instruction::SkipED(r, data) => self.skip_ed(r, data),
            Instruction::SkipNED(r, data) => self.skip_ned(r, data),
            Instruction::SkipER(r0, r1) => self.skip_er(r0, r1),
            Instruction::SetRegisterD(r, val) => self.set_register_data(r, val),
            Instruction::AddRegisterD(r, val) => self.add_register_data(r, val),
            Instruction::SetRegisterR(r0, r1) => self.set_register_register(r0, r1),
            Instruction::BinaryOR(r0, r1) => self.binary_or(r0, r1),
            Instruction::BinaryAND(r0, r1) => self.binary_and(r0, r1),
            Instruction::LogicalXOR(r0, r1) => self.logical_xor(r0, r1),
            Instruction::AddRegisterR(r0, r1) => self.add_register_register(r0, r1),
            Instruction::SubtractXY(r0, r1) => self.subtract_x_y(r0, r1),
            Instruction::ShiftRight(r0, r1) => self.shift_right(r0, r1),
            Instruction::SubtractYX(r0, r1) => self.subtract_y_x(r0, r1),
            Instruction::ShiftLeft(r0, r1) => self.shift_left(r0, r1),
            Instruction::SkipNER(r0, r1) => self.skip_ner(r0, r1),
            Instruction::SetIndex(val) => self.set_index(val),
            Instruction::OffsetJump(r, addr) => self.offset_jump(r, addr),
            Instruction::Random(r, modif) => self.random(r, modif),
            Instruction::Draw(xa, ya, n) => self.draw(xa, ya, n),
            Instruction::KeyPressed(r) => self.key_pressed(r),
            Instruction::KeyReleased(r) => self.key_released(r),
            Instruction::GetDelay(r) => self.get_delay(r),
            Instruction::KeyWait(r) => self.key_wait(r),
            Instruction::SetDelay(r) => self.set_delay(r),
            Instruction::SetSound(r) => self.set_sound(r),
            Instruction::AddIndex(r) => self.add_index(r),
            Instruction::GetFont(r) => self.get_font(r),
            Instruction::AsDecimal(r) => self.as_decimal(r),
            Instruction::Save(r) => self.save(r),
            Instruction::Load(r) => self.load(r),
        }?;
        Ok(())
    }

    // Instructions as functions.
    fn clear_screen(&mut self) -> Result<(), Error> {
        self.screen = [[false; SCREEN_HEIGHT]; SCREEN_WIDTH];
        Ok(())
    }

    fn return_subroutine(&mut self) -> Result<(), Error> {
        self.counter = self.stack.pop().unwrap() as usize;
        Ok(())
    }

    fn jump(&mut self, location: u16) -> Result<(), Error> {
        self.counter = location as usize;
        Ok(())
    }

    fn subroutine(&mut self, location: u16) -> Result<(), Error> {
        self.stack.push(self.counter as u16);
        self.counter = location as usize;
        Ok(())
    }

    fn skip_ed(&mut self, register: u8, data: u8) -> Result<(), Error> {
        self.counter += INSTRUCTION_SIZE * (self.register[register as usize] == data) as usize;
        Ok(())
    }

    fn skip_ned(&mut self, register: u8, data: u8) -> Result<(), Error> {
        self.counter += INSTRUCTION_SIZE * (self.register[register as usize] != data) as usize;
        Ok(())
    }

    fn skip_er(&mut self, register0: u8, register1: u8) -> Result<(), Error> {
        self.counter += INSTRUCTION_SIZE
            * (self.register[register0 as usize] == self.register[register1 as usize]) as usize;
        Ok(())
    }

    fn set_register_data(&mut self, register: u8, val: u8) -> Result<(), Error> {
        self.register[register as usize] = val;
        Ok(())
    }

    fn add_register_data(&mut self, register: u8, val: u8) -> Result<(), Error> {
        self.register[register as usize] = self.register[register as usize].wrapping_add(val);
        Ok(())
    }

    fn set_register_register(&mut self, register0: u8, register1: u8) -> Result<(), Error> {
        self.register[register0 as usize] = self.register[register1 as usize];
        Ok(())
    }

    fn binary_or(&mut self, register0: u8, register1: u8) -> Result<(), Error> {
        self.register[register0 as usize] |= self.register[register1 as usize];
        Ok(())
    }

    fn binary_and(&mut self, register0: u8, register1: u8) -> Result<(), Error> {
        self.register[register0 as usize] &= self.register[register1 as usize];
        Ok(())
    }

    fn logical_xor(&mut self, register0: u8, register1: u8) -> Result<(), Error> {
        self.register[register0 as usize] ^= self.register[register1 as usize];
        Ok(())
    }

    fn add_register_register(&mut self, register0: u8, register1: u8) -> Result<(), Error> {
        let sum =
            self.register[register0 as usize] as u16 + self.register[register1 as usize] as u16;
        self.register[VF] = (sum > 255) as u8;
        self.register[register0 as usize] = (sum & 0xFF) as u8;
        Ok(())
    }

    fn subtract_x_y(&mut self, register0: u8, register1: u8) -> Result<(), Error> {
        let (vx, vy) = (
            self.register[register0 as usize],
            self.register[register1 as usize],
        );
        self.register[VF] = (vx > vy) as u8;
        self.register[register0 as usize] = vx.wrapping_sub(vy);
        Ok(())
    }

    fn shift_right(&mut self, register0: u8, register1: u8) -> Result<(), Error> {
        let vy = if self.config.legacy {
            self.register[register1 as usize]
        } else {
            self.register[register0 as usize]
        };
        self.register[VF] = ((vy & 0x1) == 1) as u8;
        if self.config.legacy {
            self.register[register0 as usize] = vy >> 1;
        } else {
            self.register[register0 as usize] /= 2;
        }
        Ok(())
    }

    fn subtract_y_x(&mut self, register0: u8, register1: u8) -> Result<(), Error> {
        let (vx, vy) = (
            self.register[register0 as usize],
            self.register[register1 as usize],
        );
        self.register[VF] = (vy > vx) as u8;
        self.register[register0 as usize] = vy.wrapping_sub(vx);
        Ok(())
    }

    fn shift_left(&mut self, register0: u8, register1: u8) -> Result<(), Error> {
        let vy = if self.config.legacy {
            self.register[register1 as usize]
        } else {
            self.register[register0 as usize]
        } << 1;
        self.register[VF] = ((vy & 0x1) == 1) as u8;
        if self.config.legacy {
            self.register[register0 as usize] = vy >> 1;
        } else {
            self.register[register0 as usize] /= 2;
        }
        Ok(())
    }

    fn skip_ner(&mut self, register0: u8, register1: u8) -> Result<(), Error> {
        self.counter += INSTRUCTION_SIZE
            * (self.register[register0 as usize] != self.register[register1 as usize]) as usize;
        Ok(())
    }

    fn set_index(&mut self, val: u16) -> Result<(), Error> {
        self.index = val;
        Ok(())
    }

    fn offset_jump(&mut self, register: u8, location: u16) -> Result<(), Error> {
        self.counter = (location
            + self.register[register as usize * self.config.legacy as usize] as u16)
            as usize;
        Ok(())
    }

    fn random(&mut self, register: u8, modifier: u8) -> Result<(), Error> {
        self.register[register as usize] = rand::random::<u8>() & modifier;
        Ok(())
    }

    fn draw(&mut self, xa: u8, ya: u8, n: u8) -> Result<(), Error> {
        let x: usize = (self.register[xa as usize] & ((SCREEN_WIDTH as u8) - 1)).into();
        let y: usize = (self.register[ya as usize] & ((SCREEN_HEIGHT as u8) - 1)).into();
        self.register[0xF] = 0;
        for r in 0..n {
            let r = r as usize;
            let b = self.memory[(self.index as usize) + r];
            if y + r == SCREEN_HEIGHT - 1 {
                break;
            }
            for p in 0..8 {
                let p = p as usize;
                if x + p == SCREEN_WIDTH - 1 {
                    break;
                }
                let sprite_pixel = ((b << p) & 0b10000000) != 0;
                if sprite_pixel {
                    if self.screen[x + p][y + r] {
                        self.screen[x + p][y + r] = false;
                        self.register[VF] = 1;
                    } else {
                        self.screen[x + p][y + r] = true;
                    }
                }
            }
        }
        self.io.update_screen(&self.screen).unwrap();
        Ok(())
    }

    fn key_pressed(&mut self, register: u8) -> Result<(), Error> {
        if self.keyboard[self.register[register as usize] as usize] {
            self.keyboard[self.register[register as usize] as usize] = false;
            self.counter += INSTRUCTION_SIZE;
        }
        Ok(())
    }

    fn key_released(&mut self, register: u8) -> Result<(), Error> {
        if !self.keyboard[self.register[register as usize] as usize] {
            self.counter += INSTRUCTION_SIZE;
        }
        self.keyboard[self.register[register as usize] as usize] = false;
        Ok(())
    }

    fn get_delay(&mut self, register: u8) -> Result<(), Error> {
        self.register[register as usize] = self.timer.0;
        Ok(())
    }

    fn key_wait(&mut self, register: u8) -> Result<(), Error> {
        if let Some(key) = self.keyboard.iter().position(|x| *x == true) {
            self.keyboard[key] = false;
            self.register[register as usize] = key as u8;
        } else {
            self.counter -= INSTRUCTION_SIZE;
        }
        Ok(())
    }

    fn set_delay(&mut self, register: u8) -> Result<(), Error> {
        self.timer.0 = self.register[register as usize];
        Ok(())
    }

    fn set_sound(&mut self, register: u8) -> Result<(), Error> {
        self.timer.1 = self.register[register as usize];
        Ok(())
    }

    fn add_index(&mut self, register: u8) -> Result<(), Error> {
        self.index = self
            .index
            .wrapping_add(self.register[register as usize] as u16);
        Ok(())
    }

    fn get_font(&mut self, register: u8) -> Result<(), Error> {
        self.index = self.register[register as usize] as u16 * FONT_SIZE;
        Ok(())
    }

    fn as_decimal(&mut self, register: u8) -> Result<(), Error> {
        let mut val = self.register[register as usize];
        for i in 0..3 {
            self.memory[self.index as usize + i] = val % 10;
            val = val / 10;
        }
        Ok(())
    }

    fn save(&mut self, register: u8) -> Result<(), Error> {
        for i in 0..=register as usize {
            self.memory[self.index as usize + i] = self.register[i];
        }
        Ok(())
    }

    fn load(&mut self, register: u8) -> Result<(), Error> {
        for i in 0..=register as usize {
            self.register[i] = self.memory[self.index as usize + i];
        }
        Ok(())
    }
}
