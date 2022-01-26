use std::ffi::OsStr;
use std::ffi::OsString;
use std::io;
use std::os::unix::ffi::OsStrExt;

pub struct StreamString {
	content: String,
	tmp: OsString,
}

impl StreamString {
	pub fn new() -> StreamString {
		StreamString {
			content: String::new(),
			tmp: OsString::with_capacity(4), // UTF-8 code points are up to 4 byte long
		}
	}
	/// Returns a char if a valid UTF-8 codepoint has been completed.
	pub fn push(&mut self, byte: u8) -> Result<Option<char>, io::Error> {
		if self.tmp.len() >= 4 {
			// if after 4 bytes have been added, tmp hasn't been cleared,
			// it means these 4 bytes didn't form a valid UTF-8 sequence
			return Err(io::Error::new(
				io::ErrorKind::InvalidData,
				"Invalid UTF-8 sequence.",
			));
		}

		self.tmp.push(OsStr::from_bytes(&[byte]));

		if let Some(character) = self.tmp.to_str() {
			self.content.push_str(character);
			self.tmp.clear();

			let last_char = self.content.chars().last();

			return Ok(last_char);
		}

		Ok(None)
	}
	pub fn pop(&mut self) {
		self.content.pop();
	}
	pub fn len(&self) -> usize {
		self.content.len()
	}
	pub fn clear(&mut self) {
		self.content.clear();
		self.tmp.clear();
	}
	pub fn as_str(&self) -> &str {
		self.content.as_str()
	}
}
