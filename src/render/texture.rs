//! Here is the code for loading textures(images) from file and creating texture objects.

use std::collections::HashMap;

use indexmap::IndexSet;
use wgpu::util::DeviceExt;

/// A texture ID
pub type TextureId = u32; 

pub const MAX_TEXTURE_SIZE: [u32; 2] = [2560, 2560];
pub(crate) const DEFAULT_TEXTURE_LAYER: u32 = 4;
const TEXTURE_LAYER_MUL_THRESHOLD: u32 = 32;
const MAX_TEXTURE_LAYERS_PER_BUFFER: u32 = 256;
const MAX_TEXTURE_BUFFERS: u32 = 1;

#[derive(Debug, Clone, thiserror::Error)]
/// An error that occurs when creating a texture.
pub enum CreateTextureError {
	#[error("The image is too large to be loaded as a texture ({0}x{1}), maximum size is {2}x{3})")]
	TooLarge(u32, u32, u32, u32),
	#[error("Reached maximum number of texture buffers ({max})", max = MAX_TEXTURE_BUFFERS)]
	ReachedMaxLayers,
	#[error("updatig unexisting texture `{0}`")]
	UpdateUnexistingTexture(TextureId),
}

pub(crate) struct WgpuTexture {
	pub texture: wgpu::Texture,
	pub len: u32,
	pub bind_group: wgpu::BindGroup,
	pub layout: wgpu::BindGroupLayout,
	pub width: u32,
	pub height: u32,
}

/// A texture object that can be used to render a texture(image).
pub struct Texture {
	/// An unique identifier for the texture.
	pub texture_id: TextureId,
	/// The width of the texture.
	pub width: u32,
	/// The height of the texture.
	pub height: u32,
	pub(crate) used_in_last_frame: bool,
}

#[derive(Default)]
pub(crate) struct TexturePool {
	pub textures: HashMap<TextureId, Texture>,
	pub available_texture_ids: IndexSet<TextureId>,
	pub texture_array: Vec<WgpuTexture>,
}

pub(crate) fn create_new_texture_array(
	device: &wgpu::Device, 
	texture_page: usize, 
	layers: u32, 
	width: u32, 
	height: u32,
	label: String
) -> Result<WgpuTexture, CreateTextureError> {
	if texture_page >= MAX_TEXTURE_BUFFERS as usize {
		return Err(CreateTextureError::ReachedMaxLayers);
	}

	if width > MAX_TEXTURE_SIZE[0] || height > MAX_TEXTURE_SIZE[1] {
		return Err(CreateTextureError::TooLarge(width, height, MAX_TEXTURE_SIZE[0], MAX_TEXTURE_SIZE[1]));
	}

	let texture_size = wgpu::Extent3d {
		width,
		height,
		depth_or_array_layers: layers,
	};

	let texture = device.create_texture(&wgpu::TextureDescriptor {
		label: Some(&format!("{label} Page {}", texture_page)),
		size: texture_size,
		mip_level_count: 1,
		sample_count: 1,
		dimension: wgpu::TextureDimension::D2,
		format: wgpu::TextureFormat::Rgba8UnormSrgb,
		usage: wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_SRC,
		view_formats: &[],
	});

	let texture_view = texture.create_view(&wgpu::TextureViewDescriptor {
		label: Some(&format!("{label} View Page {}", texture_page)),
		..Default::default()
	});

	let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
		label: Some(&format!("{label} Sampler Page {}", texture_page)),
		address_mode_u: wgpu::AddressMode::ClampToEdge,
		address_mode_v: wgpu::AddressMode::ClampToEdge,
		address_mode_w: wgpu::AddressMode::ClampToEdge,
		mag_filter: wgpu::FilterMode::Linear,
		min_filter: wgpu::FilterMode::Linear,
		mipmap_filter: wgpu::FilterMode::Linear,
		// border_color: Some(wgpu::SamplerBorderColor::TransparentBlack),
		anisotropy_clamp: 64,
		..Default::default()
	});

	let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
		entries: &[
			wgpu::BindGroupLayoutEntry {
				binding: 1,
				visibility: wgpu::ShaderStages::FRAGMENT,
				ty: wgpu::BindingType::Texture {
					multisampled: false,
					view_dimension: wgpu::TextureViewDimension::D2Array,
					sample_type: wgpu::TextureSampleType::Float { filterable: true },
				},
				count: None,
			},
			wgpu::BindGroupLayoutEntry {
				binding: 0,
				visibility: wgpu::ShaderStages::FRAGMENT,
				ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
				count: None,
			},
		],
		label: Some(&format!("{label} Bind Group Layout {}", texture_page)),
	});

	let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
		layout: &bind_group_layout,
		entries: &[
			wgpu::BindGroupEntry {
				binding: 1,
				resource: wgpu::BindingResource::TextureView(&texture_view),
			},
			wgpu::BindGroupEntry {
				binding: 0,
				resource: wgpu::BindingResource::Sampler(&sampler),
			},
		],
		label: Some(&format!("{label} Bind Group {}", texture_page)),
	});

	let out = WgpuTexture {
		texture,
		len: layers,
		bind_group,
		width,
		height,
		layout: bind_group_layout,
	};
	
	Ok(out)
}

fn extend_texture_layer(
	texture_wgpu: &mut WgpuTexture, 
	device: &wgpu::Device,
	queue: &wgpu::Queue,
	new_size: u32
) -> Result<(), CreateTextureError> {
	let new_texture_wgpu = create_new_texture_array(
		device, 
		0,
		new_size, 
		texture_wgpu.width, 
		texture_wgpu.height,
		"Texture".to_string(),
	)?;

	let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
		label: Some("Extend Texture Layer"),
	});

	encoder.copy_texture_to_texture(
		wgpu::TexelCopyTextureInfo {
			texture: &texture_wgpu.texture,
			mip_level: 0,
			origin: wgpu::Origin3d::ZERO,
			aspect: wgpu::TextureAspect::All,
		},
		wgpu::TexelCopyTextureInfo {
			texture: &new_texture_wgpu.texture,
			mip_level: 0,
			origin: wgpu::Origin3d::ZERO,
			aspect: wgpu::TextureAspect::All,
		},
		wgpu::Extent3d {
			width: texture_wgpu.width,
			height: texture_wgpu.height,
			depth_or_array_layers: texture_wgpu.len,
		},
	);

	queue.submit(std::iter::once(encoder.finish()));

	texture_wgpu.len = new_size;
	texture_wgpu.texture = new_texture_wgpu.texture;
	texture_wgpu.bind_group = new_texture_wgpu.bind_group;

	Ok(())
}

impl TexturePool {
	pub(crate) fn remove_texture(&mut self, texture_id: TextureId) {
		if self.textures.remove(&texture_id).is_some() {
			self.available_texture_ids.insert(texture_id);
		}
	}

	pub(crate) fn clear(&mut self) {
		self.textures.clear();
		self.available_texture_ids.clear();
	}

	pub(crate) fn update_texture(
		&mut self, 
		device: &wgpu::Device, 
		queue: &wgpu::Queue,
		texture_id: TextureId, 
		rgba: &[u8], 
		width: u32, 
		height: u32
	) -> Result<(), CreateTextureError> {
		if !self.textures.contains_key(&texture_id) {
			return Err(CreateTextureError::UpdateUnexistingTexture(texture_id));
		}

		let array_index = texture_id / MAX_TEXTURE_LAYERS_PER_BUFFER;
		let layer_index = texture_id % MAX_TEXTURE_LAYERS_PER_BUFFER;

		let texture_wgpu = if let Some(texture_wgpu) = self.texture_array.get_mut(array_index as usize) {
			texture_wgpu
		}else {
			unreachable!("Texture array index out of range")
		};

		let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
			label: Some(&format!("Texture Buffer {}", texture_id)),
			contents: rgba,
			usage: wgpu::BufferUsages::COPY_SRC,
		});

		let texture_size = wgpu::Extent3d {
			width,
			height,
			depth_or_array_layers: layer_index + 1,
		};

		let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
			label: Some(&format!("Upload Texture {}", texture_id)),
		});

		encoder.copy_buffer_to_texture(
			wgpu::TexelCopyBufferInfo {
				buffer: &buffer,
				layout: wgpu::TexelCopyBufferLayout {
					offset: 0,
					bytes_per_row: Some((4 * width / 256 + 1) * 256),
					rows_per_image: Some(height),
				}
			},
			wgpu::TexelCopyTextureInfo {
				texture: &texture_wgpu.texture,
				mip_level: 0,
				origin: wgpu::Origin3d::ZERO,
				aspect: wgpu::TextureAspect::All,
			},
			texture_size,
		);

		queue.submit(std::iter::once(encoder.finish()));

		let texture = Texture {
			texture_id,
			width,
			height,
			used_in_last_frame: true,
		};

		self.textures.insert(texture_id, texture);

		Ok(())
	}

	pub(crate) fn insert_texture(
		&mut self, 
		device: &wgpu::Device, 
		queue: &wgpu::Queue,
		rgba: &[u8], 
		width: u32, 
		height: u32
	) -> Result<(TextureId, bool), CreateTextureError> {
		if width > MAX_TEXTURE_SIZE[0] || height > MAX_TEXTURE_SIZE[1] {
			return Err(CreateTextureError::TooLarge(width, height, MAX_TEXTURE_SIZE[0], MAX_TEXTURE_SIZE[1]));
		}
		let texture_id = self.available_texture_ids.pop().unwrap_or(self.textures.len() as u32);
		let array_index = texture_id / MAX_TEXTURE_LAYERS_PER_BUFFER;
		let layer_index = texture_id % MAX_TEXTURE_LAYERS_PER_BUFFER;
		let mut changed = false;

		let texture_wgpu = if let Some(texture_wgpu) = self.texture_array.get_mut(array_index as usize) {
			texture_wgpu
		}else {
			let new_texture_wgpu = create_new_texture_array(
				device, 
				array_index as usize,
				DEFAULT_TEXTURE_LAYER, 
				width, 
				height,
				"Texture".to_string()
			)?;
			changed = true;
			self.texture_array.push(new_texture_wgpu);
			self.texture_array.get_mut(array_index as usize).unwrap()
		};

		if texture_wgpu.width < width || texture_wgpu.height < height {
			return Err(CreateTextureError::TooLarge(width, height, texture_wgpu.width, texture_wgpu.height));
		}


		if layer_index >= texture_wgpu.len {
			let new_size = if texture_wgpu.len * 2 >= TEXTURE_LAYER_MUL_THRESHOLD { 
				texture_wgpu.len + TEXTURE_LAYER_MUL_THRESHOLD
			}else {
				texture_wgpu.len * 2
			};
			texture_wgpu.texture.destroy();
			extend_texture_layer(texture_wgpu, device, queue, new_size)?;
			changed = true;
		}

		let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
			label: Some(&format!("Texture Buffer {}", texture_id)),
			contents: rgba,
			usage: wgpu::BufferUsages::COPY_SRC,
		});

		let texture_size = wgpu::Extent3d {
			width,
			height,
			depth_or_array_layers: layer_index + 1,
		};

		let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
			label: Some(&format!("Upload Texture {}", texture_id)),
		});

		encoder.copy_buffer_to_texture(
			wgpu::TexelCopyBufferInfo {
				buffer: &buffer,
				layout: wgpu::TexelCopyBufferLayout {
					offset: 0,
					bytes_per_row: Some((4 * width / 256 + 1) * 256),
					rows_per_image: Some(height),
				}
			},
			wgpu::TexelCopyTextureInfo {
				texture: &texture_wgpu.texture,
				mip_level: 0,
				origin: wgpu::Origin3d::ZERO,
				aspect: wgpu::TextureAspect::All,
			},
			texture_size,
		);

		queue.submit(std::iter::once(encoder.finish()));

		let texture = Texture {
			texture_id,
			width,
			height,
			used_in_last_frame: true,
		};

		self.textures.insert(texture_id, texture);

		Ok((texture_id, changed))
	}

	pub(crate) fn cleanup(&mut self) {
		let mut avaiable_texture_ids = IndexSet::new();
		self.textures.retain(|id, texture| {
			if !texture.used_in_last_frame {
				avaiable_texture_ids.insert(*id);
			}
			
			texture.used_in_last_frame
		});
		for id in avaiable_texture_ids {
			self.available_texture_ids.insert(id);
		}
	}
}
