use std::{cell::RefCell, rc::Rc};

use nablo_ui::{prelude::*, widgets::inputbox::{InputBox, SimpleValidator}};
use time::Duration;

#[derive(Debug, Clone, Default)]
struct TestApp {
	data: Rc<RefCell<Data>>
}

#[derive(Debug, Clone, Default)]
struct Data {
	value: u32,
	last_frame: Duration,
}

#[derive(Debug, Clone)]
enum Sig {
	None,
	BgClicked,
	Clicked,
	SwitchPassword,
	DoubleClicked,
	OpenFLoatContainer
}

impl Signal for Sig {}

impl App<Sig> for TestApp {
	fn on_start(&mut self, ctx: &mut Context<Sig>) {
		ctx.set_advance_factor(0, 0.75);
		let data = self.data.clone();

		new_layout!(ctx.layout, Card::new(LayoutStrategy::default())
			.rounding(Vec4::same(16.0))
			.on_click(|_| Sig::BgClicked)
			.padding(Vec2::same(16.0))
			.scroll(Scroll::both())
			.pin_child(LayoutId(1), Vec2::ZERO) => 
		{
			["Float", FloatingContainer::new().draggable(true) => { Card::new(LayoutStrategy::default())
				.rounding(Vec4::same(16.0))
				.padding(Vec2::same(16.0))
				.set_size(Vec2::new(216.0, 96.0)) => {
					Label::new("这是一个浮动容器"),
					Button::new("这是一个浮动容器的按钮").on_click(|_| Sig::Clicked),
				},
			}],
			Label::title("nablo 内置 widgets 一览"),
			Label::new("内置的按钮"),
			Card::new_horizontal().dont_draw(true).padding(Vec2::same(16.0)).alignments([Alignment::Center, Alignment::Center]) => {
				Button::new("主要按钮").on_click(|_| Sig::Clicked).on_double_click(|_| Sig::DoubleClicked),
				Button::new("次要按钮").on_click(|_| Sig::Clicked).style(ButtonStyle::Secondary),
				Button::new("文字按钮").on_click(|_| Sig::Clicked).style(ButtonStyle::Text),
				Button::new("废掉的按钮").on_click(|_| Sig::Clicked).style(ButtonStyle::Disabled),
			},
			Reactive::new(Label::new("你点了 0 次按钮"), move |inner| {
				if let Ok(data) = data.try_borrow() {
					inner.text(format!("你点了 {} 次按钮", data.value))
				}else {
					inner
				}
			}),
			Divider::new(false),
			Label::new("内置的单选"),
			Card::new_horizontal().dont_draw(true).padding(Vec2::same(16.0)).alignments([Alignment::Center, Alignment::Center]) => {
				Radio::new_radio("单选"),
				Radio::new_check_box("多选"),
				Radio::new_switch("开关"),
				Radio::new_button("按钮选"),
			},
			Divider::new(false),
			Label::new("内置的能拖的玩意"),
			Slider::new(1.0, 1.0, 100.0).reverse(true).prefix("带前缀 "),
			Slider::new(1.0, 1.0, 100.0).logarithmic(true).suffix(" 带后缀"),
			DraggableValue::new(1.0, 1.0, 100.0).logarithmic(true).suffix(" 对数刻度"),
			Divider::new(false),
			Collapse::new("内置的折叠面板") => {
				Label::new("这是一个折叠面板的内容"),
				Button::new("这是个被折叠的按钮").on_click(|_| Sig::Clicked).style(ButtonStyle::Secondary),
				Card::new(LayoutStrategy::default())
					.set_size(Vec2::new(128.0 * 3.0, 128.0))
					.padding(Vec2::y(16.0))
					.dont_draw(true)
					.draw_stroke(false) => 
				{
					Label::new("This is the content of the inner card"),
					Label::new("这是一个内嵌卡片的内容"),
					Button::new("这是个内嵌卡片的按钮").on_click(|_| Sig::Clicked).style(ButtonStyle::Text),
				},
			},
			Divider::new(false),
			Label::new("内置的输入框"),
			InputBox::new(0, EM).placeholder("啥也没有").validator(SimpleValidator {
				allow_breakline: false,
				limit: Some(20),
				..Default::default()
			}),
			InputBox::new(0, EM).placeholder("点了会填充神奇文字").validator(SimpleValidator {
				allow_breakline: false,
				limit: Some(20),
				..Default::default()
			}).on_click(|inner| {
				inner.text = "神奇文字".to_string();
				inner.pointer = Pointer::new(4);
				Sig::None
			}),
			["Password", InputBox::new(0, EM).placeholder("这是密码").validator(SimpleValidator {
				allow_breakline: false,
				limit: Some(20),
				..Default::default()
			}).password(true)],
			InputBox::new(0, EM).placeholder("你用不了").validator(SimpleValidator {
				banned: true,
				..Default::default()
			}),
			Radio::new_radio("显示密码").on_click(|_| Sig::SwitchPassword),
			Divider::new(false),
			Radio::new_radio("打开浮动容器").on_click(|_| Sig::OpenFLoatContainer),
			["painter", Canvas::new(Vec2::same(256.0), |_| {}, true)],
			["progress_bar", ProgressBar::new().set_length(256.0)],
			["fps", Label::new("fps: 0.00")],
		});

		println!("Starting...");
	}

	fn on_exit(&mut self, _ctx: &mut Context<Sig>) {
		println!("Exiting...");
	}

	fn on_draw_frame(&mut self, ctx: &mut Context<Sig>) {
		let current = ctx.input_state().program_running_time();
		let t= current.as_seconds_f32().sin() * 0.5 + 0.5; 
		let t2= (current.as_seconds_f32() / 2.0).sin() * 0.5 + 0.5; 
		let t4= (current.as_seconds_f32() / 4.0).sin() * 0.5 + 0.5; 
		if let Ok(mut data) = self.data.try_borrow_mut() {
			let delta = current - data.last_frame;
			data.last_frame = current;
			ctx.layout.widget_mut_by_alias::<Label<Sig>>("fps", |inner| {
				inner.text(format!("fps: {:.2}", 1.0 / delta.as_seconds_f32()))
			});
		}

		ctx.layout.widget_mut_by_alias::<ProgressBar<Sig>>("progress_bar", |inner| {
			inner
				.set_progress_without_animation(t)
				.set_foreground_color(PRIMARY_COLOR.lerp(SUCCESS_COLOR, t))
		});

		ctx.layout.widget_mut_by_alias::<Canvas<Sig>>("painter", |_| {
			Canvas::new(Vec2::same(256.0), move |painter| {
				painter.set_fill_mode(
					FillMode::LinearGradient(
						ERROR_COLOR, 
						WARNING_COLOR, 
						Vec2::ZERO, 
						Vec2::same(256.0)
					)
				);
				
				painter.draw_rect(Rect::from_size(Vec2::same(256.0)), Vec4::same(16.0));

				painter.set_fill_mode(
					FillMode::RadialGradient(
						PRIMARY_COLOR, 
						SUCCESS_COLOR, 
						Vec2::same(128.0), 
						192.0
					)
				);
				
				painter.draw_shape(
					(Shape::from(BasicShapeData::Circle(Vec2::same(256.0) * t2, 32.0)))
					.lerp(Shape::from(BasicShapeData::Rectangle(Vec2::ZERO, Vec2::same(256.0), Vec4::same(96.0) * t4)), t)
				);

				// painter.set_fill_mode(PRIMARY_TEXT_COLOR);

				// painter.draw_text(Vec2::new(0.0, 128.0), 0, 16.0, "这个颜色还挺不错");
			}, true)
		});
	}

	fn on_signal(&mut self, ctx: &mut Context<Sig>, signal: SignalWrapper<Sig>) {
		if matches!(signal.signal, Sig::Clicked) {
			self.data.borrow_mut().value += 1;
		}

		if matches!(signal.signal, Sig::SwitchPassword) {
			ctx.layout.widget_mut_by_alias::<InputBox<Sig>>("Password", |mut inner| {
				inner.inner.password = !inner.inner.password;
				inner
			});
		}

		if matches!(signal.signal, Sig::OpenFLoatContainer) {
			ctx.layout.widget_mut_by_alias::<FloatingContainer<Sig>>("Float", |mut inner| {
				inner.inner.show = !inner.inner.show;
				inner.reset_context();
				inner
			});
		}

		println!("Received signal: {:?} from {}.", signal.signal, signal.from);
	}
}

fn main() {
	Manager::new(TestApp::default(), include_bytes!("../Maple.ttf").to_vec(), 0)
		.title("Test")
		.draw_frame_rate(60.0)
		.run();
}