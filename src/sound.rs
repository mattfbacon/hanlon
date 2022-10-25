use std::io::Cursor;

use rodio::{Decoder, OutputStreamHandle, Source as _};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Pitch(u8);

impl Pitch {
	pub const MAX: u8 = 7;

	pub const fn new(value: u8) -> Self {
		assert!(value <= Self::MAX);
		Self(value)
	}

	pub const fn increment_by(self, amount: u8) -> Self {
		Self((self.0 + amount) % (Self::MAX + 1))
	}

	pub const fn semitones(self) -> u8 {
		self.0
	}
}

#[derive(Debug, Clone, Copy)]
pub enum Type {
	Guitar,
}

#[derive(Debug, Clone, Copy)]
pub struct Sound {
	pub pitch: Pitch,
	pub ty: Type,
}

impl Sound {
	pub fn sample_for(self) -> &'static [u8] {
		match self.ty {
			Type::Guitar => [
				include_bytes!("../sounds/guitar-0.ogg") as &[u8],
				include_bytes!("../sounds/guitar-1.ogg"),
				include_bytes!("../sounds/guitar-2.ogg"),
				include_bytes!("../sounds/guitar-3.ogg"),
				include_bytes!("../sounds/guitar-4.ogg"),
				include_bytes!("../sounds/guitar-5.ogg"),
				include_bytes!("../sounds/guitar-6.ogg"),
				include_bytes!("../sounds/guitar-7.ogg"),
			][usize::from(self.pitch.semitones())],
		}
	}

	pub fn play(self, stream: &OutputStreamHandle) {
		let sample = self.sample_for();
		let decoder = Decoder::new_vorbis(Cursor::new(sample)).unwrap();
		stream.play_raw(decoder.convert_samples()).unwrap();
	}
}
