//! Contains the implementation of the WindowEvent and output event.

use std::path::PathBuf;

use winit::{event::{Ime, MouseScrollDelta, WindowEvent as WinitEvent}, keyboard::{NativeKeyCode, PhysicalKey}};
use crate::{math::vec2::Vec2, render::{font::{FontId, EM}, texture::TextureId}};


/// The output event that `nablo` requeseted host to handle.
#[derive(Debug, Clone)]
pub enum OutputEvent {
	/// Contains the new title of the window.
	SetWindowTitle(String),
	/// Contains the new size of the window.
	Resize(Vec2),
	/// Contains the new position of the window.
	Move(Vec2),
	/// Set the cursor icon of the window.
	SetCursorIcon(CursorIcon),
	/// Set the cursor position of the window.
	SetCursorPosition(Vec2),
	/// Set the cursor visibility of the window.
	SetCursorVisible(bool),
	/// request host to register a new texture.
	/// 
	/// Do NOT send this manually, use [`crate::Context::register_texture()`] instead.
	RegisterTexture(Vec2, Vec<u8>),
	/// request host to change the texture.
	/// 
	/// Do NOT send this manually, use [`crate::Context::update_texture()`] instead.
	UpdateTexture(TextureId, Vec2, Vec<u8>),
	/// request host to remove the texture.
	/// 
	/// Do NOT send this manually, use [`crate::Context::remove_texture()`] instead.
	RemoveTexture(TextureId),
	/// request host to clear the texture.
	/// 
	/// Do NOT send this manually, use [`crate::Context::clear_textures()`] instead.
	ClearTexture,
	/// Request host to add a char into font texture.
	/// 
	/// Do NOT send this manually, this will be automatically handled by `nablo`.
	/// 
	/// `char` is the character to be added.
	/// `Vec<u8>` is the msdf texture data of the font.
	/// `FontId` is the id of the font texture.
	AddChar(Vec<u8>, char, FontId),
	/// Request host to remove a whole font.
	/// 
	/// Do NOT send this manually, this will be automatically handled by `nablo`.
	RemoveFont(FontId),
	/// Request host to add given string to clipboard.
	CopyToClipboard(String),
	/// Request host to get the content of the clipboard.
	RequestClipboard,
}

/// The cursor icon of the window.
/// 
/// Mainly warping the cursor icon from the `winit` crate.
#[derive(Debug, Clone)]
pub enum CursorIcon {
	Default,
	ContextMenu,
	Help,
	Pointer,
	Progress,
	Wait,
	Cell,
	Crosshair,
	Text,
	VerticalText,
	Alias,
	Copy,
	Move,
	NoDrop,
	NotAllowed,
	Grab,
	Grabbing,
	EResize,
	NResize,
	NeResize,
	NwResize,
	SResize,
	SeResize,
	SwResize,
	WResize,
	EwResize,
	NsResize,
	NeswResize,
	NwseResize,
	ColResize,
	RowResize,
	AllScroll,
	ZoomIn,
	ZoomOut,
}

impl From<CursorIcon> for winit::window::Cursor {
	fn from(value: CursorIcon) -> Self {
		match value {
			CursorIcon::Default => winit::window::Cursor::Icon(winit::window::CursorIcon::Default),
			CursorIcon::ContextMenu => winit::window::Cursor::Icon(winit::window::CursorIcon::ContextMenu),
			CursorIcon::Help => winit::window::Cursor::Icon(winit::window::CursorIcon::Help),
			CursorIcon::Pointer => winit::window::Cursor::Icon(winit::window::CursorIcon::Pointer),
			CursorIcon::Progress => winit::window::Cursor::Icon(winit::window::CursorIcon::Progress),
			CursorIcon::Wait => winit::window::Cursor::Icon(winit::window::CursorIcon::Wait),
			CursorIcon::Cell => winit::window::Cursor::Icon(winit::window::CursorIcon::Cell),
			CursorIcon::Crosshair => winit::window::Cursor::Icon(winit::window::CursorIcon::Crosshair),
			CursorIcon::Text => winit::window::Cursor::Icon(winit::window::CursorIcon::Text),
			CursorIcon::VerticalText => winit::window::Cursor::Icon(winit::window::CursorIcon::VerticalText),
			CursorIcon::Alias => winit::window::Cursor::Icon(winit::window::CursorIcon::Alias),
			CursorIcon::Copy => winit::window::Cursor::Icon(winit::window::CursorIcon::Copy),
			CursorIcon::Move => winit::window::Cursor::Icon(winit::window::CursorIcon::Move),
			CursorIcon::NoDrop => winit::window::Cursor::Icon(winit::window::CursorIcon::NoDrop),
			CursorIcon::NotAllowed => winit::window::Cursor::Icon(winit::window::CursorIcon::NotAllowed),
			CursorIcon::Grab => winit::window::Cursor::Icon(winit::window::CursorIcon::Grab),
			CursorIcon::Grabbing => winit::window::Cursor::Icon(winit::window::CursorIcon::Grabbing),
			CursorIcon::EResize => winit::window::Cursor::Icon(winit::window::CursorIcon::EResize),
			CursorIcon::NResize => winit::window::Cursor::Icon(winit::window::CursorIcon::NResize),
			CursorIcon::NeResize => winit::window::Cursor::Icon(winit::window::CursorIcon::NeResize),
			CursorIcon::NwResize => winit::window::Cursor::Icon(winit::window::CursorIcon::NwResize),
			CursorIcon::SResize => winit::window::Cursor::Icon(winit::window::CursorIcon::SResize),
			CursorIcon::SeResize => winit::window::Cursor::Icon(winit::window::CursorIcon::SeResize),
			CursorIcon::SwResize => winit::window::Cursor::Icon(winit::window::CursorIcon::SwResize),
			CursorIcon::WResize => winit::window::Cursor::Icon(winit::window::CursorIcon::WResize),
			CursorIcon::EwResize => winit::window::Cursor::Icon(winit::window::CursorIcon::EwResize),
			CursorIcon::NsResize => winit::window::Cursor::Icon(winit::window::CursorIcon::NsResize),
			CursorIcon::NeswResize => winit::window::Cursor::Icon(winit::window::CursorIcon::NeswResize),
			CursorIcon::NwseResize => winit::window::Cursor::Icon(winit::window::CursorIcon::NwseResize),
			CursorIcon::ColResize => winit::window::Cursor::Icon(winit::window::CursorIcon::ColResize),
			CursorIcon::RowResize => winit::window::Cursor::Icon(winit::window::CursorIcon::RowResize),
			CursorIcon::AllScroll => winit::window::Cursor::Icon(winit::window::CursorIcon::AllScroll),
			CursorIcon::ZoomIn => winit::window::Cursor::Icon(winit::window::CursorIcon::ZoomIn),
			CursorIcon::ZoomOut => winit::window::Cursor::Icon(winit::window::CursorIcon::ZoomOut),
		}
	}
}

/// The event that `nablo` interested in.
/// 
/// Mainly warping the events from the `winit` crate.
pub enum WindowEvent {
	/// Contains the new size of the window.
	Resized(Vec2),
	// /// Contains the new position of the window.
	// Moved(Vec2),
	/// Emit when the window is closed.
	CloseRequested,
	DroppedFile(PathBuf),
	HoveredFile(PathBuf),
	HoveredFileCancelled,
	/// Contains the new state of the window.
	Focused(bool),
	KeyPressed(Key),
	KeyReleased(Key),
	StringInput(String),
	ImeEnabled,
	Ime(ImeEvent),
	ImeDisabled,
	MouseMoved(Vec2),
	MouseEntered,
	MouseLeft,
	MouseWheel(Vec2),
	MousePressed(MouseButton),
	MouseReleased(MouseButton),
	Touch(Touch),
	ScaleFactor(f64),
	ThemeChanged(Theme),
	RedrawRequested,
	Unknown,
}

/// Input event from the IME.
pub enum ImeEvent {
	Commit(String),
	Edit(String, Option<(usize, usize)>),
}

/// The theme of the window.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Theme {
	Dark,
	Light,
}

/// Mouse button.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MouseButton {
	Left,
	Right,
	Middle,
	Back,
	Forward,
	Other(u16)
}

/// Touch event.
pub struct Touch {
	pub id: u64,
	pub pos: Vec2,
	pub phase: TouchPhase,
}

/// Touch phase.
#[derive(PartialEq, Eq)]
pub enum TouchPhase {
	Started,
	Moved,
	Ended,
	Cancelled,
}

/// Key that `nablo` interested in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Key {
	KeyA,
	KeyB,
	KeyC,
	KeyD,
	KeyE,
	KeyF,
	KeyG,
	KeyH,
	KeyI,
	KeyJ,
	KeyK,
	KeyL,
	KeyM,
	KeyN,
	KeyO,
	KeyP,
	KeyQ,
	KeyR,
	KeyS,
	KeyT,
	KeyU,
	KeyV,
	KeyW,
	KeyX,
	KeyY,
	KeyZ,
	Key0,
	Key1,
	Key2,
	Key3,
	Key4,
	Key5,
	Key6,
	Key7,
	Key8,
	Key9,
	/// on numpad
	Num0,
	Num1,
	Num2,
	Num3,
	Num4,
	Num5,
	Num6,
	Num7,
	Num8,
	Num9,
	Escape,
	F1,
	F2,
	F3,
	F4,
	F5,
	F6,
	F7,
	F8,
	F9,
	F10,
	F11,
	F12,
	Backspace,
	Backslash,
	Backquote,
	BracketLeft,
	BracketRight,
	Comma,
	Delete,
	End,
	Enter,
	Equal,
	Grave,
	Home,
	Insert,
	KeypadAdd,
	KeypadDecimal,
	KeypadDivide,
	KeypadEnter,
	KeypadEqual,
	KeypadMultiply,
	KeypadSubtract,
	// Left,
	Menu,
	Minus,
	NumLock,
	PageDown,
	PageUp,
	Pause,
	Period,
	Quote,
	Return,
	// Right,
	ScrollLock,
	Semicolon,
	Slash,
	Tab,
	CapsLock,
	ControlLeft,
	ControlRight,
	ShiftLeft,
	ShiftRight,
	SuperLeft,
	SuperRight,
	AltLeft,
	AltRight,
	MetaLeft,
	MetaRight,
	Space,
	ArrawLeft,
	ArrawRight,
	ArrawUp,
	ArrawDown,
	Fn,
	FnLock,
	PrintScreen,
	Unknown(u32),
}

impl Key {
	pub fn get_char(&self, is_holding_shift: bool) -> Option<char> {
		match self {
			Key::KeyA => if is_holding_shift { Some('A') } else { Some('a') },
			Key::KeyB => if is_holding_shift { Some('B') } else { Some('b') },
			Key::KeyC => if is_holding_shift { Some('C') } else { Some('c') },
			Key::KeyD => if is_holding_shift { Some('D') } else { Some('d') },
			Key::KeyE => if is_holding_shift { Some('E') } else { Some('e') },
			Key::KeyF => if is_holding_shift { Some('F') } else { Some('f') },
			Key::KeyG => if is_holding_shift { Some('G') } else { Some('g') },
			Key::KeyH => if is_holding_shift { Some('H') } else { Some('h') },
			Key::KeyI => if is_holding_shift { Some('I') } else { Some('i') },
			Key::KeyJ => if is_holding_shift { Some('J') } else { Some('j') },
			Key::KeyK => if is_holding_shift { Some('K') } else { Some('k') },
			Key::KeyL => if is_holding_shift { Some('L') } else { Some('l') },
			Key::KeyM => if is_holding_shift { Some('M') } else { Some('m') },
			Key::KeyN => if is_holding_shift { Some('N') } else { Some('n') },
			Key::KeyO => if is_holding_shift { Some('O') } else { Some('o') },
			Key::KeyP => if is_holding_shift { Some('P') } else { Some('p') },
			Key::KeyQ => if is_holding_shift { Some('Q') } else { Some('q') },
			Key::KeyR => if is_holding_shift { Some('R') } else { Some('r') },
			Key::KeyS => if is_holding_shift { Some('S') } else { Some('s') },
			Key::KeyT => if is_holding_shift { Some('T') } else { Some('t') },
			Key::KeyU => if is_holding_shift { Some('U') } else { Some('u') },
			Key::KeyV => if is_holding_shift { Some('V') } else { Some('v') },
			Key::KeyW => if is_holding_shift { Some('W') } else { Some('w') },
			Key::KeyX => if is_holding_shift { Some('X') } else { Some('x') },
			Key::KeyY => if is_holding_shift { Some('Y') } else { Some('y') },
			Key::KeyZ => if is_holding_shift { Some('Z') } else { Some('z') },
			Key::Key0 => if is_holding_shift { Some(')') } else { Some('0') },
			Key::Key1 => if is_holding_shift { Some('!') } else { Some('1') },
			Key::Key2 => if is_holding_shift { Some('@') } else { Some('2') },
			Key::Key3 => if is_holding_shift { Some('#') } else { Some('3') },
			Key::Key4 => if is_holding_shift { Some('$') } else { Some('4') },
			Key::Key5 => if is_holding_shift { Some('%') } else { Some('5') },
			Key::Key6 => if is_holding_shift { Some('^') } else { Some('6') },
			Key::Key7 => if is_holding_shift { Some('&') } else { Some('7') },
			Key::Key8 => if is_holding_shift { Some('*') } else { Some('8') },
			Key::Key9 => if is_holding_shift { Some('(') } else { Some('9') },
			Key::Num0 => Some('0'),
			Key::Num1 => Some('1'),
			Key::Num2 => Some('2'),
			Key::Num3 => Some('3'),
			Key::Num4 => Some('4'),
			Key::Num5 => Some('5'),
			Key::Num6 => Some('6'),
			Key::Num7 => Some('7'),
			Key::Num8 => Some('8'),
			Key::Num9 => Some('9'),
			Key::Escape => None,
			Key::F1 => None,
			Key::F2 => None,
			Key::F3 => None,
			Key::F4 => None,
			Key::F5 => None,
			Key::F6 => None,
			Key::F7 => None,
			Key::F8 => None,
			Key::F9 => None,
			Key::F10 => None,
			Key::F11 => None,
			Key::F12 => None,
			Key::Backspace => None,
			Key::Backslash => if is_holding_shift { Some('|') } else { Some('\\') },
			Key::Backquote => if is_holding_shift { Some('~') } else { Some('`') },
			Key::BracketLeft => if is_holding_shift { Some('{') } else { Some('[') },
			Key::BracketRight => if is_holding_shift { Some('}') } else { Some(']') },
			Key::Comma => if is_holding_shift { Some('<') } else { Some(',') },
			Key::Delete => None,
			Key::End => None,
			Key::Enter => Some('\n'),
			Key::Equal => if is_holding_shift { Some('+') } else { Some('=') },
			Key::Grave => if is_holding_shift { Some('~') } else { Some('`') },
			Key::Home => None,
			Key::Insert => None,
			Key::KeypadAdd => None,
			Key::KeypadDecimal => None,
			Key::KeypadDivide => None,
			Key::KeypadEnter => Some('\n'),
			Key::KeypadEqual => None,
			Key::KeypadMultiply => None,
			Key::KeypadSubtract => None,
			// Key::Left => None,
			Key::Menu => None,
			Key::Minus => if is_holding_shift { Some('_') } else { Some('-') },
			Key::NumLock => None,
			Key::PageDown => None,
			Key::PageUp => None,
			Key::Pause => None,
			Key::Period => if is_holding_shift { Some('>') } else { Some('.') },
			Key::Quote => if is_holding_shift { Some('"') } else { Some('\'') },
			Key::Return => None,
			// Key::Right => None,
			Key::ScrollLock => None,
			Key::Semicolon => if is_holding_shift { Some(':') } else { Some(';') },
			Key::Slash => if is_holding_shift { Some('?') } else { Some('/') },
			Key::Space => Some(' '),
			Key::Tab => None,
			Key::CapsLock => None,
			Key::ControlLeft => None,
			Key::ControlRight => None,
			Key::ShiftLeft => None,
			Key::ShiftRight => None,
			Key::SuperLeft => None,
			Key::SuperRight => None,
			Key::AltLeft => None,
			Key::AltRight => None,
			Key::MetaLeft => None,
			Key::MetaRight => None,
			Key::ArrawLeft => None,
			Key::ArrawRight => None,
			Key::ArrawUp => None,
			Key::ArrawDown => None,
			Key::Unknown(_) => None,
			Key::Fn => None,
			Key::FnLock => None,
			Key::PrintScreen => None,
		}
	}
}

impl From<WinitEvent> for WindowEvent {
	fn from(event: WinitEvent) -> Self {
		match event {
			WinitEvent::Resized(size) => WindowEvent::Resized(Vec2::new(size.width as f32, size.height as f32)),
			// WinitEvent::Moved(pos) => WindowEvent::Moved(Vec2::new(pos.x as f32, pos.y as f32)),
			WinitEvent::CloseRequested => WindowEvent::CloseRequested,
			WinitEvent::DroppedFile(path) => WindowEvent::DroppedFile(path),
			WinitEvent::HoveredFile(path) => WindowEvent::HoveredFile(path),
			WinitEvent::HoveredFileCancelled => WindowEvent::HoveredFileCancelled,
			WinitEvent::Focused(focused) => WindowEvent::Focused(focused),
			WinitEvent::KeyboardInput { event, .. } => {
				let key = Key::from(event.physical_key);
				if event.state == winit::event::ElementState::Pressed {
					WindowEvent::KeyPressed(key)
				} else {
					WindowEvent::KeyReleased(key)
				}
			},
			WinitEvent::Ime(ime_event) => {
				match ime_event {
					Ime::Enabled => WindowEvent::ImeEnabled,
					Ime::Disabled => WindowEvent::ImeDisabled,
					Ime::Preedit(string, range) => WindowEvent::Ime(ImeEvent::Edit(string, range)),
					Ime::Commit(string) => WindowEvent::Ime(ImeEvent::Commit(string)),
				}
			},
			WinitEvent::CursorMoved { position, .. } => WindowEvent::MouseMoved(Vec2::new(position.x as f32, position.y as f32)),
			WinitEvent::CursorEntered { .. } => WindowEvent::MouseEntered,
			WinitEvent::CursorLeft { .. } => WindowEvent::MouseLeft,
			WinitEvent::MouseWheel { delta, .. } => {
				match delta {
					MouseScrollDelta::LineDelta(line, column) => WindowEvent::MouseWheel(Vec2::new(line, column) * EM),
					MouseScrollDelta::PixelDelta(delta) => WindowEvent::MouseWheel(Vec2::new(delta.x as f32, delta.y as f32)),
				}
			},
			WinitEvent::MouseInput { state, button, .. } => {
				let button = match button {
					winit::event::MouseButton::Left => MouseButton::Left,
					winit::event::MouseButton::Right => MouseButton::Right,
					winit::event::MouseButton::Middle => MouseButton::Middle,
					winit::event::MouseButton::Other(id) => MouseButton::Other(id),
					winit::event::MouseButton::Back => MouseButton::Back,
					winit::event::MouseButton::Forward => MouseButton::Forward,
				};
				if state == winit::event::ElementState::Pressed {
					WindowEvent::MousePressed(button)
				} else {
					WindowEvent::MouseReleased(button)
				}
			},
			WinitEvent::Touch(touch) => {
				let phase = match touch.phase {
					winit::event::TouchPhase::Started => TouchPhase::Started,
					winit::event::TouchPhase::Moved => TouchPhase::Moved,
					winit::event::TouchPhase::Ended => TouchPhase::Ended,
					winit::event::TouchPhase::Cancelled => TouchPhase::Cancelled,
				};
				WindowEvent::Touch(Touch {
					id: touch.id,
					pos: Vec2::new(touch.location.x as f32, touch.location.y as f32),
					phase,
				})
			},
			WinitEvent::ScaleFactorChanged { scale_factor, .. } => WindowEvent::ScaleFactor(scale_factor),
			WinitEvent::ThemeChanged(theme) => {
				match theme {
					winit::window::Theme::Light => WindowEvent::ThemeChanged(Theme::Light),
					winit::window::Theme::Dark => WindowEvent::ThemeChanged(Theme::Dark),
				}
			},
			WinitEvent::RedrawRequested => WindowEvent::RedrawRequested,
			_ => WindowEvent::Unknown,
		}
	}
}

impl From<PhysicalKey> for Key {
	fn from(key: PhysicalKey) -> Self {
		match key {
			PhysicalKey::Code(code) => {
				use winit::keyboard::KeyCode::*;
				match code {
					Backquote => Key::Backquote,
					Backslash => Key::Backslash,
					BracketLeft => Key::BracketLeft,
					BracketRight => Key::BracketRight,
					Comma => Key::Comma,
					Digit0 => Key::Key0,
					Digit1 => Key::Key1,
					Digit2 => Key::Key2,
					Digit3 => Key::Key3,
					Digit4 => Key::Key4,
					Digit5 => Key::Key5,
					Digit6 => Key::Key6,
					Digit7 => Key::Key7,
					Digit8 => Key::Key8,
					Digit9 => Key::Key9,
					Equal => Key::Equal,
					IntlBackslash => Key::Backslash,
					IntlRo => Key::Slash,
					IntlYen => Key::Backslash,
					KeyA => Key::KeyA,
					KeyB => Key::KeyB,
					KeyC => Key::KeyC,
					KeyD => Key::KeyD,
					KeyE => Key::KeyE,
					KeyF => Key::KeyF,
					KeyG => Key::KeyG,
					KeyH => Key::KeyH,
					KeyI => Key::KeyI,
					KeyJ => Key::KeyJ,
					KeyK => Key::KeyK,
					KeyL => Key::KeyL,
					KeyM => Key::KeyM,
					KeyN => Key::KeyN,
					KeyO => Key::KeyO,
					KeyP => Key::KeyP,
					KeyQ => Key::KeyQ,
					KeyR => Key::KeyR,
					KeyS => Key::KeyS,
					KeyT => Key::KeyT,
					KeyU => Key::KeyU,
					KeyV => Key::KeyV,
					KeyW => Key::KeyW,
					KeyX => Key::KeyX,
					KeyY => Key::KeyY,
					KeyZ => Key::KeyZ,
					Minus => Key::Minus,
					Period => Key::Period,
					Quote => Key::Quote,
					Semicolon => Key::Semicolon,
					Slash => Key::Slash,
					AltLeft => Key::AltLeft,
					AltRight => Key::AltRight,
					Backspace => Key::Backspace,
					CapsLock => Key::CapsLock,
					ContextMenu => Key::Menu,
					ControlLeft => Key::ControlLeft,
					ControlRight => Key::ControlRight,
					Enter => Key::Enter,
					SuperLeft => Key::SuperLeft,
					SuperRight => Key::SuperRight,
					ShiftLeft => Key::ShiftLeft,
					ShiftRight => Key::ShiftRight,
					Space => Key::Space,
					Tab => Key::Tab,
					Delete => Key::Delete,
					End => Key::End,
					Home => Key::Home,
					Insert => Key::Insert,
					PageDown => Key::PageDown,
					PageUp => Key::PageUp,
					ArrowDown => Key::ArrawDown,
					ArrowLeft => Key::ArrawLeft,
					ArrowRight => Key::ArrawRight,
					ArrowUp => Key::ArrawUp,
					NumLock => Key::NumLock,
					Numpad0 => Key::Num0,
					Numpad1 => Key::Num1,
					Numpad2 => Key::Num2,
					Numpad3 => Key::Num3,
					Numpad4 => Key::Num4,
					Numpad5 => Key::Num5,
					Numpad6 => Key::Num6,
					Numpad7 => Key::Num7,
					Numpad8 => Key::Num8,
					Numpad9 => Key::Num9,
					NumpadAdd => Key::KeypadAdd,
					NumpadBackspace => Key::Backspace,
					NumpadComma => Key::Comma,
					NumpadDecimal => Key::KeypadDecimal,
					NumpadDivide => Key::KeypadDivide,
					NumpadEnter => Key::Enter,
					NumpadEqual => Key::KeypadEqual,
					Escape => Key::Escape,
					Fn => Key::Fn,
					FnLock => Key::FnLock,
					PrintScreen => Key::PrintScreen,
					ScrollLock => Key::ScrollLock,
					Pause => Key::Pause,
					F1 => Key::F1,
					F2 => Key::F2,
					F3 => Key::F3,
					F4 => Key::F4,
					F5 => Key::F5,
					F6 => Key::F6,
					F7 => Key::F7,
					F8 => Key::F8,
					F9 => Key::F9,
					F10 => Key::F10,
					F11 => Key::F11,
					F12 => Key::F12,
					_ => Key::Unknown(code as u32),
				}
			},
			PhysicalKey::Unidentified(code) => {
				match code {
					NativeKeyCode::Unidentified => Key::Unknown(0),
					NativeKeyCode::Android(code) => Key::Unknown(code),
					NativeKeyCode::MacOS(code) => Key::Unknown(code as u32),
					NativeKeyCode::Windows(code) => Key::Unknown(code as u32),
					NativeKeyCode::Xkb(code) => Key::Unknown(code),
				}
			}
		}
	}
}

// pub fn poll_events() -> Vec<WindowEvent> {
// 	todo!()
// }