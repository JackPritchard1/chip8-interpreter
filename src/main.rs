mod interpreter;

fn main() {
    env_logger::init();
    let freq = 700f32;
    let vm = interpreter::State::new(freq);
    chip8_base::run(vm);

}