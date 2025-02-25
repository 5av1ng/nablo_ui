//! Some macros can be used to crate the layout of the UI to simplify the code.

/// A macro to create the layout of the UI.
/// 
/// Will clear the layout and create a new layout with the given root widget and children.
/// 
/// See more information in [`crate::layout_gen`].
#[macro_export] macro_rules! new_layout {
	($layout:expr, $root:expr => { $($children: tt)* }) => {{
		$layout.clear();
		$crate::layout_gen!($layout, $root => { $($children)* });
	}};
}

/// A macro to create the layout of the UI.
///
/// The macro has following ways to use:
/// 1. `layout_gen!(layout => { child1, child2, child3, ... })`: This will append children to the root widget.
/// 2. `layout_gen!(layout, root_widget => { child1, child2, child3 })`: 
///    This will replace the root widget with the given widget and append children to it.
/// 
/// Other arms to use the macro are not recommended and may cause unexpected behavior.
/// 
/// To append children to the given parent widget, use [`crate::layout_append`]
///
/// You can know how to use the macro by looking at the examples below.
/// 
/// ```
/// # use nablo_ui::prelude::*;
/// # let mut layout = Layout::new();
/// enum AppEvent {
///     Close,
///     Ok,
/// }
/// # impl Signal for AppEvent {}
/// 
/// layout_gen!(layout, Card::new(Default::default()) => {
///     Button::new("ok").on_click(|| AppEvent::Ok),
///     ["label", Button::new("cancel").on_click(|| AppEvent::Close)],
///     ["group", Collapse::new("collapse group") => {
///         Label::new("group label"),
///         Label::new("group label 2"),
///         Card::new(Default::default()) => {
///             Label::new("inner card label"),
///             Divider::new(false),
///         },
///     }],
///     Card::new(Default::default()) => {
///         Label::new("card label"),
///         Button::new("card button"),
///     },
/// });
/// 
/// assert_eq!(layout.widgets(), 12);
/// assert_eq!(layout.layers(), 3);
/// ```
/// 
/// is equivalent to:
/// 
/// ```
/// # use nablo_ui::prelude::*;
/// # let mut layout = Layout::new();
/// enum AppEvent {
///     Close,
///     Ok,
/// }
/// # impl Signal for AppEvent {}
/// 
/// layout.insert_root_widget(Card::new(Default::default()));
/// layout.add_widget(ROOT_LAYOUT_ID, Button::new("ok").on_click(|| AppEvent::Ok)).expect("missing root widget");
/// let id = layout.add_widget(ROOT_LAYOUT_ID, Button::new("cancel").on_click(|| AppEvent::Close)).expect("missing root widget");
/// layout.alias_widget(id, "label");
/// 
/// let collapse_id = layout.add_widget(ROOT_LAYOUT_ID, Collapse::new("collapse group")).expect("missing root widget");
/// layout.alias_widget(collapse_id, "group");
/// layout.add_widget(collapse_id, Label::new("group label")).expect("missing root widget");
/// layout.add_widget(collapse_id, Label::new("group label 2")).expect("missing root widget");
/// 
/// let inner_card_id = layout.add_widget(collapse_id, Card::new(Default::default())).expect("missing root widget");
/// layout.add_widget(inner_card_id, Label::new("inner card label")).expect("missing root widget");
/// layout.add_widget(inner_card_id, Divider::new(false)).expect("missing root widget");
/// 
/// let card_id = layout.add_widget(ROOT_LAYOUT_ID, Card::new(Default::default())).expect("missing root widget");
/// layout.add_widget(card_id, Label::new("card label")).expect("missing root widget");
/// layout.add_widget(card_id, Button::new("card button")).expect("missing root widget");
/// 
/// assert_eq!(layout.widgets(), 12);
/// assert_eq!(layout.layers(), 3);
/// ```
#[macro_export] macro_rules! layout_gen {
	($layout:expr $(, $root:expr)? => { $($children: tt)* }) => {{
		let __id = $crate::layout::ROOT_LAYOUT_ID;
		$(
			$layout.insert_root_widget($root);
		)?
		$crate::__inner_layout!(@process_children $layout, __id, $($children)*);
	}};
}

/// A macro to create the layout of the UI.
/// 
/// This macro will append children to the given parent widget.
/// 
/// The macro has following ways to use:
/// 1. `layout_append!(layout, @alias parent_alias (, parent_widget)? => { child1, child2, child3, ... })`: 
///    This will append children to the widget with the given alias.
/// 2. `layout_append!(layout, (@id)? parent_id (, parent_widget)? => { child1, child2, child3, ... })`: 
///    This will append children to the widget with the given id.
/// 
/// See more information in [`crate::layout_gen`].
#[macro_export] macro_rules! layout_append {
	($layout:expr, @alias $parent:expr $(, $root:expr)? => { $($children: tt)* }) => {{
		let __id = $layout.alias_to_id($parent).expect("missing alias");
		$(
			$layout.replace_widget(__id, $root);
		)?
		$crate::__inner_layout!(@process_children $layout, __id, $($children)*);
	}};

	($layout:expr, $(@id)? $parent:expr $(, $root:expr)? => { $($children: tt)* }) => {{
		let __id = $parent;
		$(
			$layout.replace_widget(__id, $root);
		)?
		$crate::__inner_layout!(@process_children $layout, __id, $($children)*);
	}};
}

#[doc(hidden)]
#[macro_export] macro_rules! __inner_layout {
	(@process_children $ctx_layout:expr, $parent:expr, $($child:tt)*) => {
		$crate::__inner_layout!(@process_child $ctx_layout, $parent, $($child)*);
	};

	(@process_child $ctx_layout:expr, $parent:expr, [ $alias:expr, $component: expr ], $($($rest: tt)+)?) => {{
		let __id = $ctx_layout.add_widget($parent, $component).expect("missing parent widget");
		$ctx_layout.alias_widget(__id, $alias);
		$(
			$crate::__inner_layout!(@process_children $ctx_layout, $parent, $($rest)*);
		)?
	}};

	(@process_child $ctx_layout:expr, $parent:expr, [ $alias:expr, $component: expr => { $($children:tt)* } ], $($($rest: tt)+)?) => {{
		let __id = $ctx_layout.add_widget($parent, $component).expect("missing parent widget");
		$ctx_layout.alias_widget(__id, $alias);
		$crate::__inner_layout!(@process_children $ctx_layout, __id, $($children)*);
		$(
			$crate::__inner_layout!(@process_children $ctx_layout, $parent, $($rest)*);
		)?
	}};
	
	(@process_child $ctx_layout:expr, $parent:expr, $component:expr, $($($rest: tt)+)?) => {
		$ctx_layout.add_widget($parent, $component).expect("missing parent widget");
		$(
			$crate::__inner_layout!(@process_children $ctx_layout, $parent, $($rest)*);
		)?
	};

	(@process_child $ctx_layout:expr, $parent:expr, $component:expr => { $($children:tt)* }, $($($rest: tt)+)?) => {{
		let __id = $ctx_layout.add_widget($parent, $component).expect("missing parent widget");
		$crate::__inner_layout!(@process_children $ctx_layout, __id, $($children)*);
		$(
			$crate::__inner_layout!(@process_children $ctx_layout, $parent, $($rest)*);
		)?
	}};
}