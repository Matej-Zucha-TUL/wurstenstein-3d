use glutin::{
	config::{ConfigTemplateBuilder, GlConfig},
	context::{ContextApi, ContextAttributesBuilder, NotCurrentGlContext},
	display::{GetGlDisplay, GlDisplay},
	surface::{GlSurface, SwapInterval},
};
use winit::{
	event::{DeviceEvent, ElementState, Event, WindowEvent},
	keyboard::Key, window::{CursorGrabMode, Fullscreen, Window}
};
use glutin_winit::{DisplayBuilder, GlWindow};
use raw_window_handle::HasWindowHandle;
use glow::*;
use log::*;

use std::num::NonZeroU32;
use std::time::SystemTime;
use std::sync::Arc;

mod config;
use config::Config;

mod shader;
use shader::{ProgramBuilder, ShaderType};

fn lock_cursor(window: &Window) {
	if let Err(err) = window.set_cursor_grab(CursorGrabMode::Confined)
		.or_else(|_| window.set_cursor_grab(CursorGrabMode::Locked))
	{
		log::error!("Could not enable cursor grab: {}", err);
	}
}

fn unlock_cursor(window: &Window) {
	if let Err(err) = window.set_cursor_grab(CursorGrabMode::None) {
		log::error!("Could not disable cursor grab: {}", err);
	}
}

fn main() {
	env_logger::builder().filter_level(log::LevelFilter::Info).init();

	// Load config

	let config = std::fs::read_to_string("config.toml").unwrap();
	let config = Config::from_toml(&config);

	log::info!("Loaded config:\n{:#?}", config);

	// Create window

	let event_loop = winit::event_loop::EventLoop::builder().build().unwrap();
	let window_builder = winit::window::Window::default_attributes()
		.with_title("Hello triangle!")
		.with_inner_size(winit::dpi::LogicalSize::new(config.window.width as f32, config.window.height as f32));

	let template = ConfigTemplateBuilder::new();

	let display_builder = DisplayBuilder::new().with_window_attributes(Some(window_builder));

	let (window, gl_config) = display_builder.build(&event_loop, template, |configs| {
		configs.reduce(|accum, config| {
			if config.num_samples() > accum.num_samples() {
				config
			} else {
				accum
			}
		}).unwrap()
	}).unwrap();

	let raw_window_handle = window
		.as_ref()
		.and_then(|window| window.window_handle().map(Into::into).ok());

	unsafe {
		// Inititalize OpenGL context

		let gl_display = gl_config.display();
		let context_attributes = ContextAttributesBuilder::new()
			.with_context_api(ContextApi::OpenGl(Some(glutin::context::Version {
				major: 3,
				minor: 3,
			})))
			.build(raw_window_handle);

		let not_current_gl_context = gl_display.create_context(&gl_config, &context_attributes).unwrap();

		let window = window.unwrap();

		window.set_title("Triangle");
		window.set_visible(false);

		let attrs = window.build_surface_attributes(Default::default()).unwrap();
		let gl_surface = gl_display.create_window_surface(&gl_config, &attrs).unwrap();

		let gl_context = not_current_gl_context.make_current(&gl_surface).unwrap();

		let gl = glow::Context::from_loader_function_cstr(|s| gl_display.get_proc_address(s));
		let gl = Arc::new(gl);

		gl_surface.set_swap_interval(&gl_context, SwapInterval::Wait(NonZeroU32::new(1).unwrap())).unwrap();

		let vertex_array = gl.create_vertex_array().unwrap();

		// Load shaders

		let program = ProgramBuilder::new(gl.clone())
			.add_shader(ShaderType::Vertex, include_str!("./../assets/shaders/vert/main.vert"))
			.add_shader(ShaderType::Fragment, include_str!("./../assets/shaders/frag/main.frag"))
			.link();

		let mut last_time = SystemTime::now();
		let mut last_update = SystemTime::now();
		let fps_update_interval_secs = 0.5;

		let mut egui = None;
		let mut fps_string = "FPS = ???".to_string();
		let mut vsync = true;
		let mut fullscreen = false;
		let mut cursor_lock = true;
		let mut background_color = [0.1, 0.2, 0.3, 1.0];
		let mut triangle_color = [0.5, 0.5, 0.5, 1.0];
		let mut file_dialog = egui_file_dialog::FileDialog::new().movable(false).resizable(false).anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0));
		let mut cursor_x = 0.0;
		let mut cursor_y = 0.0;

		if cursor_lock {
			lock_cursor(&window);
		} else {
			unlock_cursor(&window);
		}

		window.set_visible(true);

		#[allow(deprecated)] // Fuck you.
		let _ = event_loop.run(move |event, event_loop| {
			let middle_point = winit::dpi::LogicalPosition::new(window.inner_size().width / 2, window.inner_size().height / 2);

			if egui.is_none() {
				egui = Some(egui_glow::EguiGlow::new(event_loop, gl.clone(), None, None, true));
			}

			// We're capturing the mouse movement through DeviceEvents, as they provide direct relative coordinates straight from the mouse.
			if let Event::DeviceEvent { event, .. } = &event
				&& let DeviceEvent::MouseMotion { delta } = event
				&& cursor_lock
			{
				cursor_x += delta.0;
				cursor_y += delta.1;
			}

			if let Event::WindowEvent { window_id: _, event } = event {
				let _ = egui.as_mut().unwrap().on_window_event(&window, &event);

				match event {
					WindowEvent::KeyboardInput { event, .. } => {
						log::info!("{:?} key: {:?}, repeat: {:?}", event.state, event.physical_key, event.repeat);
						if event.state == ElementState::Pressed
							&& !event.repeat
							&& event.logical_key == Key::Character("v".into())
						{
							vsync = !vsync;
							info!("VSync = {}", vsync);
						}

						if event.state == ElementState::Pressed
							&& !event.repeat
							&& event.logical_key == Key::Character("f".into())
						{
							fullscreen = !fullscreen;
							info!("Fullscreen = {}", fullscreen);
						}

						if event.state == ElementState::Pressed
							&& !event.repeat
							&& event.logical_key == Key::Character("l".into())
						{
							cursor_lock = !cursor_lock;
							info!("Cursor lock = {}", cursor_lock);

							if cursor_lock {
								lock_cursor(&window);
							} else {
								unlock_cursor(&window);
							}
						}
					},
					WindowEvent::MouseInput { state, button, .. } => {
						info!("Mouse - {:?} {:?}", state, button);
					},
					WindowEvent::CloseRequested => {
						event_loop.exit();
					},
					WindowEvent::Resized(new_size) => {
						gl_surface.resize(
							&gl_context,
							new_size.width.try_into().unwrap(),
							new_size.height.try_into().unwrap()
						);
					},
					WindowEvent::RedrawRequested => {
						if cursor_lock {
							window.set_cursor_visible(false);
							let _ = window.set_cursor_position(middle_point);
						} else {
							window.set_cursor_visible(true);
						}

						gl.bind_vertex_array(Some(vertex_array));
						program.activate();
						program.set_uniform_f32_4("selected_color", &triangle_color);

						let [r, g, b, a] = background_color;
						gl.clear_color(r, g, b, a);
						gl.clear(glow::COLOR_BUFFER_BIT);

						gl.draw_arrays(glow::TRIANGLES, 0, 3);

						egui
							.as_mut()
							.unwrap()
							.run(&window, |ctx| {
								egui::Window::new("Wokýnko").resizable(false).show(ctx, |ui| {
									ui.label(&fps_string);

									ui.label(format!("Cursor X: {}", cursor_x));
									ui.label(format!("Cursor Y: {}", cursor_y));

									if cursor_lock {
										ui.label("Cursor is locked.");
										return
									}

									ui.checkbox(&mut vsync, "Enable Vsync");

									if ui.button("Pick model").clicked() {
										file_dialog.pick_file();
									}

									ui.horizontal(|ui| {
										ui.color_edit_button_rgb((&mut triangle_color[..3]).try_into().unwrap());
										ui.label("Triangle");
									});

									ui.horizontal(|ui| {
										ui.color_edit_button_rgb((&mut background_color[..3]).try_into().unwrap());
										ui.label("Background");
									});

									if ui.button("Quit").clicked() {
										event_loop.exit();
									}
								});

								file_dialog.update(ctx);
							});

						egui.as_mut().unwrap().paint(&window);

						gl_surface.swap_buffers(&gl_context).unwrap();

						let new_time = SystemTime::now();
						let frame_dur = 1.0 / new_time.duration_since(last_time).unwrap().as_secs_f32();
						last_time = new_time;

						if last_update.elapsed().unwrap().as_secs_f32() >= fps_update_interval_secs {
							fps_string = format!("FPS = {:.1}", frame_dur);
							let vsync_string = format!("VSync = {}", if vsync { "on" } else { "off" });
							let cursor_lock_string = format!("Cursor lock = {}", if cursor_lock { "on" } else { "off" });
							window.set_title(&format!("Triangle - {}, {}, {}", fps_string, vsync_string, cursor_lock_string));
							info!("{}, {}, {}", fps_string, vsync_string, cursor_lock_string);
							last_update = SystemTime::now();
						}

						window.request_redraw();

						window.set_fullscreen(fullscreen.then_some(Fullscreen::Borderless(window.current_monitor())));

						if vsync {
							gl_surface.set_swap_interval(&gl_context, SwapInterval::Wait(NonZeroU32::new(1).unwrap())).unwrap();
						} else {
							gl_surface.set_swap_interval(&gl_context, SwapInterval::DontWait).unwrap();
						}
					},
					_ => {},
				}
			}
		});
	}
}
