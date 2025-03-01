//! A simple GPU-accelerated painter.

use std::sync::{Arc, Mutex};

use lyon_geom::{point, CubicBezierSegment};

use crate::{math::{color::Vec4, prelude::Transform2D, rect::Rect, vec2::Vec2}, render::{commands::{CommandGpu, OperationGpu}, font::EM, font_render::FontRender}};

use super::{commands::{BlendMode, DrawCommandGpu}, font::{FontId, FontPool}, shape::{BasicShape, BasicShapeData, FillMode, Operator, Shape, ShapeOrOp}};

/// A shape to draw.
pub struct ShapeToDraw {
	/// The shape to draw.
	pub shape: Shape,
	/// The fill mode to use.
	pub fill_mode: FillMode,
	/// The blend mode to use.
	pub blend_mode: BlendMode,
	// /// The transform matrix to apply to the shape.
	// pub transform: Transform2D,
	/// The clip rect to use.
	pub clip_rect: Rect,
}

impl ShapeToDraw {
	fn is_visible_in_rect(&self, rect: Rect) -> bool {
		if self.shape.0.is_empty() {
			return false;
		}

		if (self.clip_rect & rect).is_empty() {
			return false;
		}

		if self.fill_mode.is_invisible() {
			return false;
		}

		!(self.shape.bounded_rect() & rect).is_empty()
	}
}

/// A simple GPU-accelerated painter.
/// 
/// Note: While setting transfroms, you need manually translating the position by the painter's `releative_to`
/// unlike other methods which automatically translate the position by the painter's `releative_to`.
#[derive(Default)]
pub struct Painter {
	/// The current transform matrix.
	/// 
	/// This matrix is applied to all newly drawn shapes drawn by this painter.
	pub transform: Transform2D,
	/// The current blend mode.
	/// 
	/// This blend mode is applied to all newly drawn shapes drawn by this painter.
	pub blend_mode: BlendMode,
	/// The current fill mode.
	/// 
	/// This fill mode is applied to all newly drawn shapes drawn by this painter.
	pub fill_mode: FillMode,
	/// The list of shapes to draw.
	pub shapes: Vec<ShapeToDraw>,
	/// The window size.
	pub window_size: Vec2,
	font_pool: Arc<Mutex<FontPool>>,
	releative_to: Vec2,
	clip_rect: Rect,
	scale_factor: f32,
}

impl Painter {
	/// Create a new painter.
	pub(crate) fn new(font_pool: Arc<Mutex<FontPool>>, window_size: Vec2) -> Self {
		Self {
			font_pool,
			window_size,
			..Default::default()
		}
	}

	/// Get relatvie to position.
	pub fn releative_to(&self) -> Vec2 {
		self.releative_to
	}

	/// Get current clip rect.
	pub fn clip_rect(&self) -> Rect {
		self.clip_rect
	}

	/// Reset the transform matrix to the identity matrix.
	pub fn reset_transform(&mut self) {
		self.transform = Transform2D::IDENTITY;
	}

	/// Reset the blend mode to the default mode.
	pub fn reset_blend_mode(&mut self) {
		self.blend_mode = BlendMode::default();
	}

	/// Reset the fill mode to the default mode.
	pub fn reset_fill_mode(&mut self) {
		self.fill_mode = FillMode::default();
	}

	/// Set fill mode.
	/// 
	/// This fill mode will be applied to all newly drawn shapes drawn by this painter.
	pub fn set_fill_mode(&mut self, fill_mode: impl Into<FillMode>) {
		self.fill_mode = fill_mode.into();
	}

	/// Set blend mode.
	/// 
	/// This blend mode will be applied to all newly drawn shapes drawn by this painter.
	pub fn set_blend_mode(&mut self, blend_mode: impl Into<BlendMode>) {
		self.blend_mode = blend_mode.into();
	}

	/// Set transform matrix.
	/// 
	/// This matrix will be applied to all newly drawn shapes drawn by this painter.
	pub fn set_transform(&mut self, transform: impl Into<Transform2D>) {
		self.transform = transform.into();
	}

	/// Rotate the current transform matrix by the given angle in radians.
	pub fn then_rotate(&mut self, angle: f32) {
		self.transform >>= Transform2D::rotate(angle);
	}

	/// Rotate the current transform matrix by the given angle in degrees.
	pub fn then_rotate_degrees(&mut self, angle: f32) {
		self.then_rotate(angle / 180.0 * std::f32::consts::PI);
	}

	/// Scale the current transform matrix by the given factor.
	pub fn then_translate(&mut self, to: impl Into<Vec2>) {
		self.transform >>= Transform2D::translate(to.into());
	}

	/// Scale the current transform matrix by the given factor.
	pub fn then_scale(&mut self, factors: impl Into<Vec2>) {
		self.transform >>= Transform2D::scale(factors.into());
	}

	/// Rotate the current transform matrix by the given angle in radians before current transform.
	pub fn pre_rotate(&mut self, angle: f32) {
		self.transform <<= Transform2D::rotate(angle);
	}

	/// Rotate the current transform matrix by the given angle in degrees before current transform.
	pub fn pre_rotate_degrees(&mut self, angle: f32) {
		self.pre_rotate(angle / 180.0 * std::f32::consts::PI);
	}

	/// Scale the current transform matrix by the given factor before current transform.
	pub fn pre_translate(&mut self, to: impl Into<Vec2>) {
		self.transform <<= Transform2D::translate(to.into());
	}

	/// Scale the current transform matrix by the given factor before current transform.
	pub fn pre_scale(&mut self, factors: impl Into<Vec2>) {
		self.transform <<= Transform2D::scale(factors.into());
	}

	/// Draw a shape.
	pub fn draw_shape(&mut self, shape: impl Into<Shape>) {
		let shape = shape.into().move_by(self.releative_to);
		let mut fill = self.fill_mode.clone();
		fill.move_by(self.releative_to);
		self.shapes.push(ShapeToDraw {
			shape: shape.transform(self.transform),
			fill_mode: fill,
			blend_mode: self.blend_mode,
			clip_rect: self.clip_rect,
		});
	}

	/// Draw a [`ShapeToDraw`].
	pub fn draw_shape_detailed(&mut self, shape: ShapeToDraw) {
		let mut fill_mode = shape.fill_mode;
		fill_mode.move_by(self.releative_to);

		let shape = ShapeToDraw {
			shape: shape.shape.move_by(self.releative_to).transform(self.transform),
			fill_mode,
			clip_rect: shape.clip_rect & self.clip_rect, 
			..shape
		};
		self.shapes.push(shape);
	}

	/// Draw a rectangle.
	pub fn draw_rect(&mut self, rect: impl Into<Rect>, rounding: impl Into<Vec4>) {
		let rect = rect.into();
		self.draw_shape(BasicShapeData::Rectangle(rect.lt(), rect.rb(), rounding.into()));
	}

	/// Draw a stroked rectangle.
	pub fn draw_stroked_rect(&mut self, rect: impl Into<Rect>, rounding: impl Into<Vec4>, width: f32) {
		let rect = rect.into();
		let shape = BasicShapeData::Rectangle(rect.lt(), rect.rb(), rounding.into());
		let shape = BasicShape {
			stroke: Some(width),
			..BasicShape::from(shape)
		};
		self.draw_shape(shape);
	}

	/// Draw a circle.
	pub fn draw_circle(&mut self, center: impl Into<Vec2>, radius: f32) {
		self.draw_shape(BasicShapeData::Circle(center.into() , radius));
	}

	/// Draw a stroked circle.
	pub fn draw_stroked_circle(&mut self, center: impl Into<Vec2>, radius: f32, width: f32) {
		let shape = BasicShapeData::Circle(center.into(), radius);
		let shape = BasicShape {
			stroke: Some(width),
			..BasicShape::from(shape)
		};
		self.draw_shape(shape);
	}

	/// Draw a triangle.
	pub fn draw_triangle(&mut self, a: impl Into<Vec2>, b: impl Into<Vec2>, c: impl Into<Vec2>) {
		self.draw_shape(BasicShapeData::Triangle(a.into(), b.into(), c.into()));
	}

	/// Draw a stroked triangle.
	pub fn draw_stroked_triangle(&mut self, a: impl Into<Vec2>, b: impl Into<Vec2>, c: impl Into<Vec2>, width: f32) {
		let shape = BasicShapeData::Triangle(a.into(), b.into(), c.into());
		let shape = BasicShape {
			stroke: Some(width),
			..BasicShape::from(shape)
		};
		self.draw_shape(shape);
	}

	/// Draw a half-plane.
	pub fn draw_half_plane(&mut self, a: impl Into<Vec2>, b: impl Into<Vec2>) {
		self.draw_shape(BasicShapeData::HalfPlane(a.into(), b.into()));
	}

	/// Draw a line.
	pub fn draw_line(&mut self, a: impl Into<Vec2>, b: impl Into<Vec2>, width: f32) {
		let shape = BasicShapeData::HalfPlane(a.into(), b.into());
		let shape = BasicShape {
			stroke: Some(width),
			..BasicShape::from(shape)
		};
		self.draw_shape(shape);
	}

	/// Draw a quad-half-plane.
	pub fn draw_quad_half_plane(&mut self, a: impl Into<Vec2>, b: impl Into<Vec2>, c: impl Into<Vec2>) {
		self.draw_shape(BasicShapeData::QuadBezierPlane(a.into(), b.into(), c.into()));
	}

	/// Draw a quadratic bezier curve.
	pub fn draw_quad_bezier(&mut self, a: impl Into<Vec2>, b: impl Into<Vec2>, c: impl Into<Vec2>, width: f32) {
		let shape = BasicShapeData::QuadBezierPlane(a.into(), b.into(), c.into());
		let shape = BasicShape {
			stroke: Some(width),
			..BasicShape::from(shape)
		};
		self.draw_shape(shape);
	}

	/// Draw a SDF texture.
	/// 
	/// Make sure to set the texture before calling this function.
	pub fn draw_sdf_texture(&mut self, rect: impl Into<Rect>, texture_id: u32) {
		let rect = rect.into().move_by(self.releative_to);
		self.draw_shape(BasicShapeData::SDFTexture(rect.lt(), rect.rb(), texture_id));
	}

	/// Draw a cubic bezier curve.
	/// 
	/// Note: We're using quadratic bezier curve to approximate the cubic bezier curve.
	/// Therefore, we do not support things like cubic bezier curve plane.
	pub fn draw_cubic_bezier(&mut self, 
		from: impl Into<Vec2>,
		ctrl1: impl Into<Vec2>,
		ctrl2: impl Into<Vec2>,
		to: impl Into<Vec2>,
		stroke_width: f32,
	) {
		let from = from.into();
		let ctrl1 = ctrl1.into();
		let ctrl2 = ctrl2.into();
		let to = to.into();

		let cb = CubicBezierSegment {
			from: point(from.x, from.y),
			ctrl1: point(ctrl1.x, ctrl1.y),
			ctrl2: point(ctrl2.x, ctrl2.y),
			to: point(to.x, to.y),
		};

		let num_qb = cb.num_quadratics(0.01);
		let step = 1.0 / num_qb as f32;

		let mut t = 0.0;
		let mut quads = vec!();

		for _ in 0..(num_qb - 1) {
			let t1 = t + step;
			let quad = cb.split_range(t..t1).to_quadratic();
			quads.push(
				BasicShape {
					stroke: Some(stroke_width),
					transform:Transform2D::IDENTITY,
					data: BasicShapeData::QuadBezierPlane(
						Vec2::new(quad.from.x, quad.from.y), 
						Vec2::new(quad.ctrl.x, quad.ctrl.y),
						Vec2::new(quad.to.x, quad.to.y),
					),
				}
			);
			t = t1;
		}

		let quad = cb.split_range(t..1.0).to_quadratic();
		quads.push(
			BasicShape {
				stroke: Some(stroke_width),
				transform:Transform2D::IDENTITY,
				data: BasicShapeData::QuadBezierPlane(
					Vec2::new(quad.from.x, quad.from.y), 
					Vec2::new(quad.ctrl.x, quad.ctrl.y),
					Vec2::new(quad.to.x, quad.to.y),
				),
			}
		);

		if quads.is_empty() {
			return;
		} 

		let mut start = Shape::from(quads.pop().unwrap());

		for quad in quads {
			start |= quad;
		}

		self.draw_shape(start);
	}

	/// Draw a text.
	/// 
	/// Make sure to set the font before calling this function.
	/// 
	/// Returns true if the text is successfully drawn.
	pub fn draw_text(
		&mut self, 
		pos: impl Into<Vec2>, 
		font_id: FontId, 
		font_size: f32, 
		text: impl Into<String>,
	) -> bool {
		let font_pool = if let Ok(inner) = self.font_pool.lock() {
			inner
		}else {
			return false;
		};
		let text = text.into();
		let mut pos = pos.into();
		let mut x = 0.0;
		let factor = font_size / EM * if let Some(factor) = font_pool.advance_factor(font_id) {
			factor
		}else {
			return false;
		};
		let line_height = if let Some(inner) = font_pool.line_height(font_id) {
			inner
		}else {
			return false;
		};
		// let ancestor = if let Some(inner) = font_pool.anscender(font_id) {
		// 	0.0
		// }else {
		// 	return false;
		// };
		drop(font_pool);
		for chr in text.chars() {
			let mut font_pool = if let Ok(inner) = self.font_pool.lock() {
				inner
			}else {
				return false;
			};
			if chr == '\n' {
				x = 0.0;
				pos.y += line_height * factor;
				continue;
			}
			
			let glyph =  if let Some(inner) = font_pool.get_glyph(font_id, chr) {
				inner
			}else {
				return false;
			};
			let chr_pos = pos + Vec2::new(x, 0.0) + Vec2::x(glyph.bearing.x * factor);
			drop(font_pool);
			self.draw_shape(BasicShapeData::Text(chr_pos, font_id, font_size, chr));
			x += glyph.advance.x * factor; 
		}

		true
	}

	/// Get size of a text.
	/// 
	/// Returns None if the font is not found or the text is empty.
	pub fn text_size(
		&self, 
		font_id: FontId, 
		font_size: f32, 
		text: impl Into<String>,
	) -> Option<Vec2> {
		let mut font_pool = if let Ok(inner) = self.font_pool.lock() {
			// println!("get lock!");
			inner
		}else {
			return None;
		};
		font_pool.caculate_text_size(font_id, text, font_size, false)
	}

	/// Get size of a text, but optimized for rendering pointer.
	pub fn text_size_pointer(
		&self, 
		font_id: FontId, 
		font_size: f32, 
		text: impl Into<String>,
	) -> Option<Vec2> {
		let mut font_pool = if let Ok(inner) = self.font_pool.lock() {
			// println!("get lock!");
			inner
		}else {
			return None;
		};
		font_pool.caculate_text_size(font_id, text, font_size, true)
	}

	/// Get line height of a font.
	/// 
	/// Returns None if the font is not found.
	pub fn line_height(&self, font_id: FontId, font_size: f32) -> Option<f32> {
		if let Ok(inner) = self.font_pool.lock() {
			inner.line_height_with_size(font_id, font_size)
		}else {
			None
		}
	}

	pub(crate) fn set_scale_factor(&mut self, factor: f32) {
		self.scale_factor = factor;
	}

	pub(crate) fn set_relative_to(&mut self, pos: Vec2) {
		self.releative_to = pos;
	}
	
	/// Set the clip rect.
	pub fn set_clip_rect(&mut self, rect: Rect) {
		self.clip_rect = rect;
	}

	pub(crate) fn parse(mut self, font_render: &FontRender, dirty_rect: Rect) -> (Vec<DrawCommandGpu>, u32) {
		use rayon::prelude::*;

		self.shapes.reverse();

		// let mut out = vec!();
		// let mut expect_stack_size = 0;
		// let mut current_transform = Transform2D::IDENTITY;
		// let mut current_blend_mode = BlendMode::default();

		let shapes = std::mem::take(&mut self.shapes);

		let out = shapes.into_par_iter().filter_map(|shape| {
			if !shape.is_visible_in_rect(dirty_rect) {
				return None;
			}
			Some(shape.parse(font_render))
		}).collect::<Vec<_>>();

		
		let mut expect_stack_size = 0;
		for (_, size) in out.iter() {
			expect_stack_size = (*size).max(expect_stack_size);
		}

		(out.into_iter().flat_map(|(inner, _)| inner).collect(), expect_stack_size)
	}
}

enum ShapeOrStack {
	Shape(BasicShape),
	Stack(u32)
}

fn get_stack(stack_index: u32, op: OperationGpu, parameter: f32, /* clip_rect: Rect */) -> DrawCommandGpu {
	DrawCommandGpu {
		command: CommandGpu::Load as u32,
		slots: [
			[stack_index as f32, 0.0, 0.0, 0.0],
			[0.0, 0.0, 0.0, 0.0],
			[0.0, 0.0, 0.0, 0.0],
			[0.0, 0.0, 0.0, 0.0],
		],
		stroke_width: -1.0,
		operation: op as u32,
		// smooth_function: 0,
		// smooth_parameter: 0.0,
		lhs: 0,
		parameter,
		// clip_rect_lt_x: clip_rect.lt().x,
		// clip_rect_lt_y: clip_rect.lt().y,
		// clip_rect_rb_x: clip_rect.rb().x,
		// clip_rect_rb_y: clip_rect.rb().y,
		..Default::default()
	}
}

fn get_transform(transform: Transform2D) -> DrawCommandGpu {
	DrawCommandGpu {
		command: CommandGpu::SetMat3x3 as u32,
		slots: [
			[transform[0][0], transform[1][0], transform[2][0], transform[0][1]],
			[transform[1][1], transform[2][1], transform[0][2], transform[1][2]],
			[transform[2][2], 0.0, 0.0, 0.0],
			[0.0, 0.0, 0.0, 0.0],
		],
		stroke_width: -1.0,
		operation: OperationGpu::None as u32,
		lhs: 0,
		parameter: 0.0,
		// clip_rect_lt_x: 0.0,
		// clip_rect_lt_y: 0.0,
		// clip_rect_rb_x: 0.0,
		// clip_rect_rb_y: 0.0,
		..Default::default()
	}
}

fn hanle_binary_op(
	op: Operator, 
	lhs: ShapeOrStack, 
	rhs: ShapeOrStack,
	current_transform: &mut Transform2D,
	font_render: &FontRender,
	stack_index: &mut u32,
	// clip_rect: Rect
) -> Option<(Vec<DrawCommandGpu>, ShapeOrStack)> {
	let (op, parameter) = match op {
		Operator::And => (OperationGpu::And, 0.0),
		Operator::Or => (OperationGpu::Or, 0.0),
		Operator::Xor => (OperationGpu::Xor, 0.0),
		Operator::Minus => (OperationGpu::Sub, 0.0),
		Operator::Lerp(t) => (OperationGpu::Lerp, t),
		Operator::SmoothStep(t) => (OperationGpu::SmoothStep, t),
		Operator::Sigmoid(t) => (OperationGpu::Sigmoid, t),
		_ => unreachable!("not a binary operator")
	};

	let mut out = vec!();

	let stack_index = match (lhs, rhs) {
		(ShapeOrStack::Shape(shape), ShapeOrStack::Shape(shape2)) => {
			*stack_index += 1;
			if current_transform != &shape.transform {
				*current_transform = shape.transform;
				out.push(get_transform(shape.transform));
			}
			let (command, slots) = shape.data.compile(font_render)?;
			let stroke_width = shape.stroke.unwrap_or(-1.0);
			out.push(DrawCommandGpu {
				command: command as u32,
				stroke_width,
				slots,
				operation: OperationGpu::Replace as u32,
				lhs: *stack_index,
				// clip_rect_lt_x: clip_rect.lt().x,
				// clip_rect_lt_y: clip_rect.lt().y,
				// clip_rect_rb_x: clip_rect.rb().x,
				// clip_rect_rb_y: clip_rect.rb().y,
				parameter: 0.0,
				..Default::default()
			});
			if current_transform != &shape2.transform {
				*current_transform = shape2.transform;
				out.push(get_transform(shape2.transform));
			}
			let (command, slots) = shape2.data.compile(font_render)?;
			let stroke_width = shape2.stroke.unwrap_or(-1.0);
			out.push(DrawCommandGpu {
				command: command as u32,
				slots,
				stroke_width,
				operation: op as u32,
				lhs: *stack_index,
				parameter,
				// clip_rect_lt_x: clip_rect.lt().x,
				// clip_rect_lt_y: clip_rect.lt().y,
				// clip_rect_rb_x: clip_rect.rb().x,
				// clip_rect_rb_y: clip_rect.rb().y,
				..Default::default()
			});
			*stack_index
		},
		(ShapeOrStack::Stack(index), ShapeOrStack::Shape(shape)) | 
		(ShapeOrStack::Shape(shape), ShapeOrStack::Stack(index)) => {
			let (command, slots) = shape.data.compile(font_render)?;
			let stroke_width = shape.stroke.unwrap_or(-1.0);
			out.push(DrawCommandGpu {
				command: command as u32,
				slots,
				stroke_width,
				operation: OperationGpu::Replace as u32,
				lhs: *stack_index,
				// clip_rect_lt_x: clip_rect.lt().x,
				// clip_rect_lt_y: clip_rect.lt().y,
				// clip_rect_rb_x: clip_rect.rb().x,
				// clip_rect_rb_y: clip_rect.rb().y,
				parameter: 0.0,
				..Default::default()
			});
			out.push(DrawCommandGpu {
				command: CommandGpu::Load as u32,
				slots: [
					[*stack_index as f32, 0.0, 0.0, 0.0],
					[0.0, 0.0, 0.0, 0.0],
					[0.0, 0.0, 0.0, 0.0],
					[0.0, 0.0, 0.0, 0.0],
				],
				stroke_width,
				operation: op as u32,
				lhs: index,
				parameter,
				// clip_rect_lt_x: clip_rect.lt().x,
				// clip_rect_lt_y: clip_rect.lt().y,
				// clip_rect_rb_x: clip_rect.rb().x,
				// clip_rect_rb_y: clip_rect.rb().y,
				..Default::default()
			});
			index
		},
		(ShapeOrStack::Stack(l_index), ShapeOrStack::Stack(r_index)) => {
			*stack_index -= 1;
			out.push(DrawCommandGpu {
				lhs: l_index,
				..get_stack(r_index, op, parameter, /* clip_rect */)
			});
			l_index
		}
	};

	Some((out, ShapeOrStack::Stack(stack_index)))
}

impl ShapeToDraw {
	pub(crate) fn parse(self, font_render: &FontRender) -> (Vec<DrawCommandGpu>, u32) {
		// let clip_rect = self.clip_rect;
		
		let mut current_transform = Transform2D::IDENTITY; 
		// let current_blend_mode = BlendMode::default();

		let mut stack = vec!();
		let mut max_stack_size = 0;
		let mut used_stack_amount  = 0;
		let mut out = vec!();

		if self.fill_mode.is_invisible() {
			return (vec!(), 0);
		}

		for elem in self.shape.0 {
			match elem {
				ShapeOrOp::Shape(shape) => {
					stack.push(ShapeOrStack::Shape(shape));
				},
				ShapeOrOp::Op(op) => {
					if matches!(op, Operator::Not) {
						let lhs = stack.pop().unwrap();
						match lhs {
							ShapeOrStack::Shape(shape) => {
								used_stack_amount += 1;
								max_stack_size = max_stack_size.max(used_stack_amount);
								if current_transform != shape.transform {
									current_transform = shape.transform;
									out.push(get_transform(shape.transform));
								}
								let (command, slots) = shape.data.compile(font_render).unwrap();
								let stroke_width = shape.stroke.unwrap_or(-1.0);
								out.push(DrawCommandGpu {
									command: command as u32,
									slots,
									stroke_width,
									operation: OperationGpu::Neg as u32,
									lhs: used_stack_amount,
									// clip_rect_lt_x: clip_rect.lt().x,
									// clip_rect_lt_y: clip_rect.lt().y,
									// clip_rect_rb_x: clip_rect.rb().x,
									// clip_rect_rb_y: clip_rect.rb().y,
									parameter: 0.0,
									..Default::default()
								});
							},
							ShapeOrStack::Stack(stack_index) => {
								out.push(get_stack(stack_index, OperationGpu::Neg, 0.0, /* clip_rect */));
							}
						}
						continue;
					}
					
					let rhs = stack.pop().unwrap();
					let lhs = stack.pop().unwrap();

					if let Some((commands, stack_index)) = 
					hanle_binary_op(
						op, 
						lhs, 
						rhs, 
						&mut current_transform, 
						font_render, 
						&mut used_stack_amount, 
						// clip_rect
					) {
						max_stack_size = max_stack_size.max(used_stack_amount);
						stack.push(stack_index);
						out.extend(commands);
					}
				}
			}
		}

		assert!(stack.len() == 1);

		if let Some(shape) = stack.pop() {
			match shape {
				ShapeOrStack::Shape(shape) => {
					if current_transform != shape.transform {
						current_transform = shape.transform;
						out.push(get_transform(shape.transform));
					}
					let (command, slots) = if let Some(inner) = shape.data.compile(font_render) {
						inner
					}else {
						return (vec!(), 0);
					};
					let stroke_width = shape.stroke.unwrap_or(-1.0);
					out.push(DrawCommandGpu {
						command: command as u32,
						slots,
						stroke_width,
						operation: OperationGpu::Replace as u32,
						lhs: 1,
						// clip_rect_lt_x: clip_rect.lt().x,
						// clip_rect_lt_y: clip_rect.lt().y,
						// clip_rect_rb_x: clip_rect.rb().x,
						// clip_rect_rb_y: clip_rect.rb().y,
						parameter: 0.0,
						..Default::default()
					});
				},
				ShapeOrStack::Stack(stack_index) => {
					assert!(stack_index == 1)
				},
			}
		}

		if current_transform != Transform2D::IDENTITY {
			// current_transform = Transform2D::IDENTITY;
			out.push(get_transform(Transform2D::IDENTITY));
		}

		out.push(DrawCommandGpu {
			command: CommandGpu::DrawRectangle as u32,
			slots: [
				[self.clip_rect.lt().x, self.clip_rect.lt().y, self.clip_rect.rb().x, self.clip_rect.rb().y],
				[0.0, 0.0, 0.0, 0.0],
				[0.0, 0.0, 0.0, 0.0],
				[0.0, 0.0, 0.0, 0.0],
			],
			stroke_width: -1.0,
			operation: OperationGpu::And as u32,
			smooth_function: 0,
			smooth_parameter: 0.0,
			lhs: 1,
			parameter: 0.0,
			__padding: Default::default(),
			// ..Default::default()
		});

		out.push(DrawCommandGpu {
			command: CommandGpu::Load as u32,
			slots: [
				[1.0, 0.0, 0.0, 0.0],
				[0.0, 0.0, 0.0, 0.0],
				[0.0, 0.0, 0.0, 0.0],
				[0.0, 0.0, 0.0, 0.0],
			],
			stroke_width: -1.0,
			operation: OperationGpu::Or as u32,
			// smooth_function: 0,
			// smooth_parameter: 0.0,
			lhs: 0,
			parameter: 0.0,
			// clip_rect_lt_x: clip_rect.lt().x,
			// clip_rect_lt_y: clip_rect.lt().y,
			// clip_rect_rb_x: clip_rect.rb().x,
			// clip_rect_rb_y: clip_rect.rb().y,
			// __padding: Default::default(),
			..Default::default()
		});

	
		// current_blend_mode = self.blend_mode;
		out.push(DrawCommandGpu {
			command: CommandGpu::SetBlendMode as u32,
			slots: [
				[self.blend_mode as u32 as f32, 0.0, 0.0, 0.0],
				[0.0, 0.0, 0.0, 0.0],
				[0.0, 0.0, 0.0, 0.0],
				[0.0, 0.0, 0.0, 0.0],
			],
			stroke_width: -1.0,
			operation: OperationGpu::None as u32,
			// smooth_function: 0,
			// smooth_parameter: 0.0,
			lhs: 0,
			parameter: 0.0,
			// clip_rect_lt_x: clip_rect.lt().x,
			// clip_rect_lt_y: clip_rect.lt().y,
			// clip_rect_rb_x: clip_rect.rb().x,
			// clip_rect_rb_y: clip_rect.rb().y,
			// __padding: Default::default(),
			..Default::default()
		});
		

		let (fill, slots) = self.fill_mode.compile();
		
		// println!("{:?}, {:?}", fill, slots);

		out.push(DrawCommandGpu {
			command: fill as u32,
			slots,
			stroke_width: -1.0,
			operation: OperationGpu::None as u32,
			// smooth_function: 0,
			// smooth_parameter: 0.0,
			lhs: 0,
			parameter: 0.0,
			// clip_rect_lt_x: clip_rect.lt().x,
			// clip_rect_lt_y: clip_rect.lt().y,
			// clip_rect_rb_x: clip_rect.rb().x,
			// clip_rect_rb_y: clip_rect.rb().y,
			// __padding: Default::default(),
			..Default::default()
		});
		
		(out, max_stack_size + 1)
	}
}

impl FillMode {
	fn compile(self) -> (CommandGpu, [[f32; 4]; 4]) {
		match self {
			Self::Color(color) => {
				let color = color.premultiply();
				(CommandGpu::Fill, [
					[color.r, color.g, color.b, color.a],
					[0.0, 0.0, 0.0, 0.0],
					[0.0, 0.0, 0.0, 0.0],
					[0.0, 0.0, 0.0, 0.0],
				])
			},
			Self::LinearGradient(from_color, to_color, start, end) => {
				let from_color = from_color.premultiply();
				let to_color = to_color.premultiply();
				(CommandGpu::FillLinearGradient, [
					[from_color.r, from_color.g, from_color.b, from_color.a],
					[to_color.r, to_color.g, to_color.b, to_color.a],
					[start.x, start.y, end.x, end.y],
					[0.0, 0.0, 0.0, 0.0],
				])
			},
			Self::RadialGradient(inner_color, outer_color, center, radius) => {
				let inner_color = inner_color.premultiply();
				let outer_color = outer_color.premultiply();
				(CommandGpu::FillRadialGradient, [
					[inner_color.r, inner_color.g, inner_color.b, inner_color.a],
					[outer_color.r, outer_color.g, outer_color.b, outer_color.a],
					[center.x, center.y, radius, 0.0],
					[0.0, 0.0, 0.0, 0.0],
				])
			},
			Self::Texture(texture_id, lt, rb, tlt, trb)=> {
				(CommandGpu::FillTexture, [
					[lt.x, lt.y, rb.x, rb.y],
					[tlt.x, tlt.y, trb.x, trb.y],
					[texture_id as f32, 0.0, 0.0, 0.0],
					[0.0, 0.0, 0.0, 0.0]
				])
			},
		}
	}
}

impl BasicShapeData {
	fn compile(self, font_render: &FontRender) -> Option<(CommandGpu, [[f32; 4]; 4])> {
		Some(match self {
			Self::Circle(center, radius) => {
				(CommandGpu::DrawCircle, [
					[center.x, center.y, radius, 0.0],
					[0.0, 0.0, 0.0, 0.0],
					[0.0, 0.0, 0.0, 0.0],
					[0.0, 0.0, 0.0, 0.0],
				])
			},
			Self::Triangle(a, b, c) => {
				(CommandGpu::DrawTriangle, [
					[a.x, a.y, b.x, b.y],
					[c.x, c.y, 0.0, 0.0],
					[0.0, 0.0, 0.0, 0.0],
					[0.0, 0.0, 0.0, 0.0],
				])
			},
			Self::Rectangle(a, b, rounding) => {
				(CommandGpu::DrawRectangle, [
					[a.x, a.y, b.x, b.y],
					[rounding.x(), rounding.y(), rounding.z(), rounding.w()],
					[0.0, 0.0, 0.0, 0.0],
					[0.0, 0.0, 0.0, 0.0],
				])
			},
			Self::HalfPlane(a, b) => {
				(CommandGpu::DrawHalfPlane, [
					[a.x, a.y, b.x, b.y],
					[0.0, 0.0, 0.0, 0.0],
					[0.0, 0.0, 0.0, 0.0],
					[0.0, 0.0, 0.0, 0.0],
				])
			},
			Self::QuadBezierPlane(a, b, c) => {
				(CommandGpu::DrawQuadPlane, [
					[a.x, a.y, b.x, b.y],
					[c.x, c.y, 0.0, 0.0],
					[0.0, 0.0, 0.0, 0.0],
					[0.0, 0.0, 0.0, 0.0],
				])
			},
			Self::SDFTexture(a, b, texture_id) => {
				(CommandGpu::DrawSDFTexture, [
					[a.x, a.y, b.x, b.y],
					[texture_id as f32, 0.0, 0.0, 0.0],
					[0.0, 0.0, 0.0, 0.0],
					[0.0, 0.0, 0.0, 0.0],
				])
			},
			Self::Text(pos, font_id, font_size, chr) => {
				let char_id = *font_render.char_texture_map.get(&(chr, font_id))?;
				(CommandGpu::DrawChar, [
					[pos.x, pos.y, font_size, char_id as f32],
					[0.0, 0.0, 0.0, 0.0],
					[0.0, 0.0, 0.0, 0.0],
					[0.0, 0.0, 0.0, 0.0],
				])
			}
		})
	}
}