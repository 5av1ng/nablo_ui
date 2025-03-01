//! Defines text rendering related types and constants.

use std::collections::{HashMap, HashSet};

use rayon::prelude::*;

use image::DynamicImage;
use indexmap::IndexSet;
use mint::Vector2;
use msdf::{GlyphLoader, Projection, SDFTrait};
use owned_ttf_parser::{AsFaceRef, OwnedFace};

use crate::{math::vec2::Vec2, prelude::MAXIUM_CHAR_UPLOAD_PER_FRAME, window::event::OutputEvent};

/// The size of the font texture in pixels.
/// 
/// The font texture is a square texture with a size of 2048x2048 pixels.
pub const FONT_TEXTURE_SIZE: u32 = 2048;

/// The size of each character texture in pixels.
/// 
/// Each character texture is a square texture with a size of 64x64 pixels.
pub const CHAR_TEXTURE_SIZE: u32 = 64;

/// The base size nablo using for font rendering.
pub const EM: f32 = 16.0;

/// Maxium number of fonts that can be loaded.
pub const MAX_FONTS: usize = 16;

/// The font id type.
pub type FontId = u32;

/// The font pool, used to store and manage font textures and character textures.
pub struct FontPool {
	fonts: HashMap<FontId, Font>,
	removed_fonts: HashSet<FontId>,
	new_id: FontId,
}

impl Default for FontPool {
	fn default() -> Self {
		Self::new()
	}
}

impl FontPool{
	/// Creates a new font pool.
	pub fn new() -> Self {
		Self {
			fonts: HashMap::new(),
			removed_fonts: HashSet::new(),
			new_id: 0,
		}
	}

	/// Inserts a new font into the pool and returns its id.
	pub fn insert_font(&mut self, font_data: Vec<u8>, index: u32) -> FontId {
		if MAX_FONTS <= self.fonts.len() {
			panic!("Too many fonts loaded");
		}

		let font_id = self.new_id;
		self.new_id += 1;

		let font = Font::new(font_data, index);

		self.fonts.insert(font_id, font);

		font_id
	}

	/// Removes a font from the pool.
	/// 
	/// Returns `true` if the font was removed, `false` otherwise.
	pub fn remove_font(&mut self, font_id: FontId) -> bool {
		if self.fonts.remove(&font_id).is_some() {
			self.removed_fonts.insert(font_id);
			true
		}else {
			false
		}
	}

	/// Clear the font pool.
	pub fn clear(&mut self) {
		self.fonts.clear();
		self.new_id = 0;
	}

	/// Returns the line height of the font with the given id.
	/// 
	/// Will use [`EM`] as font size. To use a different size, use [`Self::line_height_with_size`].
	pub fn line_height(&self, font_id: FontId) -> Option<f32> {
		self.line_height_with_size(font_id, EM)
	}

	/// Returns the line height of the font with the given id and size.
	pub fn line_height_with_size(&self, font_id: FontId, size: f32) -> Option<f32> {
		self.fonts.get(&font_id).map(|font| font.line_height * size / EM)
	}

	/// Returns the anscender of the font with the given id.
	/// 
	/// Will use [`EM`] as font size. To use a different size, use [`Self::anscender_with_size`].
	pub fn anscender(&self, font_id: FontId) -> Option<f32> {
		self.anscender_with_size(font_id, EM)
	}

	/// Returns the anscender of the font with the given id and size.
	pub fn anscender_with_size(&self, font_id: FontId, size: f32) -> Option<f32> {
		self.fonts.get(&font_id).map(|font| font.anscender * size / EM)
	}

	/// Gets the glyph for the given character and font id.
	pub fn get_glyph(&mut self, font_id: FontId, chr: char) -> Option<Glyph> {
		if let Some(font) = self.fonts.get_mut(&font_id) {
			font.get_glyph(chr)
		}else {
			None
		}
	}

	/// Sets the advance factor for the font with the given id.
	pub fn set_advance_factor(&mut self, id: FontId, factor: f32) {
		if let Some(font) = self.fonts.get_mut(&id) {
			font.advance_factor = factor;
		}
	}

	/// Gets the advance factor for the font with the given id.
	pub fn advance_factor(&self, id: FontId) -> Option<f32> {
		self.fonts.get(&id).map(|font| font.advance_factor)
	}

	/// Caculates the size of the given text with the given font id and size.
	pub fn caculate_text_size(&mut self, font_id: FontId, text: impl Into<String>, font_size: f32, is_pointer: bool) -> Option<Vec2> {
		if let Some(font) = self.fonts.get_mut(&font_id) {
			// println!("found font");
			font.caculate_text_size(text.into(), font_size, is_pointer)
		}else {
			None
		}
	}

	pub(crate) fn generate_textures(&mut self) -> Vec<OutputEvent> {
		let mut out = vec!();
		for (id, font) in self.fonts.iter_mut() {
			out.extend(font.generate_textures(*id));
		}
		for id in self.removed_fonts.drain() {
			out.push(OutputEvent::RemoveFont(id));
		}
		out
	}
}

/// A single character glyph.
#[derive(Debug, Clone)]
pub struct Glyph {
	/// The character represented by this glyph.
	pub chr: char,
	/// The bearing of the character.
	pub bearing: Vec2,
	/// The advance of the character.
	pub advance: Vec2,
	/// The size of the character texture.
	pub size: Vec2,
}

pub(crate) struct Font {
	/// Contains the font data.
	pub face: OwnedFace,
	/// The characters that need to be added to the texture.
	pub to_add_to_texture: IndexSet<char>,
	pub char_map: HashMap<char, Glyph>,
	pub anscender: f32,
	/// warpped line height.
	pub line_height: f32,
	pub base_units_per_em: f32,
	pub advance_factor: f32,
}

impl Font {
	fn new(font_data: Vec<u8>, index: u32) -> Self {
		const ASCII: [char; 95] = [
			' ', '!', '"', '#', '$', '%', '&', '\'', '(', ')', '*', '+', ',', '-', '.', '/',
			'0', '1', '2', '3', '4', '5', '6', '7', '8', '9', ':', ';', '<', '=', '>', '?',
			'@', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O',
			'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z', '[', '\\', ']', '^', '_',
			'`', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l','m', 'n', 'o',
			'p', 'q', 'r','s', 't', 'u', 'v', 'w', 'x', 'y', 'z', '{', '|', '}', '~',
		];

		let face = OwnedFace::from_vec(font_data, index).unwrap();
		let face_ref = face.as_face_ref();
		let base_units_per_em = face_ref.units_per_em() as f32;
		let line_height = face_ref.height() as f32 * EM / base_units_per_em;
		let anscender = face_ref.ascender() as f32 * EM / base_units_per_em;
		let mut font = Self {
			face,
			char_map: HashMap::new(),
			to_add_to_texture: IndexSet::new(),
			anscender,
			line_height,
			base_units_per_em,
			advance_factor: 1.0
		};

		for chr in ASCII {
			font.get_glyph(chr);
		}

		font
	}
}

impl Font {
	fn get_glyph(&mut self, chr: char) -> Option<Glyph> {
		if let Some(chr) = self.char_map.get(&chr) {
			return Some(chr.clone());
		}

		let face = self.face.as_face_ref();
		let index = face.glyph_index(chr)?;
		self.base_units_per_em = face.units_per_em() as f32;
		let bearing_x = face.glyph_hor_side_bearing(index).unwrap_or(0) as f32 * EM / self.base_units_per_em;
		let bearing_y = face.glyph_ver_side_bearing(index).unwrap_or(0) as f32 * EM / self.base_units_per_em;
		let advance_x = face.glyph_hor_advance(index).unwrap_or(0) as f32 * EM / self.base_units_per_em;
		let advance_y = face.glyph_ver_advance(index).unwrap_or(0) as f32 * EM / self.base_units_per_em;
		let size = face.glyph_bounding_box(index).map(|inner| {
			let height = inner.height() as f32 * EM / self.base_units_per_em;
			let width = inner.width() as f32 * EM / self.base_units_per_em;
			Vec2::new(width, height)
		}).unwrap_or_default();
		let glyph = Glyph {
			chr,
			bearing: Vec2::new(bearing_x, bearing_y),
			advance: Vec2::new(advance_x, advance_y),
			size,
		};
		// println!("{:?}", glyph);
		self.char_map.insert(chr, glyph);
		self.to_add_to_texture.insert(chr);
		self.char_map.get(&chr).cloned()
	}
	
	fn caculate_text_size(&mut self, text: String, font_size: f32, is_pointer: bool) -> Option<Vec2> {
		let line_height = self.line_height;
		let mut size = Vec2::new(0.0, 0.0);
		let mut x: f32 = 0.0;
		// let mut max_line_height: f32 = 0.0;
		let len = text.chars().count();
		for (i, chr) in text.chars().enumerate() {
			if chr == '\n' {
				x = 0.0;
				// max_line_height = 0.0;
				size.y += line_height;
				size.x = x.max(size.x);
				// continue;
			}else {
				let glyph = self.get_glyph(chr)?;
				if i == len - 1 {
					if is_pointer {
						x += glyph.advance.x * self.advance_factor;
					}else {
						x += glyph.advance.x;
					}
				}else {
					x += glyph.advance.x * self.advance_factor;
				}
				// max_line_height = max_line_height.max(glyph.size.y + glyph.bearing.y);
			}
		}
		size.x = x.max(size.x);
		size.y += self.anscender;
		Some(size * font_size / EM)
	}

	pub(crate) fn generate_textures(&mut self, font_id: FontId) -> Vec<OutputEvent> {
		let face = self.face.as_face_ref();
		let len = self.to_add_to_texture.len();
		let chars = self.to_add_to_texture.drain(0..len.min(MAXIUM_CHAR_UPLOAD_PER_FRAME)).collect::<Vec<_>>();
		let factor = face.height() as f32 / self.base_units_per_em;
		let descender = face.descender() as f32;
		let proj = Projection {
			scale: Vector2 {
				x: 1.0 / (CHAR_TEXTURE_SIZE as f32 * factor / 4.0) as f64, 
				y: 1.0 / (CHAR_TEXTURE_SIZE as f32 * factor / 4.0) as f64,
			},
			translation: Vector2 {
				x: 0.0, 
				y: - descender as f64,
			},
		};
		chars.into_par_iter().filter_map(|chr| {
			// println!("generating texture for char: {}", chr);
			let index = face.glyph_index(chr)?;
			let shape = face.load_shape(index)?;

			let colored_shape = shape.color_edges_ink_trap(3.0);

			let msdf  = colored_shape.generate_msdf(
				CHAR_TEXTURE_SIZE, 
				CHAR_TEXTURE_SIZE, 
				1280.0, 
				&proj, 
				&Default::default()
			);

			let img = msdf.render_colored(CHAR_TEXTURE_SIZE, CHAR_TEXTURE_SIZE);
			let dynamic_image = DynamicImage::from(img).to_rgba8();
			// let _ = dynamic_image.save_with_format(format!("./{}.png", chr), image::ImageFormat::Png).unwrap();

			let data = dynamic_image.into_vec();

			Some(OutputEvent::AddChar(data, chr, font_id))
		}).collect::<Vec<_>>()
	}
}