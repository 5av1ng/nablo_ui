//! A floating container widget that can be used as modal, message, tooltip, window, dropdown, etc.

use std::{cell::RefCell, collections::HashMap};

use indexmap::IndexMap;

use crate::{layout::{Layout, LayoutId}, prelude::{InputState, Painter, Rect, Vec2}, App};

use super::{Signal, SignalGenerator, Widget};

/// A floating container widget that can be used as modal, message, tooltip, window, dropdown, etc.
/// 
/// 
/// Better use with [`crate::prelude::Card`]
pub struct FloatingContainer<S: Signal, A: App<Signal = S>> {
	/// The inner properties of the floating container.
	pub inner: FloatingContainerInner,
	/// The signals of the floating container.
	pub signals: SignalGenerator<S, FloatingContainerInner, A>,
	current_pos: Option<Vec2>,
	content_size: Option<Vec2>,
	current_size: Option<Vec2>,
	parent_area: RefCell<Rect>,
	widget_pos: RefCell<Vec2>,
	parent_pos: RefCell<Vec2>,
}

/// The inner properties of the floating container.
pub struct FloatingContainerInner {
	/// The position of the floating container.
	pub position: FloatPostion,
	/// if show the content of the floating container.
	pub show: bool,
	/// if the floating container is draggable.
	pub draggable: bool,
	/// The size of the floating container.
	/// 
	/// If `None`, the size of the floating container will be the size of its content.
	pub size: Option<Vec2>,
	/// Whether the floating container is resizeable.
	/// 
	/// Contains the minimum size and maximum size of the floating container.
	/// 
	/// If `None`, the floating container is not resizeable.
	pub resizeable: Option<(Vec2, Vec2)>,
	/// The padding of the floating container.
	pub padding: Vec2,
}

/// The position of the floating container.
pub enum FloatPostion {
	/// The absolute position of the floating container.
	Absolote(Vec2),
	/// The relative position ralative to the parent widget of the floating container.
	Relative(Vec2),
	/// The relative position ralative to the given widget of the floating container.
	RelativeWidget(LayoutId, Vec2),
	/// The anchored position relative to the parent widget of the floating container.
	Anchored {
		anchor: Anchor,
		padding: Vec2,
	},
	/// The relative position ralative to the cursor position.
	RelativeCursor(Vec2)
}

/// The anchor of the floating container.
pub enum Anchor {
	TopLeft,
	TopCenter,
	TopRight,
	MiddleLeft,
	MiddleCenter,
	MiddleRight,
	BottomLeft,
	BottomCenter,
	BottomRight,
}

impl Default for FloatingContainerInner {
	fn default() -> Self {
		Self {
			position: FloatPostion::Relative(Vec2::ZERO),
			show: false,
			draggable: false,
			size: None,
			resizeable: None,
			padding: Vec2::ZERO,
		}
	}
}

impl<S: Signal, A: App<Signal = S>> Default for FloatingContainer<S, A> {
	fn default() -> Self {
		Self {
			inner: FloatingContainerInner::default(),
			signals: SignalGenerator::default(),
			current_pos: None,
			content_size: None,
			current_size: None,
			parent_area: RefCell::new(Rect::ZERO),
			widget_pos: RefCell::new(Vec2::ZERO),
			parent_pos: RefCell::new(Vec2::ZERO),
		}
	}
}

impl<S: Signal, A: App<Signal = S>> FloatingContainer<S, A> {
	/// Create a new floating container.
	pub fn new() -> Self {
		Self::default()
	}

	/// Set the position of the floating container.
	pub fn position(self, position: FloatPostion) -> Self {
		Self {
			inner: FloatingContainerInner { position, ..self.inner },
			..self
		}
	}

	/// Set if show the content of the floating container.
	pub fn show(self, show: bool) -> Self {
		Self {
			inner: FloatingContainerInner { show, ..self.inner },
			..self
		}
	}

	/// Set if the floating container is draggable.
	pub fn draggable(self, draggable: bool) -> Self {
		Self {
			inner: FloatingContainerInner { draggable, ..self.inner },
			..self
		}
	}

	/// Set the size of the floating container.
	/// 
	/// If `None`, the size of the floating container will be the size of its content.
	pub fn size(self, size: Option<Vec2>) -> Self {
		Self {
			inner: FloatingContainerInner { size, ..self.inner },
			..self
		}
	}

	/// Set whether the floating container is resizeable.
	/// 
	/// Contains the minimum size and maximum size of the floating container.
	/// 
	/// If `None`, the floating container is not resizeable.
	pub fn resizeable(self, resizeable: Option<(Vec2, Vec2)>) -> Self {
		Self {
			inner: FloatingContainerInner { resizeable, ..self.inner },
			..self
		}
	}

	/// Set the padding of the floating container.
	pub fn padding(self, padding: Vec2) -> Self {
		Self {
			inner: FloatingContainerInner { padding, ..self.inner },
			..self
		}
	}

	/// Reset the context of the floating container.
	pub fn reset_context(&mut self) {
		self.current_pos = None;
		self.content_size = None;
		self.current_size = None;
		self.parent_area.replace(Rect::ZERO);
	}
}

impl FloatPostion {
	fn get_pos(&self, 
		parent_area: Rect, 
		size: Vec2, 
		widget_pos: Vec2,
		cursor_pos: Vec2
	) -> Vec2 {
		match self {
			FloatPostion::Absolote(pos) => *pos,
			FloatPostion::Relative(pos) => parent_area.lt() + *pos,
			FloatPostion::RelativeWidget(_, pos) => {
				widget_pos + *pos
			},
			FloatPostion::RelativeCursor(pos) => cursor_pos + *pos,
			FloatPostion::Anchored { anchor, padding } => {
				let (x, y) = match anchor {
					Anchor::TopLeft => (
						parent_area.x, 
						parent_area.y
					),
					Anchor::TopCenter => (
						parent_area.x + parent_area.w / 2.0 - size.x / 2.0,
						parent_area.y 
					),
					Anchor::TopRight => (
						parent_area.x + parent_area.w - size.x,
						parent_area.y	
					),
					Anchor::MiddleLeft => (
						parent_area.x,
						parent_area.y + parent_area.h / 2.0 - size.y / 2.0
					),
					Anchor::MiddleCenter => (
						parent_area.x + parent_area.w / 2.0 - size.x / 2.0,
						parent_area.y + parent_area.h / 2.0 - size.y / 2.0
					),
					Anchor::MiddleRight => (
						parent_area.x + parent_area.w - size.x,
						parent_area.y + parent_area.h / 2.0 - size.y / 2.0
					),
					Anchor::BottomLeft => (
						parent_area.x,
						parent_area.y + parent_area.h - size.y
					),
					Anchor::BottomCenter => (
						parent_area.x + parent_area.w / 2.0 - size.x / 2.0,
						parent_area.y + parent_area.h - size.y
					),
					Anchor::BottomRight => (
						parent_area.x + parent_area.w - size.x,
						parent_area.y + parent_area.h - size.y
					),
				};
				Vec2::new(x, y) + *padding
			}
		}
	}
}

impl<S: Signal, A: App<Signal = S>> Widget for FloatingContainer<S, A> {
	type Signal = S;
	type Application = A;

	fn size(&self, id: LayoutId, painter: &Painter, layout: &Layout<Self::Signal, A>) -> Vec2 {
		*self.parent_area.borrow_mut() = if let Some(parent_id) = layout.get_parent_id(id) {
			layout.get_widget_area(parent_id).unwrap_or_default()
		}else {
			Rect::ZERO
		};
		
		if let FloatPostion::RelativeWidget(widget_id, _) = &self.inner.position {
			*self.widget_pos.borrow_mut() = layout.get_widget_pos(*widget_id).unwrap_or_default();
		}

		self.parent_pos.replace(painter.releative_to());

		Vec2::ZERO
	}

	fn inner_padding(&self) -> Vec2 {
		self.inner.padding
	}

	fn draw(&mut self, _: &mut Painter, _: Vec2) {}

	fn handle_event(&mut self, app: &mut A, input_state: &mut InputState<Self::Signal>, id: LayoutId, _: Rect, _: Vec2) -> bool {
		if !self.inner.show {
			return false;
		}

		let cursor_pos = input_state.touch_positions().first().cloned().unwrap_or(Vec2::INF);
		
		// println!("what");

		if self.current_pos.is_none() {
			self.current_pos = Some(
				self.inner.position.get_pos(*self.parent_area.borrow(), self.inner.size.unwrap_or(
					if let Some(size) = self.content_size {
						size
					}else {
						return false;
					}
				), *self.widget_pos.borrow(), cursor_pos)
			);
		}

		if self.current_size.is_none() || self.current_size.map(|f| f == Vec2::ZERO).unwrap_or_default() {
			self.current_size = Some(
				self.inner.size.unwrap_or(
					if let Some(size) = self.content_size {
						size
					}else {
						return false;
					}
				)
			);
		}

		let (current_pos, current_size) = if let (Some(pos), Some(size)) = (&mut self.current_pos, &mut self.current_size) {
			(pos, size)
		}else {
			return false;
		};

		let area = Rect::from_lt_size(*current_pos, *current_size);
		let draggable = self.inner.draggable || self.inner.resizeable.is_some();

		// println!("{}", area);

		let res = self.signals.generate_signals(
			app,
			&mut self.inner, 
			input_state, 
			id, 
			area, 
			false, 
			draggable
		);

		if let Some(delta) = res.drag_delta {
			// println!("{}", delta);
			let current_dragging = if let Some(inner) = self.signals.dragging_by() {
				inner
			}else {
				// actually unreachable
				return false;
			};
			if let Some((min, max)) = self.inner.resizeable {
				let touch = input_state.get_touch_pos(current_dragging).unwrap_or(Vec2::INF);
				if area.is_close_to_edge(touch, Vec2::same(16.0)) {
					*current_size += delta;
					*current_size = current_size.clamp_both(min, max);
				}else if self.inner.draggable {
					*current_pos += delta;
				}
				if delta != Vec2::ZERO {
					input_state.mark_all_dirty();
				}
			}else if self.inner.draggable {
				*current_pos += delta;
				if delta != Vec2::ZERO {
					input_state.mark_all_dirty();
				}
			}
		}


		false
	}

	fn handle_child_layout(&mut self, childs: IndexMap<LayoutId, Vec2>, _: Rect, id: LayoutId) -> HashMap<LayoutId, Option<Rect>> {
		if self.inner.show {
			let mut out = HashMap::new();
			out.insert(id, Rect::WINDOW);
			let mut current_y = self.inner.padding.y;
			let mut max_width = 0.0;
			for (id, child_size) in childs {
				let child_pos = Vec2::new(self.inner.padding.x, current_y);
				max_width = child_size.x.max(max_width);
				current_y += child_size.y + self.inner.padding.y;
				let rect = Rect::from_lt_size(child_pos, child_size);
				out.insert(id, rect);
			}
			self.content_size = Some(Vec2::new(max_width + self.inner.padding.x * 2.0, current_y));
			out.into_iter().map(|(k, v)| (k, Some(
				v.move_to(self.current_pos.unwrap_or_default())
				.move_by(- *self.parent_pos.borrow())
			))).collect()
		}else {
			HashMap::new()
		}
	}

	fn continuous_event_handling(&self) -> bool {
		self.inner.show
	}
}