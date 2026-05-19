use glutin::config::ConfigTemplateBuilder;
use glutin_winit::DisplayBuilder;
use log::*;
use winit::{
	application::ApplicationHandler,
	event::{DeviceEvent, WindowEvent},
	event_loop::ActiveEventLoop,
};

use std::sync::OnceLock;

mod app;
use app::App;

mod background;

mod camera;

mod config;
use config::Config;

mod model;

mod playfield;

mod player;

mod shader;

struct WinitApp {
	preinit: Option<PreInitData>,
	app: OnceLock<App>,
}

struct PreInitData {
	config: Config,
	display_builder: DisplayBuilder,
	template: ConfigTemplateBuilder,
}

impl ApplicationHandler for WinitApp {
	fn resumed(&mut self, event_loop: &ActiveEventLoop) {
		let Some(PreInitData {
			config,
			display_builder,
			template,
		}) = self.preinit.take()
		else {
			return;
		};
		let _ = self
			.app
			.set(unsafe { App::init(event_loop, config, display_builder, template) });
	}

	fn device_event(
		&mut self,
		event_loop: &ActiveEventLoop,
		_device_id: winit::event::DeviceId,
		event: DeviceEvent,
	) {
		self.app
			.get_mut()
			.unwrap()
			.handle_device_event(event_loop, event);
	}

	fn window_event(
		&mut self,
		event_loop: &ActiveEventLoop,
		_window_id: winit::window::WindowId,
		event: WindowEvent,
	) {
		self.app
			.get_mut()
			.unwrap()
			.handle_window_event(event_loop, event);
	}
}

fn main() {
	env_logger::builder().filter_level(LevelFilter::Info).init();

	// Load config

	let config = std::fs::read_to_string("config.toml").unwrap();
	let config = Config::from_toml(&config);

	info!("Loaded config:\n{:#?}", config);

	// Create window

	let event_loop = winit::event_loop::EventLoop::builder().build().unwrap();
	let window_builder = winit::window::Window::default_attributes()
		.with_title("Hello triangle!")
		.with_inner_size(winit::dpi::LogicalSize::new(
			config.window.width as f32,
			config.window.height as f32,
		));

	let template = ConfigTemplateBuilder::new();

	let display_builder = DisplayBuilder::new().with_window_attributes(Some(window_builder));

	// Run the app

	let mut app = WinitApp {
		preinit: Some(PreInitData {
			config,
			display_builder,
			template,
		}),
		app: OnceLock::new(),
	};

	let _ = event_loop.run_app(&mut app);
}
