//! A struct that can be used to convert a static widget into a reactive widget.

use std::collections::HashMap;

use indexmap::IndexMap;

use crate::{layout::{Layout, LayoutId}, prelude::{InputState, Painter, Rect, Vec2}, App};

use super::{Signal, Widget};

/// A struct that can be used to convert a static widget into a reactive widget.
pub struct Reactive<W, S: Signal, A: App<Signal = S>> 
where 
	W: Widget<Signal = S, Application = A>,
{
	/// The original static widget.
	widget: Option<W>,
	/// The function that used to update the display element of the widget.
	#[allow(clippy::type_complexity)]
	pub on_update: Box<dyn Fn(&mut A, W) -> W>,
}

impl <W, S, A> Reactive<W, S, A>
where 
	W: Widget<Signal = S, Application = A>,
	S: Signal,
	A: App<Signal = S>,
{
	/// Creates a new reactive widget.
	pub fn new(widget: W, on_update: impl Fn(&mut A, W) -> W + 'static) -> Self {
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

impl<W, S, A> Widget for Reactive<W, S, A> 
where 
	W: Widget<Signal = S, Application = A>,
	S: Signal,
	A: App<Signal = S>,
{
	type Signal = S;
	type Application = A;

	fn handle_event(&mut self, app: &mut A, input_state: &mut InputState<Self::Signal>, id: LayoutId, area: Rect, pos: Vec2) -> bool {
		let widget = self.widget.take().unwrap();
		self.widget = Some((*self.on_update)(app, widget));
		self.get_widget_mut().handle_event(app, input_state, id, area, pos);
		true
	}

	fn draw(&mut self, painter: &mut Painter, size: Vec2) {
		self.get_widget_mut().draw(painter, size)
	}

	fn size(&self, id: LayoutId, painter: &Painter, layout: &Layout<Self::Signal, A>) -> Vec2 {
		self.get_widget().size(id, painter, layout)
	}

	fn handle_child_layout(&mut self, childs: IndexMap<LayoutId, Vec2>, area: Rect, id: LayoutId) -> HashMap<LayoutId, Option<Rect>> {
		self.get_widget_mut().handle_child_layout(childs, area, id)
	}

	fn inner_padding(&self) -> Vec2 {
		self.get_widget().inner_padding()
	}

	fn continuous_event_handling(&self) -> bool {
		self.get_widget().continuous_event_handling()
	}
} 