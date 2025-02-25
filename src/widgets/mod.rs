//! Widgets module.
//! 
//! For convenience, the `prelude` module is included, which re-exports all the types and functions from this module.

pub mod button;
pub mod canvas;
pub mod card;
pub mod collapse;
pub mod divider;
pub mod draggable_value;
pub mod inputbox;
pub mod label;
pub mod progress_bar;
pub mod radio;
pub mod slider;
pub mod styles;
pub mod floating_container;

pub mod reactive;

pub mod prelude;

use std::{any::Any, collections::HashMap};

use indexmap::IndexMap;
use time::Duration;

use crate::{layout::{Layout, LayoutId}, math::{rect::Rect, vec2::Vec2}, render::painter::Painter, window::input_state::InputState};

pub const DOUBLE_CLICK_THRESHOLD: Duration = Duration::milliseconds(250);

/// The main trait for all widgets.
/// 
/// You can implement this trait for your own widgets.
/// So you can use your own widgets in your UI.
/// 
/// The widget will not be dropped until the element is removed from the layout.
/// Therefore you can safely store any data in the widget.
pub trait Widget: Any {
	type Signal: Signal;

	/// Handle window events. 
	/// 
	/// Return `true` if you need to redraw the UI.
	/// 
	/// The `area` is the absolote viewport of the widget,
	/// The `pos` is the position of the widget's left top absolute position.
	/// 
	/// The area may be smaller than the widget size. Since we may have scrolled the viewport,
	fn handle_event(&mut self, input_state: &mut InputState<Self::Signal>, id: LayoutId, area: Rect, pos: Vec2) -> bool;

	/// Draw the widget.
	/// 
	/// The origin of the widget is the left top corner of the layout.
	/// You can get absolute position by call [`Painter::releative_to()`].
	/// 
	/// You can use [`Painter::clip_rect()`] to get the current clip rect.
	fn draw(&mut self, painter: &mut Painter, size: Vec2);

	/// Get the size of the widget.
	fn size(&self, id: LayoutId, painter: &Painter, layout: &Layout<Self::Signal>) -> Vec2;

	/// Handle child layout, if any.
	/// 
	/// By default, this method will not put any child layout, which means the widget will not be able to have child widgets.
	/// If you'd like to make a container widget, you can override this method to handle child layout.
	/// You need to return the area allocated for the child widget relative to the left top corner of the parent widget.
	/// 
	/// You can include the rect of the parent widget in the output to specify the clip rect of the child widget.
	/// Otherwise, the child widget will be drawn inside of the parent widget.
	/// 
	/// The `childs` is a map of the child layout id and its size which is sorted in the order of adding time.
	/// Will automatically using the cooridnate system of the parent widget.
	/// 
	/// You need to return the area allocated for the child widget relative to the left top corner of the parent widget.
	/// return empty map if you don't want to handle child layout.
	/// 
	/// If you returned `None`, the child will be removed from the layout.
	/// 
	/// Note: You needn't to return all the childs, only the childs that you want to handle.
	fn handle_child_layout(&mut self, childs: IndexMap<LayoutId, Vec2>, area: Rect, id: LayoutId) -> HashMap<LayoutId, Option<Rect>> {
		let _ = (childs, area, id);
		HashMap::new()
	}

	/// Get the padding of the widget.
	/// 
	/// Usful for creating widgets like dividers.
	fn inner_padding(&self) -> Vec2 {
		Vec2::ZERO
	}
}

/// The main trait for all signals.
pub trait Signal: Send + Sync + 'static {}

impl Signal for () {}

impl<T: Signal> Signal for Option<T> {}

impl<S: Signal> dyn Widget<Signal = S> {
	/// Get concrete reference type of the widget.
	pub fn downcast_ref<T: Widget<Signal = S> + Any>(&self) -> Option<&T> {
		if self.type_id() == std::any::TypeId::of::<T>() {
			Some(unsafe { &*(self as *const dyn Widget<Signal = S> as *const T) })
		} else {
			None
		}
	}

	/// Check if the widget is of the specified type.
	pub fn is<T: Widget<Signal = S> + Any>(&self) -> bool {
		self.type_id() == std::any::TypeId::of::<T>()
	}
}

/// A wrapper for signals.
pub struct SignalWrapper<S: Signal> {
	/// The wrapped signal.
	pub signal: S,
	/// The sender of the signal.
	pub from: LayoutId,
}

/// Callbacks that can lead to a signal.
/// 
/// Defined for convenience.
/// 
/// The callback will contains muttable part of the widget during runtime, usually the style of the widget.
/// 
/// To get full part of the widget, use [`crate::widgets::reactive::Reactive`] instead.
#[allow(clippy::type_complexity)]
pub struct SignalGenerator<S: Signal, T> {
	/// The signal to be generated when the widget is clicked.
	pub on_click: Option<Box<dyn Fn(&mut T) -> S>>,
	/// The signal to be generated when the widget is pressed.
	pub on_pressed: Option<Box<dyn Fn(&mut T) -> S>>,
	/// The signal to be generated when the widget is released.
	pub on_released: Option<Box<dyn Fn(&mut T) -> S>>,
	/// The signal to be generated when the widget is hovered.
	pub on_hover: Option<Box<dyn Fn(&mut T) -> S>>,
	/// The signal to be generated when the widget is unhovered.
	pub on_unhover: Option<Box<dyn Fn(&mut T) -> S>>,
	/// The signal to be generated when the widget is dragged.
	/// 
	/// Also contains the scroll event,
	/// Will construct a signal with the scroll delta.
	pub on_drag: Option<Box<dyn Fn(&mut T, Vec2) -> S>>,
	/// The signal to be generated when the widget is double clicked.
	/// 
	/// Note: you need to set [`Self::on_click`] to use this correctly.
	pub on_double_click: Option<Box<dyn Fn(&mut T) -> S>>,
	last_click_time: Option<Duration>,
	dragging_by: Option<u64>,
	is_hovering: bool,
}

/// Result of the signal generation.
pub struct SignalGeneratorResult {
	/// Whether the widget is clicked.
	pub is_clicked: bool,
	/// The drag delta of the widget.
	pub drag_delta: Option<Vec2>,
}

impl<S: Signal, T> Default for SignalGenerator<S, T> {
	fn default() -> Self {
		Self {
			on_click: None,
			on_pressed: None,
			on_released: None,
			on_hover: None,
			on_unhover: None,
			on_drag: None,
			on_double_click: None,
			dragging_by: None,
			is_hovering: false,
			last_click_time: None,
		}
	}
}

impl<S: Signal, T> SignalGenerator<S, T> {
	/// Set the signal to be generated when the widget is clicked.
	pub fn on_click(self, signal: impl Fn(&mut T) -> S + 'static) -> Self {
		Self {
			on_click: Some(Box::new(signal)),
			..self
		}
	}

	/// Remove the signal to be generated when the widget is clicked.
	pub fn remove_on_click(self) -> Self {
		Self {
			on_click: None,
			..self
		}
	}

	/// Set the signal to be generated when the widget is pressed.
	pub fn on_pressed(self, signal: impl Fn(&mut T) -> S + 'static) -> Self {
		Self {
			on_pressed: Some(Box::new(signal)),
			..self
		}
	}

	/// Remove the signal to be generated when the widget is pressed.
	pub fn remove_on_pressed(self) -> Self {
		Self {
			on_pressed: None,
			..self
		}
	}

	/// Set the signal to be generated when the widget is released.
	pub fn on_released(self, signal: impl Fn(&mut T) -> S + 'static) -> Self {
		Self {
			on_released: Some(Box::new(signal)),
			..self
		}
	}

	/// Remove the signal to be generated when the widget is released.
	pub fn remove_on_released(self) -> Self {
		Self {
			on_released: None,
			..self
		}
	}

	/// Set the signal to be generated when the widget is hovered.
	pub fn on_hover(self, signal: impl Fn(&mut T) -> S + 'static) -> Self {
		Self {
			on_hover: Some(Box::new(signal)),
			..self
		}
	}

	/// Remove the signal to be generated when the widget is hovered.
	pub fn remove_on_hover(self) -> Self {
		Self {
			on_hover: None,
			..self
		}
	}

	/// Set the signal to be generated when the widget is unhovered.
	pub fn on_unhover(self, signal: impl Fn(&mut T) -> S + 'static) -> Self {
		Self {
			on_unhover: Some(Box::new(signal)),
			..self
		}
	}

	/// Remove the signal to be generated when the widget is unhovered.
	pub fn remove_on_unhover(self) -> Self {
		Self {
			on_unhover: None,
			..self
		}
	}

	/// Set the signal to be generated when the widget is dragged.
	pub fn on_drag(self, signal: impl Fn(&mut T, Vec2) -> S + 'static) -> Self {
		Self {
			on_drag: Some(Box::new(signal)),
			..self
		}
	}

	/// Remove the signal to be generated when the widget is dragged.
	pub fn remove_on_drag(self) -> Self {
		Self {
			on_drag: None,
			..self
		}
	}

	/// Set the signal to be generated when the widget is double clicked.
	pub fn on_double_click(self, signal: impl Fn(&mut T) -> S + 'static) -> Self {
		Self {
			on_double_click: Some(Box::new(signal)),
			..self
		}
	}

	/// Remove the signal to be generated when the widget is double clicked.
	pub fn remove_on_double_click(self) -> Self {
		Self {
			on_double_click: None,
			..self
		}
	}

	/// Generate signals based on the input state.
	pub fn generate_signals(
		&mut self, 
		style: &mut T,
		input_state: &mut InputState<S>, 
		from: LayoutId, 
		area: Rect,
		mut force_clickable: bool,
		force_draggable: bool,
	) -> SignalGeneratorResult {
		let touch_positions = input_state.touch_positions();
		let contains_mouse = touch_positions.into_iter().any(|pos| area.contains(pos));
		
		force_clickable = force_clickable || force_draggable;

		let mut out = false;
		let mut out_drag_delta = None;

		if input_state.any_touch_pressed_on(area) {
			self.dragging_by = input_state.get_touch_pressed_on(area).first().cloned();
		}else if let Some(touch_id) = self.dragging_by {
			if input_state.is_touch_released(touch_id) {
				self.dragging_by = None;
			}
		}

		if !contains_mouse && self.is_hovering {
			self.is_hovering = false;
			if let Some(signal) = &self.on_unhover {
				input_state.send_signal_from(from, signal(style));
			}
		}

		self.is_hovering = contains_mouse;

		if let Some(signal) = &self.on_click {
			if input_state.is_clicked(from, area) {
				out = true;
				let current = input_state.program_running_time();
				if if let Some(last_click_time) = self.last_click_time {
					// println!("{}", current - last_click_time);
					current - last_click_time < DOUBLE_CLICK_THRESHOLD 
				}else {
					false
				} {
					if let Some(signal) = &self.on_double_click {
						input_state.send_signal_from(from, signal(style));
					}else {
						input_state.send_signal_from(from, signal(style));	
					}
				}else {
					input_state.send_signal_from(from, signal(style));
				}
				self.last_click_time = Some(current);
			}
		}else if force_clickable {
			#[allow(clippy::collapsible_if)]
			if input_state.is_clicked(from, area) {
				out = true;
				self.last_click_time = Some(input_state.program_running_time());
				// input_state.send_signal_from(from, signal.clone());
			}
		}

		if let Some(signal) = &self.on_pressed {
			if input_state.any_touch_pressed_on(area) {
				input_state.send_signal_from(from, signal(style));
			}
		}

		if let Some(signal) = &self.on_released {
			if input_state.any_touch_released_on(area) {
				input_state.send_signal_from(from, signal(style));
			}
		}

		if let Some(signal) = &self.on_hover {
			if input_state.is_any_touch_pressing() && contains_mouse {
				input_state.send_signal_from(from, signal(style));
			}
		}

		if let Some(signal) = &self.on_drag {
			if let Some(id) = &self.dragging_by {
				let drag_delta = input_state.drag_delta(*id);
				input_state.send_signal_from(from, signal(style, drag_delta));
				out_drag_delta = Some(drag_delta + input_state.wheel_delta_consume());
			}else if input_state.wheel_delta() != Vec2::ZERO {
				out_drag_delta = Some(input_state.wheel_delta_consume());
			}
		}else if force_draggable {
			if let Some(id) = &self.dragging_by {
				let drag_delta = input_state.drag_delta(*id);
				// input_state.send_signal_from(from, signal(drag_delta));
				out_drag_delta = Some(drag_delta + input_state.wheel_delta_consume());
			}else if input_state.wheel_delta() != Vec2::ZERO {
				out_drag_delta = Some(input_state.wheel_delta_consume());
			}
		}

		SignalGeneratorResult {
			is_clicked: out,
			drag_delta: out_drag_delta,
		}
	}

	/// Get the touch id that is dragging the widget.
	pub fn dragging_by(&self) -> Option<u64> {
		self.dragging_by
	}
}