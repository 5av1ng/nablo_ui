struct DrawCommand {
	command: u32,
	stroke_width: f32,
	parameter: f32,
	slots: mat4x4<f32>,
	operation: u32,
	smooth_function: u32,
	smooth_parameter: f32,
	lhs: u32,
}

struct Uniforms {
	window_size: vec2<f32>,
	mouse: vec2<f32>,
	time: f32,
	scale_factor: f32,
	stack_len: u32,
	command_len: u32,
}

const EDGE_WIDTH: f32 = 1.0;
const TEXTURE_SIZE: vec2<f32> = vec2<f32>(2560.0, 2560.0);
const FONT_TEXTURE_SIZE: vec2<f32> = vec2<f32>(2048.0, 2048.0);
const CHAR_SIZE: vec2<f32> = vec2<f32>(64.0, 64.0);
const EM: f32 = CHAR_SIZE.x;
const MSDF_RANGE: f32 = 64.0;
const EPSILON: f32 = 0.0001;

@group(0) @binding(0) var<uniform> uniforms: Uniforms;
// @group(0) @binding(1) var<storage, read_write> stack: array< vec4<f32> >;
@group(1) @binding(0) var<storage, read> draw_commands: array<DrawCommand>;
@group(2) @binding(1) var texture_array: texture_2d_array<f32>;
@group(2) @binding(0) var sampler_texture: sampler;
@group(3) @binding(1) var font_texture_array: texture_2d_array<f32>;
@group(3) @binding(0) var sampler_font: sampler;

@vertex
fn vs_main(
	@builtin(vertex_index) in_vertex_index: u32,
) -> @builtin(position) vec4<f32> {
	let pos = vec2<f32>(
		(vec2(1u, 2u) + in_vertex_index) % 6u < vec2(3u, 3u)
	) * 2.0 - 1.0;
	return vec4f(pos, 0.0, 1.0);
}

fn line(pos: vec2<f32>, start: vec2<f32>, end: vec2<f32>) -> f32 {
	let a = end - start;
	let b = pos - start;
	let t = clamp(dot(b, a) / dot(a, a), 0.0, 1.0);
	let closest = start + a * t;
	return length(pos - closest);
}

fn circle(pos: vec2<f32>, center: vec2<f32>, radius: f32) -> f32 {
	return length(pos - center) - radius;
}

fn triangle(pos: vec2<f32>, p1: vec2<f32>, p2: vec2<f32>, p3: vec2<f32>) -> f32 {
	let d_0 = line(pos, p1, p2);
	let d_1 = line(pos, p2, p3);
	let d_2 = line(pos, p3, p1);

	let sgn = sign(cross(p2 - p1, pos - p1)) + sign(cross(p3 - p2, pos - p2)) + sign(cross(p1 - p3, pos - p3));

	if sgn == 3.0 {
		return - min(d_0, min(d_1, d_2));
	}else {
		return min(d_0, min(d_1, d_2));
	}
}

fn rectangle(pos: vec2<f32>, left_top: vec2<f32>, right_bottom: vec2<f32>, roundings: vec4<f32>) -> f32 {
	let size = right_bottom - left_top;
	let center = left_top + size / 2.0;
	let moved_pos = pos - center;

	var r = 0.0;
	if moved_pos.x <= 0.0 && moved_pos.y <= 0.0 {
		r = roundings.x;
	}else if moved_pos.x >= 0.0 && moved_pos.y <= 0.0 {
		r = roundings.y;
	}else if moved_pos.x <= 0.0 && moved_pos.y >= 0.0 {
		r = roundings.z;
	}else if moved_pos.x >= 0.0 && moved_pos.y >= 0.0 {
		r = roundings.w;
	}

	if r == 0.0 {
		return max(abs(moved_pos.x) - size.x / 2.0, abs(moved_pos.y) - size.y / 2.0);
	}else {
		r = min(r, min(size.x / 2.0, size.y / 2.0));

		let d = abs(moved_pos) - size / 2.0 + vec2(r, r);
		return length(max(d, vec2(0.0, 0.0))) - r;
	}

	
}

fn half_plane(pos: vec2<f32>, p1: vec2<f32>, p2: vec2<f32>) -> f32 {
	let a = p2.y - p1.y;
	let b = p1.x - p2.x;
	let c = p1.y * p2.x - p1.x * p2.y;
	return - (a * pos.x + b * pos.y + c) / sqrt(a * a + b * b);
}

fn cross(a: vec2<f32>, b: vec2<f32>) -> f32 {
	return a.x * b.y - a.y * b.x;
}

fn cos_acos_3(fx: f32) -> f32 { 
	let x = sqrt(0.5 + 0.5 * fx); 
	return x * (x * (x * (x * - 0.008972 + 0.039071) - 0.107074) + 0.576975) + 0.5; 
}

fn quad_bezier(pos: vec2<f32>, start: vec2<f32>, ctrl: vec2<f32>, end: vec2<f32>) -> f32 {
	// The algorithm is inspried by https://www.shadertoy.com/view/MlKcDD.
	// which is mainly using the Cardano formula to solve the SDF for a quadratic Bezier curve.

	let a = ctrl - start;
	let b = end - 2.0 * ctrl + start;
	let c = 2.0 * a;
	let d = start - pos;

	let denominator = dot(b, b);
	let fx = dot(a, b) / denominator;
	let fy = (2.0 * dot(a, a) + dot(d, b)) / (3.0 * denominator);
	let fz = dot(d, a) / denominator;

	var res = 0.0;
	var sgn = 0.0;

	let p = fy - fx * fx;
	let q = fx * (2.0 * fx * fx - 3.0 * fy) + fz;
	let p3 = p * p * p;
	let q2 = q * q;
	var fh = q2 + 4.0 * p3;

	if fh >= 0.0 {
		let h = sqrt(fh);
		let x = vec2(h - q, - h - q) / 2.0;
		let uv = vec2(sign(x.x), sign(x.y)) * vec2(pow(abs(x.x), 1.0 / 3.0), pow(abs(x.y), 1.0 / 3.0));
		var t = uv.x + uv.y;
		t -= (t * (t * t + 3.0 * p) + q) / (3.0 * t * t + 3.0 * p);
		t = clamp(t - fx, 0.0, 1.0);

		let w = d + (c + b * t) * t;
		res = dot(w, w);
		sgn = cross(c + 2.0 * b * t, w);
	}else {
		let z = sqrt(-p);
		let m = cos_acos_3(q / (p * z * 2.0));
		let n = sqrt(3.0) * sqrt(1.0 - m * m);
		let t = vec2(
			clamp((m + m) * z - fx, 0.0, 1.0),
			clamp(- (n + m) * z - fx, 0.0, 1.0),
		);

		let qx = d + t.x * (c + b * t.x);
		let qy = d + t.y * (c + b * t.y);
		let dx = dot(qx, qx);
		let dy = dot(qy, qy);
		let sx = cross(a + b * t.x, qx);
		let sy = cross(a + b * t.y, qy);

		if dx < dy {
			res = dx;
			sgn = sx;
		}else {
			res = dy;
			sgn = sy;
		}
	}

	return sqrt(res) * sign(sgn);
}

fn ray(pos: vec2<f32>, start: vec2<f32>, direction: vec2<f32>, is_left: bool) -> f32 {
	let sgn = select(1.0, -1.0, is_left);
	let rotated_direction = vec2(direction.y, -direction.x) * sgn;
	let rd = sgn * abs(half_plane(pos, start, start + rotated_direction)) * sign(cross(pos - start, rotated_direction));
	let d = sgn * abs(half_plane(pos, start, start + direction)) * sign(cross(pos - start, direction));
	// return min(-d, rotated_d);
	return min(-d, rd);
}

fn quad_half_plane(pos: vec2<f32>, start: vec2<f32>, ctrl: vec2<f32>, end: vec2<f32>) -> f32 {
	var quad = 0.0;
	// var ray_1 = 0.0;
	// var ray_2 = 0.0;
	if cross(start - ctrl, end - ctrl) > 0.0 {
		quad = quad_bezier(pos, start, ctrl, end);
		// ray_1 = ray(pos, start, ctrl - start, true);
		// ray_2 = ray(pos, end, ctrl - end, false);
	}else {
		quad = quad_bezier(pos, end, ctrl, start);
		// ray_1 = ray(pos, start, ctrl - start, false);
		// ray_2 = ray(pos, end, ctrl - end, true);
	}
	return quad;
}

fn sdf_texture(
	pos: vec2<f32>, 
	texture_id: u32, 
	lt: vec2<f32>, 
	rb: vec2<f32>, 
) -> f32 {
	let size = rb - lt;
	let uv = (pos - lt) / size;
	let color = textureSample(texture_array, sampler_texture, uv, texture_id);
	let grayscale = clamp(dot(color.xyz, vec3<f32>(0.299, 0.587, 0.114)), 0.0, 1.0);
	return (grayscale - 0.5) * 2.0 * min(uniforms.window_size.x, uniforms.window_size.y) / 2.0;
}

fn median(r: f32, g: f32, b: f32) -> f32 {
	return max(min(r, g), min(max(r, g), b));
}

fn msdf_char(
	pos: vec2<f32>, 
	char_pos: vec2<f32>,
	char_size: f32,
	char_id: u32,
) -> f32 {
	let mod_val = u32((FONT_TEXTURE_SIZE / CHAR_SIZE).x);
	let char_size_texture = CHAR_SIZE * char_size / EM;
	let uv = (pos - char_pos) / char_size_texture;
	if uv.x < 0.0 || uv.x > 1.0 || uv.y < 0.0 || uv.y > 1.0 {
		return 1.0;
	}
	let page = char_id / (mod_val * mod_val);
	let char_pos_id = char_id % (mod_val * mod_val);
	let char_pos_x = f32(char_pos_id % mod_val);
	let char_pos_y = f32(char_pos_id / mod_val);
	let char_lt = vec2(char_pos_x, char_pos_y) * CHAR_SIZE.x / FONT_TEXTURE_SIZE.x;
	let texture_uv = uv * CHAR_SIZE.x / FONT_TEXTURE_SIZE.x + char_lt;
	// let eps = CHAR_SIZE.x / FONT_TEXTURE_SIZE.x / EM;
	// let texture_uv1 = texture_uv + vec2(eps, 0.0);
	// let texture_uv2 = texture_uv + vec2(-eps, 0.0);
	// let texture_uv3 = texture_uv + vec2(0.0, eps);
	// let texture_uv4 = texture_uv + vec2(0.0, -eps);
	// let color1 = textureSample(font_texture_array, sampler_font, texture_uv1, page);
	// let color2 = textureSample(font_texture_array, sampler_font, texture_uv2, page);
	// let color3 = textureSample(font_texture_array, sampler_font, texture_uv3, page);
	// let color4 = textureSample(font_texture_array, sampler_font, texture_uv4, page);
	let color = textureSample(font_texture_array, sampler_font, texture_uv, page);
	// let sd1 = median(color1.x, color1.y, color1.z);
	// let sd2 = median(color2.x, color2.y, color2.z);
	// let sd3 = median(color3.x, color3.y, color3.z);
	// let sd4 = median(color4.x, color4.y, color4.z);
	let sd = median(color.x, color.y, color.z);
	// let sd_avaerage = (sd1 + sd2 + sd3 + sd4 + sd) / 5.0;
	let range = 0.5;
	return - smoothstep(0.5 - range, 0.5 + range, sd);
	// return select(1.0, -1.0, (color != vec4f(0.0, 0.0, 0.0, 0.0)));
}

fn to_stroke(d: f32, stroke_width: f32) -> f32 {
	return abs(d) - stroke_width / 2.0;
}

fn radial_gradient(
	pos: vec2<f32>,   
	center: vec2<f32>,
	radius: f32,
	inner_color: vec4<f32>, 
	outer_color: vec4<f32>, 
) -> vec4f {
	let distance = length(pos - center);
	let t = distance / radius;

	return mix(inner_color, outer_color, clamp(t, 0.0, 1.0));
}

fn linear_gradient(
	pos: vec2<f32>,
	start: vec2<f32>,
	end: vec2<f32>,
	inner_color: vec4<f32>,
	outer_color: vec4<f32>,
) -> vec4f {
	let t = dot(pos - start, end - start) / dot(end - start, end - start);
	return mix(inner_color, outer_color, clamp(abs(t), 0.0, 1.0));
}

fn texture_fill(
	pos: vec2<f32>,
	texture_id: u32,
	lt: vec2<f32>,
	rb: vec2<f32>,
	uv_lt: vec2<f32>,
	uv_rb: vec2<f32>,
) -> vec4f {
	let texture_uv_lt = uv_lt / TEXTURE_SIZE;
	let texture_uv_size = (uv_rb - uv_lt) / TEXTURE_SIZE;
	let size = rb - lt;
	let uv = (pos - lt) / size * texture_uv_size + texture_uv_lt;
	return textureSample(texture_array, sampler_texture, uv, texture_id);
}

// Simulating enum, therefore we use UpperCamelCase rather than SCREAMING_SNAKE_CASE.
// Here is `CommandGpu` in Rust, see more details in `src/render/command.rs`.
const CommandNone: u32 = 0u;
const DrawCircle: u32 = 1u;
const DrawTriangle: u32 = 2u;
const DrawRectangle: u32 = 3u;
const DrawHalfPlane: u32 = 4u;
const DrawQuadHalfPlane: u32 = 5u;
const DrawSDFTexture: u32 = 6u;
const DrawChar: u32 = 7u;
const Fill: u32 = 8u;
const LinearGradient: u32 = 9u;
const RadialGradient: u32 = 10u;
const TextureFill: u32 = 11u;
const SetTransform: u32 = 12u;
const SetBlendMode: u32 = 13u;
const Load: u32 = 14u;

// here is `BlendMode` in Rust, see more details in `src/render/command.rs`.
const MixReplace: u32 = 0u;
const Add: u32 = 1u;
const Multiply: u32 = 2u;
const Subtract: u32 = 3u;
const Divide: u32 = 4u;
const Min: u32 = 5u;
const Max: u32 = 6u;
const AlphaAdd: u32 = 7u;
// const AlphaMix: u32 = 8u;

// here is `OperationGpu` in Rust, see more details in `src/render/command.rs`.
const OperationNone: u32 = 0u;
const Replace: u32 = 1u;
const ReplaceWhenInside: u32 = 2u;
const ReplaceWhenOutside: u32 = 3u;
const And: u32 = 4u;
const Or: u32 = 5u;
const Xor: u32 = 6u;
const Sub: u32 = 7u;
const Neg: u32 = 8u;
const Lerp: u32 = 9u;
const SmoothStep: u32 = 10u;
const Sigmoid: u32 = 11u;

fn inverse(m: mat3x3f) -> mat3x3f {
	let det = m[0][0] * (m[1][1] * m[2][2] - m[2][1] * m[1][2]) 
			- m[0][1] * (m[1][0] * m[2][2] - m[2][0] * m[1][2]) 
			+ m[0][2] * (m[1][0] * m[2][1] - m[2][0] * m[1][1]);
	let inv_det = 1.0 / det;
	return mat3x3f(
		(m[1][1] * m[2][2] - m[2][1] * m[1][2]) * inv_det,
		(m[0][2] * m[2][1] - m[2][2] * m[0][1]) * inv_det,
		(m[0][1] * m[1][2] - m[1][1] * m[0][2]) * inv_det,
		(m[1][2] * m[2][0] - m[2][2] * m[1][0]) * inv_det,
		(m[0][0] * m[2][2] - m[2][0] * m[0][2]) * inv_det,
		(m[0][2] * m[1][0] - m[1][2] * m[0][0]) * inv_det,
		(m[1][0] * m[2][1] - m[2][0] * m[1][1]) * inv_det,
		(m[0][1] * m[2][0] - m[2][1] * m[0][0]) * inv_det,
		(m[0][0] * m[1][1] - m[1][0] * m[0][1]) * inv_det,
	);
}

fn mix_color(ori_color: vec4<f32>, new_color: vec4<f32>, blend_mode: u32) -> vec4<f32> {
	switch blend_mode {
		case MixReplace: {
			return new_color;
		}
		case Add: {
			return ori_color + new_color;
		}
		case Multiply: {
			return ori_color * new_color;
		}
		case Subtract: {
			return ori_color - new_color;
		}
		case Divide: {
			return ori_color / new_color;
		}
		case Min: {
			return min(ori_color, new_color);
		}
		case Max: {
			return max(ori_color, new_color);
		}
		case AlphaAdd: {
			let ori_factor = ori_color.a;
			let new_factor = new_color.a * (1.0 - ori_color.a);
			let total_factor = ori_factor + new_factor;
			return vec4f(
				(ori_color.x * ori_factor + new_color.x * new_factor) / total_factor,
				(ori_color.y * ori_factor + new_color.y * new_factor) / total_factor,
				(ori_color.z * ori_factor + new_color.z * new_factor) / total_factor,
				total_factor,
			);
		}
		// case AlphaMix: {
		// 	if new_color.a == 1.0 {
		// 		return new_color;
		// 	}else {
		// 		let ori_factor = ori_color.a;
		// 		let new_factor = new_color.a * (1.0 - ori_color.a);
		// 		let total_factor = ori_factor + new_factor;
		// 		return vec4f(
		// 			(ori_color.x * ori_factor + new_color.x * new_factor) / total_factor,
		// 			(ori_color.y * ori_factor + new_color.y * new_factor) / total_factor,
		// 			(ori_color.z * ori_factor + new_color.z * new_factor) / total_factor,
		// 			ori_color.a + new_color.a,
		// 		);	
		// 	}
		// }
		default: {
			return new_color;
		}
	}
}

@fragment
fn fs_main(@builtin(position) clip_pos: vec4<f32>) -> @location(0) vec4f {
	let pos = clip_pos.xy / uniforms.scale_factor;
	
	var current_command_index = 0u;
	var current_color = vec4f(0.0, 0.0, 0.0, 0.0);
	var current_blend_mode = AlphaAdd;
	var current_transform = mat3x3f(
		1.0, 0.0, 0.0,
		0.0, 1.0, 0.0,
		0.0, 0.0, 1.0,
	);

	var stack = array<f32, 64>();

	// stack[0] = circle(pos, vec2f(200.0, 200.0), 100.0);
	// if stack[0] <= 0.0 {
	// 	current_color = vec4f(1.0, 1.0, 1.0, 1.0);
	// }

	loop {
		if current_command_index >= uniforms.command_len {
			break;
		}

		var temp = 0.0;
		var grad = vec2f(0.0, 0.0);
		let p = (inverse(current_transform) * vec3f(pos, 1.0)).xy;
		let p_plus_x = (inverse(current_transform) * vec3f(pos + vec2f(EPSILON, 0.0), 1.0)).xy;
		let p_plus_y = (inverse(current_transform) * vec3f(pos + vec2f(0.0, EPSILON), 1.0)).xy;
		let p_minus_x = (inverse(current_transform) * vec3f(pos - vec2f(EPSILON, 0.0), 1.0)).xy;
		let p_minus_y = (inverse(current_transform) * vec3f(pos - vec2f(0.0, EPSILON), 1.0)).xy;
		let slots = transpose(draw_commands[current_command_index].slots);

		switch draw_commands[current_command_index].command {
			case CommandNone: {
				current_command_index += 1u;
				continue;
			}
			case DrawCircle: {
				let center = vec2f(
					slots[0][0], 
					slots[1][0]
				);
				let radius = slots[2][0];
				temp = circle(p, center, radius);
				grad.x = (circle(p_plus_x, center, radius) - circle(p_minus_x, center, radius)) / (EPSILON * 2.0);
				grad.y = (circle(p_plus_y, center, radius) - circle(p_minus_y, center, radius)) / (EPSILON * 2.0);
				grad /= length(grad);
			}
			case DrawTriangle: {
				let p1 = vec2f(
					slots[0][0], 
					slots[1][0],
				);
				let p2 = vec2f(
					slots[2][0], 
					slots[3][0],
				);
				let p3 = vec2f(
					slots[0][1], 
					slots[1][1],
				);
				temp = triangle(p, p1, p2, p3);
				grad.x = (triangle(p_plus_x, p1, p2, p3) - triangle(p_minus_x, p1, p2, p3)) / (EPSILON * 2.0);
				grad.y = (triangle(p_plus_y, p1, p2, p3) - triangle(p_minus_y, p1, p2, p3)) / (EPSILON * 2.0);
			}
			case DrawRectangle: {
				let lt = vec2f(
					slots[0][0], 
					slots[1][0],
				);
				let rb = vec2f(
					slots[2][0], 
					slots[3][0],
				);
				let roundings = vec4f(
					slots[0][1],
					slots[1][1],
					slots[2][1],
					slots[3][1],
				);
				temp = rectangle(p, lt, rb, roundings);
				grad.x = (rectangle(p_plus_x, lt, rb, roundings) - rectangle(p_minus_x, lt, rb, roundings)) / (EPSILON * 2.0);
				grad.y = (rectangle(p_plus_y, lt, rb, roundings) - rectangle(p_minus_y, lt, rb, roundings)) / (EPSILON * 2.0);
			}
			case DrawHalfPlane: {
				let start = vec2f(
					slots[0][0], 
					slots[1][0],
				);
				let end = vec2f(
					slots[2][0], 
					slots[3][0],
				);
				temp = half_plane(p, start, end);
				grad.x = (half_plane(p_plus_x, start, end) - half_plane(p_minus_x, start, end)) / (EPSILON * 2.0);
				grad.y = (half_plane(p_plus_y, start, end) - half_plane(p_minus_y, start, end)) / (EPSILON * 2.0);
			}
			case DrawQuadHalfPlane: {
				let start = vec2f(
					slots[0][0], 
					slots[1][0],
				);
				let ctrl = vec2f(
					slots[2][0], 
					slots[3][0],
				);
				let end = vec2f(
					slots[0][1], 
					slots[1][1],
				);
				temp = quad_half_plane(p, start, ctrl, end);
				grad.x = (quad_half_plane(p_plus_x, start, ctrl, end) - quad_half_plane(p_minus_x, start, ctrl, end)) / (EPSILON * 2.0);
				grad.y = (quad_half_plane(p_plus_y, start, ctrl, end) - quad_half_plane(p_minus_y, start, ctrl, end)) / (EPSILON * 2.0);
			}
			case DrawSDFTexture: {
				let lt = vec2f(
					slots[0][0], 
					slots[1][0],
				);
				let rb = vec2f(
					slots[2][0], 
					slots[3][0],
				);
				let texture_id = u32(slots[0][1]);
				temp = sdf_texture(p, texture_id, lt, rb);
				grad.x = (sdf_texture(p_plus_x, texture_id, lt, rb) - sdf_texture(p_minus_x, texture_id, lt, rb)) / (EPSILON * 2.0);
				grad.y = (sdf_texture(p_plus_y, texture_id, lt, rb) - sdf_texture(p_minus_y, texture_id, lt, rb)) / (EPSILON * 2.0);
			}
			case DrawChar: {
				let char_pos = vec2f(
					slots[0][0], 
					slots[1][0],
				);
				let char_size = slots[2][0];
				let char_id = u32(slots[3][0]);
				temp = msdf_char(p, char_pos, char_size, char_id);
				grad.x = (msdf_char(p_plus_x, char_pos, char_size, char_id) - msdf_char(p_minus_x, char_pos, char_size, char_id)) / (EPSILON * 2.0);
				grad.y = (msdf_char(p_plus_y, char_pos, char_size, char_id) - msdf_char(p_minus_y, char_pos, char_size, char_id)) / (EPSILON * 2.0);
			}
			case Fill: {
				if stack[1] < 0.0 {
					let color = vec4f(
						slots[0][0],
						slots[1][0],
						slots[2][0],
						slots[3][0],
					);
					let anti_aliasing = clamp(- stack[1] / EDGE_WIDTH, 0.0, 1.0);
					let new_color = vec4f(color.xyz, color.w * anti_aliasing);
					current_color = mix_color(current_color, new_color, current_blend_mode);
				}
			}
			case LinearGradient: {
				if stack[1] < 0.0 {
					let start_color = vec4f(
						slots[0][0],
						slots[1][0],
						slots[2][0],
						slots[3][0],
					);
					let end_color = vec4f(
						slots[0][1],
						slots[1][1],
						slots[2][1],
						slots[3][1],
					);
					let start_pos = vec2f(
						slots[0][2],
						slots[1][2],
					);
					let end_pos = vec2f(
						slots[2][2],
						slots[3][2],
					);
					let color = linear_gradient(p, start_pos, end_pos, start_color, end_color);
					let anti_aliasing = clamp(- stack[1] / EDGE_WIDTH, 0.0, 1.0);
					let new_color = vec4f(color.xyz, color.w * anti_aliasing);
					current_color = mix_color(current_color, new_color, current_blend_mode);
				}
			}
			case RadialGradient: {
				if stack[1] < 0.0 {
					let start_color = vec4f(
						slots[0][0],
						slots[1][0],
						slots[2][0],
						slots[3][0],
					);
					let end_color = vec4f(
						slots[0][1],
						slots[1][1],
						slots[2][1],
						slots[3][1],
					);
					let center = vec2f(
						slots[0][2],
						slots[1][2],
					);
					let radius = slots[2][2];
					let color = radial_gradient(p, center, radius, start_color, end_color);
					let anti_aliasing = clamp(- stack[1] / EDGE_WIDTH, 0.0, 1.0);
					let new_color = vec4f(color.xyz, color.w * anti_aliasing);
					current_color = mix_color(current_color, new_color, current_blend_mode);
				}
			}
			case TextureFill: {
				if stack[1] < 0.0 {
					let lt = vec2f(
						slots[0][0], 
						slots[1][0],
					);
					let rb = vec2f(
						slots[2][0], 
						slots[3][0],
					);
					let tlt = vec2f(
						slots[0][1], 
						slots[1][1],
					);
					let trb = vec2f(
						slots[2][1], 
						slots[3][1],
					);
					let texture_id = u32(slots[0][2]);
					let color = texture_fill(p, texture_id, lt, rb, tlt, trb);
					let anti_aliasing = clamp(- stack[1] / EDGE_WIDTH, 0.0, 1.0);
					let new_color = vec4f(color.xyz, color.w * anti_aliasing);
					current_color = mix_color(current_color, new_color, current_blend_mode);
				}
			}
			case SetTransform: {
				current_transform[0][0] = slots[0][0];
				current_transform[1][0] = slots[1][0];
				current_transform[2][0] = slots[2][0];
				current_transform[0][1] = slots[3][0];
				current_transform[1][1] = slots[0][1];
				current_transform[2][1] = slots[1][1];
			}
			case SetBlendMode: {
				current_blend_mode = u32(slots[0][0]);
			}
			case Load: {
				let stack_id = u32(slots[0][0]);
				temp = stack[stack_id];
			}
			default: {
				current_command_index += 1u;
				continue;
			}
		}

		if draw_commands[current_command_index].stroke_width >= 0.0 {
			temp = to_stroke(temp, draw_commands[current_command_index].stroke_width);
		}

		if length(grad) != 0.0 {
			temp = temp / length(grad);
		}

		if draw_commands[current_command_index].lhs >= 64u {
			current_command_index += 1u;
			continue;
		}

		let lhs = draw_commands[current_command_index].lhs;
		let op = draw_commands[current_command_index].operation;

		switch op {
			case OperationNone: {}
			case Replace: {
				stack[lhs] = temp;
			}
			case ReplaceWhenInside: {
				if temp < 0.0 {
					stack[lhs] = temp;
				}
			}
			case ReplaceWhenOutside: {
				if temp > 0.0 {
					stack[lhs] = temp;
				}
			}
			case And: {
				stack[lhs] = max(stack[lhs], temp);
			}
			case Or: {
				stack[lhs] = min(stack[lhs], temp);
			}
			case Xor: {
				stack[lhs] = stack[lhs] + temp - 2.0 * stack[lhs] * temp;
			}
			case Sub: {
				stack[lhs] = max(stack[lhs], -  temp);
			}
			case Neg: {
				stack[lhs] = - temp;
			}
			case Lerp: {
				stack[lhs] = mix(stack[lhs], temp, draw_commands[current_command_index].parameter);
			}
			case SmoothStep: {
				stack[lhs] = smoothstep(stack[lhs], temp, draw_commands[current_command_index].parameter);
			}
			case Sigmoid: {
				let t = 1.0 / (1.0 + exp(- draw_commands[current_command_index].parameter));
				stack[lhs] = mix(stack[lhs], temp, t);
			}
			default: {
				current_command_index += 1u;
				continue;
			}
		}

		current_command_index += 1u;
	}

	// let d = stack[0];

	// if d <= 0.0 {
	// 	current_color = vec4f(1.0, 1.0, 1.0, 1.0);
	// }

	// let anti_aliasing = clamp(- d / EDGE_WIDTH, 0.0, 1.0);

	return vec4f(
		pow(current_color.x, 2.2),
		pow(current_color.y, 2.2),
		pow(current_color.z, 2.2),
		current_color.w
	);
	// return current_color;
}