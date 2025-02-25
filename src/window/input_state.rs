//! Here we define the InputState-related struct which holds the state of the input events.

use std::{collections::HashMap, path::PathBuf};

use time::{Duration, OffsetDateTime};

use crate::{layout::{LayoutId, ROOT_LAYOUT_ID}, math::{rect::Rect, vec2::Vec2}, widgets::{Signal, SignalWrapper}, window::event::TouchPhase};

use super::event::{ImeEvent, Key, MouseButton, OutputEvent, Theme, WindowEvent};

/// We will handle mouse events as special touch events with id MOUSE_ID.
/// 
/// The id of the mouse event is fixed and will not change.
/// - Left button: MOUSE_ID
/// - Right button: MOUSE_ID + 1
/// - Middle button: MOUSE_ID + 2
/// - Back button: MOUSE_ID + 3
/// - Forward button: MOUSE_ID + 4
/// - Other buttons: MOUSE_ID + button_id
pub const MOUSE_ID: u64 = 1000;
/// The time threshold between pressed.
/// 
/// if press time is less than this threshold, it will be considered as a tap.
pub const DEFAULT_EPSILON_TIME: Duration = Duration::milliseconds(100);

/// The id of the touch event when the mouse is not pressed.
pub const MOUSE_UNPRESSED_ID: u64 = 2000;

/// The input state of the window.
/// 
/// This struct holds the state of the input events.
/// It's the main struct that will be used by the UI to handle input events.
pub struct InputState<S: Signal> {
	// /// Will always be None in touch devices. 
	// mouse_pos: Option<Vec2>,
	/// The size of the window.
	pub window_size: Vec2,
	/// The scaling factor of the window.
	pub scale_factor: f64,
	/// The list of dropped files.
	pub dropped_files: Vec<PathBuf>,
	/// The file being hovered by the mouse.
	pub hovering_file: Option<PathBuf>,
	// /// The modifiers of the keyboard.
	// pub modifiers: Modifiers,
	/// The current theme of the window.
	pub theme: Theme,
	pub(crate) input_string: String,
	pub(crate) ime_string: (String, Option<(usize, usize)>, bool),
	pub(crate) redraw_requested: bool,
	pub(crate) signals_to_send: Vec<SignalWrapper<S>>,
	pub(crate) handling_id: LayoutId,
	pub(crate) should_close: bool,
	pub(crate) window_focused: bool,
	pub(crate) program_start_time: OffsetDateTime,
	pub(crate) output_events: Vec<OutputEvent>,
	pub(crate) all_dirty: bool,
	// last_mouse_position: Option<Vec2>,
	wheel: Vec2,
	pressing_touches: HashMap<u64, TouchState>,
	released_touches: HashMap<u64, TouchState>,
	pressing_keys: HashMap<Key, (Duration, bool)>,
	released_keys: HashMap<Key, Duration>,
	raw_events: Vec<WindowEvent>,
	has_new_events: bool,
	is_ime_enabled: bool,
	pasted_text: String,
	cached_input: String,
}

/// The input string contains the ime condition.
#[derive(Debug)]
pub enum ImeString {
	/// The input string in IME mode.
	Ime {
		input: String, 
		selected: (usize, usize) 
	},
	/// The input string in IME off.
	ImeOff(String),
	/// The string is being consumed by other widget or fo not have input string.
	None,
}

impl ImeString {
	/// check if current ImeString is empty.
	pub fn is_empty(&self) -> bool {
		match self {
			ImeString::Ime { input, .. } => input.is_empty(),
			ImeString::ImeOff(input) => input.is_empty(),
			ImeString::None => true,
		}
	}
}

/// The modifiers of the keyboard.
/// 
/// true means the key is pressing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Modifiers {
	/// The shift key.
	pub shift: bool,
	/// The control key.
	pub ctrl: bool,
	/// The alt key.
	pub alt: bool,
}

struct TouchState {
	id: u64,
	time: Duration,
	pos: Vec2,
	last_pos: Vec2,
	// (widget_id, accepted_pressed)
	using_by: Option<(LayoutId, bool)>,
	last_used: bool,
}

impl<S: Signal> Default for InputState<S> {
    fn default() -> Self {
        Self::new()
    }
}

impl<S: Signal> InputState<S> {
	pub(crate) fn new() -> Self {
		Self {
			// mouse_pos: None,
			window_size: Vec2::INF,
			scale_factor: 1.0,
			signals_to_send: Vec::new(),
			handling_id: ROOT_LAYOUT_ID,
			wheel: Vec2::ZERO,
			// modifiers: Modifiers::default(),
			input_string: String::new(),
			ime_string: (String::new(), None, false),
			program_start_time: OffsetDateTime::now_utc(),
			pressing_touches: HashMap::new(),
			released_touches: HashMap::new(),
			pressing_keys: HashMap::new(),
			released_keys: HashMap::new(),
			raw_events: Vec::new(),
			has_new_events: false,
			should_close: false,
			window_focused: true,
			is_ime_enabled: false,
			redraw_requested: true,
			dropped_files: vec!(),
			hovering_file: None,
			theme: Theme::Dark,
			output_events: vec!(),
			pasted_text: String::new(),
			cached_input: String::new(),
			all_dirty: false,
			// last_mouse_position: None,
		}
	}

	/// Get the raw events of current frame.
	pub fn raw_events(&self) -> &[WindowEvent] {
		&self.raw_events
	}

	/// Get how long the program has been running.
	pub fn program_running_time(&self) -> Duration {
		OffsetDateTime::now_utc() - self.program_start_time
	}

	/// Check if current area is clicked or not.
	pub fn is_clicked(&mut self, click_by: LayoutId, hitbox: Rect) -> bool {
		if self.pressing_touches.values().any(|touch| {
			if let Some((using_by, accepted)) = &touch.using_by {
				*using_by == click_by && *accepted
			}else {
				false
			}
		}) {
			return false;
		}else if self.released_touches.values().any(|touch| {
			if let Some((using_by, accepted)) = &touch.using_by {
				*using_by == click_by && *accepted
			}else {
				false
			}
		}) {
			let mut out = false;
			self.released_touches.retain(|_, touch| {
				if touch.using_by == Some((click_by, true)) && hitbox.contains(touch.pos) {
					out = true;
					false
				}else {
					true
				}
			});
			return out;
		}

		let current = OffsetDateTime::now_utc() - self.program_start_time;

		for touch in self.pressing_touches.values_mut() {
			if let Some((using_by, _)) = &touch.using_by {
				if *using_by == click_by && hitbox.contains(touch.pos) {
					touch.last_used = true;
				}
			}else if touch.using_by.is_none() && hitbox.contains(touch.pos) {
				touch.using_by = Some((click_by, current - touch.time < DEFAULT_EPSILON_TIME));
				touch.last_used = true;
			}
		}

		false
	}

	/// Check if there is any touch pressed.
	pub fn is_any_touch_pressed(&self) -> bool {
		let current = OffsetDateTime::now_utc() - self.program_start_time;
		self.pressing_touches.iter().any(|(_, touch)| current - touch.time < DEFAULT_EPSILON_TIME)
	}

	/// Check if the given touch is pressed.
	pub fn is_touch_pressed(&self, id: u64) -> bool {
		let current = OffsetDateTime::now_utc() - self.program_start_time;
		self.pressing_touches.get(&id).map(|touch| current - touch.time < DEFAULT_EPSILON_TIME).unwrap_or(false)
	}

	/// Get all the touches pressed on the given area, repesented by their ids.
	pub fn get_touch_pressed_on(&self, area: impl Into<Rect>) -> Vec<u64> {
		let area = area.into();
		let current = OffsetDateTime::now_utc() - self.program_start_time;
		let mut result = vec!();
		for (id, state) in self.pressing_touches.iter() {
			if area.contains(state.pos) && current - state.time < DEFAULT_EPSILON_TIME && state.using_by.is_none() {
				result.push(*id);
			}
		}
		result
	}

	/// Check if there is any touch pressed on the given area.
	pub fn any_touch_pressed_on(&self, area: impl Into<Rect>) -> bool {
		!self.get_touch_pressed_on(area).is_empty()
	}

	/// Check if there is any touch pressing on the given area.
	pub fn any_touch_pressing_on(&self, area: impl Into<Rect>) -> bool {
		let area = area.into();
		self.pressing_touches.values().any(|touch| area.contains(touch.pos))
	}

	/// Check if there is any touch pressing.
	/// 
	/// Also contains the logic to handle mouse events.
	pub fn is_any_touch_pressing(&self) -> bool {
		!self.pressing_touches.is_empty()
	}

	/// Check if there is any touch released.
	/// 
	/// Also contains the logic to handle mouse events.
	pub fn is_any_touch_released(&self) -> bool {
		!self.released_touches.is_empty()
	}

	/// Check if the given touch is released.
	pub fn is_touch_released(&self, id: u64) -> bool {
		self.released_touches.contains_key(&id) || !self.pressing_touches.contains_key(&id)
	}

	/// Check if the given touch is in the given area.
	pub fn is_touch_in(&self, rect: Rect) -> bool {
		self.pressing_touches.values().any(|touch| rect.contains(touch.pos))
	}

	/// Get all the touches released on the given area, repesented by their ids.
	pub fn get_touch_released_on(&self, area: impl Into<Rect>) -> Vec<u64> {
		let area = area.into();
		let mut result = vec!();
		for (id, touch) in self.released_touches.iter() {
			if id != &MOUSE_UNPRESSED_ID && area.contains(touch.pos) {
				result.push(*id);
			}
		}
		result
	}

	/// Get touch position by id.
	pub fn get_touch_pos(&self, id: u64) -> Option<Vec2> {
		self.pressing_touches.get(&id).or_else(|| self.released_touches.get(&id)).map(|touch| touch.pos)
	}

	/// Check if there is any touch released on the given area.
	pub fn any_touch_released_on(&self, area: impl Into<Rect>) -> bool {
		!self.get_touch_released_on(area).is_empty()
	}

	/// Check if there is any key pressing.
	pub fn is_any_key_pressing(&self) -> bool {
		!self.pressing_keys.is_empty()
	}

	/// Check if the given key is pressed.
	pub fn is_any_key_pressed(&self) -> bool {
		let current = OffsetDateTime::now_utc() - self.program_start_time;
		self.pressing_keys.values().any(|(duration, used)| current - *duration < DEFAULT_EPSILON_TIME && !*used)
	}

	/// Check if there is any key released.
	pub fn is_any_key_released(&self) -> bool {
		!self.released_keys.is_empty()
	}

	/// Check if the given key is released.
	pub fn is_key_pressed(&mut self, key: Key) -> bool {
		let current = OffsetDateTime::now_utc() - self.program_start_time;
		self.pressing_keys.get_mut(&key).map(|(duration, used)| {
			if current - *duration < DEFAULT_EPSILON_TIME && !*used {
				*used = true;
				true
			}else {
				false
			}
		}).unwrap_or(false)
	}

	/// Check if the given key is released.
	pub fn is_key_released(&self, key: Key) -> bool {
		self.released_keys.contains_key(&key)
	}

	/// Check if the given key is pressing.
	pub fn is_key_pressing(&self, key: Key) -> bool {
		self.pressing_keys.contains_key(&key)
	}

	/// Get the current modifiers of the keyboard.
	pub fn modifiers(&self) -> Modifiers {
		Modifiers {
			shift: self.is_key_pressing(Key::ShiftLeft) || self.is_key_pressing(Key::ShiftRight),
			ctrl: self.is_key_pressing(Key::ControlLeft) || self.is_key_pressing(Key::ControlRight),
			alt: self.is_key_pressing(Key::AltLeft) || self.is_key_pressing(Key::AltRight),
		}
	}

	pub(crate) fn update(&mut self, events: Vec<WindowEvent>) {
		if events.is_empty() {
			return;
		}
		for event in events {
			match &event {
				WindowEvent::Resized(size) => self.window_size = *size / self.scale_factor as f32,
				WindowEvent::CloseRequested => self.should_close = true,
				WindowEvent::DroppedFile(path) => self.dropped_files.push(path.clone()),
				WindowEvent::HoveredFile(path) => self.hovering_file = Some(path.clone()),
				WindowEvent::HoveredFileCancelled => self.hovering_file = None,
				WindowEvent::Focused(window_focused) => self.window_focused = *window_focused,
				WindowEvent::KeyPressed(key) => {
					let current = OffsetDateTime::now_utc() - self.program_start_time;
					if !self.modifiers().ctrl && !self.modifiers().alt && !self.is_ime_enabled {
						if let Some(key) = key.get_char(self.modifiers().shift) {
							self.cached_input.push(key);
						}
					}
					
					self.pressing_keys.insert(*key, (current, false));
					self.released_keys.retain(|k, _| k != key);
				}
				WindowEvent::KeyReleased(key) => {
					self.released_keys.insert(*key, OffsetDateTime::now_utc() - self.program_start_time);
					self.pressing_keys.remove(key);
				}
				WindowEvent::StringInput(inner) => self.input_string.push_str(inner),
				WindowEvent::ImeEnabled => {
					// println!("ime enabled, input string: {}", self.input_string);
					self.is_ime_enabled = true;
					self.input_string.clear();
				},
				WindowEvent::ImeDisabled => {
					// println!("ime disabled");
					self.is_ime_enabled = false;
					self.ime_string = (String::new(), None, false)
				},
				WindowEvent::Ime(ime_event) => {
					match ime_event {
						ImeEvent::Commit(commit) => {
							self.input_string.push_str(commit);
							self.ime_string = (String::new(), None, false);
						},
						ImeEvent::Edit(edit, selection) => {
							self.input_string.clear();
							self.ime_string = (edit.clone(), *selection, false);
						},
					}
				},
				WindowEvent::MouseMoved(pos) => {
					let touch = if let Some(touch) = self.pressing_touches.remove(&MOUSE_UNPRESSED_ID)  {
						TouchState {
							pos: *pos / self.scale_factor as f32,
							..touch
						}
					}else {
						TouchState {
							id: MOUSE_UNPRESSED_ID,
							// to avoid the unwanted click event
							time: Duration::ZERO,
							pos: *pos / self.scale_factor as f32,
							last_pos: *pos / self.scale_factor as f32,
							using_by: None,
							last_used: false,
						}
					};
					self.pressing_touches.insert(touch.id, touch);
					for i in 0..5 {
						let id = i + MOUSE_ID;
						if let Some(touch) = self.pressing_touches.get_mut(&id) {
							touch.pos = *pos / self.scale_factor as f32;
						}
					}
				},
				WindowEvent::MouseWheel(delta) => {
					self.wheel += *delta;
				},
				WindowEvent::MouseEntered => {},
				WindowEvent::MouseLeft => {
					self.pressing_touches.remove(&MOUSE_UNPRESSED_ID);
				},
				WindowEvent::MousePressed(button) => {
					let id = match button {
						MouseButton::Left => 0,
						MouseButton::Right => 1,
						MouseButton::Middle => 2,
						MouseButton::Back => 3,
						MouseButton::Forward => 4,
						MouseButton::Other(id) => *id as u64,
					} + MOUSE_ID;

					let mouse_pos = if let Some(inner) = self.pressing_touches.get(&MOUSE_UNPRESSED_ID) {
						inner.pos
					}else {
						Vec2::INF
					};

					self.pressing_touches.insert(id, TouchState {
						id,
						time: OffsetDateTime::now_utc() - self.program_start_time,
						pos: mouse_pos,
						last_pos: mouse_pos,
						using_by: None,
						last_used: false,
					});
				},
				WindowEvent::MouseReleased(button) => {
					let id = match button {
						MouseButton::Left => 0,
						MouseButton::Right => 1,
						MouseButton::Middle => 2,
						MouseButton::Back => 3,
						MouseButton::Forward => 4,
						MouseButton::Other(id) => *id as u64,
					} + MOUSE_ID;

					if let Some(mut touch) = self.pressing_touches.remove(&id) {
						touch.time = OffsetDateTime::now_utc() - self.program_start_time;
						self.released_touches.insert(id, touch);
					}
				},
				WindowEvent::Touch(touch) => {
					let id = touch.id;

					if touch.phase == TouchPhase::Cancelled || touch.phase == TouchPhase::Ended {
						if let Some(mut inner) = self.pressing_touches.remove(&id) {
							inner.time = OffsetDateTime::now_utc() - self.program_start_time;
							self.released_touches.insert(id, inner);
						}
					}else if let Some(inner) = self.pressing_touches.get_mut(&id) {
						self.released_touches.retain(|_, touch| touch.id != id);
						inner.pos = touch.pos / self.scale_factor as f32;
					}else {
						self.released_touches.retain(|_, touch| touch.id != id);
						self.pressing_touches.insert(id, TouchState {
							id,
							time: OffsetDateTime::now_utc() - self.program_start_time,
							pos: touch.pos  / self.scale_factor as f32,
							last_pos: touch.pos / self.scale_factor as f32,
							using_by: None,
							last_used: false,
						});
					}
				},
				WindowEvent::ScaleFactor(factor) => self.scale_factor = *factor,
				WindowEvent::ThemeChanged(theme) => self.theme = *theme,
				WindowEvent::RedrawRequested => self.redraw_requested = true,
				WindowEvent::Unknown => {},
			}

			self.raw_events.push(event);
		}
		self.has_new_events = true;
		
	}
	
	/// Get the window size.
	pub fn window_size(&self) -> Vec2 {
		self.window_size
	}

	/// Get the scaling factor of the window.
	pub fn scale_factor(&self) -> f64 {
		self.scale_factor
	}

	/// Get the wheel delta.
	pub fn wheel_delta(&self) -> Vec2 {
		self.wheel
	}

	/// Get the wheel delta and set it to zero.
	pub fn wheel_delta_consume(&mut self) -> Vec2 {
		let out = self.wheel;
		self.wheel = Vec2::ZERO;
		out
	}

	/// Get drag delta of each touch, will also include the mouse wheel delta in the id [`MOUSE_UNPRESSED_ID`].
	pub fn drag_deltas(&self) -> HashMap<u64, Vec2> {
		let mut deltas = HashMap::new();
		for touch in self.pressing_touches.values() {
			deltas.insert(touch.id, touch.pos - touch.last_pos);
		}
		deltas.insert(MOUSE_UNPRESSED_ID, self.wheel);
		deltas
	}

	/// Get the drag delta of the given touch.
	pub fn drag_delta(&self, id: u64) -> Vec2 {
		self.drag_deltas().get(&id).cloned().unwrap_or_default()
	}

	/// Consume the touch with the given id, let it cant be used by other widgets.
	pub fn consume_touch(&mut self, id: u64) {
		if let Some(touch) = self.pressing_touches.get_mut(&id) {
			touch.using_by = Some((self.handling_id, false));
			touch.last_used = true;
		}
	}

	/// Get drag delta relative to the last frame by simply summing up all the drag deltas.
	pub fn drag_delta_summary(&self) -> Vec2 {
		self.drag_deltas().values().sum()
	}

	/// Get the touch positions, will also include the mouse position if any.
	pub fn touch_positions(&self) -> Vec<Vec2> {
		self.pressing_touches.values().map(|touch| touch.pos).collect::<Vec<_>>()
	}

	/// Send a signal to the app, the id is automatically set to the widget's id which handles the event.
	/// 
	/// If you call maually (outside of event handling loop), the sender will be root.
	/// If you want to send a signal with a specific sender, use the `send_signal_from` method.
	pub fn send_signal(&mut self, signal: S) {
		self.signals_to_send.push(SignalWrapper {
			signal,
			from: self.handling_id,
		});
	}

	/// Send a signal to the app, with a specific sender.
	pub fn send_signal_from(&mut self, from: LayoutId, signal: S) {
		self.signals_to_send.push(SignalWrapper {
			signal,
			from,
		});
	}

	/// Set the window title.
	pub fn set_title(&mut self, title: impl Into<String>) {
		self.output_events.push(OutputEvent::SetWindowTitle(title.into()));
	}

	/// Set the cursor icon.
	pub fn set_cursor_icon(&mut self, icon: super::event::CursorIcon) {
		self.output_events.push(OutputEvent::SetCursorIcon(icon));
	}

	/// Set the cursor position.
	pub fn set_cursor_position(&mut self, pos: impl Into<Vec2>) {
		self.output_events.push(OutputEvent::SetCursorPosition(pos.into()));
	}

	/// Set the cursor visibility.
	pub fn set_cursor_visible(&mut self, visible: bool) {
		self.output_events.push(OutputEvent::SetCursorVisible(visible));
	}

	/// Set the window size.
	pub fn set_window_size(&mut self, size: impl Into<Vec2>) {
		self.output_events.push(OutputEvent::Resize(size.into()));
	}

	/// Set the window position.
	pub fn set_window_position(&mut self, pos: impl Into<Vec2>) {
		self.output_events.push(OutputEvent::Move(pos.into()));
	}

	/// Returns the time since the program started.
	pub fn run_time(&self) -> Duration {
		OffsetDateTime::now_utc() - self.program_start_time
	}

	/// Get the input string of current frame.
	/// 
	/// Will consume the original input string if ime is disabled.
	pub fn get_input_string(&mut self) -> ImeString {
		// println!("{}", self.input_string);
		if let Some(pos) = self.ime_string.1 {
			if !self.ime_string.2 {
				self.ime_string.2 = true;
				ImeString::Ime { input: self.ime_string.0.clone(), selected: pos }
			}else {
				ImeString::None
			}
		}else if !self.pasted_text.is_empty() {
			let mut out = String::new(); 
			std::mem::swap(&mut self.pasted_text, &mut out);
			ImeString::ImeOff(out)
		}else if self.input_string.is_empty() {
			ImeString::None
		}else {
			let mut out = String::new();
			std::mem::swap(&mut self.input_string, &mut out);
			ImeString::ImeOff(out)
		}
	}

	/// Copy the given text to the clipboard.
	pub fn copy_text(&mut self, text: impl Into<String>) {
		self.output_events.push(OutputEvent::CopyToClipboard(text.into()));
	}

	/// Request host to paste text from the clipboard.
	pub fn request_paste_text(&mut self) {
		self.output_events.push(OutputEvent::RequestClipboard);
	}

	/// Paste the given text to the input string.
	pub fn paste_text(&mut self, text: impl Into<String>) {
		self.pasted_text.push_str(&text.into());
		println!("pasted: {}", self.pasted_text);
	}

	pub(crate) fn prepare_for_next_frame(&mut self) {
		self.raw_events.clear();
		self.has_new_events = false;
		self.signals_to_send.clear();
		self.wheel = Vec2::ZERO;
		let current = OffsetDateTime::now_utc() - self.program_start_time;
		
		self.pressing_touches.values_mut().for_each(|touch| {
			touch.last_pos = touch.pos;
		});
		self.released_keys.retain(|_, time| current - *time < DEFAULT_EPSILON_TIME);
		self.released_touches.retain(|_, touch| {
			if !touch.last_used {
				touch.using_by = None;
			}

			touch.last_used = false;
			current - touch.time < DEFAULT_EPSILON_TIME
		});
		self.handling_id = ROOT_LAYOUT_ID;
		self.input_string.clear();
		self.ime_string.2 = false;
		std::mem::swap(&mut self.input_string, &mut self.cached_input);
		// std::mem::swap(&mut self.pasted_text, &mut self.input_string);
		// self.last_mouse_position = self.mouse_pos;
	}

	pub(crate) fn mouse_pos(&self) -> Option<Vec2> {
		self.pressing_touches.get(&MOUSE_UNPRESSED_ID).map(|touch| touch.pos)
	}

	/// Mark all widgets dirty, will trigger a redraw.
	pub fn mark_all_dirty(&mut self) {
		self.redraw_requested = true;
		self.all_dirty = true;
	}

	// pub(crate) fn no_events(&self) -> bool {
	// 	if !self.window_focused {
	// 		return true;
	// 	}
		
	// 	let mut has_unprocessed_touch = false;
	// 	for touch in self.pressing_touches.values() {
	// 		if touch.id != MOUSE_UNPRESSED_ID && touch.using_by.is_none() {
	// 			has_unprocessed_touch = true;
	// 			break;
	// 		}
	// 	}

	// 	let has_unprocessed_key = !self.input_string.is_empty() || !self.ime_string.0.is_empty();

	// 	let has_hovering_file = self.hovering_file.is_some();

	// 	!(has_unprocessed_touch || has_unprocessed_key || has_hovering_file)
	// }
}