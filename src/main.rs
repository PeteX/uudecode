use std::{
    env,
    fs::{File, read_to_string},
    io::{BufWriter, Write},
};

use anyhow::bail;

fn byte_value(c: u8) -> anyhow::Result<u8> {
    match c {
        96 => Ok(0),
        _ if c > 32 && c < 127 => Ok(c - 32),
        _ => bail!("found a data character \"{c}\", which is not allowed."),
    }
}

fn is_begin_line(line: &str) -> bool {
    if !line.starts_with("begin ") {
        return false;
    }

    let after_begin = &line[6..];
    if after_begin.len() < 4 {
        return false; // Need at least three digits plus space.
    }

    // Check if next three characters are digits.
    let chars: Vec<char> = after_begin.chars().collect();
    if chars.len() < 4 {
        return false;
    }

    for i in 0..3 {
        if !chars[i].is_ascii_digit() {
            return false;
        }
    }

    // Check the fourth character is a space.
    chars[3] == ' '
}

fn decode(input_file: &str, output_file: &str) -> anyhow::Result<()> {
    let text = read_to_string(input_file)?;
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
                writer.write_all(&[(acc >> bits & 0xff).try_into().unwrap()])?;
            }
        } else if is_begin_line(line) {
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
