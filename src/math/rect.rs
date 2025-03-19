//! A simple rectangle class with logical operators and methods.

use std::{fmt::Display, ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, Neg, Sub}};

use rstar::{Envelope, Point};

use super::{prelude::Transform2D, vec2::Vec2};

/// A simple rectangle class with logical operators and methods.
#[derive(Debug, Clone, Copy, PartialEq)]
#[derive(serde::Deserialize, serde::Serialize)]
pub struct Rect {
	pub x: f32,
	pub y: f32,
	pub w: f32,
	pub h: f32,
}

impl Rect {
	pub const INF: Self = Self::new(f32::NEG_INFINITY, f32::NEG_INFINITY, f32::INFINITY, f32::INFINITY);
	pub const WINDOW: Self = Self::new(0.0, 0.0, f32::INFINITY, f32::INFINITY);
	pub const ZERO: Self = Self::new(0.0, 0.0, 0.0, 0.0);
	
	/// Create a new rectangle with the given x, y, w, and h values.
	pub const fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
		Self { x, y, w, h }
	}

	/// Create a new rectangle with the given size at the origin.
	pub fn from_size(size: impl Into<Vec2>) -> Self {
		let size = size.into();
		Self::new(0.0, 0.0, size.x, size.y)
	}

	/// Create a new rectangle with left top corner at `lt` and right bottom corner at `rb`.
	pub fn from_ltrb(lt: impl Into<Vec2>, rb: impl Into<Vec2>) -> Self {
		let lt = lt.into();
		let rb = rb.into();
		Self::new(lt.x, lt.y, rb.x - lt.x, rb.y - lt.y)
	}

	/// Create a new rectangle with left top corner at `lt` and size `size`.
	pub fn from_lt_size(lt: impl Into<Vec2>, size: impl Into<Vec2>) -> Self {
		let lt = lt.into();
		let size = size.into();
		Self::new(lt.x, lt.y, size.x, size.y)
	}

	/// Create a new rectangle with center at `center` and size `size`.
	pub fn from_center_size(center: impl Into<Vec2>, size: impl Into<Vec2>) -> Self {
		let center = center.into();
		let size = size.into();
		let half_size = size / 2.0;
		Self::new(center.x - half_size.x, center.y - half_size.y, size.x, size.y)
	}

	/// Get the left top corner of the rectangle.
	pub fn lt(self) -> Vec2 {
		Vec2::new(self.x, self.y)
	}

	/// Get the right top corner of the rectangle.
	pub fn rt(self) -> Vec2 {
		Vec2::new(self.x + self.w, self.y)
	}

	/// Get the right bottom corner of the rectangle.
	pub fn rb(self) -> Vec2 {
		Vec2::new(self.x + self.w, self.y + self.h)
	}

	/// Get the left bottom corner of the rectangle.
	pub fn lb(self) -> Vec2 {
		Vec2::new(self.x, self.y + self.h)
	}

	/// Get the center of the rectangle.
	pub fn center(self) -> Vec2 {
		Vec2::new(self.x + self.w / 2.0, self.y + self.h / 2.0)
	}

	/// Get the size of the rectangle.
	pub fn size(self) -> Vec2 {
		Vec2::new(self.w, self.h)
	}

	/// Get the width of the rectangle.
	pub fn width(self) -> f32 {
		self.w
	}

	/// Get the height of the rectangle.
	pub fn height(self) -> f32 {
		self.h
	}

	/// Check if the rectangle is empty (width or height <= zero).
	/// 
	/// Also we can call a rectangle is positive if it is not empty.
	pub fn is_empty(self) -> bool {
		self.w < 0.0 || self.h < 0.0
	}

	/// Check if the rectangle is positive (not empty).
	pub fn is_positive(self) -> bool {
		!self.is_empty()
	}

	/// Calculate the area of the rectangle.
	pub fn area(self) -> f32 {
		self.w * self.h
	}

	/// Calculate the perimeter of the rectangle.
	pub fn perimeter(self) -> f32 {
		2.0 * (self.w + self.h)
	}

	/// Check if the point is inside the rectangle.
	pub fn contains(self, point: impl Into<Vec2>) -> bool {
		let point = point.into();
		point.x >= self.x && point.x <= self.x + self.w && point.y >= self.y && point.y <= self.y + self.h
	}

	/// Check if the rectangle intersects with another rectangle.
	pub fn intersects(self, other: Self) -> bool {
		self.x + self.w > other.x && self.x < other.x + other.w && self.y + self.h > other.y && self.y < other.y + other.h
	}

	/// Calculate the intersection of two rectangles.
	/// 
	/// For short, use `self & other` instead of `self.intersection(other)`.
	pub fn intersection(self, other: Self) -> Self {
		let x = self.x.max(other.x);
		let y = self.y.max(other.y);
		let w = (self.x + self.w).min(other.x + other.w) - x;
		let h = (self.y + self.h).min(other.y + other.h) - y;
		Self::new(x, y, w, h)
	}

	/// Calculate the union of two rectangles.
	/// 
	/// For short, use `self | other` instead of `self.union(other)`.
	pub fn union(self, other: Self) -> Self {
		let x = self.x.min(other.x);
		let y = self.y.min(other.y);
		let w = (self.x + self.w).max(other.x + other.w) - x;
		let h = (self.y + self.h).max(other.y + other.h) - y;
		Self::new(x, y, w, h)
	}

	/// Calculate the difference between two rectangles.
	/// 
	/// For short, use `self - other` instead of `self.difference(other)`.
	pub fn difference(self, other: Self) -> Self {
		let mut result = self;
		if other.intersects(self) {
			if other.x < self.x {
				result.w -= other.x - self.x;
				result.x = other.x;
			}
			if other.y < self.y {
				result.h -= other.y - self.y;
				result.y = other.y;
			}
			if other.x + other.w > self.x + self.w {
				result.w = (self.x + self.w) - other.x;
			}
			if other.y + other.h > self.y + self.h {
				result.h = (self.y + self.h) - other.y;
			}
		}
		result
	}

	/// Flip lt and rb corners of the rectangle.
	/// 
	/// for short, use `- self` instead of `self.rev()`.
	pub fn rev(self) -> Self {
		let lt = self.lt();
		let rb = self.rb();
		Self::from_ltrb(rb, lt)
	}

	/// Turn a rectangle into positive or zero form.
	/// 
	/// In other words, if the rectangle is negative, turn it into positive form by flipping the lt and rb corners.
	pub fn abs(self) -> Self {
		if self.is_empty() {
			self.rev()
		}else {
			self
		}
	}

	/// Shrink the rectangle by the given amount.
	/// 
	/// Will keep center unchanged.
	pub fn shrink(self, amount: impl Into<Vec2>) -> Self {
		let amount = amount.into();
		let x = self.x + amount.x;
		let y = self.y + amount.y;
		let w = self.w - amount.x * 2.0;
		let h = self.h - amount.y * 2.0;
		Self::new(x, y, w, h)
	}

	/// Shrink the rectangle's size by the given amount.
	pub fn shrink_size(self, amount: impl Into<Vec2>) -> Self {
		let amount = amount.into();
		let w = self.w - amount.x;
		let h = self.h - amount.y;
		Self::new(self.x, self.y, w, h)
	}

	/// Move the rectangle to the given position.
	pub fn move_to(self, pos: impl Into<Vec2>) -> Self {
		let pos = pos.into();
		Self::new(pos.x, pos.y, self.w, self.h)
	}

	/// Move the rectangle by the given offset.
	pub fn move_by(self, offset: impl Into<Vec2>) -> Self {
		let offset = offset.into();
		Self::new(self.x + offset.x, self.y + offset.y, self.w, self.h)
	}

	/// Check if the given point is close to the edge of the rectangle.
	pub fn is_close_to_edge(&self, point: impl Into<Vec2>, epsilon: impl Into<Vec2>) -> bool {
		let point = point.into();
		let other_rect = self.shrink(epsilon.into());
		!other_rect.contains(point) && self.contains(point)
	}

	/// Transform the rectangle by the given matrix.
	/// 
	/// Will be the larget possible rectangle that contains the transformed rectangle.
	pub fn transformed(self, mat: Transform2D) -> Self {
		let lt = mat >> self.lt();
		let rb = mat >> self.rb();
		let lb = mat >> self.lb();
		let rt = mat >> self.rt();
		let lt_x = lt.x.min(rb.x).min(lb.x).min(rt.x);
		let lt_y = lt.y.min(rb.y).min(lb.y).min(rt.y);
		let rb_x = lt.x.max(rb.x).max(lb.x).max(rt.x);
		let rb_y = lt.y.max(rb.y).max(lb.y).max(rt.y);
		Self::from_ltrb(Vec2::new(lt_x, lt_y), Vec2::new(rb_x, rb_y))
	}

	/// Linearly interpolate between two rectangles.
	pub fn lerp(self, other: Self, t: f32) -> Self {
		let x = self.x + (other.x - self.x) * t;
		let y = self.y + (other.y - self.y) * t;
		let w = self.w + (other.w - self.w) * t;
		let h = self.h + (other.h - self.h) * t;
		Self::new(x, y, w, h)
	}
}

impl Default for Rect {
	fn default() -> Self {
		Self::ZERO
	}
}

impl BitAndAssign for Rect {
	fn bitand_assign(&mut self, other: Self) {
		*self = self.intersection(other);
	}
}

impl BitAnd for Rect {
	type Output = Self;

	fn bitand(self, other: Self) -> Self {
		self.intersection(other)
	}
}

impl BitOrAssign for Rect {
	fn bitor_assign(&mut self, other: Self) {
		*self = self.union(other);
	}
}

impl BitOr for Rect {
	type Output = Self;

	fn bitor(self, other: Self) -> Self {
		self.union(other)
	}
}

impl Sub for Rect {
	type Output = Self;

	fn sub(self, other: Self) -> Self {
		self.difference(other)
	}
}

impl Neg for Rect {
	type Output = Self;

	fn neg(self) -> Self {
		self.rev()
	}
}

/// Create a new rectangle with the given x, y, w, and h values.
pub fn rect(x: f32, y: f32, w: f32, h: f32) -> Rect {
	Rect::new(x, y, w, h)
}

/// Create a new rectangle with left top corner at `lt` and right bottom corner at `rb`.
pub fn rect_ltrb(lt: impl Into<Vec2>, rb: impl Into<Vec2>) -> Rect {
	Rect::from_ltrb(lt, rb)
}

impl Display for Rect {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "({} - {})", self.lt(), self.rb())
	}
}

// impl From<AABB<Vec2>> for Rect {
// 	fn from(aabb: AABB<Vec2>) -> Self {
// 		Rect::from_ltrb(aabb.lower(), aabb.upper())
// 	}
// }

// impl From<Rect> for AABB<Vec2> {
// 	fn from(rect: Rect) -> Self {
// 		AABB::from_corners(rect.lt(), rect.rb())
// 	}
// }

impl Envelope for Rect {
	type Point = Vec2;

	fn area(&self) -> <Self::Point as rstar::Point>::Scalar {
		(self.w * self.h).max(0.0)
	}	

	fn center(&self) -> Self::Point {
		Vec2::new(self.x + self.w / 2.0, self.y + self.h / 2.0)
	}

	fn contains_envelope(&self, other: &Self) -> bool {
		self.contains(other.lt()) && self.contains(other.rb())
	}

	fn contains_point(&self, point: &Self::Point) -> bool {
		self.contains(point)
	}

	fn distance_2(&self, point: &Self::Point) -> <Self::Point as rstar::Point>::Scalar {
		if self.contains(point) {
			return 0.0;
		}
		
		let lt = self.lt();
		let rb = self.rb();

		(lt.max(rb.min(*point)) - *point).length_squared()
	}

	fn intersection_area(&self, other: &Self) -> <Self::Point as rstar::Point>::Scalar {
		(*self & *other).area()
	}

	fn intersects(&self, other: &Self) -> bool {
		(*self & *other).is_positive()
	}

	fn merge(&mut self, other: &Self) {
		*self |= *other;
	}

	fn merged(&self, other: &Self) -> Self {
		*self | *other
	}

	fn min_max_dist_2(&self, point: &Self::Point) -> <Self::Point as rstar::Point>::Scalar {
		let lt = self.lt();
		let rb = self.rb();

		let x_contrib = if point.x < lt.x {
			(rb.x - point.x).powi(2)
		}else if point.x > rb.x {
			(point.x - lt.x).powi(2)
		}else {
			(lt.x - rb.x).powi(2) / 4.0
		};

		let y_contrib = if point.y < lt.y {
			(rb.y - point.y).powi(2)
		}else if point.y > rb.y {
			(point.y - lt.y).powi(2)
		}else {
			(lt.y - rb.y).powi(2) / 4.0
		};

		x_contrib + y_contrib
	}

	fn new_empty() -> Self {
		Self::ZERO
	}

	fn partition_envelopes<T: rstar::RTreeObject<Envelope = Self>>(
			axis: usize,
			envelopes: &mut [T],
			selection_size: usize,
		) {
		envelopes.select_nth_unstable_by(selection_size, |l, r| {
            l.envelope()
                .lt()
                .nth(axis)
                .partial_cmp(&r.envelope().lt().nth(axis))
                .unwrap()
        });
	}

	fn perimeter_value(&self) -> <Self::Point as rstar::Point>::Scalar {
		(self.w + self.h) * 2.0
	}

	fn sort_envelopes<T: rstar::RTreeObject<Envelope = Self>>(axis: usize, envelopes: &mut [T]) {
		envelopes.sort_unstable_by(|l, r| {
            l.envelope()
                .lt()
                .nth(axis)
                .partial_cmp(&r.envelope().lt().nth(axis))
                .unwrap()
        });
	}
}