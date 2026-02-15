use std::time::Duration;
use chip8_base::*;
use log::{debug, error, log_enabled, info, Level};


pub struct State {
    memory: [u8; 4096],
    registers: [u8; 16],
    pc: u16,
    index: u16,
    stack: [u16; 16],
    sp: u8,
    delay_timer: u8,
    sound_timer: u8,
    display: Display,
    freq: f32
}

impl State {
    pub fn new(f: f32) -> Self {
        State {
            memory: [0; 4096],
            registers: [0; 16],
            pc: 0x200,
            index: 0,
            stack: [0; 16],
            sp: 0,
            delay_timer: 0,
            sound_timer: 0,
            display: [[Pixel::Black; 64]; 32],
            freq: f
        }
    }
}

impl Interpreter for State {
    fn step(&mut self, keys: &Keys) -> Option<Display> {
        let op = self.memory[self.pc];

        debug!(target: "Interpreter/mod", "PC: {:#06X} | Opcode: {:#06X}", self.pc, op)
        if self.pc < 4094 {
            self.pc += 2;
        } else {
            self.pc = 0x200;
        }
        Some(self.display)
    }

    fn speed(&self) -> Duration {
        Duration::from_secs_f32(1.0 / self.freq)
    }
    fn buzzer_active(&self) -> bool {
        self.sound_timer != 0
    }
}

fn execute(opcode: (u8, u8)) {
    let nib = (opcode.0 & 0b1111_0000, opcode.0 & 0b0000_1111, opcode.1 & 0b1111_0000, opcode.1 & 0b0000_1111);
    let x = nib.1;
    let y = nib.2;
    let n = nib.3;
    let kk = nib.2 << 4 | nib.3;
    let nnn = ((nib.1 as u16) << 8 | (nib.2 as u16) << 4 | (nib.3 as u16));
    match nib {
        (0, 0, 0, 0) => nop(),
        (0, 0, 0xE, 0x0) => clear(),
        (0, 0, 0xE, 0xE) => ret(),
        (0, _, _, _) => sys(nnn),
        (1, _, _, _) => jp(nnn),
        (2, _, _, _) => call(nnn),
        (3, _, _, _) => se(x, kk),
        (4, _, _, _) => sne(x, kk),
        (5, _, _, 0) => se(x, y),
        (6, _, _, _) => ld(x, kk),
        (7, _, _, _) => add(x, kk),
        (8, _, _, 0) => ld(x, y),
        (8, _, _, 1) => or(x, y),
        (8, _, _, 2) => and(x, y),
        (8, _, _, 3) => xor(x, y),
        (8, _, _, 4) => add(x, y),
        (8, _, _, 5) => sub(x, y),
        (8, _, _, 6) => shr(x, y),
        (8, _, _, 7) => subn(x, y),
        (8, _, _, 0xE) => shl(x, y),
        (9, _, _, 0) => sne_reg(x, y),
        (0xA, _, _, _) => ld(nnn),
        (0xB, _, _, _) => jp_v0(nnn),
        (0xC, _, _, _) => rnd(x, kk),
        (0xD, _, _, _) => drw(x, y, n),
        (0xE, _, 9, 0xE) => skp(x),
        (0xE, _, 0xA, 1) => sknp(x),
        (0xF, _, 0, 7) => ld_Vx_DT(x),
        (0xF, _, 0, 0xA) => ld_K(x),
        (0xF, _, 1, 5) => ld_DT_Vx(x),
        (0xF, _, 1, 8) => ld_ST_Vx(x),
        (0xF, _, 1, 0xE) => add_I(x),
        (0xF, _, 2, 9) => ld_F(x),
        (0xF, _, 3, 3) => ld_B(x),
        (0xF, _, 5, 5) => ld_I_Vx(x),
        (0xF, _, 6, 5) => ld_Vx_I(x),
    }
}

