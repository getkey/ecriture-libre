use chrono;
use clap::Parser;
use nix::sys::termios;
use std::fs;
use std::io;
use std::io::{Read, Write};
use std::os::unix::io::AsRawFd;
use std::path;

mod stream_string;

#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct Args {
	file: String,

	/// Add journal formatting in Markdown
	#[clap(short, long)]
	journal: bool,
}

fn get_file_handle(filename: &str, journal: bool) -> io::Result<fs::File> {
	let path = path::Path::new(filename);
	let file_exists = path.exists();

	let mut f = fs::OpenOptions::new()
		.write(true)
		.create(true)
		.append(true)
		.open(filename)?;

	if journal {
		if !file_exists {
			if let Some(title) = path.file_name() {
				if let Some(title) = title.to_str() {
					f.write_all(format!("# {}", title).as_bytes())?;
				}
			}
		}

		let now = chrono::Local::now();

		f.write_all(now.format("\n\n## %F %R\n\n").to_string().as_bytes())?;
	}

	Ok(f)
}

fn handle_input(f: &mut fs::File) -> io::Result<()> {
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

fn process() -> io::Result<()> {
	let args = Args::parse();
	let mut f = get_file_handle(&args.file, args.journal)?;

	handle_input(&mut f)
}

fn main() -> io::Result<()> {
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

	// `main` should only go to canonical mode and back
	// to avoid crashes that mess up with the terminal.
	// All the actual computation happens in `process`.
	let res = process();

	termios::tcsetattr(raw_stdin_fd, termios::SetArg::TCSADRAIN, &old_termios)?;

	res
}
