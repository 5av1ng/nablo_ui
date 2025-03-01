//! Here is the implementation of the DrawCommand struct.

/// The DrawCommand used by gpu to render the graphics.
/// 
/// Here is compiled version of the struct.
/// You can see orignal at [`crate::render::shape::Shape`]
/// 
/// Due to the memory alignment strategy of the wgpu, the struct actually contains a field which is used for padding.
#[derive(bytemuck::Pod, bytemuck::Zeroable, Debug, Clone, Copy, Default)]
#[derive(serde::Deserialize, serde::Serialize)]
#[repr(C, align(16))]
pub struct DrawCommandGpu {
	/// See [`CommandGpu`] for possible values.
	pub command: u32,
	/// The stroke width of the shape.
	/// 
	/// set to -1.0 to disable stroke.
	pub stroke_width: f32,
	/// The padding to align the struct to 16 bytes.
	/// 
	/// actually done nothing, but it's required to align the struct to 16 bytes.
	/// The parameter may used by operation.
	pub parameter: f32,
	// /// The clip rect's left-top x coordinate of the shape.
	// pub clip_rect_lt_x: f32,
	// /// The clip rect's left-top y coordinate of the shape.
	// pub clip_rect_lt_y: f32,
	// /// The clip rect's right-bottom x coordinate of the shape.
	// pub clip_rect_rb_x: f32,
	// /// The clip rect's right-bottom y coordinate of the shape.
	// pub clip_rect_rb_y: f32,
	/// See [`OperationGpu`] for possible values.
	pub smooth_function: u32,
	/// values may be used by command variants.
	pub slots: [[f32; 4]; 4],
	/// The way to combine with the previous content.
	/// 
	/// See [`OperationGpu`] for possible values.
	pub operation: u32,
	/// The parameter may used by smooth function.
	pub smooth_parameter: f32,
	/// The index of the shape to combine with the previous content.
	pub lhs: u32,
	pub(crate) __padding: [u8; 4],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[derive(serde::Deserialize, serde::Serialize)]
#[repr(u32)]
/// The possible commands that can be sent to the gpu.
/// 
/// All color related commands will be set as output color if the output value of stack(stack\[0\]) is negative.
pub enum CommandGpu {
	/// Do nothing.
	#[default] None = 0,
	/// Draw a circle.
	/// 
	/// Will expect 3 values in `slot`:
	/// 1. center.x
	/// 2. center.y
	/// 3. radius
	DrawCircle = 1,
	/// Draw a triangle.
	/// 
	/// Will expect 6 values in `slot`:
	/// 1. p1.x
	/// 2. p1.y
	/// 3. p2.x
	/// 4. p2.y
	/// 5. p3.x
	/// 6. p3.y
	DrawTriangle = 2,
	/// Draw a rectangle.
	/// 
	/// Will expect 4 values in `slot`:
	/// 1. left-top.x
	/// 2. left-top.y
	/// 3. right-bottom.x
	/// 4. right-bottom.y
	/// 5. left_top_rounding
	/// 6. right_top_rounding
	/// 7. right_bottom_rounding
	/// 8. left_bottom_rounding
	DrawRectangle = 3,
	/// Draw a half plane.
	/// 
	/// Will expect 4 values in `slot`:
	/// 1. p1.x
	/// 2. p1.y
	/// 3. p2.x
	/// 4. p2.y
	DrawHalfPlane = 4,
	/// Draw a quadratic bezier half plane.
	/// 
	/// Will expect 6 values in `slot`:
	/// 1. p1.x
	/// 2. p1.y
	/// 3. p2.x
	/// 4. p2.y
	/// 5. p3.x
	/// 6. p3.y
	DrawQuadPlane = 5,
	/// Draw a SDF texture.
	/// 
	/// Will expect 5 values in `slot`:
	/// 1. top-left.x
	/// 2. top-left.y
	/// 3. right-bottom.x
	/// 4. right-bottom.y
	/// 5. texture id as u32
	DrawSDFTexture = 6,
	/// Draw a single character text.
	/// 
	/// Will expect 7 values in `slot`:
	/// 1. position.x
	/// 2. position.y
	/// 3. font size as f32
	/// 4. char_id as u32
	/// 
	/// Will get the char pos from the char id and draw it.
	/// We can caculate the char pos id by the char id as follows.
	/// ```wgsl
	/// fn char_pos(char_id: u32) -> vec3<f32> {
	///     // FONT_TEXTURE_SIZE = 2048
	///     // CHAR_TEXTURE_SIZE = 64
	///     let module = (2048 * 2048) / (64 * 64);
	///     let page = char_id / module + 1;
	///     let char_pos_id = char_id % module;
	///     let char_pos_x = char_pos_id % 32;
	///     let char_pos_y = char_pos_id / 32;
	///     return vec3<f32>(
	///         f32(char_pos_x) * 64.0,
	///         f32(char_pos_y) * 64.0,
	///         f32(page)
	///     );
	/// }
	/// ```
	/// 
	/// the font texture_size will be always [`crate::render::font::CHAR_TEXTURE_SIZE`]
	DrawChar = 7,
	/// Fill the current path with a solid color.
	/// 
	/// Will expect 4 values in `slot`:
	/// 1. color.r
	/// 2. color.g
	/// 3. color.b
	/// 4. color.a
	Fill = 8,
	/// Fill the current path with a linear gradient.
	/// 
	/// Will expect 12 values in `slot`:
	/// 1. start.r
	/// 2. start.g
	/// 3. start.b
	/// 4. start.a
	/// 5. end.r
	/// 6. end.g
	/// 7. end.b
	/// 8. end.a
	/// 9. from.x
	/// 10. from.y
	/// 11. to.x
	/// 12. to.y
	FillLinearGradient = 9,
	/// Fill the current path with a radial gradient.
	/// 
	/// Will expect 11 values in `slot`:
	/// 1. inner.r
	/// 2. inner.g
	/// 3. inner.b
	/// 4. inner.a
	/// 5. outer.r
	/// 6. outer.g
	/// 7. outer.b
	/// 8. outer.a
	/// 9. center.x
	/// 10. center.y
	/// 11. radius
	FillRadialGradient = 10,
	/// Fill the current path with a texture.
	/// 
	/// Will expect 5 values in `slot`:
	/// 1. top_left.x -> 0.0 of texture coordinate
	/// 2. top_left.y -> 0.0 of texture coordinate
	/// 3. right_bottom.x -> 1.0 of texture coordinate
	/// 4. right_bottom.y -> 1.0 of texture coordinate
	/// 5. texture_left_top.x
	/// 6. texture_left_top.y
	/// 7. texture_right_bottom.x
	/// 8. texture_right_bottom.y
	/// 9. texture id as u32
	FillTexture = 11,
	/// Set the current transform matrix.
	/// 
	/// Will expect 6 values in `slot`:
	/// 1. m00
	/// 2. m10
	/// 3. m20
	/// 4. m01
	/// 5. m11
	/// 6. m21
	/// 
	/// is equivalent to:
	/// $$
	/// \begin{bmatrix}
	/// m00 & m10 & m20 \\
	/// m01 & m11 & m21 \\
	/// m02 & m12 & m22 \\
	/// \end{bmatrix}
	/// $$
	SetMat3x3 = 12,
	/// Set the blend mode for the current shape.
	/// 
	/// Will expect 1 value in `slot`:
	/// 1. blend mode as u32
	/// 
	/// See [`BlendMode`] for possible values.
	SetBlendMode = 13,
	/// Load a shape from the stack.
	/// 
	/// Will expect 1 value in `slot`:
	/// 1. index of the shape in the stack as u32
	Load = 14,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[derive(serde::Deserialize, serde::Serialize)]
#[repr(u16)]
/// The possible operations that can be done between two distance fields.
/// 
/// The rhs of the operation will be the field that is caculating.
/// The lhs of the operation will be the field that is setted by the operation.
/// 
/// Operation may use a parameter or a smooth function.
///
/// if [`DrawCommandGpu::smooth_function`] is 0, the smooth function will not be applied,
/// otherwise the function id will be used to select the smooth function.
/// The smooth function id is also defined here ([`Self::Lerp`], [`Self::SmoothStep`], [`Self::Sigmoid`]).
pub enum OperationGpu {
	/// Do nothing.
	None = 0,
	/// Replace lhs with rhs.
	Replace = 1,
	/// Replace lhs with rhs if rhs the point inside the shape.
	#[default] ReplaceWhenInside = 2,
	/// Replace lhs with rhs if rhs shows the point outside the shape.
	ReplaceWhenOutside = 3,
	/// Apply and operation between lhs and rhs equivalent to max(lhs, rhs).
	And = 4,
	/// Apply or operation between lhs and rhs equivalent to min(lhs, rhs).
	Or = 5,
	/// Apply xor operation between lhs and rhs evaulated as (lhs + rhs - 2 * min(lhs, rhs)).
	Xor = 6,
	/// Apply subtraction operation between lhs and rhs equivalent to max(lhs, - rhs).
	Sub = 7,
	/// Apply negation operation on rhs.
	Neg = 8,
	/// Apply lerp operation between lhs and rhs.
	/// 
	/// The t parameter is setted by the parameter value of the operation.
	Lerp = 9,
	/// Apply smoothstep operation between lhs and rhs.
	/// 
	/// The t parameter is setted by the parameter value of the operation.
	SmoothStep = 10,
	/// Apply sigmoid operation between lhs and rhs.
	/// 
	/// Also known as logistic function.
	/// It's defined as 1 / (1 + exp(-t)).
	/// 
	/// The t parameter is setted by the parameter value of the operation.
	/// 
	/// Will be applied as follows:
	/// ```wgsl
	/// fn sigmoid(lhs: f32, rhs: f32, operation: u32) -> f32 {
	///     let t = parse_parameter(operation);
	///     let sigmoid = 1.0 / (1.0 + exp(-t));
	///     return lerp(lhs, rhs, sigmoid);
	/// }
	/// ```
	Sigmoid = 11,
}

/// The possible blend modes for the current shape.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[derive(serde::Deserialize, serde::Serialize)]
#[repr(u32)]
pub enum BlendMode {
	/// Simply replace the color of the shape.
	Replace = 0,
	/// Add the color of the shape to the current color.
	Add = 1,
	/// Multiply the color of the shape with the current color.
	Multiply = 2,
	/// Subtract the color of the shape from the current color.
	Subtract = 3,
	/// Divide the color of the shape by the current color.
	Divide = 4,
	/// The color of the shape will be the minimum of the current color and the shape color.
	Min = 5,
	/// The color of the shape will be the maximum of the current color and the shape color.
	Max = 6,
	/// The color will be multiplied by the alpha of the shape and added to the current color.
	#[default] AlphaAdd = 7,
	// /// Does exact same thing as [`Self::AlphaAdd`] when the current color's alpha is not 1.0, otherwise it's the same as [`Self::Replace`].
	// #[default] AlphaMix = 8,
}