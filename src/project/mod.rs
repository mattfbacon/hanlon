use std::collections::HashMap;
use std::fs::File;
use std::path::Path;
use std::time::Duration;

use serde::{Deserialize, Serialize};

mod colors;
pub mod component;
pub mod direction;
pub mod pellet;
pub mod position;

pub use self::component::Component;
use self::component::ShouldEmit;
pub use self::direction::Direction;
pub use self::pellet::Pellet;
pub use self::position::Position;
use crate::sound::Sound;

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
#[repr(transparent)]
pub struct BeatsPerMinute(pub f32);

impl BeatsPerMinute {
	pub fn beat_time(self) -> Duration {
		Duration::from_secs_f32(60.0 / self.0)
	}
}

#[serde_with::serde_as]
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Project {
	#[serde_as(as = "Vec<(_, _)>")]
	pub components: HashMap<Position, Component>,
	pub tempo: BeatsPerMinute,
	pub pellets: Vec<Pellet>,
	#[serde(skip)]
	steps: u32,
}

impl Project {
	pub fn read(file: &Path) -> Result<Self, String> {
		let file = File::open(file).map_err(|err| format!("could not open file at {file:?}: {err}"))?;
		serde_json::from_reader(std::io::BufReader::new(file))
			.map_err(|err| format!("invalid project data: {err}"))
	}

	pub fn write(&self, file: &Path) -> Result<(), String> {
		let file =
			File::create(file).map_err(|err| format!("could not open/create file at {file:?}: {err}"))?;
		serde_json::to_writer(file, self).map_err(|err| {
			if err.is_io() {
				format!("failed to write to file: {err:?}")
			} else {
				unreachable!("serde_json error other than IO: {err:?}");
			}
		})
	}
}

impl Project {
	#[must_use]
	pub fn step_pellets(&mut self) -> Vec<Sound> {
		self.steps = self.steps.wrapping_add(1);

		self.pellets.retain_mut(|pellet| {
			pellet.advance_by(self.tempo.0 / 1000.0);
			!pellet.should_remove()
		});

		// steps run at 60 fps
		let seconds = az::cast::<_, f32>(self.steps) / 60.0;
		let minutes = seconds / 60.0;
		if minutes * self.tempo.0 > 1.0 {
			self.run_emitters();
			self.steps = 0;
		}

		self.check_collisions()
	}

	fn run_emitters(&mut self) {
		for (&pos, component) in &mut self.components {
			self.pellets.extend(
				component
					.on_emit()
					.iter()
					.map(|direction| Pellet::new_at(pos, direction)),
			);
		}
	}

	#[must_use]
	fn check_collisions(&mut self) -> Vec<Sound> {
		let mut new_pellets = vec![];
		let mut sounds = vec![];

		self.pellets.retain_mut(|pellet| {
			let pos = pellet.pos_rounded();

			if pos != pellet.immune_pos() {
				if let Some(component) = self.components.get_mut(&pos) {
					let ShouldEmit {
						sound,
						pitch,
						directions,
					} = component.on_pellet(*pellet);
					sounds.extend(sound);
					let mut directions = directions.iter();
					return if let Some(first) = directions.next() {
						*pellet = Pellet::new_at(pos, first).with_pitch(pitch);
						new_pellets
							.extend(directions.map(|direction| Pellet::new_at(pos, direction).with_pitch(pitch)));
						true
					} else {
						false
					};
				}
			}

			true
		});

		self.pellets.extend(new_pellets);

		sounds
	}
}
