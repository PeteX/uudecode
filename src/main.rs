use std::{
    env,
    fs::File,
    io::{Write, read_to_string, BufWriter},
    sync::LazyLock,
};

use anyhow::bail;
use regex::Regex;

fn byte_value(c: u8) -> anyhow::Result<u8> {
    match c {
        96 => Ok(0),
        _ if c > 32 && c < 127 => Ok(c - 32),
        _ => bail!("found a data character \"{c}\", which is not allowed."),
    }
}

fn decode(input_file: &str, output_file: &str) -> anyhow::Result<()> {
    static START_LINE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^begin \d{3} ").unwrap());
    let text = read_to_string(File::open(input_file)?)?;
    let output = File::create(output_file)?;
    let mut writer = BufWriter::new(output);
    let mut reading_data = false;
    for line in text.lines() {
        if reading_data {
            if line == "end" {
                writer.flush()?;
                return Ok(());
            }

            let mut source = line.chars().map(|c| c as u8);
            let length = match source.next() {
                Some(l) => byte_value(l)?,
                _ => bail!("found an empty line, which is not allowed."),
            };

            let mut acc: u32 = 0;
            let mut bits = 0;
            for _ in 0..length {
                while bits < 8 {
                    let next = match source.next() {
                        Some(c) => byte_value(c)?,
                        None => bail!("line was too short."),
                    };
                    acc = acc << 6 | next as u32;
                    bits += 6;
                }
                bits -= 8;
                writer
                    .write_all(&[(acc >> bits & 0xff).try_into().unwrap()])?;
            }
        } else if START_LINE.is_match(line) {
            reading_data = true;
        }
    }

    if reading_data {
        bail!("data didn't end with \"end\".");
    } else {
        bail!("no starting line found.");
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() != 3 {
        eprintln!("uudecode: usage: {} <input_file> <output_file>", args[0]);
        return;
    }
    
    let input_file = &args[1];
    let output_file = &args[2];
    
    match decode(input_file, output_file) {
        Err(e) => eprintln!("uudecode: {}", e),
        _ => {}
    }
}
