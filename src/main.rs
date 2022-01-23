use clap::Parser;
use nix::sys::termios;
use std::fs::OpenOptions;
use std::io;
use std::io::{Read, Write};
#[cfg(unix)]
use std::os::unix::io::AsRawFd;

#[derive(Parser, Debug)]
struct Args {
	file: String,
}

fn main() -> io::Result<()> {
	let args = Args::parse();
	let stdin = io::stdin();

	let old_termios = termios::tcgetattr(stdin.as_raw_fd())?;

	let mut non_canonical_termios = old_termios.clone();
	non_canonical_termios
		.local_flags
		.remove(termios::LocalFlags::ICANON | termios::LocalFlags::ECHO);
	termios::tcsetattr(
		stdin.as_raw_fd(),
		termios::SetArg::TCSADRAIN,
		&non_canonical_termios,
	)?;

	let mut buffer = String::new();
	let mut f = OpenOptions::new()
		.write(true)
		.create(true)
		.append(true)
		.open(args.file)?;

	let mut deletable_chars = 0;

	for byte in stdin.bytes() {
		let b = byte? as char;
		match b {
			'\x03' | '\x04' => break,
			' ' | '.' | '!' | '?' | ':' | ',' | ';' => {
				buffer.push(b);
				f.write(buffer.as_bytes())?;
				buffer.clear();
				print!("{}", b);
				deletable_chars = 0;
			}
			'\x7f' => {
				if deletable_chars > 0 {
					print!("\x1B[1D \x1B[1D");
					buffer.pop();
					deletable_chars -= 1;
				}
			}
			_ => {
				buffer.push(b);
				print!("{}", b);
				deletable_chars += 1;
			}
		}
		io::stdout().flush()?;
	}

	Ok(())
}
