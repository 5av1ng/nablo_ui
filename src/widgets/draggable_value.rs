//! A widget that can be dragged to change its value.

use crate::{layout::{Layout, LayoutId}, prelude::{Animatedf32, FillMode, FontId, InputState, Painter, Rect, Vec2, Vec4}};

use super::{styles::{BRIGHT_FACTOR, CONTENT_TEXT_SIZE, DEFAULT_PADDING, DEFAULT_ROUNDING, INPUT_BACKGROUND_COLOR, INPUT_BORDER_COLOR, SECONDARY_TEXT_COLOR}, Signal, SignalGenerator, Widget};

/// A draggable value widget.
pub struct DraggableValue<S: Signal> {
	/// The inner properties of the draggable value widget.
	pub inner: DraggableValueInner,
	/// The signal to emit when the value changes.
	pub signals: SignalGenerator<S, DraggableValueInner>,
	hover_factor: Animatedf32,
	pressed_factor: Animatedf32,
}

/// The inner properties of the draggable value widget.
#[derive(Clone, Debug, PartialEq)]
pub struct DraggableValueInner {
	/// The current value of the draggable value widget.
	pub value: f32,
	/// The minimum value of the draggable value widget.
	pub min: f32,
	/// The maximum value of the draggable value widget.
	pub max: f32,
	/// Whether the slider is logarithmic.
	pub is_logarithmic: bool,
	/// The background color of the draggable value widget.
	pub background_color: FillMode,
	/// The border color of the draggable value widget.
	pub border_color: FillMode,
	/// The prefix of the draggable value widget.
	pub prefix: String,
	/// The suffix of the draggable value widget.
	pub suffix: String,
	/// The font size of the draggable value widget.
	pub font_size: f32,
	/// The font color of the draggable value widget.
	pub font_color: FillMode,
	/// The font of the draggable value widget.
	pub font: FontId,
	/// The padding of the draggable value widget.
	pub padding: Vec2,
	/// The number of decimal places to display.
	pub decimal_places: usize,
	/// The drag speed of the draggable value widget.
	pub speed: f32,
	/// The rounding of the draggable value widget.
	pub rounding: Vec4,
}

impl Default for DraggableValueInner {
	fn default() -> Self {
		Self {
			value: 0.0,
			min: 0.0,
			max: 1.0,
			is_logarithmic: false,
			background_color: FillMode::Color(INPUT_BACKGROUND_COLOR),
			border_color: FillMode::Color(INPUT_BORDER_COLOR),
			prefix: "".to_string(),
			suffix: "".to_string(),
			font_size: CONTENT_TEXT_SIZE,
			font_color: FillMode::Color(SECONDARY_TEXT_COLOR),
			font: 0,
			padding: Vec2::same(DEFAULT_PADDING),
			decimal_places: 2,
			speed: 0.01,
			rounding: Vec4::same(DEFAULT_ROUNDING)
		}
	}
}

impl<S: Signal> Default for DraggableValue<S> {
	fn default() -> Self {
		Self {
			inner: DraggableValueInner::default(),
			signals: SignalGenerator::default(),
			hover_factor: Animatedf32::default(),
			pressed_factor: Animatedf32::default(),
		}
	}
}

impl<S: Signal> DraggableValue<S> {
	/// Creates a new draggable value widget.
	/// 
	/// # Panics
	/// 
	/// Panics if `min` is greater than `max`.
	/// Panics if `value` is outside the range [`min`, `max`].
	pub fn new(value: f32, min: f32, max: f32) -> Self {
		assert!(min <= max, "min must be less than or equal to max");
		assert!(value >= min && value <= max, "value must be within the range [min, max]");
		let decimal_places = ((max - min) / 100.0).log10().floor() as usize;
		// let speed = 0.01;
		Self {
			inner: DraggableValueInner {
				value,
				min,
				max,
				decimal_places,
				// speed,
				..DraggableValueInner::default()
			},
			..Default::default()
		}
	}

	/// Sets the minimum value of the slider.
	pub fn min(self, min: f32) -> Self {
		Self {
			inner: DraggableValueInner { min, ..self.inner },
			..self
		}
	}

	/// Sets the maximum value of the slider.
	pub fn max(self, max: f32) -> Self {
		Self {
			inner: DraggableValueInner { max, ..self.inner },
			..self
		}
	}

	/// Set whether the slider is logarithmic.
	pub fn logarithmic(self, is_logarithmic: bool) -> Self {
		Self {
			inner: DraggableValueInner { is_logarithmic, ..self.inner },
			..self
		}
	}

	/// Sets the background color of the draggable value widget.
	pub fn background_color(self, background_color: impl Into<FillMode>) -> Self {
		Self {
			inner: DraggableValueInner { background_color: background_color.into(), ..self.inner },
			..self
		}
	}

	/// Sets the border color of the draggable value widget.
	pub fn border_color(self, border_color: impl Into<FillMode>) -> Self {
		Self {
			inner: DraggableValueInner { border_color: border_color.into(), ..self.inner },
			..self
		}
	}

	/// Sets the prefix of the draggable value widget.
	pub fn prefix(self, prefix: impl Into<String>) -> Self {
		Self {
			inner: DraggableValueInner { prefix: prefix.into(), ..self.inner },
			..self
		}
	}

	/// Sets the suffix of the draggable value widget.
	pub fn suffix(self, suffix: impl Into<String>) -> Self {
		Self {
			inner: DraggableValueInner { suffix: suffix.into(), ..self.inner },
			..self
		}
	}

	/// Sets the font size of the draggable value widget.
	pub fn font_size(self, font_size: f32) -> Self {
		Self {
			inner: DraggableValueInner { font_size, ..self.inner },
			..self
		}
	}

	/// Sets the font color of the draggable value widget.
	pub fn font_color(self, font_color: impl Into<FillMode>) -> Self {
		Self {
			inner: DraggableValueInner { font_color: font_color.into(), ..self.inner },
			..self
		}
	}

	/// Sets the font of the draggable value widget.
	pub fn font(self, font: FontId) -> Self {
		Self {
			inner: DraggableValueInner { font, ..self.inner },
			..self
		}
	}

	/// Sets the padding of the draggable value widget.
	pub fn padding(self, padding: impl Into<Vec2>) -> Self {
		Self {
			inner: DraggableValueInner { padding: padding.into(), ..self.inner },
			..self
		}
	}

	/// Sets the number of decimal places to display.
	pub fn decimal_places(self, decimal_places: usize) -> Self {
		Self {
			inner: DraggableValueInner { decimal_places, ..self.inner },
			..self
		}
	}

	/// Sets the drag speed of the draggable value widget.
	pub fn speed(self, speed: f32) -> Self {
		Self {
			inner: DraggableValueInner { speed, ..self.inner },
			..self
		}
	}

	/// Sets the rounding of the draggable value widget.
	pub fn rounding(self, rounding: impl Into<Vec4>) -> Self {
		Self {
			inner: DraggableValueInner { rounding: rounding.into(), ..self.inner },
			..self
		}
	}
}

impl<S: Signal> Widget for DraggableValue<S> {
	type Signal = S;

	fn size(&self, _: LayoutId, painter: &Painter, _: &Layout<Self::Signal>) -> Vec2 {
		let text_to_draw = format!("{}{:.3$}{}", 
			self.inner.prefix, 
			self.inner.value, 
			self.inner.suffix, 
			self.inner.decimal_places
		);

		let text_size = painter.text_size(self.inner.font, self.inner.font_size, text_to_draw).unwrap_or_default();

		text_size + 2.0 * self.inner.padding
	}

	fn draw(&mut self, painter: &mut Painter, size: Vec2) {
		let bright_factor = BRIGHT_FACTOR * (self.hover_factor.value() - self.pressed_factor.value()).max(0.0); 

		let text_to_draw = format!("{}{:.3$}{}", 
			self.inner.prefix, 
			self.inner.value, 
			self.inner.suffix, 
			self.inner.decimal_places
		);

		let mut backgound_color = self.inner.background_color.clone();
		let mut border_color = self.inner.border_color.clone();
		let mut font_color = self.inner.font_color.clone();

		backgound_color.brighter(bright_factor);
		border_color.brighter(bright_factor);
		font_color.brighter(bright_factor);

		painter.set_fill_mode(backgound_color);
		painter.draw_rect(Rect::from_size(size), self.inner.rounding);
		painter.set_fill_mode(border_color);
		let stroke_width = 1.5;
		painter.draw_stroked_rect(Rect::from_size(size).shrink(Vec2::same(stroke_width / 2.0)), self.inner.rounding, stroke_width);

		painter.set_fill_mode(font_color);
		painter.draw_text(self.inner.padding, self.inner.font, self.inner.font_size, text_to_draw);
	}

	fn handle_event(&mut self, input_state: &mut InputState<Self::Signal>, from: LayoutId, area: Rect, _: Vec2) -> bool {
		let res = self.signals.generate_signals(&mut self.inner, input_state, from, area, true, true);
		
		if input_state.any_touch_pressing_on(area) {
			self.hover_factor.set(1.0);
		}else {
			self.hover_factor.set(0.0);
		}
		
		if input_state.any_touch_pressing_on(area) && input_state.is_any_touch_pressed() {
			self.pressed_factor.set(1.0);
		}

		if input_state.is_any_touch_released() {
			self.pressed_factor.set(0.0);
		}

		let changed = if let Some(delta) = res.drag_delta {
			let step = delta.x * self.inner.speed;
			let step = if self.inner.is_logarithmic {
				step * (self.inner.max.log10() - self.inner.min.log10())
			}else {
				step * (self.inner.max - self.inner.min)
			};
			self.inner.value = if self.inner.is_logarithmic {
				10.0_f32.powf(self.inner.value.log10() + step)
			}else {
				self.inner.value + step
			};
			self.inner.value = self.inner.value.clamp(self.inner.min, self.inner.max);
			delta.x != 0.0
		}else {
			false
		};

		self.hover_factor.is_animating() || self.pressed_factor.is_animating() || changed
	}
}