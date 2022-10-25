use egui::Vec2;
use serde::{Deserialize, Serialize};

use super::Direction;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub struct Position {
	pub x: i16,
	pub y: i16,
}

impl Position {
	pub fn position_along(self, main_direction: Direction) -> i16 {
		match main_direction {
			Direction::Up | Direction::Down => self.y,
			Direction::Left | Direction::Right => self.x,
		}
	}
}

impl From<Position> for Vec2 {
	fn from(pos: Position) -> Self {
		Self {
			x: f32::from(pos.x),
			y: f32::from(pos.y),
		}
	}
}
