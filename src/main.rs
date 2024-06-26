use std::{fs::File, io::BufReader, path::Path};

use simulator_8086::cpu::Cpu;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if let Err(e) = run(&args[1]) {
        eprint!("An error occurred {}", e);
        std::process::exit(1);
    }
}

fn run(path: impl AsRef<Path>) -> Result<(), std::io::Error> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut cpu = Cpu::new(reader);
    cpu.run();

    cpu.print_registers();

    Ok(())
}
