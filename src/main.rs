mod chip_oxide;
use chip_oxide::{ChipIO, ChipOxide, SCREEN_HEIGHT, SCREEN_WIDTH};

const HEIGHT: u16 = SCREEN_HEIGHT as u16;
const WIDTH: u16 = SCREEN_WIDTH as u16;

use std::{
    ops::Drop,
    fs::read as fread,
    io::{stdout, Write, Error, ErrorKind},
    time::Duration,
};

use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{self, poll, read, Event, KeyCode, KeyEvent},
    execute, queue,
    style::Print,
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen, SetSize, SetTitle},
};

fn chip_oxide_cli<W>(console: &mut W) -> Result<(), Error>
where
    W: Write,
{
    let program = fread("roms/IBM Logo.ch8").unwrap();
    let (x, y) = terminal::size().unwrap();
    let x = (x / 2) - (WIDTH / 2);
    let y = (y / 2) - (HEIGHT / 2);
    let mut io = TerminalIO::new(console)?;
    ChipOxide::start(&program[..], &mut io)?;
    Ok(())
}

struct TerminalIO<W: Write> {
    write: W,
    x: u16,
    y: u16,
}

impl<W> TerminalIO<W> where W: Write {
    fn new (mut write: W) -> Result<Self, Error>
    {
        terminal::enable_raw_mode()?;
        let (x, y) = terminal::size()?;
        if x < (WIDTH + 5) {
            terminal::disable_raw_mode()?;
            return Err(Error::new(ErrorKind::Other, "Small Terminal"));
        };
        queue!(write, SetTitle("Chip Oxide"), EnterAlternateScreen, Hide,)?;
        let x = (x / 2) - (WIDTH / 2) - 2;
        let y = (y / 2) - (HEIGHT / 2) - 2;
        queue!(
            write,
            MoveTo(x, y),
            Print("╭"),
            MoveTo(x + WIDTH + 2, y),
            Print("╮"),
            MoveTo(x + WIDTH + 2, y + HEIGHT + 2),
            Print("╯"),
            MoveTo(x, y + HEIGHT + 2),
            Print("╰"),
        )?;
        for p in 1..WIDTH+2 {
            for b in 0..2{
                queue!(
                    write,
                    MoveTo(x+p, y + (HEIGHT+2)*b),
                    Print("─"),
                )?;
            }
        }
        for p in 1..HEIGHT+2 {
            for b in 0..2{
                queue!(
                    write,
                    MoveTo(x+(WIDTH+2)*b, y+p),
                    Print("│"),
                )?;
            }
        }
        write.flush()?;
        Ok(Self{write, x, y})
    }
}

impl<W> Drop for TerminalIO<W> where W: Write {
    fn drop(&mut self) {
        execute!(self.write, Show, LeaveAlternateScreen).unwrap();
        terminal::disable_raw_mode().unwrap();
    }
}

impl<W> ChipIO for TerminalIO<W>
where
    W: Write,
{
    fn update_screen(
        &mut self,
        screen: &[[bool; SCREEN_HEIGHT]; SCREEN_WIDTH],
    ) -> Result<(), &'static str> {
        let mut x = 0;
        for row in screen {
            let mut y = 0;
            for pixel in row {
                if *pixel {
                    queue!(self.write, MoveTo(self.x + x, self.y + y), Print("█"),).unwrap();
                }
                y += 1;
            }
            x += 1
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
                } else if let KeyCode::Char(c) = event.code {
                    return Ok(())
                }
            }
        }
        Ok(())
    }
}

fn main() {
    let mut console = stdout();
    chip_oxide_cli(&mut console).unwrap();
}
