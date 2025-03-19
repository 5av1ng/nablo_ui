struct Uniforms {
	window_size: vec2<f32>,
}

@group(0) @binding(0) var texture_sampler: sampler;
@group(0) @binding(1) var rendered_texture: texture_2d<f32>;
@group(0) @binding(2) var<uniform> uniforms: Uniforms;

@vertex
fn vs_main(
	@builtin(vertex_index) in_vertex_index: u32,
) -> @builtin(position) vec4<f32> {
	let pos = vec2<f32>(
		(vec2(1u, 2u) + in_vertex_index) % 6u < vec2(3u, 3u)
	) * 2.0 - 1.0;
	return vec4f(pos, 0.0, 1.0);
}

@fragment
fn fs_main(@builtin(position) clip_pos: vec4<f32>) -> @location(0) vec4f {
	let pos = clip_pos.xy;
	let uv = pos / uniforms.window_size;
	return textureSample(rendered_texture, texture_sampler, uv);
}