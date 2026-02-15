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
            (5, _, _, 0) => se_reg(x, y),
            (6, _, _, _) => ld(x, kk),
            (7, _, _, _) => add(x, kk),
            (8, _, _, 0) => ld(x, y),
            (8, _, _, 1) => or(x, y),
            (8, _, _, 2) => and(x, y),
            (8, _, _, 3) => xor(x, y),
            (8, _, _, 4) => add_carry(x, y),
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

    fn nop(self) -> Self {
        self
    }

    fn clear(&mut self) {
        self.display = [[Pixel::Black; 64]; 32];
    }

    fn ret(&mut self) {
        self.pc = self.stack[self.sp as usize];
        self.sp -= 1;
    }

    fn jp(&mut self, nnn: u16) {
        self.pc = nnn;
    }

    fn call(&mut self, nnn: u16) {
        self.sp += 1;
        self.stack[self.sp as usize] = self.pc;
        self.pc = nnn;
    }

    fn se(&mut self, x: u8, kk: u8) {
        if self.registers[x as usize] == kk {
            self.pc += 2;
        }
    }

    fn sne(&mut self, x: u8, kk: u8) {
        if self.registers[x as usize] != kk {
            self.pc += 2;
        }
    }

    fn se_reg(&mut self, x: u8, y: u8) {
        if self.registers[x as usize] == self.registers[y as usize] {
            self.pc += 2;
        }
    }

    fn ld(&mut self, x: u8, kk: u8) {
        self.registers[x as usize] = kk;
    }

    fn add(&mut self, x: u8, kk: u8) {
        self.registers[x as usize] += kk;
    }

    fn or(&mut self, x: u8, y: u8) {
        self.registers[x as usize] |= self.registers[y as usize];
    }

    fn and(&mut self, x: u8, y: u8) {
        self.registers[x as usize] &= self.registers[y as usize];
    }

    fn xor(&mut self, x: u8, y: u8) {
        self.registers[x as usize] ^= self.registers[y as usize];
    }

    fn add_carry(&mut self, x : u8, y : u8){
        let sum : u16 = (self.registers[x as usize] as u16) + (self.registers[y as usize] as u16);
        self.registers[0xF] = if sum > 255 { 1 } else { 0 };
        self.registers[x as usize] = sum as u8;
    }

    fn sub(&mut self, x: u8, y: u8) {
        self.registers[0xF] = if self.registers[x as usize] > self.registers[y as usize] { 1 } else { 0 };
        self.registers[x as usize] -= self.registers[y as usize];
    }

    fn shr(&mut self, x: u8, y : u8) {
        self.registers[0xF] = self.registers[x as usize] & 0x1;
        self.registers[x as usize] >>= 1;
    }

    fn subn(&mut self, x: u8, y: u8) {
        self.registers[0xF] = if self.registers[y as usize] > self.registers[x as usize] { 1 } else { 0 };
        self.registers[x as usize] = self.registers[y as usize].wrapping_sub(self.registers[x as usize]);
    }

    fn shl(&mut self, x: u8, y : u8) {
        self.registers[0xF] = (self.registers[x as usize] & 0x80) >> 7;
        self.registers[x as usize] <<= 1;
    }

    fn sne_reg(&mut self, x: u8, y: u8) {
        if self.registers[x as usize] != self.registers[y as usize] {
            self.pc += 2;
        }
    }

    fn ld(&mut self, nnn: u16) {
        self.index = nnn;
    }

    fn jp_v0(&mut self, nnn: u16) {
        self.pc = (self.registers[0] as u16 + nnn) & 0xFFF;
    }

    fn rnd(&mut self, x: u8, kk: u8){
        let rand : u8 = (rand::thread_rng().gen_range(1..=255) as u8);
        self.registers[x as usize] = rand & kk;
    }

    fn drw(&mut self, x: u8, y: u8, n: u8) {

    }

    fn skp(&mut self, x: u8) {

    }

    fn sknp(&mut self, x: u8) {

    }

    fn ld_Vx_DT(&mut self, x: u8) {

    }

    fn ld_K(&mut self, x: u8) {

    }

    fn ld_DT_Vx(&mut self, x: u8) {

    }

    fn ld_ST_Vx(&mut self, x: u8) {

    }

    fn add_I(&mut self, x: u8) {

    }

    fn ld_F(&mut self, x: u8) {

    }

    fn ld_B(&mut self, x: u8) {

    }

    fn ld_I_Vx(&mut self, x: u8) {
    }

    fn ld_Vx_I(&mut self, x: u8) {

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

