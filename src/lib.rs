#![doc = include_str!(".././doc.md")]

use std::{collections::HashMap, sync::{Arc, Mutex}};

use indexmap::IndexSet;
use layout::Layout;
use math::vec2::Vec2;
use prelude::FontId;
use render::{font::FontPool, texture::{Texture, TextureId}};
use widgets::{Signal, SignalWrapper};
use window::{event::OutputEvent, input_state::InputState};

pub mod layout;
pub mod render;
pub mod window;
pub mod widgets;
pub mod math;
pub mod prelude;

// TODO: Implement Context struct.
/// The context for Nablo UI.
/// 
/// Saves the layout, input state, and other data for the Nablo UI.
#[derive(Default)]
pub struct Context<S: Signal, A: App<Signal = S>> {
	/// The layout of the app.
	pub layout: Layout<S, A>,
	/// If true, the app will be redrawn every frame, even if there are no changes,
	/// and will redraw the entire screen instead of just the changed parts.
	pub force_redraw_per_frame: bool,
	/// The font pool for the app.
	/// 
	/// used to save and load fonts.
	pub fonts: Arc<Mutex<FontPool>>,
	textures: HashMap<TextureId, Texture>,
	available_texture_ids: IndexSet<TextureId>,
	input_state: InputState<S>,
	exit: bool,
	// pub(crate) painter_context: PainterCtx,
	// padding: Vec2,
}

impl<S: Signal, A: App<Signal = S>> Context<S, A> {
	/// Creates a new context with default values.
	pub fn new(font_data: Vec<u8>, index: u32) -> Self {
		let mut font_pool = FontPool::new();
		font_pool.insert_font(font_data, index);

		Self {
			input_state: InputState::new(),
			force_redraw_per_frame: false,
			textures: HashMap::new(),
			available_texture_ids: IndexSet::new(),
			layout: Layout::new(),
			exit: false,
			// padding: Vec2::same(EM),
			fonts: Arc::new(Mutex::new(font_pool)),
			// painter_context: PainterCtx::default(),
		}
	}

	/// Insert a font into the font pool.
	pub fn insert_font(&mut self, font_data: Vec<u8>, index: u32) -> FontId {
		self.fonts.lock().unwrap().insert_font(font_data, index)
	}

	/// Set the advance factor of the font pool.
	pub fn set_advance_factor(&mut self, index: FontId, factor: f32) {
		self.fonts.lock().unwrap().set_advance_factor(index, factor);
		self.layout.make_all_dirty();
	}

	/// Get a reference to the input state.
	pub fn input_state(&self) -> &InputState<S> {
		&self.input_state
	}

	/// Register a texture into the context.
	/// 
	/// Note: Do NOT call this method every frame, as it will cause a lot of unnecessary texture uploads.
	pub fn register_texture(&mut self, rgba: Vec<u8>, size: Vec2) -> TextureId {
		self.input_state.output_events.push(OutputEvent::RegisterTexture(size, rgba));
		let id =self.available_texture_ids.pop().unwrap_or(self.textures.len() as u32);
		self.textures.insert(id, Texture {
			texture_id: id,
			width: size.x as u32,
			height: size.y as u32,
			used_in_last_frame: false,
		});

		id
	}

	/// Update a texture in the context.
	/// 
	/// Note: Do NOT call this method every frame, as it will cause a lot of unnecessary texture uploads.
	/// 
	/// Returns true if the texture was updated, false otherwise.
	pub fn update_texture(&mut self, texture_id: TextureId, rgba: Vec<u8>, new_size: Vec2) -> bool {
		if let Some(texture) = self.textures.get_mut(&texture_id) {
			self.input_state.output_events.push(OutputEvent::UpdateTexture(texture_id, new_size, rgba));
			texture.width = new_size.x as u32;
			texture.height = new_size.y as u32;
			texture.used_in_last_frame = true;
			true
		} else {
			false
		}
	}
	
	/// Remove a texture from the context.
	pub fn remove_texture(&mut self, texture_id: TextureId) -> Option<Texture> {
		self.input_state.output_events.push(OutputEvent::RemoveTexture(texture_id));
		self.available_texture_ids.insert(texture_id);
		self.textures.remove(&texture_id)
	}

	/// Clear all textures from the context.
	pub fn clear_textures(&mut self) {
		self.input_state.output_events.push(OutputEvent::ClearTexture);
		self.textures.clear();
		self.available_texture_ids.clear();
	}

	/// Get a reference to the texture with the given id.
	pub fn get_texture(&self, texture_id: TextureId) -> Option<&Texture> {
		self.textures.get(&texture_id)
	}
}

/// The main trait for Nablo UI.
pub trait App: 'static + Sized {
	type Signal: Signal;

	/// Here you can setup your app with the given context. And add widgets to the layout.
	fn on_start(&mut self, ctx: &mut Context<Self::Signal, Self>);
	/// Here you can handle the given signal emitted by widgets.
	fn on_signal(&mut self, ctx: &mut Context<Self::Signal, Self>, signal: SignalWrapper<Self::Signal>);
	/// Here you can update your app every event loop frame.
	fn on_event_frame(&mut self, ctx: &mut Context<Self::Signal, Self>) {
		let _ = ctx;
	}
	/// Here you can hanlde your app every draw loop frame.
	fn on_draw_frame(&mut self, ctx: &mut Context<Self::Signal, Self>) {
		let _ = ctx;
	}
	/// Will be called when the os requests the app to exit. If you want to exit the app, return true.
	fn on_request_exit(&mut self, ctx: &mut Context<Self::Signal, Self>) -> bool { 
		let _ = ctx;
		true 
	}
	/// Here you can do some cleanup when the app exits.
	fn on_exit(&mut self, ctx: &mut Context<Self::Signal, Self>) {
		let _ = ctx;
	}
}