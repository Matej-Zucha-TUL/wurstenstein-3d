use glutin::{
	config::{ConfigTemplateBuilder, GlConfig},
	context::{ContextApi, ContextAttributesBuilder, NotCurrentGlContext},
	display::{GetGlDisplay, GlDisplay},
	surface::{GlSurface, SwapInterval},
};
use winit::{
	event::{ElementState, Event, WindowEvent},
	keyboard::Key, window::Fullscreen
};
use glutin_winit::{DisplayBuilder, GlWindow};
use raw_window_handle::HasWindowHandle;
use glow::*;
use log::*;
use image::ImageReader;

use std::num::NonZeroU32;
use std::time::SystemTime;
use std::sync::Arc;

fn main() {
	env_logger::builder().filter_level(log::LevelFilter::Info).init();

	// Create window

	let event_loop = winit::event_loop::EventLoop::builder().build().unwrap();
	let window_builder = winit::window::Window::default_attributes()
		.with_title("Hello triangle!")
		.with_inner_size(winit::dpi::LogicalSize::new(1024.0, 768.0));

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

		let attrs = window.build_surface_attributes(Default::default()).unwrap();
		let gl_surface = gl_display.create_window_surface(&gl_config, &attrs).unwrap();

		let gl_context = not_current_gl_context.make_current(&gl_surface).unwrap();

		let gl = glow::Context::from_loader_function_cstr(|s| gl_display.get_proc_address(s));

		gl_surface.set_swap_interval(&gl_context, SwapInterval::Wait(NonZeroU32::new(1).unwrap())).unwrap();

		let vertex_array = gl.create_vertex_array().unwrap();

		// Load shaders

		let program = gl.create_program().unwrap();

		let shader_sources = [
			(glow::VERTEX_SHADER, include_str!("main.vert")),
			(glow::FRAGMENT_SHADER, include_str!("main.frag")),
		];

		let shaders = shader_sources.map(|(shader_type, shader_source)| {
			let shader = gl.create_shader(shader_type).unwrap();

			gl.shader_source(shader, shader_source);
			gl.compile_shader(shader);

			if !gl.get_shader_compile_status(shader) {
				panic!("{}", gl.get_shader_info_log(shader));
			}

			gl.attach_shader(program, shader);

			shader
		});

		gl.link_program(program);
		if !gl.get_program_link_status(program) {
			panic!("{}", gl.get_program_info_log(program));
		}

		for shader in shaders {
			gl.detach_shader(program, shader);
			gl.delete_shader(shader);
		}

		gl.clear_color(0.1, 0.2, 0.3, 1.0);

		let img = ImageReader::open("./assets/textures/ferris.png").unwrap().decode().unwrap().into_rgb8();
		let imgdata = PixelUnpackData::Slice(Some(img.as_raw()));

		let tex = gl.create_texture().unwrap();
		gl.bind_texture(TEXTURE_2D, Some(tex));
		gl.tex_image_2d(TEXTURE_2D, 0, RGB as i32, img.width() as i32, img.height() as i32, 0, RGB, UNSIGNED_BYTE, imgdata);

		let mut last_time = SystemTime::now();
		let mut last_update = SystemTime::now();
		let fps_update_interval_secs = 0.5;

		let gl = Arc::new(gl);

		let mut egui = None;
		let mut fps_string = String::new();
		let mut vsync = true;
		let mut fullscreen = true;
		let mut triangle_color = [0.5; 3];

		#[allow(deprecated)] // Fuck you.
		let _ = event_loop.run(move |event, event_loop| {
			if egui.is_none() {
				egui = Some(egui_glow::EguiGlow::new(event_loop, gl.clone(), None, None, true));
			}

			let Event::WindowEvent { window_id: _, event } = event else {
				return
			};

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
					gl.bind_vertex_array(Some(vertex_array));
					gl.use_program(Some(program));
					gl.clear(glow::COLOR_BUFFER_BIT);
					gl.draw_arrays(glow::TRIANGLES, 0, 3);

					egui
						.as_mut()
						.unwrap()
						.run(&window, |ctx| {
							egui::Window::new("Wokýnko").resizable(false).show(ctx, |ui| {
								ui.heading("Hello World!");
								ui.label(&fps_string);
								ui.checkbox(&mut vsync, "Enable Vsync");
								ui.color_edit_button_rgb(&mut triangle_color);
								if ui.button("Quit").clicked() {
									event_loop.exit();
								}
							});
						});

					egui.as_mut().unwrap().paint(&window);

					gl_surface.swap_buffers(&gl_context).unwrap();

					let new_time = SystemTime::now();
					let frame_dur = 1.0 / new_time.duration_since(last_time).unwrap().as_secs_f32();
					last_time = new_time;

					if last_update.elapsed().unwrap().as_secs_f32() >= fps_update_interval_secs {
						fps_string = format!("FPS = {:.1}", frame_dur);
						window.set_title(&format!("Triangle - {}", fps_string));
						info!("{}", fps_string);
						last_update = SystemTime::now();
					}

					window.request_redraw();
				},
				_ => (),
			}

			window.set_fullscreen(fullscreen.then_some(Fullscreen::Borderless(window.current_monitor())));

			if vsync {
				gl_surface.set_swap_interval(&gl_context, SwapInterval::Wait(NonZeroU32::new(1).unwrap())).unwrap();
			} else {
				gl_surface.set_swap_interval(&gl_context, SwapInterval::DontWait).unwrap();
			}
		});
	}
}
