use glutin::{config::{ConfigTemplateBuilder, GlConfig}, context::{ContextApi, ContextAttributesBuilder, PossiblyCurrentContext}, display::GetGlDisplay, prelude::{GlDisplay, NotCurrentGlContext}, surface::{Surface, WindowSurface}};
use glutin_winit::{DisplayBuilder, GlWindow};
use glow::*;
use log::*;
use raw_window_handle::HasWindowHandle;
use winit::{
	application::ApplicationHandler,
	event::{DeviceEvent, WindowEvent},
	event_loop::ActiveEventLoop, window::Window,
};

use std::sync::{Arc, OnceLock};

mod app;
use app::App;

mod background;

mod camera;

mod config;
use config::Config;

mod model;

mod playfield;

mod player;

mod screenshot;

mod shader;

mod transparent;

struct WinitApp {
	preinit: Option<PreInitData>,
	app: OnceLock<App>,
}

struct PreInitData {
	window: Window,
	gl: Arc<Context>,
	gl_context: PossiblyCurrentContext,
	gl_surface: Surface<WindowSurface>
}

impl ApplicationHandler for WinitApp {
	fn resumed(&mut self, event_loop: &ActiveEventLoop) {
		let Some(PreInitData {
			window,
			gl,
			gl_context,
			gl_surface
		}) = self.preinit.take()
		else {
			return;
		};
		let _ = self
			.app
			.set(App::init(event_loop, window, gl, gl_context, gl_surface));
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

fn opengl_callback(src: u32, kind: u32, id: u32, severity: u32, msg: &str) {
	let src = match src {
		DEBUG_SOURCE_API => "API",
		DEBUG_SOURCE_WINDOW_SYSTEM => "WINDOW SYSTEM",
		DEBUG_SOURCE_SHADER_COMPILER => "SHADER COMPILER",
		DEBUG_SOURCE_THIRD_PARTY => "THIRD PARTY",
		DEBUG_SOURCE_APPLICATION => "APPLICATION",
		DEBUG_SOURCE_OTHER => "OTHER",
		_ => "Unknown",
	};

	let kind = match kind {
		DEBUG_TYPE_ERROR => "ERROR",
		DEBUG_TYPE_DEPRECATED_BEHAVIOR => "DEPRECATED_BEHAVIOR",
		DEBUG_TYPE_UNDEFINED_BEHAVIOR => "UNDEFINED_BEHAVIOR",
		DEBUG_TYPE_PORTABILITY => "PORTABILITY",
		DEBUG_TYPE_PERFORMANCE => "PERFORMANCE",
		DEBUG_TYPE_MARKER => "MARKER",
		DEBUG_TYPE_OTHER => "OTHER",
		_ => "Unknown",
	};

	let severity = match severity {
		DEBUG_SEVERITY_NOTIFICATION => return,
		DEBUG_SEVERITY_LOW => "LOW",
		DEBUG_SEVERITY_MEDIUM => "MEDIUM",
		DEBUG_SEVERITY_HIGH => "HIGH",
		_ => "Unknown",
	};

	warn!(target: "GL", "{:?}", msg);
	warn!(target: "GL", " -> from {src}, kind {kind}, severity {severity}, id {id}");
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

	let (window, gl_config) = display_builder
		.build(&event_loop, template, |configs| {
			configs
				.reduce(|accum, new| {
					if new.num_samples() == config.graphics.antialiasing {
						new
					} else {
						accum
					}
				})
				.unwrap()
		})
		.unwrap();

	info!("Antialiasing level: {}", gl_config.num_samples().max(1));

	let raw_window_handle = window
		.as_ref()
		.and_then(|window| window.window_handle().map(Into::into).ok());

	// Inititalize OpenGL context

	let gl_display = gl_config.display();
	let context_attributes = ContextAttributesBuilder::new()
		.with_context_api(ContextApi::OpenGl(Some(glutin::context::Version {
			major: 4,
			minor: 6,
		})))
		.build(raw_window_handle);

	let window = window.unwrap();

	window.set_title("Triangle");
	window.set_visible(false);

	let (gl, gl_context, gl_surface) = unsafe {
		let not_current_gl_context = gl_display
			.create_context(&gl_config, &context_attributes)
			.unwrap();

		let attrs = window.build_surface_attributes(Default::default()).unwrap();
		let gl_surface = gl_display
			.create_window_surface(&gl_config, &attrs)
			.unwrap();

		let gl_context = not_current_gl_context.make_current(&gl_surface).unwrap();

		let mut gl = Context::from_loader_function_cstr(|s| gl_display.get_proc_address(s));

		gl.debug_message_callback(opengl_callback);
		gl.enable(DEBUG_OUTPUT);

		let gl = Arc::new(gl);

		(gl, gl_context, gl_surface)
	};

	// Run the app

	let mut app = WinitApp {
		preinit: Some(PreInitData {
			window,
			gl,
			gl_context,
			gl_surface
		}),
		app: OnceLock::new(),
	};

	let _ = event_loop.run_app(&mut app);
}
