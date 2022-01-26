use clap::Parser;
use nix::sys::termios;
use std::fs::OpenOptions;
use std::io;
use std::io::{Read, Write};
use std::os::unix::io::AsRawFd;

mod stream_string;

#[derive(Parser, Debug)]
struct Args {
	file: String,
}

fn process(filename: &str) -> io::Result<()> {
	let mut f = OpenOptions::new()
		.write(true)
		.create(true)
		.append(true)
		.open(filename)?;

	let mut word = stream_string::StreamString::new();

	for byte in io::stdin().bytes() {
		let byte = byte?;
		match byte {
			b'\x03' | b'\x04' => break,
			b'\x7f' => {
				if word.len() > 0 {
					word.pop();
					print!("\x1B[1D \x1B[1D");
				}
			}
			_ => {
				if let Some(character) = word.push(byte)? {
					print!("{}", character);

					if character.is_whitespace() || character.is_ascii_punctuation() {
						f.write_all(word.as_str().as_bytes())?;
						word.clear();
					}
				}
			}
		}
		io::stdout().flush()?;
	}

	Ok(())
}

fn main() -> io::Result<()> {
	let args = Args::parse();
	let raw_stdin_fd = io::stdin().as_raw_fd();

	let old_termios = termios::tcgetattr(raw_stdin_fd)?;

	let mut non_canonical_termios = old_termios.clone();
	non_canonical_termios
		.local_flags
		.remove(termios::LocalFlags::ICANON | termios::LocalFlags::ECHO);
	termios::tcsetattr(
		raw_stdin_fd,
		termios::SetArg::TCSADRAIN,
		&non_canonical_termios,
	)?;

	let res = process(&args.file);

	termios::tcsetattr(raw_stdin_fd, termios::SetArg::TCSADRAIN, &old_termios)?;

	res
}
