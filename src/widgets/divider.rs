//! A widget that draws a horizontal line.

use crate::{layout::{Layout, LayoutId}, prelude::{FillMode, InputState, Painter, Rect, Vec2, Vec4}, App};

use super::{styles::CARD_BORDER_COLOR, Signal, SignalGenerator, Widget};

/// A widget that draws a horizontal or vertical line.
#[derive(Default)]
pub struct Divider<S: Signal, A: App<Signal = S>> {
	/// The inner properties of the `Divider` widget.
	pub inner: DividerInner,
	/// The signals generated by this widget.
	pub signals: SignalGenerator<S, DividerInner, A>,
	last_area: Rect,
}

/// The inner properties of the `Divider` widget.
#[derive(Clone, Debug, PartialEq)]
pub struct DividerInner {
	/// The color of the line.
	pub color: FillMode,
	/// The width of the line.
	pub width: f32,
	/// The length of the line.
	/// 
	/// If `None`, the length will be the size of the parent widget.
	pub length: Option<f32>,
	/// If `true`, the line will be vertical.
	pub vertical: bool,
	/// The padding of the widget.
	pub padding: f32,
}

impl Default for DividerInner {
	fn default() -> Self {
		Self {
			color: CARD_BORDER_COLOR.into(),
			width: 4.0,
			length: None,
			vertical: false,
			padding: 0.0,
		}
	}
}

impl<S: Signal, A: App<Signal = S>> Divider<S, A> {
	/// Creates a new `Divider` widget.
	pub fn new(vertical: bool) -> Self {
		Self {
			inner: DividerInner {
				vertical,
				..Default::default()
			},
			signals: Default::default(),
			last_area: Rect::ZERO,
		}
	}

	/// Sets the padding of the widget.
	pub fn padding(self, padding: f32) -> Self {
		Self { inner: DividerInner { padding, ..self.inner }, ..self }
	}

	/// Sets the color of the line.
	pub fn color(self, color: impl Into<FillMode>) -> Self {
		Self { inner: DividerInner { color: color.into(), ..self.inner } , ..self }
	}

	/// Sets the width of the line.
	pub fn width(self, width: f32) -> Self {
		Self { inner: DividerInner { width, ..self.inner }, ..self }
	}

	/// Sets the length of the line.
	/// 
	/// If `None`, the length will be the size of the parent widget.
	pub fn length(self, length: f32) -> Self {
		Self { inner: DividerInner { length: Some(length), ..self.inner }, ..self }
	}

	/// Sets the length of the line to the size of the parent widget.
	pub fn full_length(self) -> Self {
		Self { inner: DividerInner { length: None, ..self.inner }, ..self }
	}

	/// Sets if the line is vertical.
	pub fn vertical(self, vertical: bool) -> Self {
		Self { inner: DividerInner { vertical, ..self.inner }, ..self }
	}
}

impl<S: Signal, A: App<Signal = S>> Widget for Divider<S, A> {
	type Signal = S;
	type Application = A;

	fn handle_event(&mut self, app: &mut A, input_state: &mut InputState<S>, from: LayoutId, area: Rect, _: Vec2) -> bool {
		self.signals.generate_signals(app, &mut self.inner, input_state, from, area, false, false);
		if self.last_area != area {
			self.last_area = area;
			true
		}else {
			false
		}
	}

	fn draw(&mut self, painter: &mut Painter, size: Vec2) {
		painter.set_fill_mode(self.inner.color.clone());
		// let size = size;
		let size = size - if self.inner.vertical { Vec2::new(0.0, self.inner.padding * 2.0) } else { Vec2::new(self.inner.padding * 2.0, 0.0) };
		let pos = if self.inner.vertical { Vec2::new(0.0, self.inner.padding / 2.0) } else { Vec2::new(self.inner.padding / 2.0, 0.0) };
		// println!("pos: {}, size: {}, window_size: {}", pos, size, painter.window_size);
		painter.draw_rect(Rect::from_lt_size(pos, size), Vec4::same(self.inner.width / 2.0));
	}

	fn size(&self, id: LayoutId, painter: &Painter, layout: &Layout<S, A>) -> Vec2 {
		if let Some(length) = self.inner.length {
			if self.inner.vertical {
				return Vec2::new(length, self.inner.width);
			}else {
				return Vec2::new(self.inner.width, length);
			}
		}

		let parent_id = layout.get_parent_id(id);
		if let Some(parent_id) = parent_id {
			if let Some(inner) = layout.get_widget_area(parent_id) {
				let inner_size = inner.size().min(painter.window_size);
				let parent_padding = layout.get_widget_padding(parent_id).unwrap_or_default();
				if self.inner.vertical {
					Vec2::new(self.inner.width, inner_size.y - parent_padding.y * 2.0)
				}else {
					Vec2::new(inner_size.x - parent_padding.x * 2.0, self.inner.width)
				}
			}else {
				Vec2::ZERO
			}
		}else {
			Vec2::ZERO
		}
	}
}