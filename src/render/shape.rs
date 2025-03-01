//! Here we define the Shape related types.

use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not, Sub, SubAssign};

use crate::{math::{color::{Color, Vec4}, transform2d::Transform2D, vec2::Vec2}, prelude::Rect};

/// The operator types currently supported by the library.
/// 
/// Note: there's no precedence defined for the operators,
/// in other words, the order of the caculation will alway be from left to right.
#[derive(Debug, Clone, Copy, PartialEq)]
#[derive(serde::Serialize, serde::Deserialize)]
pub enum Operator {
	/// Get the intersection of two shapes.
	And,
	/// Get the union of two shapes.
	Or,
	/// Get the difference of two shapes.
	Minus,
	/// Get the symmetric difference of two shapes.
	Xor,
	/// Get the complement of a shape.
	Not,
	/// Linear interpolation between two shapes.
	Lerp(f32),
	/// Smooth step interpolation between two shapes.
	SmoothStep(f32),
	/// Sigmoid interpolation between two shapes.
	Sigmoid(f32),
}

/// A basic shape defined by its data, fill mode, and blend mode.
#[derive(Debug, PartialEq, Clone)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct BasicShape {
	/// The data of the basic shape.
	pub data: BasicShapeData,
	/// The transform matrix to be applied to the shape.
	pub transform: Transform2D,
	/// The stroke width and color of the shape.
	/// 
	/// Note: if stroke is setted, the shape will be rendered as stroke instead of fill,
	/// its **not** the superposition of fill and stroke.
	pub stroke: Option<f32>,
}

impl From<BasicShapeData> for BasicShape {
	fn from(data: BasicShapeData) -> Self {
		Self {
			data,
			transform: Transform2D::IDENTITY,
			stroke: None,
		}
	}
}

impl From<BasicShapeData> for Shape {
	fn from(data: BasicShapeData) -> Self {
		Shape::from(BasicShape::from(data))
	}
}

impl BasicShape {
	/// Set the transform matrix for the basic shape.
	pub fn transform(mut self, transform: Transform2D) -> Self {
		self.transform = transform;
		self
	}

	/// Set the stroke width of the basic shape.
	pub fn stroke(mut self, width: f32) -> Self {
		self.stroke = Some(width);
		self
	}

	/// Rotate the basic shape by the given angle in radians.
	pub fn then_rotate(mut self, angle: f32) -> Self {
		self.transform >>= Transform2D::rotate(angle);
		self
	}

	/// Rotate the basic shape by the given angle in degrees.
	pub fn then_rotate_degrees(mut self, angle: f32) -> Self {
		self.transform >>= Transform2D::rotate_degrees(angle);
		self
	}

	/// Rotate the basic shape by the given angle in radians before current transform.
	pub fn pre_rotate(mut self, angle: f32) -> Self {
		self.transform <<= Transform2D::rotate(angle);
		self
	}

	/// Rotate the basic shape by the given angle in degrees before current transform.
	pub fn pre_rotate_degrees(mut self, angle: f32) -> Self {
		self.transform <<= Transform2D::rotate_degrees(angle);
		self
	}

	/// Scale the basic shape by the given factors.
	pub fn then_scale(mut self, factor: impl Into<Vec2>) -> Self {
		self.transform >>= Transform2D::scale(factor);
		self
	}

	/// Scale the basic shape by the given factors before current transform.
	pub fn pre_scale(mut self, factor: impl Into<Vec2>) -> Self {
		self.transform <<= Transform2D::scale(factor);
		self
	}

	/// Translate the basic shape by the given offset.
	pub fn then_translate(mut self, offset: impl Into<Vec2>) -> Self {
		self.transform >>= Transform2D::translate(offset);
		self
	}

	/// Translate the basic shape by the given offset before current transform.
	pub fn pre_translate(mut self, offset: impl Into<Vec2>) -> Self {
		self.transform <<= Transform2D::translate(offset);
		self
	}

	/// Move the basic shape by the given offset.
	pub fn move_by(&mut self, offset: impl Into<Vec2>) {
		self.data.move_by(offset);
	}

	/// Get the bounding rect of the basic shape.
	pub fn bounded_rect(&self) -> Rect {
		self.data.bounded_rect().transformed(self.transform).shrink(if let Some(width) = self.stroke {
			- Vec2::same(width / 2.0)
		}else {
			Vec2::ZERO
		})
	}
}


impl BasicShape {
	/// Create a new basic shape from the given data, and fill color.
	pub fn new(data: BasicShapeData) -> Self {
		Self {
			data,
			transform: Transform2D::IDENTITY,
			stroke: None,
		}
	}
}

/// The fill mode of the basic shape.
#[derive(Debug, Clone, PartialEq)]
#[derive(serde::Serialize, serde::Deserialize)]
pub enum FillMode {
	/// Fill the shape with the given color.
	Color(Color),
	/// Fill the shape with the given texture.
	/// 
	/// Given texture id, top-left corner, right-bottom corner, and the texture left-top corner and right-bottom corner.
	Texture(u32, Vec2, Vec2, Vec2, Vec2),
	/// Fill the shape with linear gradient.
	/// 
	/// Given start and end color, and the start and end position of the gradient.
	LinearGradient(Color, Color, Vec2, Vec2),
	/// Fill the shape with radial gradient.
	/// 
	/// Given start and end color, center position, and the radiusof the gradient.
	RadialGradient(Color, Color, Vec2, f32),
}

impl FillMode {
	/// Check if the fill mode is invisible.
	pub fn is_invisible(&self) -> bool {
		match self {
			FillMode::Color(color) => color.a <= 0.0,
			FillMode::Texture(_, _, _, _, _) => false,
			FillMode::LinearGradient(from, to, _, _) => from.a <= 0.0 && to.a <= 0.0,
			FillMode::RadialGradient(from, to, _, _) => from.a <= 0.0 && to.a <= 0.0,
		}
	}

	/// Make the fill mode brighter by the given factor.
	/// 
	/// Will do nothing if the fill mode is texture.
	pub fn brighter(&mut self, bright_factor: f32) {
		if self.is_invisible() {
			return;
		}

		match self {
			FillMode::Color(color) => {
				*color += bright_factor * Color::WHITE;
			},
			FillMode::Texture(_, _, _, _, _) => {},
			FillMode::LinearGradient(from, to, _, _) => {
				*from += bright_factor * Color::WHITE;
				*to += bright_factor * Color::WHITE;
			},
			FillMode::RadialGradient(from, to, _, _) => {
				*from += bright_factor * Color::WHITE;
				*to += bright_factor * Color::WHITE;
			},
		}
	}

	/// Multiply the alpha channel of the fill mode by the given factor.
	/// 
	/// Will do nothing if the fill mode is texture.
	pub fn mul_alpha(&mut self, alpha: f32) {
		if self.is_invisible() {
			return;
		}

		match self {
			FillMode::Color(color) => {
				color.a *= alpha;
			},
			FillMode::Texture(_, _, _, _, _) => {},
			FillMode::LinearGradient(from, to, _, _) => {
				from.a *= alpha;
				to.a *= alpha;
			},
			FillMode::RadialGradient(from, to, _, _) => {
				from.a *= alpha;
				to.a *= alpha;
			},
		}
	}

	pub(crate) fn move_by(&mut self, offset: impl Into<Vec2>) {
		let offset = offset.into();
		match self {
			FillMode::Texture(_, top_left, right_bottom, _, _) => {
				*top_left += offset;
				*right_bottom += offset;
			},
			FillMode::LinearGradient(_, _, start, end) => {
				*start += offset;
				*end += offset;
			},
			FillMode::RadialGradient(_, _, center, _) => {
				*center += offset;
			},
			_ => {},
		}
	}
}

impl<T> From<T> for FillMode
where T: Into<Color>,
{
	fn from(color: T) -> Self {
		FillMode::Color(color.into())
	}
}

impl Default for FillMode {
	fn default() -> Self {
		FillMode::Color(Color::TRANSPARENT)
	}
}

/// The basic shape types currently supported by the library.
/// 
/// Noticed that we don't have cubic bezier curve support, 
/// since it's hard to define "inside" or "outside" for a general cubic bezier curve.
/// 
/// If you need to draw a general cubic bezier curve, you can use combination of `QuadHalfPlane` shape,
/// which is simple due to sdf based rendering approach.
#[derive(Debug, PartialEq, Clone)]
#[derive(serde::Serialize, serde::Deserialize)]
pub enum BasicShapeData {
	/// A circle defined by center and radius.
	Circle(Vec2, f32),
	/// A triangle defined by three points.
	Triangle(Vec2, Vec2, Vec2),
	/// A rectangle defined by left-top point, right-bottom point and the corner radius.
	Rectangle(Vec2, Vec2, Vec4),
	/// A half plane defined by two point.
	/// 
	/// The plane can be defined by the following formula:
	/// $$
	/// (x - x_0) (y_1 - y_0) - (y - y_0) (x_1 - x_0) \ge 0
	/// $$
	HalfPlane(Vec2, Vec2),
	/// A quadratic bezier plane defined by three points.
	/// 
	/// Defines the concave part as the negative(outside) part of the plane,
	/// the convex part as the positive(inside) part of the plane.
	QuadBezierPlane(Vec2, Vec2, Vec2),
	/// A SDF texture defined by its top-left corner, its right-bottom corner and its texture id.
	SDFTexture(Vec2, Vec2, u32),
	/// A single character text defined by its position, font id, font size, and character.
	Text(Vec2, u32, f32, char)
}
/// A shape that saves shape in reverse polish notation.
/// 
/// Can be used to define complex shapes with operators.
/// But pay attention: do **not** use this shape to draw too complex shapes, since it's not optimized for performance for now.
/// 
/// Also can use `Shape::from(BasicShape)` to convert a `BasicShape` to a `ShapeComplex`.
/// 
/// Shape implements the following operators:
/// - `Shape | Shape` to get the union of two shapes.
/// - `Shape - Shape` to get the difference of two shapes.
/// - `Shape ^ Shape` to get the symmetric difference of two shapes.
/// - `Shape & Shape` to get the intersection of two shapes.
/// - `!Shape` to get the complement of a shape.
/// 
/// Note: be careful when using `-`, since they have higher precedence than other bitwise operator in rust.
#[derive(serde::Serialize, serde::Deserialize)]
#[derive(Debug, Clone)]
pub struct Shape(pub Vec<ShapeOrOp>);

impl Shape {
	/// Get the union of two shapes.
	/// 
	/// For shorthand, you can use `self | rhs` operator instead of `self.union(rhs)`.
	pub fn union(mut self, rhs: Self) -> Self {
		self.0.extend(rhs.0);
		self.0.push(ShapeOrOp::Op(Operator::Or));
		self
	}

	/// Get the difference of two shapes.
	/// 
	/// For shorthand, you can use `self - rhs` operator instead of `self.difference(rhs)`.
	/// 
	/// Note: be careful when using `-`, since they have higher precedence than other bitwise operator in rust.
	pub fn difference(mut self, rhs: Self) -> Self {
		self.0.extend(rhs.0);
		self.0.push(ShapeOrOp::Op(Operator::Minus));
		self
	}

	/// Get the symmetric difference of two shapes.
	/// 
	/// For shorthand, you can use `self ^ rhs` operator instead of `self.symmetric_difference(rhs)`.
	pub fn symmetric_difference(mut self, rhs: Self) -> Self {
		self.0.extend(rhs.0);
		self.0.push(ShapeOrOp::Op(Operator::Xor));
		self
	}

	/// Get the intersection of two shapes.
	/// 
	/// For shorthand, you can use `self & rhs` operator instead of `self.intersection(rhs)`.
	pub fn intersection(mut self, rhs: Self) -> Self {
		self.0.extend(rhs.0);
		self.0.push(ShapeOrOp::Op(Operator::And));
		self
	}

	/// Get the complement of a shape.
	/// 
	/// For shorthand, you can use `!self` operator instead of `self.complement()`.
	pub fn complement(mut self) -> Self {
		self.0.push(ShapeOrOp::Op(Operator::Not));
		self
	}

	/// Get the lerp of two shapes.
	pub fn lerp(mut self, rhs: Self, t: f32) -> Self {
		self.0.extend(rhs.0);
		self.0.push(ShapeOrOp::Op(Operator::Lerp(t)));
		self
	}

	/// Get the smoothstep interpolation of two shapes.
	pub fn smoothstep(mut self, rhs: Self, t: f32) -> Self {
		self.0.extend(rhs.0);
		self.0.push(ShapeOrOp::Op(Operator::SmoothStep(t)));
		self
	}

	/// Get sigmoid interpolation of of two shapes.
	pub fn sigmoid(mut self, rhs: Self, t: f32) -> Self {
		self.0.extend(rhs.0);
		self.0.push(ShapeOrOp::Op(Operator::Sigmoid(t)));
		self
	}

	/// Apply transform matrix for the shape.
	pub fn transform(mut self, transform: Transform2D) -> Self {
		for shape_or_op in &mut self.0 {
			if let ShapeOrOp::Shape(shape) = shape_or_op {
				shape.transform >>= transform;
			}
		}
		self
	}

	/// Move the shape by the given offset.
	pub fn move_by(mut self, offset: impl Into<Vec2>) -> Self {
		let offset = offset.into();
		for shape_or_op in &mut self.0 {
			if let ShapeOrOp::Shape(shape) = shape_or_op {
				shape.move_by(offset);
			}
		}
		self
	}

	/// Get the bounding rect of the shape.
	/// 
	/// # Panics
	/// 
	/// Panics if the shape is invalid.
	pub fn bounded_rect(&self) -> Rect {
		let mut stack = vec!();
		for shape_or_op in &self.0 {
			match shape_or_op {
				ShapeOrOp::Shape(shape) => {
					stack.push(shape.bounded_rect());
				},
				ShapeOrOp::Op(op) => {
					if let Operator::Not = op {
						stack.pop();
						stack.push(Rect::WINDOW);
						continue;
					}

					let rhs = stack.pop().unwrap();
					let lhs = stack.pop().unwrap();
					match op {
						Operator::Or => stack.push(lhs | rhs),
						Operator::Minus => stack.push(rhs),
						Operator::And => stack.push(lhs & rhs),
						Operator::Xor => stack.push(lhs | rhs),
						Operator::Lerp(t) => stack.push(lhs.lerp(rhs, *t)),
						Operator::SmoothStep(t) => stack.push(lhs.lerp(rhs, *t)),
						Operator::Sigmoid(t) => {
							let sigmoid = 1.0 / (1.0 + t.exp());
							stack.push(lhs.lerp(rhs, sigmoid))
						},
						Operator::Not => unreachable!(),
					}
				},
			}
		}
		stack.pop().unwrap_or_default()
	}
}

impl<R> BitAnd<R> for Shape 
where 
	R: Into<Shape>
{
	type Output = Shape;

	fn bitand(self, rhs: R) -> Self::Output {
		let rhs = rhs.into();
		self.intersection(rhs)
	}
}

impl<R> BitAndAssign<R> for Shape 
where 
	R: Into<Shape>
{
	fn bitand_assign(&mut self, rhs: R) {
		let result = Shape(self.0.drain(..).collect());
		*self = result.intersection(rhs.into());
	}
}

impl<R> BitOr<R> for Shape 
where 
	R: Into<Shape>
{
	type Output = Self;

	fn bitor(self, rhs: R) -> Self::Output {
		self.union(rhs.into())
	}
}

impl<R> BitOrAssign<R> for Shape 
where 
	R: Into<Shape>
{
	fn bitor_assign(&mut self, rhs: R) {
		let result = Shape(self.0.drain(..).collect());
		*self = result.union(rhs.into());
	}
}

impl<R> Sub<R> for Shape 
where 
	R: Into<Shape>
{
	type Output = Self;

	fn sub(self, rhs: R) -> Self::Output {
		self.difference(rhs.into())
	}
}

impl<R> SubAssign<R> for Shape 
where
	R: Into<Shape>
{
	fn sub_assign(&mut self, rhs: R) {
		let result = Shape(self.0.drain(..).collect());
		*self = result.difference(rhs.into());
	}
}

impl<R> BitXor<R> for Shape 
where
	R: Into<Shape>
{
	type Output = Self;

	fn bitxor(self, rhs: R) -> Self::Output {
		self.symmetric_difference(rhs.into())
	}
}

impl<R> BitXorAssign<R> for Shape 
where
	R: Into<Shape>
{
	fn bitxor_assign(&mut self, rhs: R) {
		let result = Shape(self.0.drain(..).collect());
		*self = result.symmetric_difference(rhs.into());
	}
}

impl Not for Shape {
	type Output = Self;

	fn not(self) -> Self::Output {
		self.complement()
	}
}

/// A shape or operator used in the complex shape.
#[derive(serde::Serialize, serde::Deserialize)]
#[derive(Debug, Clone)]
pub enum ShapeOrOp {
	/// A basic shape.
	Shape(BasicShape),
	/// An operator.
	Op(Operator),
}

impl From<BasicShape> for Shape {
	fn from(shape: BasicShape) -> Self {
		Self(vec![ShapeOrOp::Shape(shape)])
	}
}

impl BasicShapeData {
	#[inline]
	/// Move the shape by the given offset.
	pub fn move_by(&mut self, offset: impl Into<Vec2>) {
		let offset = offset.into();
		match self {
			Self::Circle(center, _) => {
				*center += offset;
			},
			Self::Triangle(p1, p2, p3) => {
				*p1 += offset;
				*p2 += offset;
				*p3 += offset;
			},
			Self::Rectangle(left_top, right_bottom, _) => {
				*left_top += offset;
				*right_bottom += offset;
			},
			Self::HalfPlane(p1, p2) => {
				*p1 += offset;
				*p2 += offset;
			},
			Self::QuadBezierPlane(p1, p2, p3) => {
				*p1 += offset;
				*p2 += offset;
				*p3 += offset;
			},
			Self::SDFTexture(top_left, right_bottom, _) => {
				*top_left += offset;
				*right_bottom += offset;
			},
			Self::Text(pos, _, _, _) => {
				*pos += offset;
			},
		}
	}

	/// Get the bounding rectangle of the shape.
	pub fn bounded_rect(&self) -> Rect {
		match self {
			Self::Circle(center, radius) => Rect::from_center_size(*center, Vec2::same(*radius * 2.0)),
			Self::Triangle(p1, p2, p3) => {
				let min_x = p1.x.min(p2.x).min(p3.x);
				let min_y = p1.y.min(p2.y).min(p3.y);
				let max_x = p1.x.max(p2.x).max(p3.x);
				let max_y = p1.y.max(p2.y).max(p3.y);
				Rect::from_ltrb(Vec2::new(min_x, min_y), Vec2::new(max_x, max_y))
			},
			Self::Rectangle(lt, rb, _) => Rect::from_ltrb(*lt, *rb),
			Self::HalfPlane(_, _) => Rect::WINDOW,
			Self::QuadBezierPlane(p1, p2, p3) => {
				let min_x = p1.x.min(p2.x).min(p3.x);
				let min_y = p1.y.min(p2.y).min(p3.y);
				let max_x = p1.x.max(p2.x).max(p3.x);
				let max_y = p1.y.max(p2.y).max(p3.y);
				Rect::from_ltrb(Vec2::new(min_x, min_y), Vec2::new(max_x, max_y))
			},
			Self::SDFTexture(top_left, right_bottom, _) => {
				Rect::from_ltrb(*top_left, *right_bottom)
			},
			Self::Text(pos, _, size, _) => {
				Rect::from_lt_size(*pos, Vec2::same(*size))
			},
		}
	}
}

// /// A Builder for creating [`ShapeInner`] a path.
// /// 
// /// Currently, this is a simple implementation, and may not work correctly for all cases.
// /// Do not support gpu rendering yet.
// pub struct PathBuilder {
// 	pub(crate) start_pos: Vec2,
// 	pub(crate) fill_mode: FillMode,
// 	pub(crate) path: Vec<PathCommand>,
// }

// enum PathCommand {
// 	LineTo(Vec2),
// 	CubicTo(Vec2, Vec2, Vec2),
// 	QuadraticTo(Vec2, Vec2),
// }

// impl PathBuilder {
// 	/// Create a new path builder with the given start position.
// 	pub fn new(start_pos: Vec2) -> Self {
// 		Self {
// 			start_pos,
// 			fill_mode: FillMode::default(),
// 			path: vec![],
// 		}
// 	}

// 	/// Set the fill mode for the path.
// 	pub fn fill_mode(mut self, fill_mode: FillMode) -> Self {
// 		self.fill_mode = fill_mode;
// 		self
// 	}

// 	/// Adds a line from the current position to the given position.
// 	pub fn line_to(mut self, pos: Vec2) -> Self {
// 		self.path.push(PathCommand::LineTo(pos));
// 		self
// 	}

// 	/// Adds a cubic bezier curve from the current position to the given position with the given control points.
// 	pub fn cubic_to(mut self, ctrl1: Vec2, ctrl2: Vec2, pos: Vec2) -> Self {
// 		self.path.push(PathCommand::CubicTo(ctrl1, ctrl2, pos));
// 		self
// 	}

// 	/// Adds a quadratic bezier curve from the current position to the given position with the given control point.
// 	pub fn quadratic_to(mut self, ctrl: Vec2, pos: Vec2) -> Self {
// 		self.path.push(PathCommand::QuadraticTo(ctrl, pos));
// 		self
// 	}

// 	/// Ends the path and returns the resulting shape.
// 	pub fn end(mut self, close: bool) -> ShapeInner {
// 		if close {
// 			self.path.push(PathCommand::LineTo(self.start_pos));
// 		}
// 		todo!()
// 	}
// }