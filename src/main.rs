use std::{
    fs::File,
    io::{BufReader, BufWriter, Write},
    path::Path,
};

use simulator_8086::{cpu::Cpu, decoder::Codec};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if let Err(e) = run(&args[1]) {
        eprint!("An error occurred {}", e);
        std::process::exit(1);
    }
}

fn run(path: impl AsRef<Path>) -> Result<(), std::io::Error> {
    let mut cpu = Cpu::new();
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    let out = File::create("output.asm")?;
    let mut writer = BufWriter::new(out);

    writer.write_all(b"bits 16\n")?;

    let c = Codec::new(reader);

    for code in c {
        writer.write_all(code.to_string().as_bytes())?;
        writer.write_all(b"\n")?;
        cpu.execute_instruction(code);
    }
    cpu.print_registers();

    Ok(())
}
