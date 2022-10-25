// based on https://flatuicolors.com/palette/ru

use epaint::Color32;

#[derive(Debug, Clone, Copy)]
pub enum Category {
	Emitter,
	Routing,
	Drum,
	Instrument,
	Tone,
	Scale,
	Arpeggio,
	Tuning,
	Step,
	Debug,
}

#[derive(Debug, Clone, Copy)]
pub struct Palette {
	pub background: Color32,
	pub foreground: Color32,
}

impl Category {
	// background, foreground
	#[must_use]
	fn palette_(self) -> (Color32, Color32) {
		match self {
			// red
			Self::Emitter => (
				Color32::from_rgb(0xe6, 0x67, 0x67),
				Color32::from_rgb(0xea, 0x86, 0x85),
			),
			// yellow
			Self::Routing => (
				Color32::from_rgb(0xf5, 0xcd, 0x79),
				Color32::from_rgb(0xf7, 0xd7, 0x94),
			),
			// blue
			Self::Drum => (
				Color32::from_rgb(0x54, 0x6d, 0xe5),
				Color32::from_rgb(0xf3, 0xa6, 0x83),
			),
			// purple
			Self::Instrument => (
				Color32::from_rgb(0x57, 0x4b, 0x90),
				Color32::from_rgb(0x78, 0x6f, 0xa6),
			),
			// cyan
			Self::Tone => (
				Color32::from_rgb(0x3d, 0xc1, 0xd3),
				Color32::from_rgb(0x63, 0xcd, 0xda),
			),
			// pink
			Self::Scale => (
				Color32::from_rgb(0xf7, 0x8f, 0xb3),
				Color32::from_rgb(0xf8, 0xa5, 0xc2),
			),
			// dark pink
			Self::Arpeggio => (
				Color32::from_rgb(0xc4, 0x45, 0x69),
				Color32::from_rgb(0xcf, 0x6a, 0x87),
			),
			// orange
			Self::Tuning => (
				Color32::from_rgb(0xf1, 0x90, 0x66),
				Color32::from_rgb(0xf3, 0xa6, 0x83),
			),
			// gray
			Self::Step => (
				Color32::from_rgb(0x30, 0x39, 0x52),
				Color32::from_rgb(0x59, 0x62, 0x75),
			),
			Self::Debug => (Color32::DEBUG_COLOR, Color32::DEBUG_COLOR),
		}
	}

	#[must_use]
	pub fn palette(self) -> Palette {
		let (background, foreground) = self.palette_();
		Palette {
			background,
			foreground,
		}
	}
}
