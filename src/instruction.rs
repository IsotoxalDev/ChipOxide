use std::io::{Error, ErrorKind};

#[derive(Debug)]
pub enum Instruction {
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
