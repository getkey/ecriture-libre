use clap::Parser;
use nix::sys::termios;
use std::ffi::OsStr;
use std::ffi::OsString;
use std::fs::OpenOptions;
use std::io;
use std::io::{Read, Write};
use std::os::unix::ffi::OsStrExt;
use std::os::unix::io::AsRawFd;

#[derive(Parser, Debug)]
struct Args {
	file: String,
}

struct PartialString {
	content: String,
	tmp: OsString,
}

impl PartialString {
	fn new() -> PartialString {
		PartialString {
			content: String::new(),
			tmp: OsString::with_capacity(4), // UTF-8 code points are up to 4 byte long
		}
	}
	fn push(&mut self, byte: u8) -> Option<char> {
		self.tmp.push(OsStr::from_bytes(&[byte]));

		if let Some(character) = self.tmp.to_str() {
			self.content.push_str(character);
			self.tmp.clear();

			let last_char = self.content.chars().last();

			return last_char;
		}

		None
	}
	fn pop(&mut self) {
		self.content.pop();
	}
	fn len(&self) -> usize {
		self.content.len()
	}
	fn clear(&mut self) {
		self.content.clear();
		self.tmp.clear();
	}
}

fn process(filename: &str) -> io::Result<()> {
	println!("{:?}", "éèëêàù".as_bytes());

	let mut f = OpenOptions::new()
		.write(true)
		.create(true)
		.append(true)
		.open(filename)?;

	let mut word = PartialString::new();

	for byte in io::stdin().bytes() {
		let byte = byte?;
		match byte {
			b'\x03' | b'\x04' => break,
			b' ' | b'.' | b'!' | b'?' | b':' | b',' | b';' => {
				if let Some(character) = word.push(byte) {
					print!("{}", character);
				}
				f.write_all(word.content.as_bytes())?;
				word.clear();
			}
			b'\x7f' => {
				if word.len() > 0 {
					word.pop();
					print!("\x1B[1D \x1B[1D");
				}
			}
			_ => {
				if let Some(character) = word.push(byte) {
					print!("{}", character);
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
