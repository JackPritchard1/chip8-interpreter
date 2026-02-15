use std::time::Duration;
use chip8_base::*;
use log::{debug, error, log_enabled, info, Level};
use rand::Rng;


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
    keys: Keys,
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
            keys: [false; 16],
            display: [[Pixel::Black; 64]; 32],
            freq: f
        }
    }

    fn execute(&mut self, opcode: (u8, u8)) {
        let nib = (opcode.0 & 0b1111_0000, opcode.0 & 0b0000_1111, opcode.1 & 0b1111_0000, opcode.1 & 0b0000_1111);
        let x = nib.1;
        let y = nib.2;
        let n = nib.3;
        let kk = nib.2 << 4 | nib.3;
        let nnn = ((nib.1 as u16) << 8 | (nib.2 as u16) << 4 | (nib.3 as u16));
        match nib {
            (0, 0, 0, 0) => self.nop(),
            (0, 0, 0xE, 0x0) => self.clear(),
            (0, 0, 0xE, 0xE) => self.ret(),
            (0, _, _, _) => self.sys(nnn),
            (1, _, _, _) => self.jp(nnn),
            (2, _, _, _) => self.call(nnn),
            (3, _, _, _) => self.se(x, kk),
            (4, _, _, _) => self.sne(x, kk),
            (5, _, _, 0) => self.se_reg(x, y),
            (6, _, _, _) => self.ld(x, kk),
            (7, _, _, _) => self.add(x, kk),
            (8, _, _, 0) => self.ld(x, y),
            (8, _, _, 1) => self.or(x, y),
            (8, _, _, 2) => self.and(x, y),
            (8, _, _, 3) => self.xor(x, y),
            (8, _, _, 4) => self.add_carry(x, y),
            (8, _, _, 5) => self.sub(x, y),
            (8, _, _, 6) => self.shr(x, y),
            (8, _, _, 7) => self.subn(x, y),
            (8, _, _, 0xE) => self.shl(x, y),
            (9, _, _, 0) => self.sne_reg(x, y),
            (0xA, _, _, _) => self.ld_I(nnn),
            (0xB, _, _, _) => self.jp_v0(nnn),
            (0xC, _, _, _) => self.rnd(x, kk),
            (0xD, _, _, _) => self.drw(x, y, n),
            (0xE, _, 9, 0xE) => self.skp(x),
            (0xE, _, 0xA, 1) => self.sknp(x),
            (0xF, _, 0, 7) => self.ld_Vx_DT(x),
            (0xF, _, 0, 0xA) => self.ld_K(x),
            (0xF, _, 1, 5) => self.ld_DT_Vx(x),
            (0xF, _, 1, 8) => self.ld_ST_Vx(x),
            (0xF, _, 1, 0xE) => self.add_I(x),
            (0xF, _, 2, 9) => self.ld_F(x),
            (0xF, _, 3, 3) => self.ld_B(x),
            (0xF, _, 5, 5) => self.ld_I_Vx(x),
            (0xF, _, 6, 5) => self.ld_Vx_I(x),
            _ => panic!("Invalid opcode")
        }
    }

    fn nop(&mut self)  {

    }

    fn sys(&mut self, nnn: u16) {
        self.pc = nnn;
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

    fn ld_I(&mut self, nnn: u16) {
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
        let x_pos = self.registers[x as usize] % 64;
        let y_pos = self.registers[y as usize] % 32;
        self.registers[0xF] = 0;
        for i in 0..n {
            if y_pos + i >= 32 { break; }
            for b in 0..8 {
                if x_pos + b >= 64 { break; }
                let pixel = self.display[(y_pos + i) as usize][(x_pos + b) as usize];
                self.display[(y_pos + i) as usize][(x_pos + b) as usize] = {
                        if pixel == Pixel::Black {
                            Pixel::White
                        } else {
                            self.registers[0xF] = 1;
                            Pixel::Black
                        }
                }
            }
        }

    }

    fn skp(&mut self, x: u8) {
        if self.keys[x as usize] { self.pc += 2;}
    }

    fn sknp(&mut self, x: u8) {
        if !self.keys[x as usize] { self.pc += 2;}
    }

    fn ld_Vx_DT(&mut self, x: u8) {
        self.registers[x as usize] = self.delay_timer;
    }

    fn ld_K(&mut self, x: u8) {
        while self.keys == [false; 16] {
            for key in 0..16 {
                if self.keys[key] { self.registers[x as usize] = (key as u8); }
            }
        }
    }

    fn ld_DT_Vx(&mut self, x: u8) {
        self.delay_timer = self.registers[x as usize];
    }

    fn ld_ST_Vx(&mut self, x: u8) {
        self.sound_timer = self.registers[x as usize];
    }

    fn add_I(&mut self, x: u8) {
        self.index += self.registers[x as usize] as u16;
    }

    fn ld_F(&mut self, x: u8) {

    }

    fn ld_B(&mut self, x: u8) {
        self.memory[self.index as usize] = self.registers[x as usize] / 100;
        self.memory[(self.index + 1) as usize] = (self.registers[x as usize] / 10) % 10;
        self.memory[(self.index + 2) as usize] = self.registers[x as usize] % 10;
    }

    fn ld_I_Vx(&mut self, x: u8) {
        for i in 0..x {
            self.memory[self.index as usize + i as usize] = self.registers[i as usize];
        }
    }

    fn ld_Vx_I(&mut self, x: u8) {
        for i in 0..x {
            self.registers[i as usize] = self.memory[self.index as usize + i as usize];
        }
    }

    fn initialise_sprites(&mut self) {
        let initial = 0x50;
        for i in 0..16{
            let sprite = Sprite::new(i);
            for j in 0..5 {
                self.memory[(initial + (i * 5) + j) as usize] = sprite.hex[j as usize];
            }
        }
    }

}

impl Interpreter for State {
    fn step(&mut self, keys: &Keys) -> Option<Display> {
        let op: (u8, u8) = (self.memory[self.pc as usize], self.memory[(self.pc + 1) as usize]);

        info!(target: "Interpreter/mod", "PC: {:#06X} | Opcode: {:#06X}{:#06X}", self.pc, op.0, op.1);
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

pub struct Sprite {
    key : u8,
    hex : [u8; 5]
}

impl Sprite {
    pub fn new(key : u8) -> Self {
        let hex = match key {
            0 => [0xF0, 0x90, 0x90, 0x90, 0xF0],
            1 => [0x20, 0x60, 0x20, 0x20, 0x70],
            2 => [0xF0, 0x10, 0xF0, 0x80, 0xF0],
            3 => [0xF0, 0x10, 0xF0, 0x10, 0xF0],
            4 => [0x90, 0x90, 0xF0, 0x10, 0x10],
            5 => [0xF0, 0x80, 0xF0, 0x10, 0xF0],
            6 => [0xF0, 0x80, 0xF0, 0x90, 0xF0],
            7 => [0xF0, 0x10, 0x20, 0x40, 0x40],
            8 => [0xF0, 0x90, 0xF0, 0x90, 0xF0],
            9 => [0xF0, 0x90, 0xF0, 0x10, 0xF0],
            0xA => [0xF0, 0x90, 0xF0, 0x90, 0x90],
            0xB => [0xE0, 0x90, 0xE0, 0x90, 0xE0],
            0xC => [0xF0, 0x80, 0x80, 0x80, 0xF0],
            0xD => [0xE0, 0x90, 0x90, 0x90, 0xE0],
            0xE => [0xF0, 0x80, 0xF0, 0x80, 0xF0],
            0xF => [0xF0, 0x80, 0xF0, 0x80, 0x80],
            _ => panic!("Invalid sprite key")
        };
        Sprite { key, hex }
    }
}