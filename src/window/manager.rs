//! A simple window manager for Nablo, based on winit.

use std::sync::Arc;

use arboard::Clipboard;
use time::{Duration, OffsetDateTime};
use winit::{application::ApplicationHandler, dpi::{PhysicalPosition, PhysicalSize, Position, Size}, event_loop::ActiveEventLoop, window::{self, Icon, Window}};

use crate::{layout::ROOT_LAYOUT_ID, math::{rect::Rect, vec2::Vec2}, render::{backend::{crate_wgpu_state, Uniform, WgpuState}, painter::Painter}, widgets::Signal, App, Context};

use super::event::{OutputEvent, Theme};

const STACK_SIZE: u32 = 64;

/// Settings for the window.
/// 
/// All the position and size values are in physical pixels ranther than logical pixels.
#[derive(Debug, Clone)]
pub struct WindowSettings {
	/// The title of the window.
	pub title: String,
	/// Allows the window to be resized.
	pub resizable: bool,
	/// The icon of the window.
	/// 
	/// The icon should be a tuple of the image data(rgba), width, and height.
	pub icon: Option<(Vec<u8>, u32, u32)>,
	/// The theme of the window.
	pub theme: Theme,
	/// The min size of the window.
	/// 
	/// If the min size is `None`, the window will have no minimum size.
	pub min_size: Option<Vec2>,
	/// The max size of the window.
	/// 
	/// If the max size is `None`, the window will have no maximum size.
	pub max_size: Option<Vec2>,
	/// The default size of the window.
	/// 
	/// If the default size is `None`, the window will be created with the default size of the system.
	pub default_size: Option<Vec2>,
	/// The position of the window.
	/// 
	/// If the position is `None`, the window will be centered on the screen.
	pub position: Option<Vec2>,
	/// The control flow of the event loop.
	pub control_flow: winit::event_loop::ControlFlow,
	/// The event frame per second of the window.
	/// 
	/// Set to zero to not limit the frame rate.
	/// 
	/// By default, the frame rate is set to 0.0.
	pub event_frame_rate: f32,
	/// The draw frame per second of the window.
	/// 
	/// Set to zero to not limit the frame rate.
	/// 
	/// By default, the frame rate is set to 0.0.
	pub draw_frame_rate: f32,
}

impl Default for WindowSettings {
	fn default() -> Self {
		Self {
			title: "Nablo UI".to_string(),
			resizable: true,
			icon: None,
			min_size: None,
			max_size: None,
			default_size: None,
			position: None,
			control_flow: winit::event_loop::ControlFlow::Poll,
			event_frame_rate: 0.0,
			draw_frame_rate: 0.0,
			theme: Theme::Dark,
		}
	}
}

/// A Simple window manager for Nablo UI.
#[allow(dead_code)]
pub struct Manager<'w, A, S: Signal> 
where A: App<S>,
{
	/// The settings of the window.
	pub window_settings: WindowSettings,
	/// The app to run.
	pub app: A,
	ctx: Context<S>,
	window: Option<(Arc<Window>, WgpuState<'w>)>,
	last_event_time: Duration,
	last_draw_time: Duration,
	clipboard: Option<Clipboard>,
}

impl<'w, A, S> ApplicationHandler for Manager<'w, A, S> 
where 
	A: App<S>,
	S: Signal + 'static,
{
	fn resumed(&mut self, event_loop: &ActiveEventLoop) {
		let mut attributes = Window::default_attributes();
		attributes.title = self.window_settings.title.clone();
		attributes.resizable = self.window_settings.resizable;
		if let Some((icon_data, width, height)) = &self.window_settings.icon {
			attributes.window_icon = Some(Icon::from_rgba(icon_data.clone(), *width, *height).expect("Failed to create icon"));
		}
		if let Some(min_size) = self.window_settings.min_size {
			attributes.min_inner_size = Some(Size::Physical(PhysicalSize::from([min_size.x as u32, min_size.y as u32])));
		}
		if let Some(max_size) = self.window_settings.max_size {
			attributes.max_inner_size = Some(Size::Physical(PhysicalSize::from([max_size.x as u32, max_size.y as u32])));
		}
		if let Some(default_size) = self.window_settings.default_size {
			attributes.inner_size = Some(Size::Physical(PhysicalSize::from([default_size.x as u32, default_size.y as u32])));
		}
		if let Some(position) = self.window_settings.position {
			attributes.position = Some(Position::Physical(PhysicalPosition::from([position.x as i32, position.y as i32])));
		}
		attributes.preferred_theme = Some(match &self.window_settings.theme {
			Theme::Dark => winit::window::Theme::Dark,
			Theme::Light => winit::window::Theme::Light,
		});
		let window = event_loop.create_window(attributes).expect("Failed to create window");
		window.set_ime_allowed(true);
		self.ctx.input_state.scale_factor = window.scale_factor();
		self.ctx.input_state.window_size = Vec2::new(window.inner_size().width as f32, window.inner_size().height as f32);
		self.app.on_start(&mut self.ctx);
		self.ctx.input_state.window_focused = true;
		let size = self.ctx.input_state.window_size;
		let window = Arc::new(window);
		let state = crate_wgpu_state(window.clone(), size);
		self.window = Some((window, state));
	}

	fn window_event(
		&mut self,
		event_loop: &ActiveEventLoop,
		_: window::WindowId,
		event: winit::event::WindowEvent,
	) {
		if self.window.is_none() {
			return;
		}

		if let winit::event::WindowEvent::Resized(size) = &event {
			self.ctx.input_state.window_size = Vec2::new(size.width as f32, size.height as f32);
			if let Some((window, state)) = &mut self.window {
				state.resized(self.ctx.input_state.window_size);
				self.ctx.input_state.scale_factor = window.scale_factor();
			}
		}

		// if let winit::event::WindowEvent::Focused(focused) = &event {
		// 	if let Some((window, state)) = &mut self.window {
		// 		if *focused {
		// 			state.resized(self.ctx.input_state.window_size);
		// 			self.ctx.input_state.scale_factor = window.scale_factor();
		// 		}else {
		// 			state.resized(Vec2::same(1.0));
		// 		}
		// 	}
		// }

		self.ctx.input_state.update(vec!(event.into()));
		#[allow(clippy::collapsible_if)]
		if self.ctx.input_state.should_close {
			if self.app.on_request_exit(&mut self.ctx) {
				event_loop.exit();
			}
		}

		let event_delta_time = OffsetDateTime::now_utc() - self.ctx.input_state.program_start_time;

		let should_handle_events = if self.window_settings.event_frame_rate == 0.0 {
			true
		}else {
			event_delta_time - self.last_event_time >= Duration::seconds_f32(1.0 / self.window_settings.event_frame_rate)
		};

		if should_handle_events {
			self.last_event_time = event_delta_time;
			self.ctx.layout.handle_events(ROOT_LAYOUT_ID, &mut self.ctx.input_state);
			let signals = self.ctx.input_state.signals_to_send.drain(..).collect::<Vec<_>>();
			for signal in signals {
				self.app.on_signal(&mut self.ctx, signal);
			}

			let events = if let Ok(mut events) = self.ctx.fonts.lock() {
				events.generate_textures()
			}else {
				panic!("Failed to lock font pool")
			};

			self.ctx.input_state.output_events.extend(events);

			self.ctx.input_state.prepare_for_next_frame();

			if self.ctx.input_state.all_dirty {
				self.ctx.input_state.all_dirty = false;
				self.ctx.layout.make_all_dirty();
			}

			if let Some((window, state)) = &mut self.window {
				let output_events = self.ctx.input_state.output_events.drain(..).collect::<Vec<_>>();
				
				if self.ctx.input_state.redraw_requested {
					window.request_redraw();
				}

				for event in output_events {
					match event {
						OutputEvent::SetWindowTitle(title) => {
							window.set_title(&title);
						},
						OutputEvent::Resize(size) => {
							window.request_inner_size(Size::Physical(PhysicalSize::from([size.x as u32, size.y as u32])))
								.expect("Failed to resize window");
						},
						OutputEvent::Move(position) => {
							window.set_outer_position(Position::Physical(PhysicalPosition::from([position.x as i32, position.y as i32])));
						},
						OutputEvent::SetCursorIcon(icon) => {
							window.set_cursor(icon);
						},
						OutputEvent::SetCursorPosition(position) => {
							window.set_cursor_position(Position::Physical(PhysicalPosition::from([position.x as i32, position.y as i32])))
								.expect("Failed to set cursor position");
						},
						OutputEvent::SetCursorVisible(visible) => {
							window.set_cursor_visible(visible);
						},
						OutputEvent::RegisterTexture(size, data) => {
							state.insert_texture(&data, size.x as u32, size.y as u32).expect("Failed to create texture");
						},
						OutputEvent::UpdateTexture(texture_id, size, data) => {
							state.update_texture(texture_id, &data,size.x as u32, size.y as u32).expect("Failed to update texture");
						},
						OutputEvent::RemoveTexture(texture_id) => {
							state.remove_texture(texture_id);
						},
						OutputEvent::ClearTexture => {
							state.clear_texture();
						},
						OutputEvent::AddChar(data, chr, font_id) => {
							state.add_char(font_id, chr, data);
						},
						OutputEvent::RemoveFont(font_id) => {
							state.remove_font(font_id);
						},
						OutputEvent::CopyToClipboard(text) => {
							if let Some(cb) = &mut self.clipboard {
								if let Err(e) = cb.set_text(text) {
									println!("Failed to set clipboard: {}", e);
								}
							}else {
								println!("WARN: Failed to create clipboard")
							}
						},
						OutputEvent::RequestClipboard => {
							if let Some(cb) = &mut self.clipboard {
								match cb.get_text() {
									Ok(text) => {
										self.ctx.input_state.paste_text(text);
									},
									Err(e) => {
										println!("Failed to get clipboard: {}", e);
									}
								}
							}else {
								println!("WARN: Failed to create clipboard")
							}
						},
					}
				}
			
				self.app.on_event_frame(&mut self.ctx);
			}
		}

		let draw_delta_time = OffsetDateTime::now_utc() - self.ctx.input_state.program_start_time;

		let should_draw = if self.window_settings.draw_frame_rate <= 0.0 {
			true
		}else {
			(draw_delta_time - self.last_draw_time) >= Duration::seconds_f32(1.0 / self.window_settings.draw_frame_rate)
		} && (self.ctx.input_state.redraw_requested || self.ctx.layout.any_widget_dirty() || self.ctx.force_redraw_per_frame);

		if should_draw {
			self.ctx.input_state.redraw_requested = false;
			let mut painter = Painter::new(self.ctx.fonts.clone(), self.ctx.input_state.window_size);
			painter.set_scale_factor(self.ctx.input_state.scale_factor as f32);
			
			if self.ctx.force_redraw_per_frame {
				self.ctx.layout.make_all_dirty();
			}
			
			self.app.on_draw_frame(&mut self.ctx);
			let refresh_area = self.ctx.layout.handle_draw(&mut painter);
			let refresh_area = if self.ctx.force_redraw_per_frame {
				Rect::WINDOW
			}else if let Some(area) = refresh_area {
				area
			}else {
				return;
			};
			if let Some((window, state)) =  &mut self.window {
				// painter.shapes.reverse();
				let (commands, stack_len) = painter.parse(
					&state.font_render,
					refresh_area
				);

				if stack_len >= STACK_SIZE {
					panic!("Gpu Stack overflows, max size is {} but current size is {}", STACK_SIZE, stack_len);
				}
				// println!("commands: {:#?}", commands);
				// panic!();
				let window_size = self.ctx.input_state.window_size();
				let mouse_pos = self.ctx.input_state.mouse_pos().unwrap_or(Vec2::INF);
				let time = (OffsetDateTime::now_utc() - self.ctx.input_state.program_start_time).as_seconds_f32();

				let uniform = Uniform {
					window_size: [
						window_size.x, 
						window_size.y
					],
					mouse: [
						mouse_pos.x, 
						mouse_pos.y
					],
					time,
					scale_factor: self.ctx.input_state.scale_factor as f32,
					command_len: commands.len() as u32,
					stack_len,
				};
				state.draw(
					refresh_area, 
					commands,
					// stack_len as u64,
					uniform, 
				);
				if self.ctx.force_redraw_per_frame {
					window.request_redraw();
				}
				state.cleanup();
			}
			self.ctx.input_state.redraw_requested = false;
			self.last_draw_time = draw_delta_time;
			// render::backend::render(painter.parse());
		}

		if self.ctx.exit {
			event_loop.exit();
		}
	}

	fn suspended(&mut self, _: &ActiveEventLoop) {
		self.window = None;
	}

	fn exiting(&mut self, _: &ActiveEventLoop) {
		self.app.on_exit(&mut self.ctx);
	}
}

impl<A, S: Signal + 'static> Manager<'_, A, S>
where A: App<S>,
{
	/// Creates a new manager with the given app.
	pub fn new(app: A, font_data: Vec<u8>, font_index: u32) -> Self {
		Self {
			app,
			ctx: Context::new(font_data, font_index),
			window: None,
			last_event_time: Duration::ZERO,
			last_draw_time: Duration::ZERO,
			window_settings: WindowSettings::default(),
			clipboard: match Clipboard::new() {
				Ok(clipboard) => Some(clipboard),
				Err(e) => {
					eprintln!("Failed to create clipboard: {}", e);
					None
				}
			},
		}
	}

	/// Sets the title of the window.
	pub fn title(self, title: impl Into<String>) -> Self {
		Self {
			window_settings: WindowSettings {
				title: title.into(),
				..self.window_settings
			},
			..self
		}
	}

	/// Sets whether the window can be resized.
	pub fn resizable(self, resizable: bool) -> Self {
		Self {
			window_settings: WindowSettings {
				resizable,
				..self.window_settings
			},
			..self
		}
	}

	/// Sets the icon of the window.
	pub fn icon(self, icon: Option<(Vec<u8>, u32, u32)>) -> Self {
		Self {
			window_settings: WindowSettings {
				icon,
				..self.window_settings
			},
			..self
		}
	}

	/// Sets the theme of the window.
	pub fn theme(self, theme: Theme) -> Self {
		Self {
			window_settings: WindowSettings {
				theme,
				..self.window_settings
			},
			..self
		}
	}

	/// Sets the min size of the window.
	pub fn min_size(self, min_size: Option<Vec2>) -> Self {
		Self {
			window_settings: WindowSettings {
				min_size,
				..self.window_settings
			},
			..self
		}
	}

	/// Sets the max size of the window.
	pub fn max_size(self, max_size: Option<Vec2>) -> Self {
		Self {
			window_settings: WindowSettings {
				max_size,
				..self.window_settings
			},
			..self
		}
	}

	/// Sets the default size of the window.
	pub fn default_size(self, default_size: Option<Vec2>) -> Self {
		Self {
			window_settings: WindowSettings {
				default_size,
				..self.window_settings
			},
			..self
		}
	}

	/// Sets the position of the window.
	pub fn position(self, position: Option<Vec2>) -> Self {
		Self {
			window_settings: WindowSettings {
				position,
				..self.window_settings
			},
			..self
		}
	}

	/// Sets the control flow of the event loop.
	pub fn control_flow(self, control_flow: winit::event_loop::ControlFlow) -> Self {
		Self {
			window_settings: WindowSettings {
				control_flow,
				..self.window_settings
			},
			..self
		}
	}

	/// Sets the event frame per second of the window.
	pub fn event_frame_rate(self, event_frame_rate: f32) -> Self {
		Self {
			window_settings: WindowSettings {
				event_frame_rate,
				..self.window_settings
			},
			..self
		}
	}

	/// Sets the draw frame per second of the window.
	pub fn draw_frame_rate(self, draw_frame_rate: f32) -> Self {
		Self {
			window_settings: WindowSettings {
				draw_frame_rate,
				..self.window_settings
			},
			..self
		}
	}

	/// Runs the manager.
	/// 
	/// # Panics
	/// 
	/// Panics if the window creation fails.
	pub fn run(&mut self) {
		let event_loop = winit::event_loop::EventLoop::new().expect("Failed to create event loop");
		event_loop.set_control_flow(self.window_settings.control_flow);

		let last_draw_time = OffsetDateTime::now_utc() - self.ctx.input_state.program_start_time;
		let last_event_time = OffsetDateTime::now_utc() - self.ctx.input_state.program_start_time;

		self.last_draw_time = last_draw_time;
		self.last_event_time = last_event_time;

		event_loop.run_app(self).expect("error while running app");
	}
}