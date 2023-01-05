/// Struct for configuring the emulator.
pub struct ChipConfig {
    pub opcodes_per_cycle: usize,
    pub timer_hz: u8,
    pub legacy: bool,
}

impl ChipConfig {
    /// Default Config
    pub fn default(legacy: bool) -> Self {
        Self {
            opcodes_per_cycle: 8,
            timer_hz: 60,
            legacy,
        }
    }
}
