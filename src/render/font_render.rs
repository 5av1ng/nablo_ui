use std::collections::{HashMap, HashSet};

use indexmap::IndexSet;
use wgpu::util::DeviceExt;

use super::{font::{FontId, CHAR_TEXTURE_SIZE, FONT_TEXTURE_SIZE}, texture::{create_new_texture_array, CreateTextureError}};

const DEFAULT_FONT_LAYERS: u32 = 4;

pub(crate) struct FontRender {
	pub texture: wgpu::Texture,
	pub bind_group: wgpu::BindGroup,
	pub bind_group_layout: wgpu::BindGroupLayout,
	pub char_texture_map: HashMap<(char, FontId), u32>,
	pub empty_positions: IndexSet<u32>,
	pub layers: u32
}

impl FontRender {
	pub fn new(device: &wgpu::Device) -> Result<Self, CreateTextureError>{
		let texture = create_new_texture_array(
			device,
			0,
			DEFAULT_FONT_LAYERS,
			FONT_TEXTURE_SIZE,
			FONT_TEXTURE_SIZE,
			"Font texture".to_string()
		)?;

		Ok(Self {
			texture: texture.texture,
			bind_group: texture.bind_group,
			bind_group_layout: texture.layout,
			char_texture_map: HashMap::new(),
			empty_positions: IndexSet::new(),
			layers: DEFAULT_FONT_LAYERS,
		})
	}

	pub fn extend_texture(
		&mut self, 
		device: &wgpu::Device, 
		queue: &wgpu::Queue,
	) -> Result<(), CreateTextureError> {
		let new_layer = self.layers + 1;
		let new_texture = create_new_texture_array(
			device,
			0,
			new_layer,
			FONT_TEXTURE_SIZE,
			FONT_TEXTURE_SIZE,
			"Font texture".to_string()
		)?;

		let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { 
			label: Some("Extend font texture encoder") 
		});

		encoder.copy_texture_to_texture(
			wgpu::TexelCopyTextureInfo {
				texture: &self.texture,
				mip_level: 0,
				origin: wgpu::Origin3d::ZERO,
				aspect: wgpu::TextureAspect::All,
			},
			wgpu::TexelCopyTextureInfo {
				texture: &new_texture.texture,
				mip_level: 0,
				origin: wgpu::Origin3d::ZERO,
				aspect: wgpu::TextureAspect::All,
			},
			wgpu::Extent3d {
				width: FONT_TEXTURE_SIZE,
				height: FONT_TEXTURE_SIZE,
				depth_or_array_layers: self.layers,
			},
		);

		queue.submit(std::iter::once(encoder.finish()));
		self.texture = new_texture.texture;
		self.bind_group = new_texture.bind_group;
		self.bind_group_layout = new_texture.layout;
		self.layers = new_layer;

		Ok(())
	}

	pub fn add_char(
		&mut self,
		device: &wgpu::Device,
		queue: &wgpu::Queue,
		font_id: FontId,
		chr: char,
		rgba: Vec<u8>, 
	) -> Result<bool, CreateTextureError> {
		let pos_id = self.empty_positions.pop().unwrap_or(self.char_texture_map.len() as u32);
		let module = FONT_TEXTURE_SIZE / CHAR_TEXTURE_SIZE;
		let layer = pos_id / (module * module);
		let pos = pos_id % (module * module);
		let x = pos % module * CHAR_TEXTURE_SIZE;
		let y = pos / module * CHAR_TEXTURE_SIZE;
		
		let updated = if layer >= self.layers {
			self.extend_texture(device, queue)?;
			true
		}else {
			false
		};

		let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
			label: Some(&format!("Char {} texture buffer", chr)),
			contents: bytemuck::cast_slice(&rgba),
			usage: wgpu::BufferUsages::COPY_SRC,
		});

		// let out_buffer = device.create_buffer(&wgpu::BufferDescriptor {
		// 	label: Some(&format!("Char {} output texture buffer", chr)),
		// 	size: (4 * FONT_TEXTURE_SIZE * FONT_TEXTURE_SIZE) as u64,
		// 	usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
		// 	mapped_at_creation: false,
		// });

		let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { 
			label: Some("Add char to font texture encoder") 
		});

		let texture_size = wgpu::Extent3d {
			width: CHAR_TEXTURE_SIZE,
			height: CHAR_TEXTURE_SIZE,
			depth_or_array_layers: layer + 1,
		};

		encoder.copy_buffer_to_texture(
			wgpu::TexelCopyBufferInfo {
				buffer: &buffer,
				layout: wgpu::TexelCopyBufferLayout {
					offset: 0,
					bytes_per_row: Some(4 * CHAR_TEXTURE_SIZE),
					rows_per_image: Some(CHAR_TEXTURE_SIZE),
				}
			},
			wgpu::TexelCopyTextureInfo {
				texture: &self.texture,
				mip_level: 0,
				origin: wgpu::Origin3d {
					x,
					y,
					z: 0,
				},
				aspect: wgpu::TextureAspect::All,
			},
			texture_size,
		);

		// encoder.copy_texture_to_buffer(
		// 	wgpu::TexelCopyTextureInfo {
		// 		texture: &self.texture,
		// 		mip_level: 0,
		// 		origin: wgpu::Origin3d {
		// 			x: 0,
		// 			y: 0,
		// 			z: layer,
		// 		},
		// 		aspect: wgpu::TextureAspect::All,
		// 	},
		// 	wgpu::TexelCopyBufferInfo {
		// 		buffer: &out_buffer,
		// 		layout: wgpu::TexelCopyBufferLayout {
		// 			offset: 0,
		// 			bytes_per_row: Some(4 * FONT_TEXTURE_SIZE),
		// 			rows_per_image: Some(FONT_TEXTURE_SIZE),
		// 		}
		// 	},
		// 	wgpu::Extent3d {
		// 		width: FONT_TEXTURE_SIZE,
		// 		height: FONT_TEXTURE_SIZE,
		// 		depth_or_array_layers: layer + 1,
		// 	},
		// );

		queue.submit(std::iter::once(encoder.finish()));

		// let out_buffer = std::sync::Arc::new(out_buffer);
		// let cap = out_buffer.clone();
		// let img = out_buffer.slice(..);
		// img.map_async(wgpu::MapMode::Read, move |_| {
		// 	let img = cap.slice(..).get_mapped_range().to_vec();
		// 	let img: image::ImageBuffer<image::Rgba<u8>, Vec<u8>> = image::ImageBuffer::from_raw(FONT_TEXTURE_SIZE, FONT_TEXTURE_SIZE, img).unwrap();
		// 	let img = image::DynamicImage::ImageRgba8(img);
		// 	img.save_with_format("./test.png", image::ImageFormat::Png).unwrap();
		// });

		self.char_texture_map.insert((chr, font_id), pos_id);

		Ok(updated)
	}

	pub fn remove_font(&mut self, font_id: FontId) {
		let mut to_remove = HashSet::new();
		for ((_, key), value) in self.char_texture_map.iter() {
			if *key == font_id {
				to_remove.insert(*value);
			}
		}
		for pos in to_remove {
			self.empty_positions.insert(pos);
		}
		self.char_texture_map.retain(|(_ ,key), _| *key != font_id);
	}
}