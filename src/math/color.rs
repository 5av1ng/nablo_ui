//! A simple color struct with red, green, and blue and alpha components.
//! 
//! You can also use the `Color` as a `Vec4` Type.

use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};


/// You can use the `Color` as a `Vec4` Type.
pub type Vec4 = Color;

/// A simple color struct with red, green, and blue and alpha components.
/// 
/// You can also use the `Color` as a `Vec4` Type if you want by using `x`, `y`, `z`, and `w` methods.
/// The color is unpremultiplied. You need use `premultiply` method to get the premultiplied color manually.
#[derive(Debug, Clone, Copy, PartialEq)]
#[derive(serde::Deserialize, serde::Serialize)]
pub struct Color {
	pub r: f32,
	pub g: f32,
	pub b: f32,
	pub a: f32,
}

impl Color {
	pub const BLACK: Self = Self::new(0.0, 0.0, 0.0, 1.0);
	pub const WHITE: Self = Self::new(1.0, 1.0, 1.0, 1.0);
	pub const RED: Self = Self::new(1.0, 0.0, 0.0, 1.0);
	pub const GREEN: Self = Self::new(0.0, 1.0, 0.0, 1.0);
	pub const BLUE: Self = Self::new(0.0, 0.0, 1.0, 1.0);
	pub const YELLOW: Self = Self::new(1.0, 1.0, 0.0, 1.0);
	pub const CYAN: Self = Self::new(0.0, 1.0, 1.0, 1.0);
	pub const MAGENTA: Self = Self::new(1.0, 0.0, 1.0, 1.0);
	pub const TRANSPARENT: Self = Self::new(0.0, 0.0, 0.0, 0.0);

	pub const ZERO: Self = Self::same(0.0);
	pub const ONE: Self = Self::same(1.0);

	/// Create a new color with the given red, green, blue, and alpha components.
	pub const fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
		Self { r, g, b, a }
	}

	pub const fn same(input: f32) -> Self {
		Self::new(input, input, input, input)
	}

	/// Create a new color with the given red, green, blue, and alpha components.
	pub fn from_rgba_u8(r: u8, g: u8, b: u8, a: u8) -> Self {
		Self::new(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, a as f32 / 255.0)
	}

	/// Create a new color with the given red, green, blue, and alpha components.
	/// 
	/// Will clamp the values to the range [0.0, 1.0].
	pub  fn from_rgba_f32(r: f32, g: f32, b: f32, a: f32) -> Self {
		Self::new(r.clamp(0.0, 1.0), g.clamp(0.0, 1.0), b.clamp(0.0, 1.0), a.clamp(0.0, 1.0))
	}

	/// Create a new color with the given red, green, blue components.
	/// 
	/// The alpha component will be set to 1.0.
	pub fn from_rgb_f32(r: f32, g: f32, b: f32) -> Self {
		Self::new(r.clamp(0.0, 1.0), g.clamp(0.0, 1.0), b.clamp(0.0, 1.0), 1.0)
	}

	/// Create a new color with the given red, green, blue components.
	/// 
	/// The alpha component will be set to 1.0.
	pub fn from_rgb_u8(r: u8, g: u8, b: u8) -> Self {
		Self::from_rgb_f32(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0)
	}

	/// Create a new color with the given hex value. 
	/// 
	/// The hex value should be in the format 0xRRGGBBAA.
	pub fn from_hex(hex: u32) -> Self {
		let r = ((hex >> 24) & 0xff) as f32 / 255.0;
		let g = ((hex >> 16) & 0xff) as f32 / 255.0;
		let b = ((hex >> 8) & 0xff) as f32 / 255.0;
		let a = (hex & 0xff) as f32 / 255.0;
		Self::new(r, g, b, a)
	}

	/// Create a new color with the given gray value.
	/// 
	/// Will clamp the values to the range [0.0, 1.0].
	pub fn from_gray_f32(value: f32) -> Self {
		let val = value.clamp(0.0, 1.0);
		Self::new(val, val, val, 1.0)
	}

	/// Create a new color with the given gray value.
	/// 
	/// The alpha component will be set to 1.0.
	pub fn from_gray_u8(value: u8) -> Self {
		let val = value as f32 / 255.0;
		Self::from_gray_f32(val)
	}
	/// Create a new color with gray value and alpha.
	/// 
	/// Will clamp the values to the range [0.0, 1.0].
	pub fn from_gray_alpha_f32(value: f32, alpha: f32) -> Self {
		let val = value.clamp(0.0, 1.0);
		Self::new(val, val, val, alpha.clamp(0.0, 1.0))
	}

	/// Create a new color with gray value and alpha.
	pub fn from_gray_alpha_u8(value: u8, alpha: u8) -> Self {
		let val = value as f32 / 255.0;
		let alpha = alpha as f32 / 255.0;
		Self::from_gray_alpha_f32(val, alpha)
	}

	/// Create a new color with the given HSL values.
	/// 
	/// The alpha component will be set to 1.0.
	/// Will not clamp the values.
	pub fn from_hsl(h: f32, s: f32, l: f32) -> Self {
		let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
		let x = c * (1.0 - ((h / 60.0).abs() % 2.0 - 1.0).abs());
		let m = l - c / 2.0;
		let (r, g, b) = match h {
			0.0..=60.0 => (c, x, 0.0),
			60.0..=120.0 => (x, c, 0.0),
			120.0..=180.0 => (0.0, c, x),
			180.0..=240.0 => (0.0, x, c),
			240.0..=300.0 => (x, 0.0, c),
			_ => (c, 0.0, x),
		};
		Self::new((r + m) * 255.0, (g + m) * 255.0, (b + m) * 255.0, 1.0)
	}

	/// Create a new color with the given HSLA values.
	/// 
	/// Will not clamp the values.
	pub fn from_hsla(h: f32, s: f32, l: f32, a: f32) -> Self {
		let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
		let x = c * (1.0 - ((h / 60.0).abs() % 2.0 - 1.0).abs());
		let m = l - c / 2.0;
		let (r, g, b) = match h {
			0.0..=60.0 => (c, x, 0.0),
			60.0..=120.0 => (x, c, 0.0),
			120.0..=180.0 => (0.0, c, x),
			180.0..=240.0 => (0.0, x, c),
			240.0..=300.0 => (x, 0.0, c),
			_ => (c, 0.0, x),
		};
		Self::new((r + m) * 255.0, (g + m) * 255.0, (b + m) * 255.0, a)
	}

	/// Create a new color with the given HSV values.
	/// 
	/// Will not clamp the values.
	pub fn from_hsv(h: f32, s: f32, v: f32) -> Self {
		let c = v * s;
		let x = c * (1.0 - ((h / 60.0).abs() % 2.0 - 1.0).abs());
		let m = v - c;
		let (r, g, b) = match h {
			0.0..=60.0 => (c, x, 0.0),
			60.0..=120.0 => (x, c, 0.0),
			120.0..=180.0 => (0.0, c, x),
			180.0..=240.0 => (0.0, x, c),
			240.0..=300.0 => (x, 0.0, c),
			_ => (c, 0.0, x),
		};
		Self::new((r + m) * 255.0, (g + m) * 255.0, (b + m) * 255.0, 1.0)
	}

	/// Create a new color with the given HSVA values.
	/// 
	/// Will not clamp the values.
	pub fn from_hsva(h: f32, s: f32, v: f32, a: f32) -> Self {
		let c = v * s;
		let x = c * (1.0 - ((h / 60.0).abs() % 2.0 - 1.0).abs());
		let m = v - c;
		let (r, g, b) = match h {
			0.0..=60.0 => (c, x, 0.0),
			60.0..=120.0 => (x, c, 0.0),
			120.0..=180.0 => (0.0, c, x),
			180.0..=240.0 => (0.0, x, c),
			240.0..=300.0 => (x, 0.0, c),
			_ => (c, 0.0, x),
		};
		Self::new((r + m) * 255.0, (g + m) * 255.0, (b + m) * 255.0, a)
	}

	/// Create a new color with the given CMYK values.
	/// 
	/// The alpha component will be set to 1.0.
	/// Will not clamp the values.
	pub fn from_cmyk(c: f32, m: f32, y: f32, k: f32) -> Self {
		let r = (1.0 - c) * (1.0 - k);
		let g = (1.0 - m) * (1.0 - k);
		let b = (1.0 - y) * (1.0 - k);
		Self::new(r, g, b, 1.0)
	}

	/// Create a new color with the given CMYKA values.
	/// 
	/// Will not clamp the values.
	pub fn from_cmyka(c: f32, m: f32, y: f32, k: f32, a: f32) -> Self {
		let r = (1.0 - c) * (1.0 - k);
		let g = (1.0 - m) * (1.0 - k);
		let b = (1.0 - y) * (1.0 - k);
		Self::new(r, g, b, a)
	}

	/// Encode the color as a 32-bit integer with the format 0xRRGGBBAA.
	/// 
	/// The alpha component is not premultiplied. You need to use `premultiply` method to get the premultiplied color.
	pub fn to_hex(self) -> u32 {
		let r = self.r.clamp(0.0, 1.0) * 255.0;
		let g = self.g.clamp(0.0, 1.0) * 255.0;
		let b = self.b.clamp(0.0, 1.0) * 255.0;
		let a = self.a.clamp(0.0, 1.0) * 255.0;

		((r as u32) << 24) |
		((g as u32) << 16) |
		((b as u32) << 8) |
		(a as u32)
	}

	/// Get premultiplied color.
	pub fn premultiply(self) -> Self {
		let a = self.a;
		let r = self.r * a;
		let g = self.g * a;
		let b = self.b * a;
		Self::new(r, g, b, a)
	}

	/// Get the inverse color.
	pub fn inverse(self) -> Self {
		Self::new(1.0 - self.r, 1.0 - self.g, 1.0 - self.b, self.a)
	}
	/// Get the grayscale color.
	pub fn grayscale(self) -> f32 {
		(self.r + self.g + self.b) / 3.0
	}

	/// Get the luminance of the color.
	pub fn luminance(self) -> f32 {
		let r = self.r;
		let g = self.g;
		let b = self.b;
		
		0.2126 * r + 0.7152 * g + 0.0722 * b
	}

	/// convert the color to HSLA color space.
	pub fn to_hsla(self) -> Self {
		let r = self.r;
		let g = self.g;
		let b = self.b;
		let a = self.a;

		let max = r.max(g).max(b);
		let min = r.min(g).min(b);
		let delta = max - min;

		let mut h = 0.0;
		let mut s = 0.0;
		let l = (max + min) / 2.0;

		if delta > 0.0001 {
			if max == r {
				h = (g - b) / delta;
			} else if max == g {
				h = 2.0 + (b - r) / delta;
			} else {
				h = 4.0 + (r - g) / delta;
			}

			h = (h * 60.0).rem_euclid(360.0);
			s = if delta == 0.0 {
				0.0
			} else {
				delta / (1.0 - (2.0 * l - 1.0).abs())
			};
		}

		Self::new(h, s, l, a)
	}

	/// convert the color to HSVA color space.
	pub fn to_hsva(self) -> Self {
		let r = self.r;
		let g = self.g;
		let b = self.b;
		let a = self.a;

		let max = r.max(g).max(b);
		let min = r.min(g).min(b);
		let delta = max - min;

		let mut h = 0.0;
		let s = if max == 0.0 {
			0.0
		} else {
			delta / max
		};
		let v = max;

		if delta > 0.0001 {
			if max == r {
				h = (g - b) / delta;
			} else if max == g {
				h = 2.0 + (b - r) / delta;
			} else {
				h = 4.0 + (r - g) / delta;
			}

			h = (h * 60.0).rem_euclid(360.0);
		}

		Self::new(h, s, v, a)
	}

	/// Convert the color to CMYK color space.
	pub fn to_cmyka(self) -> Self {
		let r = self.r;
		let g = self.g;
		let b = self.b;

		let k = 1.0 - r.max(g).max(b);
		let c = (1.0 - r - k) / (1.0 - k);
		let m = (1.0 - g - k) / (1.0 - k);
		let y = (1.0 - b - k) / (1.0 - k);

		Self::new(c, m, y, k)
	}

	/// Convert the color to LAB color space.
	/// 
	/// Will clamp the values to the range [0, 1].
	pub fn to_lab(self) -> Self {
		const XN: f32 = 95.047;
		const YN: f32 = 100.0;
		const ZN: f32 = 108.883;

		fn f(t: f32) -> f32 {
			if t > 0.008856 {
				t.cbrt()
			} else {
				7.787 * t + 16.0 / 116.0
			}
		}

		self.clamp(0.0, 1.0);
		let x = 0.412453 * self.r + 0.357580 * self.g + 0.180423 * self.b;
		let y = 0.212671 * self.r + 0.715160 * self.g + 0.072169 * self.b;
		let z = 0.019334 * self.r + 0.119193 * self.g + 0.950227 * self.b;

		let l = 116.0 * f(y / YN) - 16.0;
		let a = 500.0 * (f(x / XN) - f(y / YN));
		let b = 200.0 * (f(y / YN) - f(z / ZN));

		Self::new(l, a, b, self.a)
	}

	/// Clamp the color values to the range [min, max].
	pub fn clamp(self, min: f32, max: f32) -> Self {
		Self::new(
			self.r.clamp(min, max),
			self.g.clamp(min, max),
			self.b.clamp(min, max),
			self.a.clamp(min, max),
		)
	}

	/// Caculate the similarity between two colors by using LAB color space.
	/// 
	/// Will clamp the values to the range [0, 1].
	pub fn similarity(self, other: Self) -> f32 {
		let self_lab = self.to_lab();
		let other_lab = other.to_lab();

		let delta_l = (self_lab.x() - other_lab.x()).powi(2);
		let delta_a = (self_lab.y() - other_lab.y()).powi(2);
		let delta_b = (self_lab.z() - other_lab.z()).powi(2);

		(delta_l + delta_a + delta_b).sqrt()
	}

	/// Calculate the similarity by using weighting HSL distance color space.
	/// 
	/// the weighting factors are OMEGA_1 = 1.0, OMEGA_2 = 0.5, OMEGA_3 = 0.2.
	pub fn hsl_similarity(self, other: Self) -> f32 {
		const OMEGA_1: f32 = 1.0;
		const OMEGA_2: f32 = 0.5;
		const OMEGA_3: f32 = 0.2;

		let self_hsl = self.to_hsla();
		let other_hsl = other.to_hsla();
		
		let delta_h = (self_hsl.x() - other_hsl.x()).powi(2);
		let delta_s = (self_hsl.y() - other_hsl.y()).powi(2);
		let delta_l = (self_hsl.z() - other_hsl.z()).powi(2);

		let delta_h_weighted = delta_h * OMEGA_1;
		let delta_s_weighted = delta_s * OMEGA_2;
		let delta_l_weighted = delta_l * OMEGA_3;

		(delta_h_weighted + delta_s_weighted + delta_l_weighted).sqrt()
	}

	/// Calculate the length of the color vector.
	pub fn length(self) -> f32 {
		(self.r.powi(2) + self.g.powi(2) + self.b.powi(2) + self.a.powi(2)).sqrt()
	}

	/// Calculate the dot product of two colors.
	pub fn dot(self, other: Self) -> f32 {
		self.r * other.r + self.g * other.g + self.b * other.b + self.a * other.a
	}

	/// Calculate the cross product of two colors.
	/// 
	/// ignore the alpha component.
	pub fn cross(self, other: Self) -> Self {
		Self::new(
			self.g * other.b - self.b * other.g,
			self.b * other.r - self.r * other.b,
			self.r * other.g - self.g * other.r,
			0.0,
		)
	}

	/// Calculate the angle between two colors.
	pub fn angle(self, other: Self) -> f32 {
		let dot = self.dot(other);
		let len1 = self.length();
		let len2 = other.length();
		let angle = dot / (len1 * len2);
		angle.acos()
	}

	/// Linearly interpolate between two colors.
	pub fn lerp(self, other: Self, t: f32) -> Self {
		self * (1.0 - t) + other * t
	}

	/// Brighten the color by a factor.
	pub fn brighten(self, factor: f32) -> Self {
		self + factor * Color::WHITE
	}
}

impl Color {
	pub fn x(&self) -> f32 { self.r }
	pub fn y(&self) -> f32 { self.g }
	pub fn z(&self) -> f32 { self.b }
	pub fn w(&self) -> f32 { self.a }
	pub fn set_x(&mut self, x: f32) { self.r = x }
	pub fn set_y(&mut self, y: f32) { self.g = y }
	pub fn set_z(&mut self, z: f32) { self.b = z }
	pub fn set_w(&mut self, w: f32) { self.a = w }
}

impl Default for Color {
	fn default() -> Self {
		Self::new(0.0, 0.0, 0.0, 1.0)
	}
}

impl Add for Color {
	type Output = Self;

	fn add(self, other: Self) -> Self {
		Self::new(
			self.r + other.r,
			self.g + other.g,
			self.b + other.b,
			self.a + other.a,
		)
	}
}

impl Sub for Color {
	type Output = Self;

	fn sub(self, other: Self) -> Self {
		Self::new(
			self.r - other.r,
			self.g - other.g,
			self.b - other.b,
			self.a - other.a,
		)
	}
}

impl Mul for Color {
	type Output = Self;

	fn mul(self, other: Self) -> Self {
		Self::new(
			self.r * other.r,
			self.g * other.g,
			self.b * other.b,
			self.a * other.a,
		)
	}
}

impl Div for Color {
	type Output = Self;

	fn div(self, other: Self) -> Self {
		Self::new(
			self.r / other.r,
			self.g / other.g,
			self.b / other.b,
			self.a / other.a,
		)
	}
}

impl Mul<f32> for Color {
	type Output = Self;

	fn mul(self, other: f32) -> Self {
		Self::new(self.r * other, self.g * other, self.b * other, self.a * other)
	}
}

impl Div<f32> for Color {
	type Output = Self;

	fn div(self, other: f32) -> Self {
		Self::new(self.r / other, self.g / other, self.b / other, self.a / other)
	}
}

impl Mul<Color> for f32 {
	type Output = Color;

	fn mul(self, other: Color) -> Color {
		Color::new(self * other.r, self * other.g, self * other.b, self * other.a)
	}
}

impl AddAssign for Color {
	fn add_assign(&mut self, other: Self) {
		self.r += other.r;
		self.g += other.g;
		self.b += other.b;
		self.a += other.a;
	}
}

impl SubAssign for Color {
	fn sub_assign(&mut self, other: Self) {
		self.r -= other.r;
		self.g -= other.g;
		self.b -= other.b;
		self.a -= other.a;
	}
}

impl MulAssign for Color {
	fn mul_assign(&mut self, other: Self) {
		self.r *= other.r;
		self.g *= other.g;
		self.b *= other.b;
		self.a *= other.a;
	}
}

impl DivAssign for Color {
	fn div_assign(&mut self, other: Self) {
		self.r /= other.r;
		self.g /= other.g;
		self.b /= other.b;
		self.a /= other.a;
	}
}

impl MulAssign<f32> for Color {
	fn mul_assign(&mut self, other: f32) {
		self.r *= other;
		self.g *= other;
		self.b *= other;
		self.a *= other;
	}
}

impl DivAssign<f32> for Color {
	fn div_assign(&mut self, other: f32) {
		self.r /= other;
		self.g /= other;
		self.b /= other;
		self.a /= other;
	}
}

/// Create a new color from RGBA values.
pub fn rgba(r: f32, g: f32, b: f32, a: f32) -> Color {
	Color::new(r, g, b, a)
}

/// Create a new color from HSLA values.
pub fn hsla(h: f32, s: f32, l: f32, a: f32) -> Color {
	Color::from_hsla(h, s, l, a)
}

/// Create a new color from HSVA values.
pub fn hsva(h: f32, s: f32, v: f32, a: f32) -> Color {
	Color::from_hsva(h, s, v, a)
}

/// Create a new color from CMYKA values.
pub fn cmyka(c: f32, m: f32, y: f32, k: f32, a: f32) -> Color {
	Color::from_cmyka(c, m, y, k, a)
}

/// Create a Vec4
pub fn vec4(x: f32, y: f32, z: f32, w: f32) -> Color {
	Color::new(x, y, z, w)
}

/// Create a color
pub fn color(r: f32, g: f32, b: f32, a: f32) -> Color {
	Color::new(r, g, b, a)
}

impl std::fmt::Display for Color {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "({}, {}, {}, {})", self.r, self.g, self.b, self.a)
	}
}