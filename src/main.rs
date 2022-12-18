mod chip_oxide;
use chip_oxide::{
    ChipOxide,
    ChipIO,
    SCREEN_HEIGHT,
    SCREEN_WIDTH,
};

const height: u16 = SCREEN_HEIGHT as u16;
const width: u16 = SCREEN_WIDTH as u16;

use std::{
    time::Duration,
    fs::read as fread,
    io::{stdout, Write},
};

use crossterm::{
    queue, execute,
    terminal::{
        self, EnterAlternateScreen,
        LeaveAlternateScreen,
        SetTitle, SetSize,
    },
    cursor::{Hide, Show, MoveTo},
    style::Print,
    event::{self, read, poll, Event,
        KeyCode, KeyEvent},
    Result as CrossResult,
};

fn start_cli<W>(console: &mut W) -> CrossResult<()>
where
    W: Write,
{
    queue!(
        console,
        SetTitle("Chip Oxide"),
        EnterAlternateScreen,
        Hide,
    )?;
    terminal::enable_raw_mode()?;
    let (x, y) = terminal::size().unwrap();
    //if x < 134 {end_cli(console).unwrap();}
    let x = (x/2) - (width/2) - 2;
    let y = (y/2) - (height/2) - 2;
    queue!(
        console,
        MoveTo(x, y),
        Print("╭"),
        MoveTo(x+width+2, y),
        Print("╮"),
        MoveTo(x+width+2, y+height+2),
        Print("╯"),
        MoveTo(x, y+height+2),
        Print("╰"),
    )?;
    console.flush()?;
    Ok(())
}

fn end_cli<W>(console: &mut W) -> CrossResult<()>
where
    W: Write,
{
    execute!(
        console,
        Show,
        LeaveAlternateScreen,
    )?;
    terminal::disable_raw_mode()?;
    Ok(())
}

fn chip_oxide_cli<W>(console: &mut W) -> CrossResult<()>
where
    W: Write,
{
    let program = fread("roms/IBM Logo.ch8").unwrap();
    let (x, y) = terminal::size().unwrap();
    let x = (x/2) - (width/2);
    let y = (y/2) - (height/2);
    let mut io = IO {
        write: console, x, y
    };
    let chip_oxide = ChipOxide::start(&program[..], &mut io);
    Ok(())
}

struct IO <W: Write> {
    write: W,
    x: u16,
    y: u16,
}

impl<W> ChipIO for IO<W>
where
    W: Write,
{
    fn update_screen(&mut self, screen:
        &[[bool; SCREEN_HEIGHT]; SCREEN_WIDTH]) -> Result<(), &'static str> {
        let (mut x, mut y) = (0, 0);
        for row in screen {
            y = 0;
            for pixel in row {
                if *pixel {
                    queue!(
                        self.write,
                        MoveTo(self.x+x, self.y+y),
                        Print("█"),
                    ).unwrap();
                } y += 1;
            } x += 1
        }
        self.write.flush().unwrap();
        Ok(())
    }
    fn start_beep(&mut self) -> Result<(), &'static str> {
        Ok(())
    }
    fn end_beep(&mut self) -> Result<(), &'static str> {
        Ok(())
    }
    fn get_keyboard_state(&mut self, keyboard: &mut [bool]) -> Result<(), &'static str> {
        if poll(Duration::from_secs(0)).unwrap() {
            if let Event::Key(event) = read().unwrap() {
                if event.code == KeyCode::Esc {
                    panic!("You Quit!")
                }else if let KeyCode::Char(c) = event.code {
                    todo!{}
                }
            }
        }
        Ok(())
    }
}

fn main() {
    let mut console = stdout();
    start_cli(&mut console).unwrap();
    chip_oxide_cli(&mut console).unwrap();
    end_cli(&mut console).unwrap();
}
