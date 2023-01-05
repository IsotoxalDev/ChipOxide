use crate::{SCREEN_HEIGHT, SCREEN_WIDTH};
use std::io::Error;

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
