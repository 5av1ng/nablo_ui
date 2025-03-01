// use crate::prelude::{Rect, Vec2};

// use super::LayoutId;

// pub const CAPACITY: usize = 10;

// pub struct QuadTree {
// 	pub area: Rect,
// 	pub inner_widget: Vec<(LayoutId, Rect)>,
// 	pub children: Option<[Box<QuadTree>; 4]>,
// }

// impl QuadTree {
// 	pub fn new(area: Rect) -> Self {
// 		Self {
// 			area,
// 			inner_widget: Vec::new(),
// 			children: None,
// 		}
// 	}

// 	pub fn insert(&mut self, widget: LayoutId, rect: Rect) -> bool {
// 		if rect | self.area != self.area  {
// 			return false;
// 		}

// 		if let Some(children) = &mut self.children {
// 			if !children.iter_mut().any(|child| child.insert(widget, rect)) {
// 				self.inner_widget.push((widget, rect));
// 			}

// 			true
// 		}else {
// 			if self.inner_widget.len() < CAPACITY {
// 				self.inner_widget.push((widget, rect));
// 				return true;
// 			}

// 			self.inner_widget.push((widget, rect));

// 			let mut children = [
// 				Box::new(QuadTree::new(Rect {
// 					x: self.area.x,
// 					y: self.area.y,
// 					w: self.area.w / 2.0,
// 					h: self.area.h / 2.0,
// 				})),
// 				Box::new(QuadTree::new(Rect {
// 					x: self.area.x + self.area.w / 2.0,
// 					y: self.area.y,
// 					w: self.area.w / 2.0,
// 					h: self.area.h / 2.0,
// 				})),
// 				Box::new(QuadTree::new(Rect {
// 					x: self.area.x,
// 					y: self.area.y + self.area.h / 2.0,
// 					w: self.area.w / 2.0,
// 					h: self.area.h / 2.0,
// 				})),
// 				Box::new(QuadTree::new(Rect {
// 					x: self.area.x + self.area.w / 2.0,
// 					y: self.area.y + self.area.h / 2.0,
// 					w: self.area.w / 2.0,
// 					h: self.area.h / 2.0,
// 				})),
// 			];

// 			// let mut out = false;

// 			let inner = std::mem::take(&mut self.inner_widget);

// 			for (w, r) in inner {
// 				if !children.iter_mut().any(|child| child.insert(w, r)) {
// 					self.inner_widget.push((w, r));
// 				}
// 			}

// 			self.children = Some(children);
// 			true
// 		}
// 	}

// 	pub fn query(&self, point: Vec2) -> Vec<LayoutId> {
// 		if let Some(children) = &self.children {
// 			let mut out = Vec::new();

// 			for child in children {
// 				if child.area.contains(point) {
// 					out.extend(child.query(point));
// 				}
// 			}

// 			for (w, r) in &self.inner_widget {
// 				if r.contains(point) {
// 					out.push(*w);
// 				}
// 			}

// 			out
// 		}else {
// 			if !self.area.contains(point) {
// 				return Vec::new();
// 			}

// 			let mut out = Vec::new();

// 			for (w, r) in &self.inner_widget {
// 				if r.contains(point) {
// 					out.push(*w);
// 				}
// 			}

// 			out
// 		}
// 	}

// 	pub fn query_single(&self, point: Vec2) -> Option<LayoutId> {
// 		let mut out = self.query(point);

// 		out.sort_by(|a, b| a.0.cmp(&b.0));
// 		out.pop()
// 	}
// }