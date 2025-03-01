//! A tree-based layout for the Nablo UI.

mod macros;
mod quad_tree;

use std::{any::Any, collections::{HashMap, HashSet, VecDeque}, fmt::Display, hash::Hash};

use indexmap::{IndexMap, IndexSet};
// use quad_tree::QuadTree;

use crate::{math::rect::Rect, prelude::Vec2, render::painter::Painter, widgets::{Signal, Widget}, window::input_state::InputState, App};

/// A unique identifier for a layout element.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub struct LayoutId(pub usize);

impl Display for LayoutId {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "LayoutId({})", self.0)
	}
}

/// The root element's id.
pub const ROOT_LAYOUT_ID: LayoutId = LayoutId(0);

/// A tree-based layout for the Nablo UI.
pub struct Layout<S: Signal, A: App<Signal = S>> {
	/// we will save the widgets in a hashmap with their id as the key to make it easy to find the widget by id and keep efficient.
	widgets: HashMap<LayoutId, LayoutElement<S, A>>,
	/// the adjacency list of the tree-based layout.
	tree: HashMap<LayoutId, Vec<LayoutId>>,
	/// the inversed adjacency list of the tree-based layout.
	/// This is used to find the parent of a widget.
	/// root has [`ROOT_LAYOUT_ID`] as its parent.
	inverse_tree: HashMap<LayoutId, LayoutId>,
	/// the avaiable id.
	next_id: usize,
	/// the alias map for the layout.
	alias_map: HashMap<String, LayoutId>,
	/// the inversed alias map for the layout.
	inversed_alias_map: HashMap<LayoutId, String>,

	// quad_tree: QuadTree,
	continous_widgets: HashSet<LayoutId>,
}

/// A layout element that holds a widget and its properties.
pub struct LayoutElement<S: Signal, A: App<Signal = S>> {
	/// The unique identifier of the layout element.
	pub id: LayoutId,
	/// The area and the position of the layout element within its parent.
	/// 
	/// `None` means that the element is not added to the layout and the widget will done nothing.
	/// This may caused by the parent's layout method or adding the element to a widget do not support to have child.
	/// This will be automatically set to [`Rect::WINDOW`] if the element is added to the root of the layout.
	/// Will be intersectioned with the parent's area if the element is added to a widget that has a parent.
	/// 
	/// The left top position of the element will be the later element of the tuple.
	pub area_and_pos: Option<(Rect, Vec2)>,
	/// The widget of the layout element.
	pub widget: Box<dyn Widget<Signal = S, Application = A>>,
	/// Whether the widget needs to be redrawn. 
	/// 
	/// We will also call the widget is dirty if it needs to be redrawn.
	pub redraw_request: bool,
}

impl<S: Signal, A: App<Signal = S>> Default for Layout<S, A> {
	fn default() -> Self {
		Self::new()
	}
}

impl<S: Signal, A: App<Signal = S>> Layout<S, A> {
	/// Create a new empty layout.
	pub fn new() -> Self {
		Self {
			widgets: HashMap::new(),
			tree: HashMap::new(),
			inverse_tree: HashMap::new(),
			next_id: 1,
			alias_map: HashMap::new(),
			inversed_alias_map: HashMap::new(),
			// quad_tree: QuadTree::new(Rect::ZERO),
			continous_widgets: HashSet::new(),
		}
	}

	/// Insert a root widget to the layout.
	/// 
	/// Returns the id of the new widget.
	/// There will be only one root widget in the layout. 
	/// If there is already a root widget, the new widget will be switched to the root widget and true will be returned.
	pub fn insert_root_widget(&mut self, widget: impl Widget<Signal = S, Application = A>) -> bool {
		if let Some(root) = self.widgets.get_mut(&ROOT_LAYOUT_ID) {
			root.widget = Box::new(widget);
			root.redraw_request = true;
			true
		}else {
			self.widgets.insert(
				ROOT_LAYOUT_ID,
				LayoutElement {
					id: ROOT_LAYOUT_ID,
					area_and_pos: Some((Rect::WINDOW, Vec2::ZERO)),
					widget: Box::new(widget),
					redraw_request: true,
				},
			);
			self.tree.insert(ROOT_LAYOUT_ID, Vec::new());
			self.inverse_tree.insert(ROOT_LAYOUT_ID, ROOT_LAYOUT_ID);
			false
		}
	}

	/// Add a new widget to the layout.
	/// 
	/// Returns the id of the new widget.
	/// 
	/// If the parent_id is not in the layout, the widget will not be added and None will be returned.
	pub fn add_widget(&mut self, parent_id: LayoutId, widget: impl Widget<Signal = S, Application = A>) -> Option<LayoutId> {
		if self.widgets.contains_key(&parent_id) {
			let id = LayoutId(self.next_id);
			if widget.continuous_event_handling() {
				self.continous_widgets.insert(id);
			}
			self.next_id += 1;
			self.widgets.insert(
				id,
				LayoutElement {
					id,
					area_and_pos: None,
					widget: Box::new(widget),
					redraw_request: true,
				},
			);
			self.widgets.get_mut(&parent_id).unwrap().redraw_request = true;
			self.tree.entry(parent_id).or_default().push(id);
			self.inverse_tree.insert(id, parent_id);
			Some(id)
		}else {
			None
		}
	}

	/// Alias a widget by its id.
	/// 
	/// This will allow you to refer to the widget by its alias name instead of its id.
	pub fn alias_widget(&mut self, id: LayoutId, alias: impl Into<String>) {
		let alias = alias.into();
		self.alias_map.insert(alias.clone(), id);
		self.inversed_alias_map.insert(id, alias);
	}

	/// Remove a widget from the layout.
	/// 
	/// Returns None if the widget is not in the layout.
	/// 
	/// Will also remove all the children of the widget.
	pub fn remove_widget(&mut self, id: LayoutId) -> Vec<Box<dyn Widget<Signal = S, Application = A>>> {
		if let Some(element) = self.widgets.remove(&id) {
			let mut out = vec!();
			if let Some(children) = self.tree.remove(&id) {
				for child_id in children {
					out.extend(self.remove_widget(child_id));
				}
			}
			if let Some(parent_id) = self.inverse_tree.remove(&id) {
				self.tree.entry(parent_id).or_default().retain(|&x| x != id);
				if let Some(inner) = self.widgets.get_mut(&parent_id) { inner.redraw_request = true };
			}
			out.push(element.widget);
			out
		}else {
			vec!()
		}
	}

	/// Remove a widget by its alias.
	pub fn remove_widget_by_alias(&mut self, alias: impl Into<String>) -> Vec<Box<dyn Widget<Signal = S, Application = A>>> {
		let alias = alias.into();
		if let Some(id) = self.alias_map.get(&alias) {
			self.remove_widget(*id)
		}else {
			vec!()
		}
	}

	/// Remove a widget's childer.
	pub fn remove_widget_children(&mut self, id: LayoutId) -> Vec<Box<dyn Widget<Signal = S, Application = A>>> {
		if let Some(children) = self.tree.remove(&id) {
			let mut out = vec!();
			for child_id in children {
				out.extend(self.remove_widget(child_id));
			}
			out
		}else {
			vec!()
		}
	}

	/// Remove a widget's childer by its alias.
	pub fn remove_widget_children_by_alias(&mut self, alias: impl Into<String>) -> Vec<Box<dyn Widget<Signal = S, Application = A>>> {
		let alias = alias.into();
		if let Some(id) = self.alias_map.get(&alias) {
			self.remove_widget_children(*id)
		}else {
			vec!()
		}
	}

	/// Replace the given widget, will return the old widget and its children if any.
	/// 
	/// # Panics
	/// 
	/// Panics if missing root widget in the layout or the widget is not in the layout.
	pub fn replace_widget(&mut self, id: LayoutId, widget: impl Widget<Signal = S, Application = A>) -> Vec<Box<dyn Widget<Signal = S, Application = A>>> {
		let parent_id = if let Some(parent_id) = self.inverse_tree.get(&id) {
			*parent_id
		}else {
			panic!("The given widget {id} is not in the layout.")
		};

		let out = self.remove_widget_children(id);

		if self.widgets.contains_key(&parent_id) {
			if widget.continuous_event_handling() {
				self.continous_widgets.insert(id);
			}
			self.widgets.insert(
				id,
				LayoutElement {
					id,
					area_and_pos: None,
					widget: Box::new(widget),
					redraw_request: true,
				},
			);
			self.widgets.get_mut(&parent_id).unwrap().redraw_request = true;
			// self.tree.entry(parent_id).or_default().push(id);
			// self.inverse_tree.insert(id, parent_id);
		}else {
			panic!("The given widget {id} is not in the layout.")
		}

		out
	}

	/// Turn an alias to an id.
	pub fn alias_to_id(&self, alias: impl Into<String>) -> Option<LayoutId> {
		self.alias_map.get(&alias.into()).cloned()
	}

	/// Turn an id to an alias.
	pub fn id_to_alias(&self, id: LayoutId) -> Option<&str> {
		self.inversed_alias_map.get(&id).map(|x| x.as_str())
	}

	/// Replace the given widget by its alias, will return the old widget and its children if any.
	pub fn replace_widget_by_alias(
		&mut self, 
		alias: impl Into<String>,
		widget: impl Widget<Signal = S, Application = A>
	) -> Vec<Box<dyn Widget<Signal = S, Application = A>>> {
		let alias = alias.into();
		if let Some(id) = self.alias_map.get(&alias) {
			self.replace_widget(*id, widget)
		}else {
			vec!()
		}
	}

	/// Get the widget by its id.
	pub fn get_widget<T: Widget<Signal = S, Application = A> + Any>(&self, id: LayoutId) -> Option<&T> {
		if let Some(inner) = self.widgets.get(&id) {
			inner.widget.downcast_ref::<T>()
		}else {
			None
		}
	}

	/// Get the widget by its alias.
	pub fn get_widget_by_alias<T: Widget<Signal = S, Application = A> + Any>(&self, alias: impl Into<String>) -> Option<&T> {
		let alias = alias.into();
		if let Some(id) = self.alias_map.get(&alias) {
			self.get_widget(*id)
		}else {
			None
		}
	}

	/// Get the widget mutably by its id.
	/// 
	/// This function will automatically mark the widget as dirty.
	/// 
	/// Due to the limitation of Rust's type system, we cannot return a mutable reference to the widget.
	/// 
	/// Instead, we will use a closure to modify the widget.
	pub fn widget_mut<W: Widget<Signal = S, Application = A> + Any>(&mut self, id: LayoutId, f: impl FnOnce(W) -> W) {
		if let Some(element) = self.widgets.remove(&id) {
			let area_and_pos = element.area_and_pos;
			if element.widget.is::<W>() {
				let widget = *unsafe { Box::from_raw(Box::into_raw(element.widget) as *mut W) };
				let widget = f(widget);
				self.widgets.insert(id, LayoutElement {
					id,
					area_and_pos,
					widget: Box::new(widget),
					redraw_request: true,
				});
			}else {
				self.widgets.insert(id, LayoutElement {
					id,
					area_and_pos,
					widget: element.widget,
					redraw_request: true,
				});
			}
		}
	}

	/// Get the widget mutably by its alias.
	/// 
	/// This function will automatically mark the widget as dirty.
	/// 
	/// Due to the limitation of Rust's type system, we cannot return a mutable reference to the widget.
	/// 
	/// Instead, we will use a closure to modify the widget.
	pub fn widget_mut_by_alias<W: Widget<Signal = S, Application = A> + Any>(&mut self, alias: impl Into<String>, f: impl FnOnce(W) -> W) {
		let alias = alias.into();
		if let Some(id) = self.alias_map.get(&alias) {
			self.widget_mut(*id, f);
		}
	}

	/// Get the area of a widget.
	pub fn get_widget_area(&self, id: LayoutId) -> Option<Rect> {
		if let Some(element) = self.widgets.get(&id) {
			element.area_and_pos.map(|(area, _)|  area)
		}else {
			None
		}
	}

	/// Get the position of a widget.
	pub fn get_widget_pos(&self, id: LayoutId) -> Option<Vec2> {
		if let Some(element) = self.widgets.get(&id) {
			element.area_and_pos.map(|(_, pos)| pos)
		}else {
			None
		}
	}

	/// Get the padding of a widget.
	pub fn get_widget_padding(&self, id: LayoutId) -> Option<Vec2> {
		self.widgets.get(&id).map(|inner| inner.widget.inner_padding())
	}

	/// Get the parent id of a widget.
	pub fn get_parent_id(&self, id: LayoutId) -> Option<LayoutId> {
		self.inverse_tree.get(&id).cloned()
	}

	/// Get the parent id of a widget by its alias.
	pub fn get_parent_id_by_alias(&self, alias: impl Into<String>) -> Option<LayoutId> {
		let alias = alias.into();
		if let Some(id) = self.alias_map.get(&alias) {
			self.get_parent_id(*id)
		}else {
			None
		}
	}

	/// Get the parents of a widget.
	pub fn get_parents(&self, id: LayoutId) -> Vec<LayoutId> {
		let mut out = vec!();
		let mut current = Some(id);

		while let Some(id) = current {
			if let Some(id) = self.get_parent_id(id) {
				out.push(id);
				current = Some(id);
			}
		}

		out.reverse();
		out
	}

	/// Get the parents of a widget by its alias.
	/// 
	/// Returns empty vector if the alias is not found.
	pub fn get_parents_by_alias(&self, alias: impl Into<String>) -> Vec<LayoutId> {
		let alias = alias.into();
		if let Some(id) = self.alias_map.get(&alias) {
			self.get_parents(*id)
		}else {
			vec!()
		}
	}


	/// Get the children ids of a widget.
	pub fn get_children_ids(&self, id: LayoutId) -> Option<&[LayoutId]> {
		self.tree.get(&id).map(|x| x.as_slice())
	}

	/// Get the number of the widgets.
	pub fn widgets(&self) -> usize {
		self.widgets.len()
	}

	/// Get the number of the layers.
	pub fn layers(&self) -> usize {
		let mut out = 0;
		self.layers_inner(HashSet::from([ROOT_LAYOUT_ID]), &mut out);
		out
	}

	fn layers_inner(&self, layers: HashSet<LayoutId>, layer_count: &mut usize) {
		if layers.is_empty() {
			return;
		}
		let mut next_layers = HashSet::new();
		for id in layers {
			if let Some(children) = self.tree.get(&id) {
				next_layers.extend(children.iter().cloned());
			}
		}
		if !next_layers.is_empty() {
			*layer_count += 1;
			self.layers_inner(next_layers, layer_count);
		}
	}

	fn sperate_dirty_widgets(&mut self) {
		let mut traversed_widgets = HashSet::new();
		let mut dirty_widgets = self.widgets
			.values().filter_map(|inner| {
				if inner.redraw_request {
					Some(inner.id)
				}else {
					None
				}
			}).collect::<Vec<_>>();
		
		while let Some(id) = dirty_widgets.pop() {
			if traversed_widgets.contains(&id) {
				continue;
			}
			traversed_widgets.insert(id);
			if let Some(children) = self.tree.get(&id) {
				for child_id in children {
					dirty_widgets.push(*child_id);
				}
			}
			if let Some(element) = self.widgets.get_mut(&id) {
				element.redraw_request = true;
			}
		}
	}

	fn reanrrage_widgets(
		&mut self, 
		mut parent_window: Rect, 
		parent_pos: Vec2, 
		layout_id: LayoutId, 
		painter: &mut Painter,
		widget_to_remove: &mut Vec<LayoutId>
	) {
		// if let Some(element) = self.widgets.get_mut(&layout_id) {
		// 	if !element.redraw_request {
		// 		return;
		// 	}
		// }

		let children = if let Some(child) = self.tree.get(&layout_id) {
			child.clone()
		}else {
			return;
		};

		let mut children_set = children.iter().copied().collect::<IndexSet<_>>();

		let children_size_map = children.iter().filter_map(|child_id| {
			painter.set_relative_to(parent_pos);
			self.widgets.get(child_id).map(|child| (*child_id, child.widget.size(*child_id, painter, self)))
		}).collect::<IndexMap<_, _>>();

		let mut children_size_map = if let Some(parent) = self.widgets.get_mut(&layout_id) {
			if let Some((rect, _)) = parent.area_and_pos {
				parent.widget.handle_child_layout(children_size_map, rect, layout_id)
			}else {
				return;
			}
		}else {
			return;
		};

		if let Some(Some(rect)) = children_size_map.remove(&layout_id) {
			parent_window = rect.move_by(parent_pos);
		}

		for (child_id, child_window) in children_size_map {
			if let Some(child_window) = child_window {
				if let Some(child) = self.widgets.get_mut(&child_id) {
					let child_pos = parent_pos + child_window.lt();
					let child_window = child_window.move_by(parent_pos) & parent_window;
					// self.quad_tree.insert(child_id, child_window);
					child.area_and_pos = Some((child_window, child_pos));
					self.reanrrage_widgets(child_window, child_pos, child_id, painter, widget_to_remove);
					children_set.swap_remove(&child_id);
				}
			}else {
				widget_to_remove.push(child_id)
			}
		}

		while let Some(id) = children_set.pop() {
			if let Some(element) = self.widgets.get_mut(&id) {
				element.area_and_pos = None;
			}
			if let Some(children) = self.tree.get(&id) {
				for child_id in children {
					children_set.insert(*child_id);
				}
			}
		}
	}

	/// Clear the layout.
	pub fn clear(&mut self) {
		self.widgets.clear();
		self.tree.clear();
		self.inverse_tree.clear();
		self.next_id = 1;
		self.alias_map.clear();
	}

	// #[cfg(debug_assertions)]
	// fn check_overlap(&self, current_layer: Vec<LayoutId>) {
	// 	let mut rects = vec!();
	// 	let mut next_layer = vec!();
	// 	for id in current_layer {
	// 		if let Some(element) = self.widgets.get(&id) {
	// 			if let Some((area, _)) = element.area_and_pos {
	// 				rects.push((id, area));
	// 			}
	// 		}
	// 		next_layer.extend(self.tree.get(&id).unwrap_or(&vec!()).iter().cloned());
	// 	}

	// 	for i in 0..rects.len() {
	// 		for j in i+1..rects.len() {
	// 			if !(rects[i].1 & rects[j].1).is_empty() {
	// 				println!("[WARN] widget(id: {:?}) and widget(id: {:?}) overlap!", rects[i].0, rects[j].0);
	// 			}
	// 		}
	// 	}

	// 	if !next_layer.is_empty() {
	// 		self.check_overlap(next_layer);
	// 	}
	// }

	pub(crate) fn handle_draw(&mut self, painter: &mut Painter, window_size: Vec2) -> Option<Rect> {
		let mut widget_to_remove = vec!();

		self.sperate_dirty_widgets();
		// self.quad_tree = QuadTree::new(Rect::from_size(window_size));
		self.reanrrage_widgets(Rect::from_size(window_size), Vec2::ZERO, ROOT_LAYOUT_ID, painter, &mut widget_to_remove);
		// #[cfg(debug_assertions)]
		// self.check_overlap(vec![ROOT_LAYOUT_ID]);

		for id in widget_to_remove {
			self.remove_widget(id);
		}

		self.handle_paint(painter)
	}

	pub(crate) fn make_all_dirty(&mut self) {
		for element in self.widgets.values_mut() {
			element.redraw_request = true;
		}
	}

	fn handle_paint(
		&mut self,
		painter: &mut Painter,
	) -> Option<Rect> {
		let mut refresh_area = None; 

		let mut child_ids = VecDeque::new();

		child_ids.push_back(ROOT_LAYOUT_ID);

		while let Some(id) = child_ids.pop_front() {
			if let Some(element) = self.widgets.get_mut(&id) {
				if let Some((area, pos)) = element.area_and_pos {
					if element.redraw_request {
						if let Some(refresh) = &mut refresh_area {
							*refresh |= area;
						}else {
							refresh_area = Some(area);
						}
					}

					if area.is_empty() {
						continue;
					}

					painter.set_clip_rect(area);
					painter.set_relative_to(pos);
					painter.reset_blend_mode();
					painter.reset_fill_mode();
					painter.reset_transform();
					let size = if area.size().has_inf() {
						painter.window_size
					}else {
						area.rb() - pos
					};
					element.widget.draw(painter, size);
				}
				element.redraw_request = false;
			}
			if let Some(children) = self.tree.get(&id) {
				for child_id in children {
					child_ids.push_back(*child_id);
				}
			}
		}

		refresh_area
	}

	// pub(crate) fn handle_continous_events(&mut self, state: &mut InputState<S>) {
	// 	let widgets = std::mem::take(&mut self.continous_widgets);

	// 	for child_id in widgets {
	// 		if let Some(element) = self.widgets.get_mut(&child_id) {
	// 			if let Some((area, pos)) = element.area_and_pos {
	// 				if area.is_positive() {
	// 					element.redraw_request |= element.widget.handle_event(state, child_id, area, pos);
	// 					if element.widget.continuous_event_handling() {
	// 						self.continous_widgets.insert(child_id);
	// 					}
	// 				}
	// 			}
	// 		}
	// 	}
	// }

	pub(crate) fn handle_events(&mut self, parent_id: LayoutId, state: &mut InputState<S>, app: &mut A) {
		// if state.no_touch_available() {
		// 	return;
		// }

		let children = self.tree.get(&parent_id).unwrap_or(&vec!()).clone();
		
		for child_id in children {
			self.handle_events(child_id, state, app);
		}

		// self.continous_widgets.clear();

		state.handling_id = parent_id;
		if let Some(element) = self.widgets.get_mut(&parent_id) {
			if let Some((area, pos)) = element.area_and_pos {
				if area.is_positive() {
					element.redraw_request |= element.widget.handle_event(app, state, parent_id, area, pos);
					if element.widget.continuous_event_handling() {
						self.continous_widgets.insert(element.id);
					}
				}
			}
		}


		// let widgets = std::mem::take(&mut self.continous_widgets);

		// for child_id in widgets {
		// 	if let Some(element) = self.widgets.get_mut(&child_id) {
		// 		if let Some((area, pos)) = element.area_and_pos {
		// 			if area.is_positive() {
		// 				element.redraw_request |= element.widget.handle_event(state, child_id, area, pos);
		// 				if element.widget.continuous_event_handling() {
		// 					self.continous_widgets.push(child_id);
		// 				}
		// 			}
		// 		}
		// 	}
		// }

		// let window = Rect::from_size(state.window_size);

		// for pos in state.get_touch_on(window) {
		// 	if let Some(id) = self.quad_tree.query_single(state.get_touch_pos(pos).unwrap_or(Vec2::INF)) {
		// 		if let Some(element) = self.widgets.get_mut(&id) {
		// 			if let Some((area, pos)) = element.area_and_pos {
		// 				if area.is_positive() {
		// 					element.redraw_request |= element.widget.handle_event(state, id, area, pos);
		// 					if element.widget.continuous_event_handling() {
		// 						self.continous_widgets.push(id);
		// 					}
		// 				}
		// 			}
		// 		}
		// 	}
		// }
	}

	pub(crate) fn any_widget_dirty(&self) -> bool {
		self.widgets.values().any(|x| x.redraw_request)
	}
}