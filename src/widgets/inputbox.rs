//! A simple input box widget.

use crate::{layout::{Layout, LayoutId}, prelude::{AnimatedColor, Animatedf32, Color, FillMode, FontId, ImeString, InputState, Key, Painter, Rect, Vec2, Vec4}, App};

use super::{styles::{BRIGHT_FACTOR, CONTENT_TEXT_SIZE, DEFAULT_PADDING, DEFAULT_ROUNDING, DISABLE_TEXT_COLOR, INPUT_BACKGROUND_COLOR, INPUT_BORDER_COLOR, PRIMARY_COLOR, SECONDARY_TEXT_COLOR, SELECTED_TEXT_COLOR}, Signal, SignalGenerator, Widget};

/// The word splitter for the input box.
pub static WORD_SPLITER: &[char] = &[' ', '\t', '\n', ';', ',', '.', ':', '!', '?', '(', ')', '[', ']', '{', '}', '<', '>', '/', '\\', '\'', '\"', '@', '#', '$', '%', '^', '&', '*', '-', '_', '+', '=', '|', '`', '~'];

/// A simple input box widget.
pub struct InputBox<S: Signal, A: App<Signal = S>> {
	/// The inner properties of the input box.
	pub inner: InputBoxInner,
	/// The signal to send when the input box is submitted.
	/// 
	/// The signal will be constructed with the current text in the input box.
	#[allow(clippy::type_complexity)]
	pub on_submit: Option<Box<dyn Fn(&mut InputBoxInner) -> S>>,
	/// The signal to send when the input box changes.
	/// 
	/// The signal will be constructed with the current text in the input box.
	#[allow(clippy::type_complexity)]
	pub on_change: Option<Box<dyn Fn(&mut InputBoxInner) -> S>>,
	/// The general signal to send when the input box is interacted with.
	pub signals: SignalGenerator<S, InputBoxInner, A>,
	is_typing: bool,
	hover_factor: Animatedf32,
}

/// The inner properties of the input box.
pub struct InputBoxInner {
	/// The placeholder text to display when the input box is empty.
	pub placeholder: String,
	/// Set wheather the input box is a password input.
	pub password: bool,
	/// The current text in the input box.
	pub text: String,
	/// The size of the input box.
	pub size: Vec2,
	/// The font id of the input box.
	pub font: FontId,
	/// The font size of the input box.
	pub font_size: f32,
	/// The validator to use for the input box.
	pub validator: Option<Box<dyn Validator>>,
	// /// The highlighter to use for the input box.
	// pub highligher: Option<Box<dyn Highlighter>>,
	// /// The completer to use for the input box.
	// pub completer: Option<Box<dyn Completer>>,
	/// The current pointer position in the input box.
	pub pointer: Pointer,
	/// The current scroll position in the input box.
	pub scroll_position: Vec2,
	/// The background color of the input box.
	pub background_color: FillMode,
	/// The text color of the input box.
	pub text_color: FillMode,
	/// The border color of the input box.
	pub border_color: AnimatedColor,
	/// The padding of the input box.
	pub padding: Vec2,
	/// The roundings of the input box.
	pub roundings: Vec4,
	/// The color of the placeholder text.
	pub placeholder_color: FillMode,
	/// The color of the selected text.
	pub selected_color: FillMode,
}

impl Default for InputBoxInner {
	fn default() -> Self {
		Self {
			placeholder: "".to_string(),
			password: false,
			text: "".to_string(),
			size: Vec2::new(200.0, CONTENT_TEXT_SIZE),
			font: 0,
			font_size: CONTENT_TEXT_SIZE,
			validator: None,
			pointer: Pointer::default(),
			scroll_position: Vec2::ZERO,
			background_color: FillMode::Color(INPUT_BACKGROUND_COLOR),
			text_color: FillMode::Color(SECONDARY_TEXT_COLOR),
			border_color: AnimatedColor::default_with_value(INPUT_BORDER_COLOR),
			padding: Vec2::same(DEFAULT_PADDING),
			roundings: Vec4::same(DEFAULT_ROUNDING),
			placeholder_color: FillMode::Color(DISABLE_TEXT_COLOR),
			selected_color: FillMode::Color(SELECTED_TEXT_COLOR),
			// highligher: None,
			// completer: None,
		}
	}
}

/// The current pointer position in the input box.
#[derive(Clone, Copy, Debug, Default)]
pub struct Pointer {
	/// The start index of the selected text.
	/// 
	/// if it is equal to the end index, there is no selected text.
	start: usize,
	/// The end index of the selected text.
	/// 
	/// if it is equal to the start index, there is no selected text.
	end: usize,
	/// Whether the start index is the current index.
	is_start_current: bool,
}

/// A enum to represent the position of the pointer.
/// 
/// Used for drawing the selection box and the cursor.
pub enum PointerPos {
	/// The [`Pointer`] do not have any selected text.
	Single(Vec2),
	/// The [`Pointer`] has selected text.
	Selected {
		/// Where current Pointer is.
		pointer_pos: Vec2,
		/// The area of the selected text.
		selection_rect: Vec<Rect>,
	},
}

impl PointerPos {
	/// Get the position of the pointer.
	pub fn pos(&self) -> Vec2 {
		match self {
			PointerPos::Single(pos) => *pos,
			PointerPos::Selected { pointer_pos,.. } => *pointer_pos,
		}
	}
}

/// A enum to represent the amount of the pointer movement.
pub enum PointerAmount {
	/// Move the pointer by one character.
	Char(isize),
	/// Move the pointer by one word.
	Word(isize),
	/// Move the pointer by one line.
	Line(isize),
}

impl Pointer {
	/// Create a new [`Pointer`] with the given current index.
	pub fn new(current_pos: usize) -> Self {
		Self {
			start: current_pos,
			end: current_pos,
			is_start_current: false,
		}
	}

	/// Get the current index of the selected text.
	/// 
	/// The method need the original text because the index pointer save has taken into account the utf-8 encoding
	/// and it is not possible to get the index of the current character in the text without the original text.
	pub fn current_index(&self, text: &str) -> usize {
		if self.is_start_current {
			convert_index(text, self.start)
		}else {
			convert_index(text, self.end)
		}
	}

	/// Get the current index of the selected text but in utf-8 encoding.
	#[inline]
	pub fn current_index_utf8(&self) -> usize {
		if self.is_start_current {
			self.start
		}else {
			self.end
		}
	}

	/// Move the pointer by given amount.
	pub fn move_by(&mut self, text: &str, amount: PointerAmount, with_selection: bool) {
		match amount {
			PointerAmount::Char(amount) => {
				let new_index = self.current_index_utf8() as isize + amount;
				if with_selection {
					if new_index < 0 {
						if self.is_start_current {
							self.start = 0;
						}else {
							self.start = 0;
							self.end = 0;
							self.is_start_current = false;
						}
					}else if self.is_start_current {
						self.start = new_index as usize;
					}else {
						self.end = new_index as usize;
					}
				}else if new_index < 0 {
					self.start = 0;
					self.end = 0;
					self.is_start_current = false;
				}else {
					self.start = new_index as usize;
					self.end = new_index as usize;
					self.is_start_current = false;
				}
			},
			PointerAmount::Word(delta) | PointerAmount::Line(delta) => {
				let spliter = if matches!(amount, PointerAmount::Word(_)) {
					WORD_SPLITER
				}else {
					&['\n']
				};

				let words = text.split(spliter);
				let mut current_word = 0;
				let mut current_index = 0;
				let current_pointer = self.current_index_utf8();
				for word in words {
					if current_index + word.chars().count() <= current_pointer {
						current_word += 1;
						current_index += word.chars().count() + 1;
					}else {
						break;
					}
				}
				let delta_word = current_word + delta;
				if delta_word <= 0 {
					if with_selection {
						if self.is_start_current {
							self.start = 0;
						}else {
							self.end = 0;
						}
					}else {
						self.start = 0;
						self.end = 0;
						self.is_start_current = false;
					}
				}else {
					let ptr = text.split(spliter).enumerate().map(|(i, word)| {
						if i >= delta_word as usize {
							0
						}else {
							word.chars().count() + 1
						}
					}).sum();
					if with_selection {
						if self.is_start_current {
							self.start = ptr;
						}else {
							self.end = ptr;
						}
					}else {
						self.start = ptr;
						self.end = ptr;
						self.is_start_current = false;
					}
				}
			}
		}

		if self.start > self.end {
			std::mem::swap(&mut self.start, &mut self.end);
			self.is_start_current = !self.is_start_current;
		}
		let len = text.chars().count();
		self.start = self.start.min(len);
		self.end = self.end.min(len);
	}

	/// Delete the selected text only.
	pub fn delete_selected_text(&mut self, text: &mut String) {
		if self.has_selected_text() {
			let range = convert_range(text, self.start, self.end);
			text.replace_range(range, "");
			self.end = self.start;
		}
	}

	/// Delete the selected text or the character before the pointer.
	pub fn delete(&mut self, text: &mut String) {
		if self.has_selected_text() {
			let range = convert_range(text, self.start, self.end);
			text.replace_range(range, "");
			self.end = self.start;
		}else if self.current_index_utf8() > 0 && self.current_index_utf8() <= text.chars().count() {
			let current = self.current_index_utf8();
			text.replace_range(convert_range(text, current - 1, current), "");
			self.start -= 1;
			self.end -= 1;
		}
	}

	/// Move the pointer to the end of the text.
	pub fn move_to_start(&mut self) {
		self.start = 0;
		self.end = 0;
		self.is_start_current = true;
	}

	/// Move the pointer to the end of the text.
	pub fn move_to_end(&mut self, text: &str) {
		self.start = text.chars().count();
		self.end = text.chars().count();
		self.is_start_current = true;
	}

	/// Select all the text.
	/// 
	/// Refer to `ctrl + a` in most text editors.
	pub fn select_all(&mut self, text: &str) {
		self.start = 0;
		self.end = text.chars().count();
		self.is_start_current = false;
	}

	/// Insert some text at the current position of the pointer.
	pub fn insert_text(&mut self, text: &mut String, new_text: ImeString, validator: &Option<Box<dyn Validator>>) -> ValidatorResult {
		if new_text.is_empty() {
			return ValidatorResult::Valid;
		}
		if self.has_selected_text() && self.end != 0 {
			let range = convert_range(text, self.start, self.end);
			text.replace_range(range, "");
			self.end = self.start;
		}
		let out = if let ImeString::ImeOff(inner) = &new_text {
			if let Some(validator) = validator {
				validator.validate(inner, text, *self)
			}else {
				ValidatorResult::Valid
			}
		}else {
			ValidatorResult::Valid
		};

		if matches!(out, ValidatorResult::Valid) {
			match new_text {
				ImeString::None => {},
				ImeString::Ime { input, .. } => {
					text.insert_str(self.current_index(text), &input);
					self.is_start_current = false;
					self.end += input.chars().count();
				},
				ImeString::ImeOff(inner) => {
					text.insert_str(self.current_index(text), &inner);
					self.start += inner.chars().count();
					self.end = self.start;
				},
			}
		}

		out
	}

	/// Check if there is any selected text.
	pub fn has_selected_text(&self) -> bool {
		self.start != self.end
	}

	/// Get selected text but split it into lines.
	pub fn get_selected_text_lines<'a>(&self, text: &'a str) -> Vec<&'a str> {
		let range = convert_range(text, self.start, self.end);
		let text = &text[range];
		text.lines().collect()
	}

	/// Get selected text.
	pub fn get_selected_text<'a>(&self, text: &'a str) -> &'a str {
		let range = convert_range(text, self.start, self.end);
		&text[range]
	}

	/// Get lines contains the selected text.
	pub fn get_selected_lines<'a>(&self, text: &'a str) -> Vec<&'a str> {
		let lines = text.lines().collect::<Vec<_>>();
		let mut start_index = 0;
		let mut start_line = 0;
		let mut end_index = 0;
		let mut end_line = 0;
		for line in lines.iter() {
			let line_len = line.chars().count();
			if start_index + line_len < self.start {
				start_index += line_len + 1;
				start_line += 1;
			}
			if end_index + line_len < self.end {
				end_index += line_len + 1;
				end_line += 1;
			}
		}
		lines[start_line..end_line + 1].to_vec()
	}

	/// Caculate the position of the pointer.
	pub fn caculate_pointer_pos(&self, text: &str, font_size: f32, font_id: FontId, painter: &mut Painter) -> PointerPos {
		let line_height = painter.line_height(font_id, font_size).unwrap_or_default();
		let pointer_pos = {
			let current_pos = self.current_index_utf8();
			let mut line_count = 0;
			let mut index = 0;
			for line in text.lines() {
				if index + line.chars().count() < current_pos {
					index += line.chars().count() + 1;
					line_count += 1;
				}else {
					break;
				}
			}
			let line = text.lines().nth(line_count).unwrap_or_default();
			let line = &line[convert_range(line, 0, current_pos - index)];
			let text_width = painter.text_size_pointer(font_id, font_size, line).unwrap_or_default().x;
			Vec2::new(text_width, line_count as f32 * line_height)
		};

		if self.has_selected_text() {
			let selected_lines = self.get_selected_lines(text);
			let selected_text = self.get_selected_text_lines(text);
			let mut selection_rect = Vec::new();
			for (i, (total, selected)) in selected_lines.into_iter().zip(selected_text.into_iter()).enumerate() {
				let start_index = text.find(selected).unwrap();
				let start_size = painter.text_size_pointer(font_id, font_size, &total[0..start_index]).unwrap_or_default();
				let selected_size = painter.text_size_pointer(font_id, font_size, selected).unwrap_or_default();
				selection_rect.push(Rect::from_lt_size(
					Vec2::new(start_size.x, i as f32 * line_height * if self.is_start_current { 1.0 } else { -1.0 } + pointer_pos.y),
					selected_size,
				));
			}
			PointerPos::Selected { pointer_pos, selection_rect }
		}else {
			PointerPos::Single(pointer_pos)
		}
	}
}

impl<S: Signal, A: App<Signal = S>> Default for InputBox<S, A> {
	fn default() -> Self {
		Self {
			inner: InputBoxInner::default(),
			on_submit: None,
			on_change: None,
			signals: SignalGenerator::default(),
			is_typing: false,
			hover_factor: Animatedf32::default(),
		}
	}
}

impl<S: Signal, A: App<Signal = S>> InputBox<S, A> {
	/// Create a new input box.
	pub fn new(font: FontId, font_size: f32) -> Self {
		Self {
			inner: InputBoxInner {
				font,
				font_size,
				..Default::default()
			},
			..Default::default()
		}
	}

	/// Set the padding of the input box.
	pub fn padding(self, padding: Vec2) -> Self {
		Self { inner: InputBoxInner { padding, ..self.inner }, ..self } 
	}

	/// Set the background color of the input box.
	pub fn background_color(self, color: impl Into<FillMode>) -> Self {
		Self {
			inner: InputBoxInner { background_color: color.into(), ..self.inner },
			..self
		}
	}

	/// Set the text color of the input box.
	pub fn text_color(self, color: impl Into<FillMode>) -> Self {
		Self {
			inner: InputBoxInner { text_color: color.into(), ..self.inner },
			..self
		}
	}

	/// Set the placeholder text to display when the input box is empty.
	pub fn placeholder(self, placeholder: impl Into<String>) -> Self {
		Self {
			inner: InputBoxInner { placeholder: placeholder.into(), ..self.inner },
			..self
		}
	}

	/// Set wheather the input box is a password input.
	pub fn password(self, password: bool) -> Self {
		Self { inner: InputBoxInner { password, ..self.inner }, ..self }
	}

	/// Set the current text in the input box.
	pub fn text(self, text: impl Into<String>) -> Self {
		Self {
			inner: InputBoxInner { text: text.into(), ..self.inner },
			..self
		}
	}

	/// Set the size of the input box.
	pub fn size(self, size: Vec2) -> Self {
		Self { inner: InputBoxInner { size, ..self.inner }, ..self }
	}

	/// Set the validator to use for the input box.
	pub fn validator(self, validator: impl Validator + 'static) -> Self {
		Self {
			inner: InputBoxInner { validator: Some(Box::new(validator)), ..self.inner },
			..self
		}
	}

	// /// Set the highlighter to use for the input box.
	// pub fn highligher(self, highligher: impl Highlighter + 'static) -> Self {
	// 	Self {
	// 		highligher: Some(Box::new(highligher)),
	// 		..self
	// 	}
	// }

	// /// Set the completer to use for the input box.
	// pub fn completer(self, completer: impl Completer + 'static) -> Self {
	// 	Self {
	// 		completer: Some(Box::new(completer)),
	// 		..self
	// 	}
	// }

	/// Set the signal to send when the input box is submitted.
	pub fn on_submit(self, on_submit: impl Fn(&mut InputBoxInner) -> S + 'static) -> Self {
		Self {
			on_submit: Some(Box::new(on_submit)),
			..self
		}
	}

	/// Set the signal to send when the input box changes.
	pub fn on_change(self, on_change: impl Fn(&mut InputBoxInner) -> S + 'static) -> Self {
		Self {
			on_change: Some(Box::new(on_change)),
			..self
		}
	}

	/// Set the current pointer position in the input box.
	pub fn pointer(self, pointer: Pointer) -> Self {
		Self { inner: InputBoxInner { pointer, ..self.inner }, ..self }
	}

	fn submit(&mut self, input_state: &mut InputState<S>, id: LayoutId) {
		self.is_typing = false;
		self.inner.border_color.set(INPUT_BORDER_COLOR);
		if let Some(on_submit) = &self.on_submit {
			let signal = on_submit(&mut self.inner);
			input_state.send_signal_from(id, signal);
		}
	}
}

/// Possible results of input validation.
pub enum ValidatorResult {
	/// The input is valid.
	Valid,
	/// The input is invalid.
	Invalid {
		/// The error message to display.
		message: Option<String>,
		/// Whether the input should be allowed.
		/// 
		/// if true, the input will also be appended to the current text in the input box.
		/// if false, the input will be discarded.
		allow_input: bool,
	},
	/// The input is banned. 
	Banned,
	/// Remove the key foucs from the input box.
	FinishType,
}

/// A trait for input validation.
pub trait Validator {
	/// Validate the newly input text and the current text in the input box.
	/// 
	/// Returns an error message if the input is invalid, `None` for valid input.
	fn validate(&self, newly_input: &str, current_text: &str, pointer: Pointer) -> ValidatorResult;

	/// Whether to validate the input when the input box changes.
	/// 
	/// If true, the `validate` method will be called when the input box changes.
	/// If false, the `validate` method will only be called when the input box is submitted.
	fn validate_when_change(&self) -> bool;
}

// /// A trait for input highlighting.
// pub trait Highlighter {
// 	/// Highlight the input text and the current text in the input box.
// 	/// 
// 	/// Returns a list of tuples containing the highlighted text and the fill mode to use.
// 	fn highlight(&self, text: &str, pointer: Pointer) -> Vec<(String, FillMode)>;
// }

// /// A trait for input completion.
// pub trait Completer {
// 	/// Give a list of completions for the input text and the current text in the input box.
// 	/// 
// 	/// Returns a list of completions.
// 	fn complete(&self, text: &str, current_text: &str, pointer: Pointer) -> Vec<String>;
// }

/// A simple input validator for daliy use.
#[derive(Clone, Debug, Default)]
pub struct SimpleValidator {
	/// Whether to allow breakline in the input text.
	pub allow_breakline: bool,
	/// The maximum length of the input text.
	pub limit: Option<usize>,
	/// The number validation to use.
	pub number_validation: NumerValidation,
	/// Whether to ban the input or not.
	pub banned: bool,
	/// Whether to validate every input change or only when the input box is submitted.
	pub validate_when_change: bool,
}

/// The number validation to use.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub enum NumerValidation {
	/// Only allow digits.
	Integer,
	/// Allow digits and decimal point.
	Float,
	/// Do not use number validation.
	#[default] None,
}

impl Validator for SimpleValidator {
	fn validate(&self, newly_input: &str, current_text: &str, poniter: Pointer) -> ValidatorResult {
		if self.banned {
			return ValidatorResult::Banned;
		}

		if let Some(limit) = self.limit {
			if current_text.chars().count() + newly_input.chars().count() > limit {
				return ValidatorResult::Invalid {
					message: Some("Too long".to_string()),
					allow_input: false,
				};
			}
		}

		if !self.allow_breakline && newly_input.contains('\n') {
			return ValidatorResult::FinishType;
		}

		if let NumerValidation::Integer = self.number_validation {
			if newly_input.chars().any(|c| !c.is_numeric()) {
				return ValidatorResult::Invalid {
					message: Some("Only digits allowed".to_string()),
					allow_input: true,
				};
			}
		}else if let NumerValidation::Float = self.number_validation {
			let mut final_str = current_text.to_string();
			final_str.insert_str(poniter.current_index(current_text), newly_input);
			if let Err(e) = final_str.parse::<f32>() {
				return ValidatorResult::Invalid {
					message: Some(format!("Invalid number: {}", e)),
					allow_input: true,
				};
			}
		}

		ValidatorResult::Valid
	}

	fn validate_when_change(&self) -> bool {
		self.validate_when_change
	}
}

impl<S: Signal, A: App<Signal = S>> Widget for InputBox<S, A> {
	type Signal = S;
	type Application = A;

	fn size(&self, _: LayoutId, _: &Painter, _: &Layout<Self::Signal, A>) -> Vec2 {
		self.inner.size + self.inner.padding * 2.0
	}

	fn draw(&mut self, painter: &mut Painter, size: Vec2) {
		let (text, mut text_color) = if self.inner.text.is_empty() {
			(self.inner.placeholder.clone(), self.inner.placeholder_color.clone())
		}else if self.inner.password {
			(self.inner.text.chars().map(|_| "*").collect(), self.inner.text_color.clone())
		}else {
			(self.inner.text.clone(), self.inner.text_color.clone())
		};

		let stroke = 2.0;
		let mut bg_color = self.inner.background_color.clone();
		bg_color.brighter(self.hover_factor.value() * BRIGHT_FACTOR);
		painter.set_fill_mode(bg_color);
		painter.draw_rect(Rect::from_size(size), self.inner.roundings);
		painter.set_fill_mode(self.inner.border_color.value() + self.hover_factor.value() * BRIGHT_FACTOR * Color::WHITE);
		painter.draw_stroked_rect(Rect::from_size(size).shrink(Vec2::same(stroke / 2.0)), self.inner.roundings, stroke);
		
		let pointer_pos = self.inner.pointer.caculate_pointer_pos(&text, self.inner.font_size, self.inner.font, painter);
		
		let text_pos = pointer_pos.pos() + self.inner.padding;
		let text_pos = if Rect::from_size(size - Vec2::same(self.inner.font_size)).contains(text_pos) {
			Vec2::ZERO
		}else {
			- (text_pos - size + Vec2::same(self.inner.font_size)).max(Vec2::ZERO)
		} + self.inner.padding;
		let text_color = if self.is_typing {
			text_color
		}else {
			text_color.brighter(self.hover_factor.value() * BRIGHT_FACTOR);
			text_color
		};
		painter.set_fill_mode(text_color);
		painter.draw_text(text_pos, self.inner.font, self.inner.font_size, &text);
		if self.is_typing {
			// let line_height = painter.line_height(self.font, self.font_size).unwrap_or_default();
			painter.draw_rect(
				Rect::from_lt_size(
					pointer_pos.pos() + Vec2::new(text_pos.x, self.inner.padding.y), 
					Vec2::new(2.0, self.inner.font_size)
				), 
				Vec4::ZERO
			);
			if let PointerPos::Selected { selection_rect,.. } = pointer_pos {
				painter.set_fill_mode(self.inner.selected_color.clone());
				for rect in selection_rect {
					painter.draw_rect(rect.move_by(text_pos), Vec4::same(self.inner.font_size / 8.0));
				}
			}
		}
	}

	fn handle_event(&mut self, app: &mut A, input_state: &mut InputState<Self::Signal>, id: LayoutId, area: Rect, _: Vec2) -> bool {
		let res = self.signals.generate_signals(app, &mut self.inner, input_state, id, area, true, false);

		if input_state.is_touch_in(area) {
			self.hover_factor.set(1.0);
		}else {
			self.hover_factor.set(0.0);
		}

		if input_state.is_any_touch_released() && !input_state.is_touch_in(area) && self.is_typing {
			self.submit(input_state, id);
		}

		if res.is_clicked {
			self.is_typing = true;
			self.inner.border_color.set(PRIMARY_COLOR + BRIGHT_FACTOR * Color::WHITE);
		}

		if self.is_typing {
			let modifiers = input_state.modifiers();
				
			let input = input_state.get_input_string();
			match self.inner.pointer.insert_text(&mut self.inner.text, input, &self.inner.validator) {
				ValidatorResult::Valid => {
					if let Some(on_change) = &self.on_change {
						let signal = on_change(&mut self.inner);
						input_state.send_signal_from(id, signal);
					}
				},
				ValidatorResult::Invalid { .. } => {},
				ValidatorResult::Banned => {
					self.is_typing = false;
					self.inner.border_color.set(INPUT_BORDER_COLOR);
				},
				ValidatorResult::FinishType => {
					self.submit(input_state, id);
				},
			}


			let amount = |amount: isize| {
				if modifiers.ctrl || modifiers.alt {
					PointerAmount::Word(amount)
				}else {
					PointerAmount::Char(amount)
				}
			};
			
			if input_state.is_key_pressed(Key::ArrawLeft) {
				self.inner.pointer.move_by(&self.inner.text, amount(-1), modifiers.shift)
			}
			if input_state.is_key_pressed(Key::ArrawRight) {
				self.inner.pointer.move_by(&self.inner.text, amount(1), modifiers.shift)
			}

			if input_state.is_key_pressed(Key::Home) {
				self.inner.pointer.move_to_start()
			}

			if input_state.is_key_pressed(Key::End) {
				self.inner.pointer.move_to_end(&self.inner.text)
			}

			if input_state.is_key_pressed(Key::ArrawUp) {
				self.inner.pointer.move_by(&self.inner.text, PointerAmount::Line(-1), modifiers.shift)
			}

			if input_state.is_key_pressed(Key::ArrawDown) {
				self.inner.pointer.move_by(&self.inner.text, PointerAmount::Line(1), modifiers.shift)
			}

			if input_state.is_key_pressed(Key::KeyA) && modifiers.ctrl {
				self.inner.pointer.select_all(&self.inner.text)
			}
			
			if input_state.is_key_pressed(Key::Backspace) || input_state.is_key_pressed(Key::Delete) {
				// println!("delete");
				self.inner.pointer.delete(&mut self.inner.text);
			}

			if modifiers.ctrl && input_state.is_key_pressed(Key::KeyC) {
				let text = self.inner.pointer.get_selected_text(&self.inner.text);
				input_state.copy_text(text);
			}

			if modifiers.ctrl && input_state.is_key_pressed(Key::KeyX) {
				let text = self.inner.pointer.get_selected_text(&self.inner.text);
				input_state.copy_text(text);
				self.inner.pointer.delete_selected_text(&mut self.inner.text);
			}

			if modifiers.ctrl && input_state.is_key_pressed(Key::KeyV) {
				input_state.request_paste_text();
			}

			if input_state.is_key_pressed(Key::Escape) 
			|| input_state.is_key_pressed(Key::Tab) {
				self.submit(input_state, id);
			}
		}

		self.is_typing || self.inner.border_color.is_animating() || self.hover_factor.is_animating()
	}

	fn continuous_event_handling(&self) -> bool {
		self.is_typing
	}
}

#[inline]
fn convert_range(s: &str, from: usize, to: usize) -> std::ops::Range<usize>  {
	if from == 0 && to == 0 {
		return 0..0;
	}
	assert!(from <= to);

	let inner = s.char_indices().nth(from).map(|(start_pos, _)| {
		let len = s.chars().count(); 
		assert!(to <= len);

		let end_pos = s[start_pos..]
			.char_indices()
			.nth(to - from)
			.map(|(end_pos, _)| end_pos).unwrap_or(s.len() - start_pos);

		(start_pos, end_pos + start_pos)
	}).unwrap();

	inner.0..inner.1
}

#[inline]
fn convert_index(s: &str, index: usize) -> usize {
	if index >= s.chars().count() {
		s.len()
	}else {
		s.char_indices().nth(index).map(|(start_pos, _)| start_pos).expect("Invalid index")
	}
}