# Nablo UI

Nablo UI: 一个基于 Rust 的纯 SDF 渲染的跨平台 GUI 框架。

## 特性

### SDF 渲染

- 所有 UI 元素都使用 SDF 算法渲染，保证了高质量的渲染效果。
- 支持构建图形表达式（如交并差）以及旋转缩放等构建复杂形状。

### 组件化

- 组件化设计，允许开发者自定义 UI 元素，并通过组合的方式构建复杂的 UI 界面。
- 自定义 Widget 支持，可以深度定义组件行为

### 跨平台

- 支持 Windows、Linux、macOS、wasm, (计划支持 Android、iOS)
- 统一处理输入/输出事件，自适应不同设备交互模式

## 快速开始

### 简单的 Counter 示例

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

见 [Roadmap](./roadmap.md)

## 贡献

欢迎提交 PR，或者在 issues 中提出问题。

## License

MIT