use std::f32::consts::TAU;

use egui::{Align2, Color32, Painter, Rect, Stroke, Vec2};
use enumset::EnumSet;
use epaint::{CubicBezierShape, PathShape};
use serde::{Deserialize, Serialize};

use super::colors::{Category, Palette};
use super::{Direction, Pellet};
use crate::sound::{Pitch, Sound, Type as SoundType};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "_type")]
pub enum Component {
	Emitter {
		direction: Direction,
	},
	RightTurn,
	Alternator {
		#[serde(default = "default_direction")]
		current_direction: Direction,
	},
	Debug,
	IncrementPitch {
		#[serde(default)]
		current: u8,
	},
	Guitar,
	Consumer,
	Half {
		state: bool,
	},
}

const fn default_direction() -> Direction {
	Direction::Up
}

pub struct ShouldEmit {
	pub sound: Option<Sound>,
	pub pitch: Pitch,
	pub directions: EnumSet<Direction>,
}

impl ShouldEmit {
	fn sound(sound: Sound) -> Self {
		Self {
			sound: Some(sound),
			pitch: Pitch::new(0),
			directions: EnumSet::empty(),
		}
	}
}

impl Component {
	pub const PALETTE_LIST: &[Self] = &[
		Self::Emitter {
			direction: Direction::Up,
		},
		Self::Consumer,
		Self::RightTurn,
		Self::Alternator {
			current_direction: default_direction(),
		},
		Self::IncrementPitch { current: 0 },
		Self::Guitar,
		Self::Half { state: false },
	];

	pub fn on_pellet(&mut self, pellet: Pellet) -> ShouldEmit {
		let directions = match self {
			Self::Emitter { .. } => EnumSet::empty(),
			Self::Consumer { .. } => EnumSet::empty(),
			Self::RightTurn => EnumSet::only(pellet.direction().rotate90()),
			Self::Alternator { current_direction } => {
				*current_direction = current_direction.rotate90();
				if *current_direction == pellet.direction().flip() {
					*current_direction = current_direction.rotate90();
				}
				EnumSet::only(*current_direction)
			}
			Self::Debug => {
				eprintln!("debug component: got pellet {pellet:?}");
				EnumSet::empty()
			}
			Self::IncrementPitch { current } => {
				let ret = ShouldEmit {
					sound: None,
					pitch: pellet.pitch.increment_by(*current),
					directions: EnumSet::only(pellet.direction()),
				};
				*current = (*current + 1) % (Pitch::MAX + 1);
				return ret;
			}
			Self::Guitar => {
				return ShouldEmit::sound(Sound {
					pitch: pellet.pitch,
					ty: SoundType::Guitar,
				})
			}
			Self::Half { state } => {
				*state = !*state;
				if *state {
					EnumSet::only(pellet.direction())
				} else {
					EnumSet::empty()
				}
			}
		};

		ShouldEmit {
			sound: None,
			pitch: pellet.pitch,
			directions,
		}
	}

	pub fn on_emit(&mut self) -> EnumSet<Direction> {
		match self {
			Self::Emitter { direction } => EnumSet::only(*direction),
			_ => EnumSet::empty(),
		}
	}

	pub fn category(self) -> Category {
		match self {
			Component::Alternator { .. } | Component::Half { .. } | Component::RightTurn => {
				Category::Routing
			}
			Component::Debug => Category::Debug,
			Component::IncrementPitch { .. } => Category::Scale,
			Component::Guitar => Category::Instrument,
			Component::Consumer | Component::Emitter { .. } => Category::Emitter,
		}
	}

	pub fn draw(self, painter: &Painter, window_pos: Rect) {
		const MAIN_SIZE_FACTOR: f32 = 0.9;
		let main_size = window_pos.height() * MAIN_SIZE_FACTOR;
		let main_rect = window_pos.shrink(window_pos.height() * (1.0 - MAIN_SIZE_FACTOR));

		let Palette {
			background,
			foreground,
		} = self.category().palette();

		let center = main_rect.center();
		painter.circle_filled(center, main_size / 2.0, background);

		let offset = main_size * 0.3;

		match self {
			Self::Emitter { direction } => {
				let movement = direction.as_vec2() * main_size * 0.4;
				let indicator_pos = center + movement;
				let stroke = Stroke::new(main_size * 0.05, foreground);
				painter.circle_stroke(center, main_size * 0.35, stroke);
				painter.circle_stroke(center, main_size * 0.225, stroke);
				painter.circle_stroke(center, main_size * 0.1, stroke);
				painter.line_segment([center, indicator_pos], (main_size * 0.1, background));
				painter.line_segment([center, indicator_pos], stroke);
			}
			Self::Consumer => {
				let stroke = Stroke::new(main_size * 0.15, foreground);
				let offset = offset / std::f32::consts::SQRT_2;
				painter.line_segment(
					[
						center + Vec2::new(-offset, -offset),
						center + Vec2::new(offset, offset),
					],
					stroke,
				);
				painter.line_segment(
					[
						center + Vec2::new(offset, -offset),
						center + Vec2::new(-offset, offset),
					],
					stroke,
				);
			}
			Self::RightTurn => {
				let stroke = Stroke::new(main_size * 0.2, foreground);
				painter.add(PathShape::line(
					vec![
						center + Vec2::DOWN * offset,
						center,
						center + Vec2::RIGHT * offset,
					],
					stroke,
				));
			}
			Self::Alternator { .. } => {
				painter.add(epaint::PathShape::convex_polygon(
					[
						Direction::Up,
						Direction::Right,
						Direction::Down,
						Direction::Left,
					]
					.into_iter()
					.map(|direction| center + direction.as_vec2() * offset)
					.collect(),
					Color32::TRANSPARENT,
					(main_size * 0.2, foreground),
				));
			}
			Self::Debug => {}
			Self::IncrementPitch { .. } => {
				let center = center + Vec2::DOWN * main_size * 0.03;
				painter.add(epaint::PathShape::convex_polygon(
					vec![
						center + Vec2::angled(TAU * -0.25) * offset,
						center + Vec2::angled(TAU * (1.0 / 3.0 - 0.25)) * offset,
						center + Vec2::angled(TAU * (2.0 / 3.0 - 0.25)) * offset,
					],
					foreground,
					Stroke::none(),
				));
			}
			Self::Guitar => {
				painter.text(
					center,
					Align2::CENTER_CENTER,
					"G",
					epaint::FontId::proportional(main_size),
					foreground,
				);
			}
			Self::Half { .. } => {
				let icon_radius = offset;
				painter.circle_stroke(center, icon_radius, (main_size * 0.1, foreground));
				// this is an approximation of a half-circle. it works well enough.
				painter.add(CubicBezierShape {
					points: [
						center + Vec2::new(-icon_radius, 0.0),
						center + Vec2::new(-icon_radius, icon_radius * 1.3),
						center + Vec2::new(icon_radius, icon_radius * 1.3),
						center + Vec2::new(icon_radius, 0.0),
					],
					closed: true,
					fill: foreground,
					stroke: Stroke::none(),
				});
			}
		}
	}

	pub fn name(self) -> &'static str {
		match self {
			Self::Emitter { .. } => "Emitter",
			Self::Consumer => "Consumer",
			Self::Alternator { .. } => "Alternator",
			Self::Guitar => "Guitar",
			Self::Debug => "Super Secret Debug Component",
			Self::RightTurn => "Right Turn",
			Self::IncrementPitch { .. } => "Rising Pitch",
			Self::Half { .. } => "Half",
		}
	}
}
