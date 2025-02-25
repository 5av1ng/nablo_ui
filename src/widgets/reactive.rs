//! A struct that can be used to convert a static widget into a reactive widget.

use std::collections::HashMap;

use indexmap::IndexMap;

use crate::{layout::{Layout, LayoutId}, prelude::{InputState, Painter, Rect, Vec2}};

use super::{Signal, Widget};

/// A struct that can be used to convert a static widget into a reactive widget.
pub struct Reactive<W, S: Signal> 
where 
	W: Widget<Signal = S>,
{
	/// The original static widget.
	widget: Option<W>,
	/// The function that used to update the display element of the widget.
	#[allow(clippy::type_complexity)]
	pub on_update: Box<dyn Fn(W) -> W>,
}

impl <W, S> Reactive<W, S>
where 
	W: Widget<Signal = S>,
	S: Signal,
{
	/// Creates a new reactive widget.
	pub fn new(widget: W, on_update: impl Fn(W) -> W + 'static) -> Self {
		Self { widget: Some(widget), on_update: Box::new(on_update) }
	}

	/// Returns a reference to the original static widget.
	pub fn get_widget(&self) -> &W {
		self.widget.as_ref().unwrap()
	}

	/// Returns a mutable reference to the original static widget.
	pub fn get_widget_mut(&mut self) -> &mut W {
		self.widget.as_mut().unwrap()
	}
}

impl<W, S> Widget for Reactive<W, S> 
where 
	W: Widget<Signal = S>,
	S: Signal,
{
	type Signal = S;

	fn handle_event(&mut self, input_state: &mut InputState<Self::Signal>, id: LayoutId, area: Rect, pos: Vec2) -> bool {
		self.get_widget_mut().handle_event(input_state, id, area, pos);
		true
	}

	fn draw(&mut self, painter: &mut Painter, size: Vec2) {
		let widget = self.widget.take().unwrap();
		self.widget = Some((*self.on_update)(widget));
		self.get_widget_mut().draw(painter, size)
	}

	fn size(&self, id: LayoutId, painter: &Painter, layout: &Layout<Self::Signal>) -> Vec2 {
		self.get_widget().size(id, painter, layout)
	}

	fn handle_child_layout(&mut self, childs: IndexMap<LayoutId, Vec2>, area: Rect, id: LayoutId) -> HashMap<LayoutId, Option<Rect>> {
		self.get_widget_mut().handle_child_layout(childs, area, id)
	}

	fn inner_padding(&self) -> Vec2 {
		self.get_widget().inner_padding()
	}
} 