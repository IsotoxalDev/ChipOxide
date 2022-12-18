pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 32;
pub const KEYBOARD_SIZE: usize = 16;

const MEM_SIZE: usize = 4096;
const STACK_SIZE: usize = 16;
const REGISTER_SIZE: usize = 16;
const COUNTER_START: usize = 0x200;

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

// Trait for IO.
pub trait ChipIO {
    // Update the screen
    fn update_screen(&mut self, screen:
        &[[bool; SCREEN_HEIGHT]; SCREEN_WIDTH]) -> Result<(), &'static str>;
    
    // Toggle Sound
    fn start_beep(&mut self) -> Result<(), &'static str>;
    fn end_beep(&mut self) -> Result<(), &'static str>;
    
    // Get keyboard State
    fn get_keyboard_state(&mut self, keyboard: &mut [bool]) -> Result<(), &'static str>;
}

enum Instruction {
    Clear,
    //Return,
    Jump (u16),
    //SubRoutine (u16),
    SetRegister (u8, u8),
    AddRegister (u8, u8),
    SetIndex(u16),
    Draw(u8, u8, u8)
}

// Decode the instruction and take out usefull data
impl TryFrom<u16> for Instruction {
    type Error = String;
    
    fn try_from(value: u16) -> Result<Self, Self::Error> {
        let inst = ((value & 0b1111000000000000) >> 12) as u8;
        let r0 = ((value & 0b0000111100000000) >> 8) as u8;
        let r1 = ((value & 0b0000000011110000) >> 4) as u8;
        let n = (value & 0b0000000000001111) as u8;
        let nn = (value & 0b0000000011111111) as u8;
        let nnn = (value & 0b0000111111111111);
        
        match (inst, r0, r1, n) {
            (0, 0, 0xE, 0) => Ok(Instruction::Clear),
            (1, _, _, _) => Ok(Instruction::Jump (nnn)),
            (6, _, _, _) => Ok(Instruction::SetRegister(r0, nn)),
            (7, _, _, _) => Ok(Instruction::AddRegister(r0, nn)),
            (0xA, _, _, _) => Ok(Instruction::SetIndex(nnn)),
            (0xD, _, _, _) => Ok(Instruction::Draw(r0, r1 as u8, n)),
            _ => {
                Err(format!("Invalid/Unimplemented Command: {:016b}", r1))
            },
        }
    }
}

// The ChipOxid Struct
pub struct ChipOxide<'a, I: ChipIO> {
    memory: [u8; MEM_SIZE],
    screen: [[bool; SCREEN_HEIGHT]; SCREEN_WIDTH],
    stack: [u16; STACK_SIZE],
    register: [u8; REGISTER_SIZE],
    timer: (u8, u8), // Delay Timer, Sound Timer
    keyboard: [bool; KEYBOARD_SIZE],
    counter: usize,
    index: u16,
    IO: &'a mut I,
}

impl<'a, I> ChipOxide<'a,  I>
where
    I: ChipIO,
{
    // Create an empty shell.
    fn empty(io: &'a mut I) -> Self {
        Self {
            memory: [0; MEM_SIZE],
            screen: [[false; SCREEN_HEIGHT]; SCREEN_WIDTH],
            stack: [0; STACK_SIZE],
            register: [0; REGISTER_SIZE],
            timer: (0, 0),
            keyboard: [false; KEYBOARD_SIZE],
            counter: 0,
            index: 0,
            IO: io,
        }
    }
    
    // Load and put a program in loop.
    pub fn start(program: &[u8], io: &'a mut I) {
        let mut chip8 = Self::empty(io);
        
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
        
        // Main chip loop.
        loop {
            let inst = chip8.fetch_instruction().unwrap();
            chip8.execute_instruction(inst);
            chip8.IO.get_keyboard_state(&mut chip8.keyboard);
        }
    }
    
    // Update the delay timer and the sound timer.
    fn update_timer(&mut self) {
        if self.timer.0 !=0 {self.timer.0 -= 1}
        if self.timer.1 !=0 {self.timer.1 -= 1}
    }
    
    // Fetch the instruction from memory.
    fn fetch_instruction(&mut self) -> Result<Instruction, String>  {
        self.counter += 2;
        Instruction::try_from((
            self.memory[self.counter - 1] as u16) |
            ((self.memory[self.counter - 2] as u16) << 8
        ))

    }
    
    // Execute the instructions.
    fn execute_instruction(&mut self, inst: Instruction) {
        match inst {
            Instruction::Clear => self.clear_screen(),
            Instruction::Jump(addr) => self.jump(addr),
            Instruction::SetRegister(r, val) => self.set_register(r, val),
            Instruction::AddRegister(r, val) => self.add_register(r, val),
            Instruction::SetIndex(val) => self.set_index(val),
            Instruction::Draw(xa, ya, n) => self.draw(xa, ya, n),
        }
    }
    
    // Instructions as functions.
    fn clear_screen(&mut self) {
        self.screen = [[false; SCREEN_HEIGHT]; SCREEN_WIDTH];
    }
    
    fn jump(&mut self, location: u16) {
        self.counter = location as usize;
    }
    
    fn set_register(&mut self, register: u8, val: u8) {
        self.register[register as usize] = val
    }
    
    fn add_register(&mut self, register: u8, val: u8) {
        self.register[register as usize] += val
    }
    
    fn set_index(&mut self, val: u16) {
        self.index = val
    }
    
    fn draw(&mut self, xa: u8, ya: u8, n: u8) {
        let x: usize = (self.register[xa as usize]
            & (SCREEN_WIDTH as u8) - 1).into();
        let y: usize = (self.register[ya as usize]
            & (SCREEN_HEIGHT as u8) - 1).into();
        self.register[0xF] = 0;
        for r in 0..n {
            let r = r as usize;
            let b = self.memory[(self.index as usize)+r];
            if y+r == SCREEN_HEIGHT-1 { break }
            for p in 0..8 {
                let p = p as usize;
                if x+p == SCREEN_WIDTH-1 { break }
                let sprite_pixel = ((b << p) & 0b10000000) != 0;
                if  sprite_pixel {
                    if self.screen[x+p][y+r] {
                        self.screen[x+p][y+r] = false;
                        self.register[0xF] = 1;
                    }
                    else {
                        self.screen[x+p][y+r] = true;
                    }
                }
            }
        }
        self.IO.update_screen(&self.screen);
    }
}

