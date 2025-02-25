//! A simple 2D vector implementation

use std::{fmt::Display, iter::Sum, ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign}};

/// A simple 2D vector implementation
#[derive(Debug, Copy, Clone, PartialEq, Default)]
#[derive(serde::Deserialize, serde::Serialize)]
pub struct Vec2 {
	pub x: f32,
	pub y: f32,
}

impl Vec2 {
	pub const ZERO: Self = Self::new(0.0, 0.0);
	pub const ONE: Self = Self::new(1.0, 1.0);
	pub const X: Self = Self::new(1.0, 0.0);
	pub const Y: Self = Self::new(0.0, 1.0);
	pub const INF: Self = Self::new(f32::INFINITY, f32::INFINITY);
	pub const NAN: Self = Self::new(f32::NAN, f32::NAN);

	/// Create a new vector with the given x and y values
	pub const fn new(x: f32, y: f32) -> Self {
		Self { x, y }
	}

	/// Create a new vector with the same value for both x and y
	pub const fn same(value: f32) -> Self {
		Self { x: value, y: value }
	}

	/// Create a new vector with the given x value and y set to 0
	pub const fn x(x: f32) -> Self {
		Self { x, y: 0.0 }
	}

	/// Create a new vector with the given y value and x set to 0
	pub const fn y(y: f32) -> Self {
		Self { x: 0.0, y }
	}

	/// Create a new vector in polar coordinates with the given magnitude and angle
	pub fn from_polar(magnitude: f32, angle: f32) -> Self {
		Self {
			x: magnitude * angle.cos(),
			y: magnitude * angle.sin(),
		}
	}

	/// Get the p norm of the vector
	/// 
	/// p = 0: Manhattan distance.
	/// p = 1: Euclidean distance.
	/// p = 2: Minkowski distance.
	pub fn norm(self, p: f32) -> f32 {
		(self.x.powf(p) + self.y.powf(p)).powf(1.0 / p)
	}

	/// Get the dot product of two vectors
	pub fn dot(self, other: Self) -> f32 {
		self.x * other.x + self.y * other.y
	}

	/// Get the cross product of two vectors
	pub fn cross(self, other: Self) -> f32 {
		self.x * other.y - self.y * other.x
	}

	/// Get the angle between two vectors in radians
	pub fn angle(self, other: Self) -> f32 {
		(self.dot(other) / (self.norm(2.0) * other.norm(2.0))).acos()
	}

	/// Get the angle between two vectors in degrees
	pub fn angle_degrees(self, other: Self) -> f32 {
		self.angle(other) * 180.0 / std::f32::consts::PI
	}

	/// Get the projection of one vector onto another
	pub fn project(self, other: Self) -> Self {
		other * (self.dot(other) / other.dot(other))
	}

	/// Get the reflection of one vector off of another
	pub fn reflect(self, other: Self) -> Self {
		self - 2.0 * self.project(other)
	}

	/// Get the refraction of one vector through another
	pub fn refract(self, other: Self, eta: f32) -> Self {
		let dot = self.dot(other);
		let k = 1.0 - eta * eta * (1.0 - dot * dot);
		if k < 0.0 {
			Self::ZERO
		} else {
			eta * self - (eta * dot + k.sqrt()) * other
		}
	}

	/// Get the length of the vector
	pub fn length(self) -> f32 {
		self.norm(2.0)
	}

	/// Get the squared length of the vector
	pub fn length_squared(self) -> f32 {
		self.x * self.x + self.y * self.y
	}

	/// Normalize the vector to have a length of 1
	pub fn normalize(self) -> Self {
		let length = self.length();
		if length == 0.0 {
			Self::ZERO
		} else {
			Self {
				x: self.x / length,
				y: self.y / length,
			}
		}
	}

	/// Get the vector rotated by the given angle in radians
	pub fn rotated(self, angle: f32) -> Self {
		let cos = angle.cos();
		let sin = angle.sin();
		Self {
			x: self.x * cos - self.y * sin,
			y: self.x * sin + self.y * cos,
		}
	}

	/// Get the vector rotated by the given angle in degrees
	pub fn rotated_degrees(self, angle: f32) -> Self {
		self.rotated(angle * std::f32::consts::PI / 180.0)
	}

	/// Get the vector's angle in radians with respect to the x-axis
	pub fn angle_x(self) -> f32 {
		self.y.atan2(self.x)
	}

	/// Get the vector's angle in degrees with respect to the x-axis
	pub fn angle_x_degrees(self) -> f32 {
		self.angle_x() * 180.0 / std::f32::consts::PI
	}

	/// Get the vector's angle in radians with respect to the y-axis
	pub fn angle_y(self) -> f32 {
		self.x.atan2(self.y)
	}

	/// Get the vector's angle in degrees with respect to the y-axis
	pub fn angle_y_degrees(self) -> f32 {
		self.angle_y() * 180.0 / std::f32::consts::PI
	}

	/// Get the vector with the x and y components swapped
	pub fn yx(self) -> Self {
		Self {
			x: self.y,
			y: self.x,
		}
	}

	/// Clamp the vector to the given length
	pub fn clamp_length(self, max_length: f32) -> Self {
		let length = self.length();
		if length > max_length {
			self.normalize() * max_length
		} else {
			self
		}
	}

	/// Clamp the vector's x and y components to the given range
	pub fn clamp(self, min: f32, max: f32) -> Self {
		Self {
			x: self.x.clamp(min, max),
			y: self.y.clamp(min, max),
		}
	}

	/// Clamp the vector's both components to the given range
	pub fn clamp_both(self, min: Vec2, max: Vec2) -> Self {
		Self {
			x: self.x.clamp(min.x, max.x),
			y: self.y.clamp(min.y, max.y),
		}
	}

	/// Get the vector with the absolute value of each component
	pub fn abs(self) -> Self {
		Self {
			x: self.x.abs(),
			y: self.y.abs(),
		}
	}

	/// Get the vector with the sign of each component
	pub fn sign(self) -> Self {
		Self {
			x: self.x.signum(),
			y: self.y.signum(),
		}
	}

	/// Get the vector with the floor of each component
	pub fn floor(self) -> Self {
		Self {
			x: self.x.floor(),
			y: self.y.floor(),
		}
	}

	/// Get the vector with the ceil of each component
	pub fn ceil(self) -> Self {
		Self {
			x: self.x.ceil(),
			y: self.y.ceil(),
		}
	}

	/// Get the vector with the round of each component
	pub fn round(self) -> Self {
		Self {
			x: self.x.round(),
			y: self.y.round(),
		}
	}

	/// Get the vector with the trunc of each component
	pub fn trunc(self) -> Self {
		Self {
			x: self.x.trunc(),
			y: self.y.trunc(),
		}
	}

	/// Get the vector with the fract of each component
	pub fn fract(self) -> Self {
		Self {
			x: self.x - self.x.floor(),
			y: self.y - self.y.floor(),
		}
	}

	/// Get the vector with the minimum value of each component
	pub fn min(self, other: Self) -> Self {
		Self {
			x: self.x.min(other.x),
			y: self.y.min(other.y),
		}
	}

	/// Get the vector with the minimum value of each component
	pub fn min_both(self, other: Self) -> Self {
		Self {
			x: self.x.min(other.x),
			y: self.y.min(other.y),
		}
	}

	/// Get the vector with the maximum value of each component
	pub fn max(self, other: Self) -> Self {
		Self {
			x: self.x.max(other.x),
			y: self.y.max(other.y),
		}
	}

	/// Get the vector with the maximum value of each component
	pub fn max_both(self, other: Self) -> Self {
		Self {
			x: self.x.max(other.x),
			y: self.y.max(other.y),
		}
	}

	/// Check if the vector is zero
	pub fn is_zero(self) -> bool {
		self.x == 0.0 && self.y == 0.0
	}

	/// Check if the vector contains only finite values
	pub fn is_finite(self) -> bool {
		self.x.is_finite() && self.y.is_finite()
	}

	/// Check if the vector is normalized
	pub fn is_normalized(self) -> bool {
		self.length() == 1.0
	}

	/// Check if the vector contains nan values
	pub fn has_nan(self) -> bool {
		self.x.is_nan() || self.y.is_nan()
	}

	/// Check if the vector contains inf values
	pub fn has_inf(self) -> bool {
		self.x.is_infinite() || self.y.is_infinite()
	}
}

impl Add for Vec2 {
	type Output = Self;

	fn add(self, other: Self) -> Self {
		Self {
			x: self.x + other.x,
			y: self.y + other.y,
		}
	}
}

impl Sub for Vec2 {
	type Output = Self;

	fn sub(self, other: Self) -> Self {
		Self {
			x: self.x - other.x,
			y: self.y - other.y,
		}
	}
}

impl Mul<f32> for Vec2 {
	type Output = Self;

	fn mul(self, other: f32) -> Self {
		Self {
			x: self.x * other,
			y: self.y * other,
		}
	}
}

impl Div<f32> for Vec2 {
	type Output = Self;

	fn div(self, other: f32) -> Self {
		Self {
			x: self.x / other,
			y: self.y / other,
		}
	}
}

impl Neg for Vec2 {
	type Output = Self;

	fn neg(self) -> Self {
		Self {
			x: -self.x,
			y: -self.y,
		}
	}
}

impl Mul<Vec2> for f32 {
	type Output = Vec2;

	fn mul(self, other: Vec2) -> Vec2 {
		Vec2 {
			x: self * other.x,
			y: self * other.y,
		}
	}
}

impl Mul for Vec2 {
	type Output = Vec2;

	fn mul(self, other: Self) -> Vec2 {
		Vec2 {
			x: self.x * other.x,
			y: self.y * other.y,
		}
	}
}

impl Div for Vec2 {
	type Output = Vec2;

	fn div(self, other: Self) -> Vec2 {
		Vec2 {
			x: self.x / other.x,
			y: self.y / other.y,
		}
	}
}

impl From<Vec2> for [f32; 2] {
	fn from(v: Vec2) -> [f32; 2] {
		[v.x, v.y]
	}
}

impl From<[f32; 2]> for Vec2 {
	fn from(v: [f32; 2]) -> Self {
		Self { x: v[0], y: v[1] }
	}
}

impl From<Vec2> for (f32, f32) {
	fn from(v: Vec2) -> (f32, f32) {
		(v.x, v.y)
	}
}

impl From<(f32, f32)> for Vec2 {
	fn from(v: (f32, f32)) -> Self {
		Self { x: v.0, y: v.1 }
	}
}

/// Create a new vector with the given x and y values
pub fn vec2(x: f32, y: f32) -> Vec2 {
	Vec2::new(x, y)
}

impl AddAssign for Vec2 {
	fn add_assign(&mut self, other: Self) {
		*self = Self {
			x: self.x + other.x,
			y: self.y + other.y,
		}
	}
}

impl SubAssign for Vec2 {
	fn sub_assign(&mut self, other: Self) {
		*self = Self {
			x: self.x - other.x,
			y: self.y - other.y,
		}
	}
}

impl MulAssign<f32> for Vec2 {
	fn mul_assign(&mut self, other: f32) {
		*self = Self {
			x: self.x * other,
			y: self.y * other,
		}
	}
}

impl MulAssign for Vec2 {
	fn mul_assign(&mut self, other: Self) {
		*self = Self {
			x: self.x * other.x,
			y: self.y * other.y,
		}
	}
}

impl DivAssign<f32> for Vec2 {
	fn div_assign(&mut self, other: f32) {
		*self = Self {
			x: self.x / other,
			y: self.y / other,
		}
	}
}

impl DivAssign for Vec2 {
	fn div_assign(&mut self, other: Self) {
		*self = Self {
			x: self.x / other.x,
			y: self.y / other.y,
		}
	}
}

impl Display for Vec2 {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "({}, {})", self.x, self.y)
	}
}

impl<T> Sum<T> for Vec2 
where
	T: Into<Vec2>
{
	fn sum<I: Iterator<Item = T>>(iter: I) -> Self {
		iter.fold(Self::ZERO, |a, b| a + b.into())
	}
}

impl From<&Vec2> for Vec2 {
	fn from(v: &Vec2) -> Self {
		*v
	}
}