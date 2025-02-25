//! A simple progress bar widget for Nablo.

use crate::{layout::{Layout, LayoutId}, prelude::{Animatedf32, FillMode, InputState, Painter, Rect, Vec2, Vec4}};

use super::{styles::{CONTENT_TEXT_SIZE, DEFAULT_ROUNDING, INPUT_BACKGROUND_COLOR, PRIMARY_COLOR}, Signal, SignalGenerator, Widget};

/// A simple progress bar widget for Nablo.
pub struct ProgressBar<S: Signal> {
	/// The inner properties of the progress bar.
	pub inner: ProgressBarInner,
	/// The signals generated by the progress bar.
	pub signals: SignalGenerator<S, ProgressBarInner>,
}

/// The inner properties of the progress bar.
pub struct ProgressBarInner {
	/// The current progress of the progress bar, should be between 0.0 and 1.0
	pub progress: Animatedf32,
	/// The size of the progress bar.
	pub size: Vec2,
	/// The background color of the progress bar.
	pub background_color: FillMode,
	/// The foreground color of the progress bar.
	pub foreground_color: FillMode,
	/// The rounding of the progress bar.
	pub roundings: Vec4,
}

impl Default for ProgressBarInner {
	fn default() -> Self {
		Self {
			progress: Animatedf32::default(),
			size: Vec2::new(100.0, CONTENT_TEXT_SIZE / 2.0),
			background_color: FillMode::Color(INPUT_BACKGROUND_COLOR),
			foreground_color: FillMode::Color(PRIMARY_COLOR),
			roundings: Vec4::same(DEFAULT_ROUNDING),
		}
	}
}

impl<S: Signal> Default for ProgressBar<S> {
	fn default() -> Self {
		Self {
			inner: ProgressBarInner::default(),
			signals: SignalGenerator::default(),
		}
	}
}

impl ProgressBarInner {
	/// Sets the progress of the progress bar.
	pub fn set_progress(mut self, progress: f32) -> Self {
		self.progress.set(progress);
		self
	}
}

impl<S: Signal> ProgressBar<S> {
	/// Creates a new progress bar with default values.
	pub fn new() -> Self {
		Self::default()
	}

	/// Sets the progress of the progress bar.
	pub fn set_progress(mut self, progress: f32) -> Self {
		self.inner.progress.set(progress);
		self
	}

	/// Sets the progress of the progress bar but without animation.
	pub fn set_progress_without_animation(mut self, progress: f32) -> Self {
		self.inner.progress.set_without_animation(progress);
		self
	}

	/// Sets the size of the progress bar.
	pub fn set_size(self, size: impl Into<Vec2>) -> Self {
		Self {
			inner: ProgressBarInner {
				size: size.into(),
				..self.inner
			},
			..self
		}
	}

	/// Sets the length of the progress bar.
	pub fn set_length(self, length: f32) -> Self {
		Self {
			inner: ProgressBarInner {
				size: Vec2::new(length, self.inner.size.y),
				..self.inner
			},
			..self
		}
	}

	/// Sets the height of the progress bar.
	pub fn set_height(self, height: f32) -> Self {
		Self {
			inner: ProgressBarInner {
				size: Vec2::new(self.inner.size.x, height),
				..self.inner
			},
			..self
		}
	}

	/// Sets the background color of the progress bar.
	pub fn set_background_color(self, color: impl Into<FillMode>) -> Self {
		Self {
			inner: ProgressBarInner {
				background_color: color.into(),
				..self.inner
			},
			..self
		}
	}

	/// Sets the foreground color of the progress bar.
	pub fn set_foreground_color(self, color: impl Into<FillMode>) -> Self {
		Self {
			inner: ProgressBarInner {
				foreground_color: color.into(),
				..self.inner
			},
			..self
		}
	}

	/// Sets the rounding of the progress bar.
	pub fn set_roundings(self, roundings: impl Into<Vec4>) -> Self {
		Self {
			inner: ProgressBarInner {
				roundings: roundings.into(),
				..self.inner
			},
			..self
		}
	}
}

impl<S: Signal> Widget for ProgressBar<S> {
	type Signal = S;

	fn handle_event(&mut self, input_state: &mut InputState<Self::Signal>, id: LayoutId, area: Rect, _: Vec2) -> bool {
		self.signals.generate_signals(
			&mut self.inner, 
			input_state, 
			id, 
			area,
			false, 
			false
		);
		self.inner.progress.is_animating()
	}

	fn size(&self, _: LayoutId, _: &Painter, _: &Layout<Self::Signal>) -> Vec2 {
		self.inner.size
	}

	fn draw(&mut self, painter: &mut Painter, size: Vec2) {
		let progress = self.inner.progress.value();
		painter.set_fill_mode(self.inner.background_color.clone());
		painter.draw_rect(Rect::from_size(size), self.inner.roundings);
		painter.set_fill_mode(self.inner.foreground_color.clone());
		painter.draw_rect(Rect::from_size(Vec2::new(size.x * progress, size.y)), self.inner.roundings);
	}
}