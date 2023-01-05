use crate::{
    ChipIO, ChipOxide, Instruction, FONT_SIZE, INSTRUCTION_SIZE, SCREEN_HEIGHT, SCREEN_WIDTH, VF,
};
use log::info;
use std::io::Error;

impl<'a, I> ChipOxide<'a, I>
where
    I: ChipIO,
{
    // Execute the instructions.
    pub fn execute_instruction(&mut self, inst: Instruction) -> Result<(), Error> {
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
