use egui::{Color32, Painter, Pos2, Rect, Sense, Stroke, Ui, Vec2};

use crate::project::{Component, Position as ComponentPos, Project};

pub struct Editor {
	pub position: Vec2,
	pub zoom: f32,
	pub dragging: Option<Component>,
	pub drag_released_from_outer: Option<Pos2>,
}

impl Default for Editor {
	fn default() -> Self {
		Self {
			position: Vec2::ZERO,
			zoom: 1.0,
			dragging: None,
			drag_released_from_outer: None,
		}
	}
}

struct ComponentWithPosition {
	window_pos: Rect,
	component: Component,
}

impl Editor {
	pub fn show(&mut self, ui: &mut Ui, project: &mut Project) {
		let size = ui.available_size();
		let (rect, response) = ui.allocate_exact_size(size, Sense::click_and_drag());

		let (clicked, released, interact_pos) = {
			let input = response.ctx.input();
			(
				input.pointer.primary_clicked(),
				input.pointer.primary_released(),
				input.pointer.interact_pos(),
			)
		};

		if response.dragged() && clicked {
			let window_pos = response.interact_pointer_pos().unwrap();
			if let Some(component_pos) = self.window_pos_to_component_pos(window_pos, rect) {
				if let Some(component) = project.components.remove(&component_pos) {
					self.dragging = Some(component);
				}
			}
		} else if released || response.drag_released() || self.drag_released_from_outer.is_some() {
			if let Some(dragging) = self.dragging.take() {
				// `interact_pointer_pos` returns `Some` only if the pointer event was within our region.
				if let Some(window_pos) = self
					.drag_released_from_outer
					.take()
					.or_else(|| response.interact_pointer_pos())
				{
					// if the drop is outside the editor region, the user wants to get rid of the component.
					if rect.contains(window_pos) {
						if let Some(component_pos) = self.window_pos_to_component_pos(window_pos, rect) {
							project.components.insert(component_pos, dragging);
						}
					}
				}
			}
		}

		if self.dragging.is_none() {
			self.zoom(ui.input().scroll_delta.y);
			self.position += response.drag_delta() / self.zoom;
		}

		let painter = ui.painter().with_clip_rect(rect);
		painter.rect_filled(rect, 0.0, ui.style().visuals.window_fill());

		self.draw_grid(&painter, rect);

		for ComponentWithPosition {
			window_pos,
			component,
		} in self
			.components_with_positions(project, rect)
			.filter(|component| {
				// partial containment check
				rect.contains(component.window_pos.min) || rect.contains(component.window_pos.max)
			}) {
			component.draw(&painter, window_pos);
		}

		for pellet in &project.pellets {
			self.draw_pellet(&painter, pellet.pos_float(), rect);
		}

		// reset clip rect
		let painter = ui.painter();

		if let Some(dragging) = self.dragging {
			if let Some(window_pos) = interact_pos {
				dragging.draw(
					painter,
					Rect::from_center_size(window_pos, Vec2::splat(self.component_size())),
				);
			}
		}
	}

	fn draw_pellet(&self, painter: &Painter, position: Vec2, region: Rect) {
		let window_pos = self.component_pos_to_window_pos(position, region);
		painter.circle_filled(
			window_pos.center(),
			window_pos.height() * 0.1,
			Color32::BLUE,
		);
	}

	fn draw_grid(&self, painter: &Painter, region: Rect) {
		let step = self.component_size();

		let adjusted_position = self.position * self.zoom;
		let offset_from_center =
			(region.center() + adjusted_position + Vec2::splat(step / 2.0)) - region.left_top();
		let rounded_offset = Vec2 {
			x: offset_from_center.x % step,
			y: offset_from_center.y % step,
		};
		let first_intersection_pos = region.left_top() + rounded_offset;

		let stroke = Stroke::new(1.0, painter.ctx().style().visuals.faint_bg_color);

		// vertical lines
		{
			let mut x = first_intersection_pos.x;
			while x < region.right() {
				painter.line_segment(
					[Pos2::new(x, region.top()), Pos2::new(x, region.bottom())],
					stroke,
				);
				x += step;
			}
		}

		// horizontal lines
		{
			let mut y = first_intersection_pos.y;
			while y < region.bottom() {
				painter.line_segment(
					[Pos2::new(region.left(), y), Pos2::new(region.right(), y)],
					stroke,
				);
				y += step;
			}
		}
	}

	fn origin(&self, region: Rect) -> Pos2 {
		region.center() + (self.position * self.zoom)
	}

	const MIN_ZOOM: f32 = 0.4;
	const MAX_ZOOM: f32 = 3.0;
	const ZOOM_STEP: f32 = 1.005;
	fn zoom(&mut self, amount: f32) {
		let tentative = self.zoom * Self::ZOOM_STEP.powf(amount);
		self.zoom = tentative.clamp(Self::MIN_ZOOM, Self::MAX_ZOOM);
	}

	pub const RELATIVE_COMPONENT_SIZE: f32 = 40.0;
	fn component_size(&self) -> f32 {
		self.zoom * Self::RELATIVE_COMPONENT_SIZE
	}

	fn components_with_positions<'a>(
		&'a self,
		project: &'a Project,
		region: Rect,
	) -> impl Iterator<Item = ComponentWithPosition> + 'a {
		project
			.components
			.iter()
			.map(move |(&position, &component)| ComponentWithPosition {
				window_pos: self.component_pos_to_window_pos(position, region),
				component,
			})
	}

	fn window_pos_to_component_pos(&self, window_pos: Pos2, region: Rect) -> Option<ComponentPos> {
		let absolute_offset = window_pos - self.origin(region);
		let offset = absolute_offset / self.component_size();
		let rounded = offset.round();
		Some(ComponentPos {
			x: az::checked_cast(rounded.x)?,
			y: az::checked_cast(rounded.y)?,
		})
	}

	fn component_pos_to_window_pos(&self, component_pos: impl Into<Vec2>, region: Rect) -> Rect {
		let pos_f = component_pos.into();

		Rect::from_center_size(
			self.origin(region) + (pos_f * self.component_size()),
			Vec2::splat(self.component_size()),
		)
	}
}
