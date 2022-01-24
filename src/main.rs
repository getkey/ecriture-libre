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

fn process(filename: &str) -> io::Result<()> {
	println!("{:?}", "éèëêàù".as_bytes());

	let mut buffer = OsString::new();
	let mut f = OpenOptions::new()
		.write(true)
		.create(true)
		.append(true)
		.open(filename)?;

	let mut deletable_chars = 0;

	for byte in io::stdin().bytes() {
		let byte = byte?;
		match byte {
			b'\x03' | b'\x04' => break,
			b' ' | b'.' | b'!' | b'?' | b':' | b',' | b';' => {
				buffer.push(OsStr::from_bytes(&[byte]));
				f.write_all(buffer.as_bytes())?;
				io::stdout().write_all(&[byte])?;
				buffer.clear();
				deletable_chars = 0;
			}
			b'\x7f' => {
				if deletable_chars > 0 {
					print!("\x1B[1D \x1B[1D");
					// buffer.pop();
					deletable_chars -= 1;
				}
			}
			_ => {
				// buffer.push(byte);
				// let sparkle_heart = str::from_utf8(buffer.as_slice()).unwrap();
				// print!("{:?} {}", buffer, sparkle_heart);
				// print!("{}", &buffer as &[char]);
				// stdout.write_all(&buffer)?;
				buffer.push(OsStr::from_bytes(&[byte]));
				deletable_chars += 1;
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
