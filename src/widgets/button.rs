//! Button widget implementation.

use crate::{layout::{Layout, LayoutId}, prelude::{Animatedf32, InputState, Rect, Vec2, Vec4}, render::{font::FontId, painter::Painter, shape::FillMode}};

use super::{styles::{BRIGHT_FACTOR, CONTENT_TEXT_SIZE, DEFAULT_PADDING, DEFAULT_ROUNDING, DISABLE_COLOR, DISABLE_TEXT_COLOR, PRIMARY_COLOR, PRIMARY_TEXT_COLOR, TITLE_TEXT_SIZE}, Signal, SignalGenerator, Widget};

/// Button widget.
pub struct Button<S: Signal> {
	/// Button's inner properties.
	pub inner: ButtonInner,
	/// Button's signal generator.
	pub signals: SignalGenerator<S, ButtonInner>,
	hover_factor: Animatedf32,
	pressed_factor: Animatedf32,
	clicked_factor: Animatedf32,
}

/// Button's inner properties.
#[derive(Debug, PartialEq)]
pub struct ButtonInner {
	/// Button label.
	pub label: String,
	/// Button's style.
	pub style: ButtonStyle,
	/// Button's font size.
	pub size: ButtonSize,
	/// Button's font.
	pub font: FontId,
	/// Button's padding.
	pub padding: Vec2,
	/// Button's rounding.
	pub rounding: Vec4,
}

impl Default for ButtonInner {
	fn default() -> Self {
		Self {
			label: String::new(),
			style: ButtonStyle::default(),
			size: ButtonSize::default(),
			padding: Vec2::same(DEFAULT_PADDING),
			rounding: Vec4::same(DEFAULT_ROUNDING),
			font: 0,
		}
	}
}

impl<S: Signal> Default for Button<S> {
	fn default() -> Self {
		Self {
			inner: ButtonInner::default(),
			signals: SignalGenerator::default(),
			hover_factor: Animatedf32::default(),
			pressed_factor: Animatedf32::default(),
			clicked_factor: Animatedf32::default(),
		}
	}
}

/// Button's style.
#[derive(Debug, PartialEq, Default)]
pub enum ButtonStyle {
	#[default] Primary,
	Secondary,
	Text,
	Disabled,
	/// Custom style with background and text colors.
	Custom {
		/// Button's background color.
		background: FillMode, 
		/// Button's text color.
		text: FillMode, 
		/// Button's border width.
		width: Option<f32>
	},
}

/// Button's font size.
#[derive(Debug, PartialEq, Default)]
pub enum ButtonSize {
	Tiny,
	#[default] Small,
	Medium,
	Large,
	Custom(f32),
}

impl<S: Signal> Button<S> {
	/// Creates a new button with the given label and size.
	pub fn new(label: impl Into<String>) -> Self {
		Self {
			inner: ButtonInner {
				label: label.into(),
				..Default::default()
			},
			..Default::default()
		}
	}

	/// Sets the button's label.
	pub fn label(self, label: impl Into<String>) -> Self {
		Self {
			inner: ButtonInner {
				label: label.into(),
				..self.inner
			},
			..self
		}
	}

	/// Sets the button's style.
	pub fn style(self, style: ButtonStyle) -> Self {
		Self {
			inner: ButtonInner {
				style,
				..self.inner
			},
			..self
		}
	}

	/// Sets the button's size.
	pub fn set_size(self, size: ButtonSize) -> Self {
		Self {
			inner: ButtonInner {
				size,
				..self.inner
			},
			..self
		}
	}

	/// Sets the button's font.
	pub fn font(self, font: FontId) -> Self {
		Self {
			inner: ButtonInner {
				font,
				..self.inner
			},
			..self
		}
	}

	/// Sets the button's padding.
	pub fn padding(self, padding: Vec2) -> Self {
		Self {
			inner: ButtonInner {
				padding,
				..self.inner
			},
			..self
		}
	}

	/// Sets the button's rounding.
	pub fn rounding(self, rounding: Vec4) -> Self {
		Self {
			inner: ButtonInner {
				rounding,
				..self.inner
			},
			..self
		}
	}

	pub fn calc_size(&self, painter: &Painter) -> Vec2 {
		let font_size = match self.inner.size {
			ButtonSize::Tiny => CONTENT_TEXT_SIZE * 0.75,
			ButtonSize::Small => CONTENT_TEXT_SIZE,
			ButtonSize::Medium => TITLE_TEXT_SIZE * 0.75,
			ButtonSize::Large => TITLE_TEXT_SIZE,
			ButtonSize::Custom(size) => size,
		};

		let text_size = painter.text_size(self.inner.font, font_size, &self.inner.label).unwrap_or_default();
		text_size + self.inner.padding * 2.0
	}
}

impl<S: Signal> Widget for Button<S> {
	type Signal = S;

	fn draw(&mut self, painter: &mut Painter, _: Vec2) {
		let size = self.calc_size(painter);
		let font_size = match self.inner.size {
			ButtonSize::Tiny => CONTENT_TEXT_SIZE * 0.75,
			ButtonSize::Small => CONTENT_TEXT_SIZE,
			ButtonSize::Medium => TITLE_TEXT_SIZE * 0.75,
			ButtonSize::Large => TITLE_TEXT_SIZE,
			ButtonSize::Custom(size) => size,
		};

		let text_size = painter.text_size(self.inner.font, font_size, &self.inner.label).unwrap_or_default();
		// println!("size: {}, text_size: {}", size, text_size);
		let bright_factor = self.hover_factor.value() * BRIGHT_FACTOR - self.pressed_factor.value() * BRIGHT_FACTOR;
		let text_pos = (size - text_size) / 2.0;

		let (mut text_color, mut background_color) = match &self.inner.style {
			ButtonStyle::Disabled => {
				let mut fill = FillMode::from(DISABLE_COLOR);
				fill.brighter(bright_factor);
				painter.set_fill_mode(fill.clone());
				painter.draw_rect(Rect::from_size(size), self.inner.rounding);
				(FillMode::from(DISABLE_TEXT_COLOR), fill)
			},
			ButtonStyle::Primary => {
				let mut fill = FillMode::from(PRIMARY_COLOR);
				fill.brighter(bright_factor);
				painter.set_fill_mode(fill.clone());
				painter.draw_rect(Rect::from_size(size), self.inner.rounding);
				(FillMode::from(PRIMARY_TEXT_COLOR), fill)
			},
			ButtonStyle::Secondary => {
				let mut fill = FillMode::from(PRIMARY_COLOR);
				fill.brighter(bright_factor);
				painter.set_fill_mode(fill.clone());
				painter.draw_stroked_rect(Rect::from_size(size).shrink(Vec2::same(0.75)), self.inner.rounding, 1.5);
				(FillMode::from(PRIMARY_COLOR), fill)
			},
			ButtonStyle::Text => {
				let t = self.hover_factor.value();
				let fill = FillMode::from(t * PRIMARY_COLOR + (1.0 - t) * PRIMARY_TEXT_COLOR);
				(fill, PRIMARY_COLOR.into())
			},
			ButtonStyle::Custom{ background, text, width } => {
				let mut fill = background.clone();
				fill.brighter(bright_factor);
				painter.set_fill_mode(fill.clone());
				if let Some(width) = width {
					painter.draw_stroked_rect(Rect::from_size(size).shrink(Vec2::same(*width / 2.0)), self.inner.rounding, *width);
				}else {
					painter.draw_rect(Rect::from_size(size), self.inner.rounding);
				}
				(text.clone(), fill)
			}
		};

		text_color.brighter(bright_factor);
		if self.clicked_factor.is_animating() {
			let click_factor = self.clicked_factor.value();
			background_color.mul_alpha(1.0 - click_factor);
			painter.set_fill_mode(background_color);
			painter.draw_rect(Rect::from_size(size), self.inner.rounding);
		}

		painter.set_fill_mode(text_color);
		painter.draw_text(text_pos, self.inner.font, font_size, &self.inner.label);
	}

	fn size(&self, _: LayoutId, painter: &Painter, _: &Layout<Self::Signal>) -> Vec2 {
		self.calc_size(painter)
	}

	fn handle_event(&mut self, input_state: &mut InputState<Self::Signal>, id: LayoutId, area: Rect, _: Vec2) -> bool {
		let mouse_pos = input_state.touch_positions();
		let mouse_over = mouse_pos.iter().any(|pos| area.contains(*pos));

		if matches!(&self.inner.style, ButtonStyle::Disabled) {
			if mouse_over {
				// input_state.set_cursor_icon(CursorIcon::NotAllowed);
			}else {
				// input_state.set_cursor_icon(CursorIcon::Default);
			}
			return false;
		}

		if mouse_over {
			self.hover_factor.set(1.0);
			// input_state.set_cursor_icon(CursorIcon::Pointer);
		}else {
			// input_state.set_cursor_icon(CursorIcon::Default);
			self.hover_factor.set(0.0);
		}

		if mouse_over && input_state.is_any_touch_pressed() {
			self.pressed_factor.set(1.0);
		}

		if input_state.is_any_touch_released() {
			self.pressed_factor.set(0.0);
		}

		if self.signals.generate_signals(&mut self.inner, input_state, id, area, false, false).is_clicked {
			self.clicked_factor.set_start(0.0);
			self.clicked_factor.set(1.0);
		}


		self.hover_factor.is_animating() || self.pressed_factor.is_animating() || self.clicked_factor.is_animating()
	}
}