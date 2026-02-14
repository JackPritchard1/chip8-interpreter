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

        debug!(target: "Interpreter/mod", "PC: {:#06X} | Opcode: {:#06X}", self.pc, op);
        debug!(target: "Interpreter", "Frequency: {:#06X}", self.freq);
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

fn execute(opcode: u8) {

}

