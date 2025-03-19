//! This file contains the implementation of the animation related structs.

use std::{fmt::Debug, ops::{Add, Index, IndexMut, Mul}};

use lyon_geom::{point, CubicBezierSegment};
use time::{Duration, OffsetDateTime};

use super::{color::Color, vec2::Vec2};

/// The default duration of an animated f32.
pub static DEFAULT_ANIMATION_DURATION: Duration = Duration::milliseconds(150);

/// Represents a one dimensional animation.
#[derive(Default, Clone)]
#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct Animation {
	/// The start value of the animation.
	pub start_value: f32,
	/// The nodes of the animation.
	pub nodes: Vec<AnimationNode>,
}

/// Represents a node of an animation.
#[derive(Default, Clone)]
#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct AnimationNode {
	/// The time relative to the last node.
	pub time: Duration,
	/// The value of the node.
	pub value: f32,
	/// The interpolation function of the node.
	pub interpolation: Linker,
}

/// Represents a interpolation function of an animation node.
#[derive(Default, Clone)]
#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub enum Linker {
	/// Become the value of the next node instantly when reach the end of the current node.
	#[default] Mutation,
	/// Linear interpolation between the current and next node.
	Linear,
	/// Cubic interpolation between the current and next node.
	/// 
	/// Value should be normalized to the range [0, 1].
	Bezier(Vec2, Vec2),
	// /// Custom interpolation function.
	// Custom(Box<dyn Interpolation>),
}

// /// Represents a custom interpolation function.
// pub trait Interpolation {
// 	/// Calculates the interpolated value between the current and next node.
// 	/// 
// 	/// All the time values are in absolute time rather than relative to the last node.
// 	fn interpolate(
// 		&self, 
// 		last_node_time: Duration, 
// 		sustain_time: Duration, 
// 		current: Duration,
// 		previous_value: f32,
// 		next_value: f32, 
// 	) -> f32;
// }

impl Animation {
	/// Creates a new animation with the given start value and nodes.
	pub fn new(start_value: f32, nodes: Vec<AnimationNode>) -> Self {
		Self {
			start_value,
			nodes,
		}
	}

	/// Adds a new node to the animation.
	pub fn push(&mut self, node: AnimationNode) {
		self.nodes.push(node);
	}

	/// Removes the last node of the animation.
	pub fn pop(&mut self) -> Option<AnimationNode> {
		self.nodes.pop()
	}

	/// Inserts a new node at the given index of the animation.
	pub fn insert(&mut self, index: usize, node: AnimationNode) {
		self.nodes.insert(index, node);
	}

	/// Inserts a new node at the given time of the animation.
	/// 
	/// Will not affect the duration of the animation unless the given time is greater than the duration of the animation.
	/// That will cause the animation to be extended to the given time.
	/// 
	/// Will do nothing if the given time is less than or equal to 0.
	pub fn insert_at_time(&mut self, time: Duration, value: f32, interpolation: Linker) {
		if time <= Duration::ZERO {
			return;
		}

		let mut current_time = Duration::ZERO;
		for i in 0..self.nodes.len() {
			let summary_time = current_time + self.nodes[i].time;
			if summary_time > time {
				self.nodes[i].time = time - current_time;
				self.insert(i, AnimationNode {
					time: summary_time - time,
					value,
					interpolation,
				});
				return;
			}
			current_time += self.nodes[i].time;
		}
		let time = current_time - time;
		self.nodes.push(AnimationNode {
			time,
			value,
			interpolation,
		});
	}

	/// Removes the node at the given index of the animation.
	pub fn remove(&mut self, index: usize) -> Option<AnimationNode> {
		if self.nodes.get(index).is_some() {
			Some(self.nodes.remove(index))
		}else {
			None
		}
	}

	/// Removes the node at the given index of the animation and keeps the duration of the animation unchanged.
	pub fn remove_hold(&mut self, index: usize) -> Option<AnimationNode> {
		if index >= self.nodes.len() {
			return None;
		}

		let time = self.nodes.iter().take(index + 1).map(|node| node.time).sum();

		self.remove_at_time(time)
	}

	/// Removes the node at the given time of the animation.
	/// 
	/// Will not affect the duration of the animation unless the given time is exact the same as the duration of the animation.
	/// That will cause the remove of the last node of the animation.
	/// 
	/// Will do nothing if the given time is not mathcing any node.
	pub fn remove_at_time(&mut self, time: Duration) -> Option<AnimationNode> {
		if self.nodes.is_empty() {
			return None;
		}

		if time <= Duration::ZERO {
			return None;
		}

		let mut current_time = Duration::ZERO;
		for i in 0..self.nodes.len() {
			let summary_time = current_time + self.nodes[i].time;
			if summary_time == time {
				if i > 0 {
					let time = self.nodes[i].time;
					self.nodes[i - 1].time += time;
				}

				return Some(self.nodes.remove(i));
			}
			current_time += self.nodes[i].time;
		}

		None
	}

	/// changes the sustain time of the node at the given index of the animation.
	/// 
	/// Will not affect the duration of the animation unless the given node is the last node.
	/// 
	/// Will do nothing if the given index is out of range.
	pub fn change_by_time_hold(&mut self, index: usize, time: Duration) {
		if index >= self.nodes.len() {
			return;
		}

		let to_set = self.nodes[index].time + time;

		self.change_to_time_hold(index, to_set);
	}

	/// changes the sustain time of the node at the given index of the animation.
	/// 
	/// Will affect the duration of the animation.
	/// 
	/// Will do nothing if the given index is out of range.
	pub fn change_by_time_unhold(&mut self, index: usize, time: Duration) {
		if index >= self.nodes.len() {
			return;
		}

		let to_set = self.nodes[index].time + time;

		self.change_to_time_unhold(index, to_set);
	}

	/// changes the sustain time of the node at the given index of the animation.
	/// 
	/// Will not affect the duration of the animation unless the given node is the last node.
	/// 
	/// Will do nothing if the given index is out of range.
	pub fn change_to_time_hold(&mut self, index: usize, time: Duration) {
		if index >= self.nodes.len() {
			return;
		}

		if time <= Duration::ZERO {
			return;
		}

		
		if index == self.nodes.len() - 1 {
			self.nodes[index].time = time;
		}else if time > self.nodes[index].time + self.nodes[index + 1].time {
			self.nodes[index].time = self.nodes[index].time + self.nodes[index + 1].time;
			self.nodes[index + 1].time = Duration::ZERO;
		}else {
			let delta = self.nodes[index].time - time;
			self.nodes[index].time = time;
			self.nodes[index + 1].time += delta;
		}
	}

	/// changes the sustain time of the node at the given index of the animation.
	/// 
	/// Will affect the duration of the animation.
	/// 
	/// Will do nothing if the given index is out of range.
	pub fn change_to_time_unhold(&mut self, index: usize, time: Duration) {
		if index >= self.nodes.len() {
			return;
		}

		if time <= Duration::ZERO {
			return;
		}

		self.nodes[index].time = time;
	}

	/// Get the absolute time of each node of the animation.
	/// 
	/// Will not include the start time of the animation, since it is always 0.
	pub fn stages(&self) -> Vec<Duration> {
		let mut stages = Vec::new();
		let mut time = Duration::ZERO;
		for node in &self.nodes {
			time += node.time;
			stages.push(time);
		}
		stages
	}

	/// Get the total duration of the animation.
	pub fn duration(&self) -> Duration {
		let mut duration = Duration::ZERO;
		for node in &self.nodes {
			duration += node.time;
		}
		duration
	}

	/// Returns true if the animation has no nodes.
	pub fn is_empty(&self) -> bool {
		self.nodes.is_empty()
	}

	/// Get all the values of the animation.
	pub fn values(&self) -> Vec<f32> {
		let mut values = vec!(self.start_value);
		for node in &self.nodes {
			values.push(node.value);
		}
		values
	}

	/// Get the last value of the animation.
	pub fn last_value(&self) -> f32 {
		if self.nodes.is_empty() {
			self.start_value
		}else {
			self.nodes.last().unwrap().value
		}
	}

	/// Calculates the interpolated value of the animation at the given time.
	/// 
	/// If the time is greater than the duration of the animation, the last value of the animation will be returned.
	/// 
	/// If the animation has no nodes or the time is less than or equal to 0, the start value will be returned.
	pub fn value_at(&self, time: Duration) -> f32 {
		if self.nodes.is_empty() || time <= Duration::ZERO {
			return self.start_value;
		}else if time > self.duration() {
			return self.last_value();
		}

		let mut current_time = Duration::ZERO;
		let mut previous_value = self.start_value;
		let mut out_value = self.last_value();
		for node in &self.nodes {
			if current_time + node.time < time {
				current_time += node.time;
				previous_value = node.value;
				continue;
			}

			out_value = match &node.interpolation {
				Linker::Mutation => {
					previous_value
				},
				Linker::Linear => {
					let progress = ((time - current_time) / node.time) as f32;
					(1.0 - progress) * previous_value + progress * node.value
				},
				Linker::Bezier(p1, p2) => {
					let p1 = p1.clamp_both(Vec2::ZERO, Vec2::ONE);
					let p2 = p2.clamp_both(Vec2::ZERO, Vec2::ONE);
					let bezier = CubicBezierSegment {
						from: point(0.0, 0.0),
						ctrl1: point(p1.x, p1.y),
						ctrl2: point(p2.x, p2.y),
						to: point(1.0, 1.0),
					};
					let x = ((time - current_time) / node.time) as f32;
					let t = bezier.solve_t_for_x(x).first().cloned().unwrap_or_default();
					let y = bezier.y(t);
					(1.0 - y) * previous_value + y * node.value
				},
				// Linker::Custom(interpolation) => {
				// 	interpolation.interpolate(
				// 		current_time, 
				// 		node.time, 
				// 		time, 
				// 		previous_value, 
				// 		node.value,
				// 	)
				// }
			};
		}

		out_value
	}

	/// apply clamp to each node of the animation.
	pub fn clamp(&mut self, min: f32, max: f32){
		self.start_value = self.start_value.clamp(min, max);
		for node in &mut self.nodes {
			node.value = node.value.clamp(min, max);
		}
	}

	/// apply min to each node of the animation.
	pub fn min(&mut self, min: f32){
		self.start_value = self.start_value.min(min);
		for node in &mut self.nodes {
			node.value = node.value.min(min);
		}
	}

	/// apply max to each node of the animation.
	pub fn max(&mut self, max: f32){
		self.start_value = self.start_value.max(max);
		for node in &mut self.nodes {
			node.value = node.value.max(max);
		}
	}

	/// Get the minimum value of the animation.
	pub fn min_value(&self) -> f32 {
		self.nodes
			.iter()
			.map(|node| node.value)
			.min_by(|a, b| a.partial_cmp(b).unwrap())
			.unwrap_or(self.start_value).min(self.start_value)
	}

	/// Get the maximum value of the animation.
	pub fn max_value(&self) -> f32 {
		self.nodes
			.iter()
			.map(|node| node.value)
			.max_by(|a, b| a.partial_cmp(b).unwrap())
			.unwrap_or(self.start_value).max(self.start_value)
	}
}

impl Index<usize> for Animation {
	type Output = AnimationNode;

	fn index(&self, index: usize) -> &Self::Output {
		&self.nodes[index]
	}
}

impl IndexMut<usize> for Animation {
	fn index_mut(&mut self, index: usize) -> &mut Self::Output {
		&mut self.nodes[index]
	}
}

/// An animated f32 value that can be used in a UI.
pub type Animatedf32 = AnimatedValue<f32>;
/// An animated 2D vector that can be used in a UI.
pub type AnimatedVec2 = AnimatedValue<Vec2>;
/// An animated Color(4D vector) that can be used in a UI.
pub type AnimatedColor = AnimatedValue<Color>;

/// An animated value that can be used in a UI.
/// 
/// By default, the animation will be a beizer interpolation with control points (0.5, 0.0) and (0.5, 1.0) between 0.0 and 1.0.
pub struct AnimatedValue<T: Add + Mul<f32> + PartialEq + Clone> {
	animation: Animation,
	last_changes: OffsetDateTime,
	from: T,
	to: T,
}

/// Extension trait for AnimatedValue. Used for shorthand syntax.
pub trait AnimatedValueExt: Add<Output = Self> + Mul<f32, Output = Self> + PartialEq + Clone {}

impl<T: Add<Output = Self> + Mul<f32, Output = Self> + PartialEq + Clone> AnimatedValueExt for T {}


impl<T: AnimatedValueExt + Default> Default for AnimatedValue<T> {
	fn default() -> Self {
		let mut animation = Animation::default();
		animation.push(AnimationNode {
			time: DEFAULT_ANIMATION_DURATION,
			value: 1.0,
			interpolation: Linker::Bezier(Vec2::new(0.5, 0.0), Vec2::new(0.5, 1.0)),
		});

		Self {
			animation,
			last_changes: OffsetDateTime::now_utc(),
			from: T::default(),
			to: T::default(),
		}
	}
}

impl<T: AnimatedValueExt + Debug> Debug for AnimatedValue<T> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "Animated({:?} -> {:?})", self.from, self.to)
	}
}

impl<T: AnimatedValueExt> PartialEq for AnimatedValue<T> {
	fn eq(&self, other: &Self) -> bool {
		self.value() == other.value()
	}
}

impl<T: AnimatedValueExt + PartialOrd> PartialOrd for AnimatedValue<T> {
	fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
		self.value().partial_cmp(&other.value())
	}
}

impl<T: AnimatedValueExt> AnimatedValue<T> {
	/// Creates a new animated value with the given animation.
	/// 
	/// The animation should be start with 0.0 and end with 1.0.
	pub fn new(animation: Animation, value: T) -> Self {
		Self {
			animation,
			from: value.clone(),
			to: value,
			last_changes: OffsetDateTime::now_utc(),
		}
	}

	/// Creates a new animated value with default animation and the given value.
	pub fn default_with_value(value: T) -> Self {
		let mut animation = Animation::default();
		animation.push(AnimationNode {
			time: DEFAULT_ANIMATION_DURATION,
			value: 1.0,
			interpolation: Linker::Bezier(Vec2::new(0.3, 0.0), Vec2::new(0.7, 1.0)),
		});

		Self {
			animation,
			from: value.clone(),
			to: value,
			last_changes: OffsetDateTime::now_utc(),
		}
	}

	/// Returns the current value of the animation.
	pub fn value(&self) -> T {
		if self.from == self.to {
			return self.from.clone();
		}
		let now = OffsetDateTime::now_utc();
		let t = self.animation.value_at(now - self.last_changes);
		// println!("{}, {}", self.animation.start_value, self.animation.last_value());
		self.from.clone() * (1.0 - t) + self.to.clone() * t
	}

	/// Sets the new value of the animation.
	pub fn set(&mut self, new_value: T) {
		if self.to != new_value {
			let current = self.value();
			self.from = current;
			self.to = new_value;
			self.last_changes = OffsetDateTime::now_utc();
		}
	}

	/// Sets the new value of the animation by a delta.
	pub fn set_by(&mut self, delta: T) {
		self.set(self.to.clone() + delta)
	}

	/// Sets the new value of the animation without animating.
	pub fn set_without_animation(&mut self, new_value: T) {
		self.from = new_value.clone();
		self.to = new_value;
		self.last_changes = OffsetDateTime::now_utc();
	}

	/// Sets the start value of the animation.
	pub fn set_start(&mut self, new_value: T) {
		self.from = new_value;
		self.last_changes = OffsetDateTime::now_utc();
	}

	/// Returns true if the animation is currently animating.
	pub fn is_animating(&self) -> bool {
		let now = OffsetDateTime::now_utc();
		now - self.last_changes < self.animation.duration() && self.from != self.to
	}
}

impl <T: AnimatedValueExt + PartialOrd> AnimatedValue<T> {
	/// Clamps the value of the animation between the given min and max values.
	pub fn clamp(&mut self, min: T, max: T) {
		if self.to < min {
			self.set(min)
		}else if self.to > max {
			self.set(max)
		}
	}

	/// Apply min to the value of the animation.
	pub fn min(&mut self, min: T) {
		if self.to < min {
			self.set(min)
		}
	}

	/// Apply max to the value of the animation.
	pub fn max(&mut self, max: T) {
		if self.to > max {
			self.set(max)
		}
	}
}