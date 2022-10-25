use egui::Vec2;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, enumset::EnumSetType)]
#[enumset(no_super_impls)]
#[serde(rename_all = "snake_case")]
pub enum Direction {
	Up,
	Right,
	Down,
	Left,
}

impl Direction {
	pub fn is_horizontal(self) -> bool {
		match self {
			Self::Up | Self::Down => false,
			Self::Left | Self::Right => true,
		}
	}

	pub fn as_vec2(self) -> Vec2 {
		match self {
			Self::Up => Vec2::UP,
			Self::Down => Vec2::DOWN,
			Self::Left => Vec2::LEFT,
			Self::Right => Vec2::RIGHT,
		}
	}

	pub fn rotate90(self) -> Self {
		match self {
			Self::Up => Self::Right,
			Self::Right => Self::Down,
			Self::Down => Self::Left,
			Self::Left => Self::Up,
		}
	}

	pub fn flip(self) -> Self {
		match self {
			Self::Up => Self::Down,
			Self::Down => Self::Up,
			Self::Left => Self::Right,
			Self::Right => Self::Left,
		}
	}

	pub fn negative(self) -> bool {
		match self {
			Self::Up | Self::Left => true,
			Self::Down | Self::Right => false,
		}
	}

	pub fn sign(self) -> f32 {
		if self.negative() {
			-1.0
		} else {
			1.0
		}
	}
}
