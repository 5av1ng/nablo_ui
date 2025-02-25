//! re-exported widgets for convenience 

pub use crate::widgets::card::*;
pub use crate::widgets::*;
pub use crate::widgets::styles::*;
pub use crate::widgets::button::*;
pub use crate::widgets::label::*;
pub use crate::widgets::canvas::*;
pub use crate::widgets::collapse::*;
pub use crate::widgets::divider::*;
pub use crate::widgets::reactive::*;
pub use crate::widgets::inputbox::*;
pub use crate::widgets::radio::*;
pub use crate::widgets::slider::*;
pub use crate::widgets::draggable_value::*;
pub use crate::widgets::progress_bar::*;
pub use crate::widgets::floating_container::*;

macro_rules! deligate_signal_generator {
	($($widget: ty, $style: ty),* $(,)?) => {
		$(
			impl<S: Signal> $widget {
				/// Add a click signal to the widget.
				pub fn on_click(mut self, signal: impl Fn(&mut $style) -> S + 'static) -> Self {
					self.signals = self.signals.on_click(signal);
					self
				}

				/// Remove the click signal from the widget.
				pub fn remove_on_click(mut self) -> Self {
					self.signals = self.signals.remove_on_click();
					self
				}

				/// Add a pressed signal to the widget.
				pub fn on_pressed(mut self, signal: impl Fn(&mut $style) -> S + 'static) -> Self {
					self.signals = self.signals.on_pressed(signal);
					self
				}

				/// Remove the pressed signal from the widget.
				pub fn remove_on_pressed(mut self) -> Self {
					self.signals = self.signals.remove_on_pressed();
					self
				}

				/// Add a released signal to the widget.
				pub fn on_released(mut self, signal: impl Fn(&mut $style) -> S + 'static) -> Self {
					self.signals = self.signals.on_released(signal);
					self
				}

				/// Remove the released signal from the widget.
				pub fn remove_on_released(mut self) -> Self {
					self.signals = self.signals.remove_on_released();
					self
				}

				/// Add a hover signal to the widget.
				pub fn on_hover(mut self, signal: impl Fn(&mut $style) -> S + 'static) -> Self {
					self.signals = self.signals.on_hover(signal);
					self
				}

				/// Remove the hover signal from the widget.
				pub fn remove_on_hover(mut self) -> Self {
					self.signals = self.signals.remove_on_hover();
					self
				}

				/// Add an unhover signal to the widget.
				pub fn on_unhover(mut self, signal: impl Fn(&mut $style) -> S + 'static) -> Self {
					self.signals = self.signals.on_unhover(signal);
					self
				}

				/// Remove the unhover signal from the widget.
				pub fn remove_on_unhover(mut self) -> Self {
					self.signals = self.signals.remove_on_unhover();
					self
				}

				/// Add a drag signal to the widget.
				pub fn on_drag(mut self, signal: impl Fn(&mut $style, Vec2) -> S + 'static) -> Self {
					self.signals = self.signals.on_drag(signal);
					self
				}

				/// Remove the drag signal from the widget.
				pub fn remove_on_drag(mut self) -> Self {
					self.signals = self.signals.remove_on_drag();
					self
				}

				/// Add a double click signal to the widget.
				pub fn on_double_click(mut self, signal: impl Fn(&mut $style) -> S + 'static) -> Self {
					self.signals = self.signals.on_double_click(signal);
					self
				}

				/// Remove the double click signal from the widget.
				pub fn remove_on_double_click(mut self) -> Self {
					self.signals = self.signals.remove_on_double_click();
					self
				}
			}
		)*
	};
}

deligate_signal_generator!{ 
	Label<S>, LabelInner,
	Canvas<S>, CanvasInner,
	Button<S>, ButtonInner,
	Divider<S>, DividerInner,
	Card<S>, CardInner,
	Collapse<S>, CollapseInner,
	InputBox<S>, InputBoxInner,
	Radio<S>, RadioInner,
	Slider<S>, SliderInner,
	DraggableValue<S>, DraggableValueInner,
	ProgressBar<S>, ProgressBarInner,
	FloatingContainer<S>, FloatingContainerInner,
}