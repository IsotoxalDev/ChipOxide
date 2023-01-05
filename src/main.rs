mod chip_oxide;
use chip_oxide::{ChipConfig, ChipIO, ChipOxide, SCREEN_HEIGHT, SCREEN_WIDTH};

const HEIGHT: u16 = SCREEN_HEIGHT as u16;
const WIDTH: u16 = SCREEN_WIDTH as u16;

use std::{
    fs::read as fread,
    io::{stdout, Error, ErrorKind, Write},
    ops::Drop,
    time::Duration,
};

use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{poll, read, Event, KeyCode, KeyEventKind},
    execute, queue,
    style::Print,
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen, SetTitle},
};

fn chip_oxide_cli<W>(console: &mut W) -> Result<(), Error>
where
    W: Write,
{
    let program = fread("/home/abhi/Dev/Rust/chip_oxide/roms/PONG").unwrap();
    let mut io = TerminalIO::new(console)?;
    let mut config = ChipConfig::default(false);
    ChipOxide::start(&program[..], &mut io, &mut config)?;
    Ok(())
}

struct TerminalIO<W: Write> {
    write: W,
    prev: [[bool; SCREEN_HEIGHT]; SCREEN_WIDTH],
    x: u16,
    y: u16,
}

impl<W> TerminalIO<W>
where
    W: Write,
{
    fn new(mut write: W) -> Result<Self, Error> {
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
        for p in 1..WIDTH + 2 {
            for b in 0..2 {
                queue!(write, MoveTo(x + p, y + (HEIGHT + 2) * b), Print("─"),)?;
            }
        }
        for p in 1..HEIGHT + 2 {
            for b in 0..2 {
                queue!(write, MoveTo(x + (WIDTH + 2) * b, y + p), Print("│"),)?;
            }
        }
        write.flush()?;
        let (x, y) = (x + 2, y + 2);
        Ok(Self {
            write,
            x,
            y,
            prev: [[false; SCREEN_HEIGHT]; SCREEN_WIDTH],
        })
    }
}

impl<W> Drop for TerminalIO<W>
where
    W: Write,
{
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
    ) -> Result<(), Error> {
        let mut x = 0;
        for row in screen {
            let mut y = 0;
            for pixel in row {
                queue!(self.write, MoveTo(self.x + x, self.y + y))?;
                if *pixel {
                    queue!(self.write, Print("█"))?;
                } else if self.prev[x as usize][y as usize] {
                    queue!(self.write, Print(" "))?;
                }
                y += 1;
            }
            x += 1
        }
        self.write.flush().unwrap();
        self.prev = screen.to_owned();
        Ok(())
    }
    fn start_beep(&mut self) -> Result<(), Error> {
        Ok(())
    }
    fn end_beep(&mut self) -> Result<(), Error> {
        Ok(())
    }
    fn get_key(&mut self) -> Result<Option<(usize, bool)>, Error> {
        if poll(Duration::from_micros(1))? {
            if let Event::Key(event) = read()? {
                if event.code == KeyCode::Esc {
                    panic!("You Quit!")
                } else if let KeyCode::Char(c) = event.code {
                    return Ok(Some((
                        match c {
                            '1' => 1,
                            '2' => 2,
                            '3' => 3,
                            '4' => 0xC,
                            'q' => 4,
                            'w' => 5,
                            'e' => 6,
                            'r' => 0xD,
                            'a' => 7,
                            's' => 8,
                            'd' => 9,
                            'f' => 0xE,
                            'z' => 0xA,
                            'x' => 0,
                            'c' => 0xB,
                            'v' => 0xF,
                            _ => return Ok(None),
                        },
                        match event.kind {
                            KeyEventKind::Press => true,
                            KeyEventKind::Repeat => true,
                            KeyEventKind::Release => false,
                        },
                    )));
                }
            }
        }
        Ok(None)
    }
}

fn main() {
    let mut console = stdout();
    chip_oxide_cli(&mut console).unwrap();
}
