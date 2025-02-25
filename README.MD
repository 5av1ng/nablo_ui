# Nablo UI

Nablo UI: A simple pure SDF-based UI library for Rust.
[中文文档见此](./README_CN.MD)

## Features

### SDF Rendering

- All Ui elements are rendered using SDF algorithm, ensuring high-quality rendering effect.
- Support building complex shapes by combining simple shapes (such as union, intersection, and difference) and transformations (such as rotation and scaling).

### Customizable Widgets

- Widgets are designed in a component-based way, allowing developers to customize UI elements and build complex UI interfaces by combining simple shapes and transformations.
- Customizable Widgets support, allowing developers to deeply define component behaviors.

### Cross-platform

- Support Windows, Linux, macOS, and wasm (to be supported in the future: Android, iOS)
- Unified handling of input/output events, adapting to different device interaction modes.

## Getting Started

### A Simple Counter Example

```rust
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

## Roadmap

See [Roadmap](./roadmap.md)

## Contribute

Currently, Nablo UI is in an early stage of development, and there are many features and improvements to be implemented. If you are interested in contributing to Nablo UI, please feel free to open an issue or pull request.

## License

MIT