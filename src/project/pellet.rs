use egui::Vec2;
use serde::{Deserialize, Serialize};

use super::{Direction, Position};
use crate::sound::Pitch;

// "along direction" = x if direction is horizontal, y otherwise
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Pellet {
	direction: Direction,
	offset_along_direction: i32,
	origin: Position,
	pub pitch: Pitch,
}

const OFFSET_FIXED_POINT_FACTOR: i32 = 10000;
#[allow(clippy::cast_precision_loss)] // at compile-time
const OFFSET_FIXED_POINT_FACTOR_F: f32 = OFFSET_FIXED_POINT_FACTOR as f32;

impl Pellet {
	pub fn direction(self) -> Direction {
		self.direction
	}

	pub fn advance_by(&mut self, amount: f32) {
		self.offset_along_direction +=
			az::cast::<_, i32>(self.direction.sign() * amount * OFFSET_FIXED_POINT_FACTOR_F);
	}

	pub fn with_pitch(self, pitch: Pitch) -> Self {
		Self { pitch, ..self }
	}

	pub fn new_at(at: Position, direction: Direction) -> Self {
		Self {
			direction,
			offset_along_direction: i32::from(at.position_along(direction)) * OFFSET_FIXED_POINT_FACTOR,
			origin: at,
			pitch: Pitch::new(0),
		}
	}

	pub fn pos_rounded(self) -> Position {
		let along = i16::try_from(self.offset_along_direction / OFFSET_FIXED_POINT_FACTOR).unwrap();

		if self.direction.is_horizontal() {
			Position {
				x: along,
				y: self.origin.y,
			}
		} else {
			Position {
				x: self.origin.x,
				y: along,
			}
		}
	}

	pub fn pos_float(self) -> Vec2 {
		let along: f32 = az::cast::<_, f32>(self.offset_along_direction) / OFFSET_FIXED_POINT_FACTOR_F;

		if self.direction.is_horizontal() {
			Vec2 {
				x: along,
				y: self.origin.y.into(),
			}
		} else {
			Vec2 {
				x: self.origin.x.into(),
				y: along,
			}
		}
	}

	pub fn immune_pos(self) -> Position {
		self.origin
	}

	pub fn should_remove(self) -> bool {
		const MAX: i32 = i16::MAX as i32 * OFFSET_FIXED_POINT_FACTOR;
		const MIN: i32 = i16::MIN as i32 * OFFSET_FIXED_POINT_FACTOR;

		!(MIN..=MAX).contains(&self.offset_along_direction)
	}
}
