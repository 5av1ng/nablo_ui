//! The wgpu renderer backend.
//! 
//! Currently, there's no public Api in this module as it's still a work in progress.
//! 
//! In the future, this module will be responsible for creating the shader-related objects.

// use super::commands::DrawCommandGpu;

use std::{collections::HashMap, sync::Arc};

use indexmap::IndexSet;
use wgpu::{util::DeviceExt, InstanceDescriptor};
use winit::window::Window;
use pollster::FutureExt as _;

use crate::math::{rect::Rect, vec2::Vec2};

use crate::prelude::BACKGROUND_COLOR;

use super::{commands::DrawCommandGpu, font::FontId, font_render::FontRender, texture::{create_new_texture_array, CreateTextureError, TextureId, TexturePool, DEFAULT_TEXTURE_LAYER, MAX_TEXTURE_SIZE}};

// const EMPTY_STACK_DATA: [u8; 16 * 64] = [0; 16 * 64];
const COMMAND_BUFFER_MUL_THERSHOLD: u64 = 2048;
// const CLEAR_THREASHOLD: f32 = 0.75;

pub(crate) struct UniformBuffer {
	pub uniform: wgpu::Buffer,
	pub bind_group: wgpu::BindGroup,
	pub layout: wgpu::BindGroupLayout,
}

pub(crate) struct StorageBuffer {
	pub buffer: wgpu::Buffer,
	pub bind_group: wgpu::BindGroup,
	pub layout: wgpu::BindGroupLayout,
	pub size: u64,
}

#[repr(C, align(16))]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct Uniform {
	pub window_size: [f32; 2],
	pub mouse: [f32; 2],
	pub time: f32,
	pub scale_factor: f32,
	pub stack_len: u32,
	pub command_len: u32,
}

pub(crate) struct WgpuState<'a> {
	pub surface: wgpu::Surface<'a>,
	pub device: wgpu::Device,
	pub queue: wgpu::Queue,
	pub size: Vec2,
	pub surface_config: wgpu::SurfaceConfiguration,
	pub size_changed: bool,
	pub shader: wgpu::ShaderModule,
	pub render_pipeline: wgpu::RenderPipeline,
	pub uniform_and_stack: UniformBuffer,
	// pub stack: StorageBuffer,
	pub commands: StorageBuffer,
	pub commands_2: StorageBuffer,
	pub is_using_commands_2: bool,
	pub texture_pool: TexturePool,
	pub font_render: FontRender,
	pub render_texture: wgpu::Texture,
	pub render_view: wgpu::TextureView,
	pub is_first_frame: bool,
}

// pub(crate) fn create_buffer_and_bind_group(
// 	device: &wgpu::Device, 
// 	data: &[u8], 
// 	buffer_label: &'static str,
// 	bindgroup_label: &'static str,
// 	usage: wgpu::BufferUsages,
// 	buffer_type: wgpu::BufferBindingType,
// ) -> (wgpu::Buffer, wgpu::BindGroupLayout, wgpu::BindGroup) {
// 	let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
// 		label: Some(buffer_label),
// 		contents: data,
// 		usage,
// 	});

// 	let (layout, bind_group) = create_bind_group_with_buffer(
// 		device,
// 		&buffer,
// 		bindgroup_label,
// 		buffer_type,
// 	);

// 	(buffer, layout, bind_group)
// }

pub(crate) fn create_bind_group_with_buffer(
	device: &wgpu::Device,
	buffer: &wgpu::Buffer,
	bindgroup_label: &'static str,
	buffer_type: wgpu::BufferBindingType,
) -> (wgpu::BindGroupLayout, wgpu::BindGroup) {
	let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
		entries: &[
			wgpu::BindGroupLayoutEntry {
				binding: 0,
				visibility: wgpu::ShaderStages::FRAGMENT,
				ty: wgpu::BindingType::Buffer {
					ty: buffer_type,
					has_dynamic_offset: false,
					min_binding_size: None,
				},
				count: None,
			},
		],
		label: Some(&(bindgroup_label.to_owned() + "Layout")),
	});

	let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
		layout: &bind_group_layout,
		entries: &[
			wgpu::BindGroupEntry {
				binding: 0,
				resource: buffer.as_entire_binding(),
			},
		],
		label: Some(bindgroup_label),
	});

	(bind_group_layout, bind_group)
}

pub(crate) fn crate_wgpu_state<'a>(window: Arc<Window>, size: Vec2) -> WgpuState<'a> {
	let instance = wgpu::Instance::new(&InstanceDescriptor {
		backends: wgpu::Backends::PRIMARY,
		..Default::default()
	});

	// let window = window.clone();

	let surface = instance.create_surface(window.clone()).expect("Failed to create surface");

	let adapter = instance
		.request_adapter(&wgpu::RequestAdapterOptions {
			power_preference: wgpu::PowerPreference::default(),
			compatible_surface: Some(&surface),
			force_fallback_adapter: false,
		}).block_on()
		.expect("Failed to find an appropriate adapter");

	let (device, queue) = adapter.request_device(&wgpu::DeviceDescriptor {
		required_features: wgpu::Features::empty(),
		required_limits: if cfg!(target_arch = "wasm32") {
			wgpu::Limits::downlevel_webgl2_defaults()
		}else {
			wgpu::Limits::default()
		},
		label: None,
		memory_hints: wgpu::MemoryHints::Performance,
	}, None).block_on().expect("Failed to create device and queue");

	let caps = surface.get_capabilities(&adapter);
	let config = wgpu::SurfaceConfiguration {
		usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_DST,
		format: caps.formats[0],
		width: size.x as u32,
		height: size.y as u32,
		present_mode: wgpu::PresentMode::Fifo,
		alpha_mode: caps.alpha_modes[0],
		view_formats: vec![],
		desired_maximum_frame_latency: 2,
	};

	surface.configure(&device, &config);

	let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
		label: None,
		source: wgpu::ShaderSource::Wgsl(include_str!("./shader.wgsl").into()),
	});

	let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
		label: Some("Uniform Buffer"),
		contents: bytemuck::bytes_of(&Uniform {
			window_size: [size.x, size.y],
			time: 0.0,
			mouse: [0.0, 0.0],
			scale_factor: 1.0,
			stack_len: 0,
			command_len: 0,
		}),
		usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
	});

	// let stack_buffer = device.create_buffer(&wgpu::BufferDescriptor {
	// 	label: Some("Stack Buffer"),
	// 	size: 16 * 64,
	// 	usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::STORAGE,
	// 	mapped_at_creation: false,
	// });

	// queue.write_buffer(&stack_buffer, 0, &[0; 16 * 64]);

	let uniform_and_stack_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
		entries: &[
			wgpu::BindGroupLayoutEntry {
				binding: 0,
				visibility: wgpu::ShaderStages::FRAGMENT,
				ty: wgpu::BindingType::Buffer {
					ty: wgpu::BufferBindingType::Uniform,
					has_dynamic_offset: false,
					min_binding_size: None,
				},
				count: None,
			},
			// wgpu::BindGroupLayoutEntry {
			// 	binding: 1,
			// 	visibility: wgpu::ShaderStages::FRAGMENT,
			// 	ty: wgpu::BindingType::Buffer {
			// 		ty: wgpu::BufferBindingType::Storage { read_only: false },
			// 		has_dynamic_offset: false,
			// 		min_binding_size: None,
			// 	},
			// 	count: None,
			// },
		],
		label: Some("Uniform And Stack Bind Group Layout"),
	});

	let uniform_and_stack_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
		layout: &uniform_and_stack_layout,
		entries: &[
			wgpu::BindGroupEntry {
				binding: 0,
				resource: uniform_buffer.as_entire_binding(),
			},
			// wgpu::BindGroupEntry {
			// 	binding: 1,
			// 	resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
			// 		buffer: &stack_buffer,
			// 		offset: 0,
			// 		size: None,
			// 	}),
			// },
		],
		label: Some("Uniform And Stack Bind Group"),
	});

	let uniform_and_stack = UniformBuffer {
		uniform: uniform_buffer,
		// stack: stack_buffer,
		// stack_size: 16 * 64,
		bind_group: uniform_and_stack_group,
		layout: uniform_and_stack_layout,
	};

	let commands_buffer = device.create_buffer(&wgpu::BufferDescriptor {
		label: Some("Commands Buffer"),
		size: 1024 * std::mem::size_of::<DrawCommandGpu>() as u64,
		usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::STORAGE,
		mapped_at_creation: false,
	});

	let commands_2_buffer = device.create_buffer(&wgpu::BufferDescriptor {
		label: Some("Commands 2 Buffer"),
		size: 1024 * std::mem::size_of::<DrawCommandGpu>() as u64,
		usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::STORAGE,
		mapped_at_creation: false,
	});

	queue.write_buffer(&commands_buffer, 0, &[0; 1024 * std::mem::size_of::<DrawCommandGpu>()]);
	queue.write_buffer(&commands_2_buffer, 0, &[0; 1024 * std::mem::size_of::<DrawCommandGpu>()]);
	queue.submit([]);

	let (commands_layout, commands_bind_group) = create_bind_group_with_buffer(
		&device,
		&commands_buffer,
		"Commands Bind Group",
		wgpu::BufferBindingType::Storage { read_only: true },
	);

	let (commands2_layout, commands2_bind_group) = create_bind_group_with_buffer(
		&device,
		&commands_2_buffer,
		"Commands Bind Group 2",
		wgpu::BufferBindingType::Storage { read_only: true },
	);


	let commands = StorageBuffer {
		buffer: commands_buffer,
		bind_group: commands_bind_group,
		size: 1024 * std::mem::size_of::<DrawCommandGpu>() as u64,
		layout: commands_layout,
	};

	let commands_2 = StorageBuffer {
		buffer: commands_2_buffer,
		bind_group: commands2_bind_group,
		size: 1024 * std::mem::size_of::<DrawCommandGpu>() as u64,
		layout: commands2_layout,
	};

	let wgpu_texture = create_new_texture_array(
		&device, 
		0, 
		DEFAULT_TEXTURE_LAYER, 
		MAX_TEXTURE_SIZE[0], 
		MAX_TEXTURE_SIZE[1],
		"Texture".to_string(),
	).expect("Failed to create texture array");

	let texture_pool = TexturePool {
		textures: HashMap::new(),
		available_texture_ids: IndexSet::new(),
		texture_array: vec![wgpu_texture],
	};

	let font_render = FontRender::new(&device).expect("Failed to create font render");

	let render_pipeline = create_render_pipeline(
		&device, 
		&shader, 
		&config, 
		&[
			&uniform_and_stack.layout, 
			&commands.layout, 
			&texture_pool.texture_array[0].layout,
			&font_render.bind_group_layout,
		]
	);

	let render_texture = device.create_texture(&wgpu::TextureDescriptor {
		label: Some("Render Texture"),
		size: wgpu::Extent3d {
			width: size.x as u32,
			height: size.y as u32,
			depth_or_array_layers: 1,
		},
		mip_level_count: 1,
		sample_count: 1,
		dimension: wgpu::TextureDimension::D2,
		format: config.format,
		usage: wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
		view_formats: &[],
	});

	let render_view = render_texture.create_view(&wgpu::TextureViewDescriptor {
		label: Some("Render View"),
		..Default::default()
	});

	WgpuState {
		surface,
		device,
		queue,
		size,
		surface_config: config,
		size_changed: false,
		shader,
		render_pipeline,
		uniform_and_stack,
		texture_pool,
		commands,
		font_render,
		render_texture,
		render_view,
		is_first_frame: true,
		is_using_commands_2: false,
		commands_2,
	}
}

fn create_render_pipeline(
	device: &wgpu::Device, 
	shader: &wgpu::ShaderModule,
	config: &wgpu::SurfaceConfiguration,
	bind_group_layouts: &[&wgpu::BindGroupLayout],
) -> wgpu::RenderPipeline {
	let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
		label: Some("Render Pipeline Layout"),
		bind_group_layouts,
		push_constant_ranges: &[],
	});
	
	device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
		label: Some("Render Pipeline"),
		layout: Some(&render_pipeline_layout),
		vertex: wgpu::VertexState {
			module: shader,
			compilation_options: Default::default(),
			entry_point: Some("vs_main"),
			buffers: &[],
		},
		fragment: Some(wgpu::FragmentState {
			module: shader,
			compilation_options: Default::default(),
			entry_point: Some("fs_main"),
			targets: &[Some(wgpu::ColorTargetState {
				format: config.format,
				blend: Some(wgpu::BlendState::ALPHA_BLENDING),
				write_mask: wgpu::ColorWrites::ALL,
			})],
		}),
		primitive: wgpu::PrimitiveState {
			topology: wgpu::PrimitiveTopology::TriangleList,
			strip_index_format: None,
			front_face: wgpu::FrontFace::Cw,
			cull_mode: Some(wgpu::Face::Back),
			polygon_mode: wgpu::PolygonMode::Fill,
			unclipped_depth: false,
			conservative: false,
		},
		depth_stencil: None,
		multisample: wgpu::MultisampleState {
			count: 1,
			mask: !0,
			alpha_to_coverage_enabled: false,
		},
		multiview: None,
		cache: None,
	})
}

impl WgpuState<'_> {
	pub fn insert_texture(&mut self, rgba: &[u8], width: u32, height: u32) -> Result<TextureId, CreateTextureError> {
		let (id, changed) = self.texture_pool.insert_texture(&self.device, &self.queue, rgba, width, height)?;

		if changed {
			self.update_render_pipeline();
		}

		Ok(id)
	}

	pub fn remove_texture(&mut self, texture_id: TextureId) {
		self.texture_pool.remove_texture(texture_id);
	}

	pub fn update_texture(&mut self, texture_id: TextureId, rgba: &[u8], width: u32, height: u32) -> Result<(), CreateTextureError> {
		self.texture_pool.update_texture(&self.device, &self.queue, texture_id, rgba, width, height)
	}

	pub fn clear_texture(&mut self) {
		self.texture_pool.clear()
	}

	pub fn resized(&mut self, new_size: Vec2) {
		if self.size != new_size {
			self.size = new_size;
			self.size_changed = true;
		}
	}

	fn update_render_pipeline(&mut self) {
		self.render_pipeline = create_render_pipeline(
			&self.device, 
			&self.shader, 
			&self.surface_config, 
			&[
				&self.uniform_and_stack.layout, 
				&self.commands.layout,
				&self.commands_2.layout,  
				&self.texture_pool.texture_array[0].layout,
				&self.font_render.bind_group_layout,
			]
		);
	}

	fn refresh_command_buffer(&mut self, new_size: u64) {
		let new_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
			label: Some("Commands Buffer"),
			size: new_size,
			usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::STORAGE,
			mapped_at_creation: false,
		});

		let new_buffer_2 = self.device.create_buffer(&wgpu::BufferDescriptor {
			label: Some("Commands 2 Buffer"),
			size: new_size,
			usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::STORAGE,
			mapped_at_creation: false,
		});

		let (layout, bind_group) = create_bind_group_with_buffer(
			&self.device,
			&new_buffer,
			"Commands Bind Group",
			wgpu::BufferBindingType::Storage { read_only: true },
		);

		let (layout_2, bind_group_2) = create_bind_group_with_buffer(
			&self.device,
			&new_buffer,
			"Commands 2 Bind Group",
			wgpu::BufferBindingType::Storage { read_only: true },
		);
		

		self.commands.buffer.destroy();
		self.commands.buffer = new_buffer;
		self.commands.bind_group = bind_group;
		self.commands.size = new_size;
		self.commands.layout = layout;

		self.commands_2.buffer.destroy();
		self.commands_2.buffer = new_buffer_2;
		self.commands_2.bind_group = bind_group_2;
		self.commands_2.size = new_size;
		self.commands_2.layout = layout_2;

		self.update_render_pipeline();
	}

	fn resize(&mut self) -> bool {
		if self.size.x == 0.0 || self.size.y == 0.0 {
			return false;
		}

		if self.size_changed {
			self.surface_config.width = self.size.x as u32;
			self.surface_config.height = self.size.y as u32;
			self.surface.configure(&self.device, &self.surface_config);
			self.recreate_render_texture();
			self.size_changed = false;
		}

		true
	}

	fn recreate_render_texture(&mut self) {
		self.render_texture.destroy();

		self.render_texture = self.device.create_texture(&wgpu::TextureDescriptor {
			label: Some("Render Texture"),
			size: wgpu::Extent3d {
				width: self.size.x as u32,
				height: self.size.y as u32,
				depth_or_array_layers: 1,
			},
			mip_level_count: 1,
			sample_count: 1,
			dimension: wgpu::TextureDimension::D2,
			format: self.surface_config.format,
			usage: wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
			view_formats: &[],
		});

		self.render_view = self.render_texture.create_view(&wgpu::TextureViewDescriptor {
			label: Some("Render View"),
			..Default::default()
		});

		self.is_first_frame = true;
	}

	pub fn draw(&mut self, 
		mut render_area: Rect,
		commands: Vec<DrawCommandGpu>,
		// expected_stack_size: u64,
		uniform: Uniform,
	) {
		if !self.resize() {
			return;
		}
		if (commands.len() * std::mem::size_of::<DrawCommandGpu>()) as u64 > self.commands.size {
			self.refresh_command_buffer( 
				if self.commands.size * 2 <= COMMAND_BUFFER_MUL_THERSHOLD * std::mem::size_of::<DrawCommandGpu>() as u64 {
					self.commands.size * 2
				}else {
					(commands.len() * std::mem::size_of::<DrawCommandGpu>()) as u64
				}
			);
		}

		
		// if expected_stack_size > self.uniform_and_stack.stack_size {
		// 	self.refresh_stack_buffer(self.uniform_and_stack.stack_size * 2);
		// }
		
		self.queue.write_buffer(&self.uniform_and_stack.uniform, 0, bytemuck::bytes_of(&uniform));
		// self.queue.write_buffer(&self.uniform_and_stack.stack, 0, &EMPTY_STACK_DATA);
		if self.is_using_commands_2 {
			self.queue.write_buffer(&self.commands.buffer, 0, bytemuck::cast_slice(&commands));
		}else {
			self.queue.write_buffer(&self.commands_2.buffer, 0, bytemuck::cast_slice(&commands));
		}
		self.queue.submit([]);
			
		render_area = Rect::from_lt_size(render_area.lt() * uniform.scale_factor, render_area.size() * uniform.scale_factor);
		render_area &= Rect::new(0.0, 0.0, self.size.x, self.size.y);
		if render_area.is_empty() {
			return;
		}
			
		let output = self.surface.get_current_texture().expect("Failed to acquire next texture view");
		// let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

		let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
			label: Some("Main Render Encoder"),
		});

		let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
			label: Some("Main Render Pass"),
			color_attachments: &[Some(wgpu::RenderPassColorAttachment {
				view: &self.render_view,
				resolve_target: None,
				ops: wgpu::Operations {
					load: if self.is_first_frame {
						wgpu::LoadOp::Clear(wgpu::Color { 
							r: BACKGROUND_COLOR.r.powf(2.2) as f64, 
							g: BACKGROUND_COLOR.g.powf(2.2) as f64, 
							b: BACKGROUND_COLOR.b.powf(2.2) as f64, 
							a: BACKGROUND_COLOR.a as f64
						})
					}else {
						wgpu::LoadOp::Load
					},
					store: wgpu::StoreOp::Store,
				},
			})],
			depth_stencil_attachment: None,
			..Default::default()
		});

		render_area = if self.is_first_frame {
			self.is_first_frame = false;
			Rect::new(0.0, 0.0, self.size.x, self.size.y)
		}else {
			render_area
		};

		render_pass.set_scissor_rect(render_area.x as u32, render_area.y as u32, render_area.w as u32, render_area.h as u32);
		render_pass.set_pipeline(&self.render_pipeline);
		render_pass.set_bind_group(0, &self.uniform_and_stack.bind_group, &[]);
		if self.is_using_commands_2 {
			render_pass.set_bind_group(1, &self.commands_2.bind_group, &[]);
			self.is_using_commands_2 = false;
		}else {
			render_pass.set_bind_group(1, &self.commands.bind_group, &[]);
			self.is_using_commands_2 = true;
		}
		// render_pass.set_bind_group(2, &self.stack.bind_group, &[]);
		render_pass.set_bind_group(2, &self.texture_pool.texture_array[0].bind_group, &[]);
		render_pass.set_bind_group(3, &self.font_render.bind_group, &[]);
		// render_pass.set_viewport(0.0, 0.0, self.size.x, self.size.y, 0.0, 1.0);
		render_pass.draw(0..6, 0..1);

		drop(render_pass);

		encoder.copy_texture_to_texture(
			wgpu::TexelCopyTextureInfo {
				texture: &self.render_texture,
				mip_level: 0,
				origin: wgpu::Origin3d::ZERO,
				aspect: wgpu::TextureAspect::All,
			},
			wgpu::TexelCopyTextureInfo {
				texture: &output.texture,
				mip_level: 0,
				origin: wgpu::Origin3d::ZERO,
				aspect: wgpu::TextureAspect::All,
			},
			wgpu::Extent3d {
				width: self.size.x as u32,
				height: self.size.y as u32,
				depth_or_array_layers: 1,
			},
		);

		self.queue.submit(std::iter::once(encoder.finish()));
		output.present();
	} 

	pub fn cleanup(&mut self) {
		self.texture_pool.cleanup();
	}

	pub fn remove_font(&mut self, font_id: FontId) {
		self.font_render.remove_font(font_id);
	}

	pub fn add_char(&mut self, font_id: FontId, chr: char, char_data: Vec<u8>) {
		self.font_render.add_char(&self.device, &self.queue, font_id, chr, char_data).expect("Failed to add char");
	}
}