`nablo_ui`: A simple pure SDF-based UI library for Rust.

# A Simple Counter Example
```rust, no_run
const FONT: &[u8] = include_bytes!("path/to/your/font.ttf");

use nablo_ui::prelude::*;

#[derive(Default)]
struct Counter {
    data: isize,
}

impl App for Counter {
	type Signal = ();

    fn on_start(&mut self, ctx: &mut Context<(), Self>) {
        new_layout!(ctx.layout, Card::new(LayoutStrategy::default())
            .rounding(Vec4::same(16.0))
            .padding(Vec2::same(16.0)) => 
        {
            Reactive::new(Label::new("Counter: 0"), |app: &mut Self, inner| {
                inner.text(format!("Counter: {}", app.data))
            }),
            Button::new("increase").on_click(|app: &mut Self, _| app.data += 1),
            Button::new("decrease").on_click(|app: &mut Self, _| app.data -= 1),
        });
    }

	fn on_signal(&mut self, _: &mut Context<Self::Signal, Self>, _: SignalWrapper<Self::Signal>) {}
}

fn main() {
    Manager::new(Counter::default(), FONT.to_vec(), 0).title("Counter Example").run();
}
```