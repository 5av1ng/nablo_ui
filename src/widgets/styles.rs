//! The main color scheme for the application.

use crate::prelude::{Color, EM};

/// The default background color.
pub static BACKGROUND_COLOR: Color = Color::new(0x1E as f32 / 255.0, 0x1E as f32 / 255.0, 0x1E as f32 / 255.0, 1.0);
/// The default background color of the card.
pub static CARD_COLOR: Color = Color::new(0x2A as f32 / 255.0, 0x2A as f32 / 255.0, 0x2A as f32 / 255.0, 1.0);
/// The default border color of the card.
pub static CARD_BORDER_COLOR: Color = Color::new(0x3D as f32 / 255.0, 0x3D as f32 / 255.0, 0x3D as f32 / 255.0, 1.0);

/// The default background color of the button, selectable label, and other clickable elements.
pub static PRIMARY_COLOR: Color = Color::new(0x8A as f32 / 255.0, 0x6A as f32 / 255.0, 0xFF as f32 / 255.0, 1.0);
/// The default background color of the button, selectable label, and other clickable elements when disabled.
pub static DISABLE_COLOR: Color = Color::new(0x5A as f32 / 255.0, 0x4A as f32 / 255.0, 0x8F as f32 / 255.0, 1.0);
/// The default bright factoe of the widget's background color when hovered.
pub static BRIGHT_FACTOR: f32 = 0.075;

/// The default colors for the error message.
pub static ERROR_COLOR: Color = Color::new(0xFF as f32 / 255.0, 0x4D as f32 / 255.0, 0x6D as f32 / 255.0, 1.0);
/// The default colors for the success message.
pub static SUCCESS_COLOR: Color = Color::new(0x00 as f32 / 255.0, 0xC8 as f32 / 255.0, 0x97 as f32 / 255.0, 1.0);
/// The default colors for the warning message.
pub static WARNING_COLOR: Color = Color::new(0xFF as f32 / 255.0, 0xB8 as f32 / 255.0, 0x5C as f32 / 255.0, 1.0);

/// The default title colors for the application.
pub static PRIMARY_TEXT_COLOR: Color = Color::new(0xE0 as f32 / 255.0, 0xE0 as f32 / 255.0, 0xE0 as f32 / 255.0, 1.0);
/// The default text colors for the application.
pub static SECONDARY_TEXT_COLOR: Color = Color::new(0xB0 as f32 / 255.0, 0xB0 as f32 / 255.0, 0xB0 as f32 / 255.0, 1.0);
/// The default disabled text colors for the application.
pub static DISABLE_TEXT_COLOR: Color = Color::new(0x70 as f32 / 255.0, 0x70 as f32 / 255.0, 0x70 as f32 / 255.0, 1.0);

/// The default font size for the application title.
pub static TITLE_TEXT_SIZE: f32 = EM * 1.5;
/// The default font size for the application.
pub static CONTENT_TEXT_SIZE: f32 = EM;

/// The background color for input fields (e.g., text boxes).
pub static INPUT_BACKGROUND_COLOR: Color = Color::new(0x33 as f32 / 255.0, 0x33 as f32 / 255.0, 0x33 as f32 / 255.0, 1.0);

/// The border color for input fields while unfocused (e.g., text boxes).
pub static INPUT_BORDER_COLOR: Color = Color::new(0x44 as f32 / 255.0, 0x44 as f32 / 255.0, 0x44 as f32 / 255.0, 1.0);
/// The color for selected text in input fields (e.g., text boxes).
pub static SELECTED_TEXT_COLOR: Color = Color::new(0x8A as f32 / 255.0, 0x6A as f32 / 255.0, 0xFF as f32 / 255.0, 0.3);

/// The default padding for the application.
pub static DEFAULT_PADDING: f32 = EM / 2.0;
/// The default rounding for the application.
pub static DEFAULT_ROUNDING: f32 = EM / 2.0;