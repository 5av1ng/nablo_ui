`nablo_ui`: A simple pure SDF-based UI library for Rust.

# A Simple Counter Example
```rust, no_run
const FONT: &[u8] = include_bytes!("path/to/your/font.ttf");

use nablo_ui::prelude::*;
use std::rc::Rc;
use std::cell::RefCell;

#[derive(Default)]
struct Counter {
    data: Rc<RefCell<isize>>,
}

enum Msg {
    Increment,
    Decrement,
}

impl Signal for Msg {}

impl App<Msg> for Counter {
    fn on_start(&mut self, ctx: &mut Context<Msg>) {
        let data = self.data.clone();
        new_layout!(ctx.layout, Card::new(LayoutStrategy::default())
            .rounding(Vec4::same(16.0))
            .padding(Vec2::same(16.0)) => 
        {
            Reactive::new(Label::new("Counter: 0"), move |inner| {
                if let Ok(data) = data.try_borrow() {
                    inner.text(format!("Counter: {}", data))
                }else {
                    inner
                }
            }),
            Button::new("increase").on_click(|_| Msg::Increment),
            Button::new("decrease").on_click(|_| Msg::Decrement),
        });
    }

    fn on_signal(&mut self, _: &mut Context<Msg>, signal: SignalWrapper<Msg>) {
        match signal.signal {
            Msg::Increment => {
                *self.data.borrow_mut() += 1;
            },
            Msg::Decrement => {
                *self.data.borrow_mut() -= 1;
            }
        }
    }
}

fn main() {
    Manager::new(Counter::default(), FONT.to_vec(), 0).title("Counter Example").run();
}
```