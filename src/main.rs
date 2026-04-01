use glutin::{
	config::{ConfigTemplateBuilder, GlConfig},
	context::{ContextApi, ContextAttributesBuilder, NotCurrentGlContext},
	display::{GetGlDisplay, GlDisplay},
	surface::{GlSurface, SwapInterval},
};
use image::{ExtendedColorType, ImageEncoder, codecs::png::PngEncoder};
use winit::{
	event::{DeviceEvent, ElementState, Event, WindowEvent},
	keyboard::{Key, KeyCode, PhysicalKey}, window::{CursorGrabMode, Fullscreen, Window}
};
use glutin_winit::{DisplayBuilder, GlWindow};
use raw_window_handle::HasWindowHandle;
use glow::*;
use log::*;
use nalgebra_glm as glm;

use std::{io::Cursor, num::NonZeroU32};
use std::time::SystemTime;
use std::sync::Arc;

mod camera;
use camera::Camera;

mod config;
use config::Config;

mod shader;
use shader::{ProgramBuilder, ShaderType};

use crate::camera::Directions;

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

fn screenshot(gl: &Context, window: &Window) {
	let width = window.inner_size().width as usize;
	let height = window.inner_size().height as usize;

	let mut buf = vec![0; width * height * 4];

	unsafe {
		gl.read_pixels(0, 0, width as i32, height as i32, RGBA, UNSIGNED_BYTE, PixelPackData::Slice(Some(buf.as_mut_slice())));
	}

	let mut out_file = std::fs::File::create("skibidi.png").unwrap();

	let buf = buf.chunks(width * 4)
		.rev()
		.flatten()
		.copied()
		.collect::<Vec<_>>();

	let encoder = PngEncoder::new(&mut out_file);
	encoder.write_image(buf.as_slice(), width as u32, height as u32, ExtendedColorType::Rgba8).unwrap();
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
			if config.num_samples() == 4 {
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

		gl.enable(glow::CULL_FACE);
		gl.cull_face(glow::FRONT);
		gl.front_face(glow::CCW);

		gl.enable(glow::DEPTH_TEST);

		gl_surface.set_swap_interval(&gl_context, SwapInterval::Wait(NonZeroU32::new(1).unwrap())).unwrap();

		// Load shaders

		let program = ProgramBuilder::new(gl.clone())
			.add_shader(ShaderType::Vertex, include_str!("./../assets/shaders/vert/main.vert"))
			.add_shader(ShaderType::Fragment, include_str!("./../assets/shaders/frag/main.frag"))
			.link();

		let mut model_data = Cursor::new(include_bytes!("./../assets/objects/teapot_tri_vnt.obj"));
		let (model, _) = tobj::load_obj_buf(&mut model_data, &tobj::GPU_LOAD_OPTIONS, |_| Err(tobj::LoadError::ReadError)).unwrap();
		let model = model.into_iter().next().unwrap();
		let mesh = model.mesh;

		let vao = Some(gl.create_vertex_array().unwrap());
		let vbo_position = Some(gl.create_buffer().unwrap());
		let vbo_normal = Some(gl.create_buffer().unwrap());
		let ebo = Some(gl.create_buffer().unwrap());

		gl.bind_vertex_array(vao);

		gl.bind_buffer(glow::ARRAY_BUFFER, vbo_position);
		gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, bytemuck::cast_slice(&mesh.positions), glow::STATIC_DRAW);
		gl.vertex_attrib_pointer_f32(0, 3, glow::FLOAT, false, 0, 0);
		gl.enable_vertex_attrib_array(0);

		gl.bind_buffer(glow::ARRAY_BUFFER, vbo_normal);
		gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, bytemuck::cast_slice(&mesh.normals), glow::STATIC_DRAW);
		gl.vertex_attrib_pointer_f32(1, 3, glow::FLOAT, false, 0, 0);
		gl.enable_vertex_attrib_array(1);

		gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, ebo);
		gl.buffer_data_u8_slice(glow::ELEMENT_ARRAY_BUFFER, bytemuck::cast_slice(&mesh.indices), glow::STATIC_DRAW);

		gl.bind_vertex_array(None);

		let mut camera = Camera::new(glm::vec3(0.0, 0.0, 10.0));

		let start_time = SystemTime::now();
		let mut last_time = start_time;
		let mut last_update = start_time;
		let fps_update_interval_secs = 0.5;

		let mut egui = None;
		let mut fps_string = "FPS = ???".to_string();
		let mut vsync = true;
		let mut fullscreen = false;
		let mut cursor_lock = true;
		let mut background_color = [0.1, 0.2, 0.3, 1.0];
		let mut triangle_color = [0.5, 0.5, 0.5, 1.0];
		let mut rizz_mode = false;
    let mut pov_camera = false;
		let mut file_dialog = egui_file_dialog::FileDialog::new().movable(false).resizable(false).anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0));
		let mut cursor_x = 0.0;
		let mut cursor_y = 0.0;

		let mut move_forward = false;
		let mut move_backward = false;
		let mut move_left = false;
		let mut move_right = false;
		let mut move_fast = false;

		let mut model_rotate = 0.0;

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
				camera.mouse_interact(delta.0 as f32, delta.1 as f32);
				cursor_x += delta.0;
				cursor_y += delta.1;
			}

			if let Event::WindowEvent { window_id: _, event } = event {
				let _ = egui.as_mut().unwrap().on_window_event(&window, &event);

				match event {
					WindowEvent::KeyboardInput { event, .. } if !event.repeat => {
						log::info!("{:?} key: {:?}", event.state, event.physical_key);

						if event.state == ElementState::Pressed {
							if event.logical_key == Key::Character("v".into()) {
								vsync = !vsync;
								info!("VSync = {}", vsync);
							}

							if event.logical_key == Key::Character("p".into()) {
								pov_camera = !pov_camera;
								info!("POV camera = {}", pov_camera);
							}

							if event.logical_key == Key::Character("f".into()) {
								fullscreen = !fullscreen;
								info!("Fullscreen = {}", fullscreen);
							}

							if event.logical_key == Key::Character("o".into()) {
								screenshot(&gl, &window);
							}

							if event.logical_key == Key::Character("l".into()) {
								cursor_lock = !cursor_lock;
								info!("Cursor lock = {}", cursor_lock);

								if cursor_lock {
									lock_cursor(&window);
								} else {
									unlock_cursor(&window);
								}
							}

							if event.physical_key == PhysicalKey::Code(KeyCode::ShiftLeft) {
								move_fast = true;
							}

							if event.physical_key == PhysicalKey::Code(KeyCode::KeyW) {
								move_forward = true;
							}

							if event.physical_key == PhysicalKey::Code(KeyCode::KeyS) {
								move_backward = true;
							}

							if event.physical_key == PhysicalKey::Code(KeyCode::KeyA) {
								move_left = true;
							}

							if event.physical_key == PhysicalKey::Code(KeyCode::KeyD) {
								move_right = true;
							}
						}

						if event.state == ElementState::Released {
							if event.physical_key == PhysicalKey::Code(KeyCode::ShiftLeft) {
								move_fast = false;
							}

							if event.physical_key == PhysicalKey::Code(KeyCode::KeyW) {
								move_forward = false;
							}

							if event.physical_key == PhysicalKey::Code(KeyCode::KeyS) {
								move_backward = false;
							}

							if event.physical_key == PhysicalKey::Code(KeyCode::KeyA) {
								move_left = false;
							}

							if event.physical_key == PhysicalKey::Code(KeyCode::KeyD) {
								move_right = false;
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
						let new_time = SystemTime::now();
						let dt = new_time.duration_since(last_time).unwrap().as_secs_f32();
						last_time = new_time;

						if cursor_lock {
							window.set_cursor_visible(false);
							let _ = window.set_cursor_position(middle_point);
						} else {
							window.set_cursor_visible(true);
						}

            camera.set_pov(pov_camera);

						{
							let dt = if move_fast { dt * 3.0 } else { dt };

							if move_forward {
								camera.key_interact(Directions::Forward, dt);
							}

							if move_backward {
								camera.key_interact(Directions::Backward, dt);
							}

							if move_left {
								camera.key_interact(Directions::Left, dt);
							}

							if move_right {
								camera.key_interact(Directions::Right, dt);
							}
						}

						let [r, g, b, a] = background_color;

						gl.clear_color(r, g, b, a);
						gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);

						gl.bind_vertex_array(vao);

						model_rotate += dt * 50.0;

						let aspect = window.inner_size().width as f32 / window.inner_size().height as f32;
						let projection_mtx = glm::perspective(aspect, camera.get_zoom().to_radians(), 0.1f32, 100.0f32);
						let model_mtx = glm::translate(&glm::Mat4::identity(), &glm::vec3(0.0, 0.0, -10.0));
						let model_mtx = glm::rotate_y(&model_mtx, model_rotate.to_radians());

						program.activate();
						program.set_uniform_f32_4("selected_color", &triangle_color);
						program.set_uniform_matrix_f32_4("view", camera.get_view_matrix().as_slice().try_into().unwrap());
						program.set_uniform_matrix_f32_4("projection", projection_mtx.as_slice().try_into().unwrap());
						program.set_uniform_matrix_f32_4("model", model_mtx.as_slice().try_into().unwrap());
						program.set_uniform_f32("screen_w", window.inner_size().width as f32);
						program.set_uniform_f32("screen_h", window.inner_size().height as f32);
						program.set_uniform_f32("time", SystemTime::now().duration_since(start_time).unwrap().as_secs_f32());
						program.set_uniform_u32("rizz_mode", rizz_mode as u32);

						gl.draw_elements(glow::TRIANGLES, mesh.indices.len() as i32, glow::UNSIGNED_INT, 0);

						egui
							.as_mut()
							.unwrap()
							.run(&window, |ctx| {
								egui::Window::new("Wokýnko").resizable(false).show(ctx, |ui| {
									ui.label(&fps_string);

									ui.add_space(4.0);

									ui.label(format!("Cursor X: {:.3}", cursor_x));
									ui.label(format!("Cursor Y: {:.3}", cursor_y));

									ui.add_space(4.0);

									let [x, y, z] = camera.get_position().as_slice().try_into().unwrap();
									let (yaw, pitch) = camera.get_yaw_pitch();

									ui.horizontal(|ui| {
										ui.vertical(|ui| {
											ui.label(format!("Camera X: {:.3}", x));
											ui.label(format!("Camera Y: {:.3}", y));
											ui.label(format!("Camera Z: {:.3}", z));
										});

										ui.vertical(|ui| {
											ui.label(format!("Camera yaw: {:.3}", yaw));
											ui.label(format!("Camera pitch: {:.3}", pitch));
										})
									});

									ui.add_space(4.0);

									if cursor_lock {
										ui.label("Cursor is locked.");
										return
									}

									ui.checkbox(&mut vsync, "Enable Vsync");

									ui.checkbox(&mut rizz_mode, "Rizz mode");

									ui.checkbox(&mut pov_camera, "POV camera");

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

						let fps = 1.0 / dt;

						if last_update.elapsed().unwrap().as_secs_f32() >= fps_update_interval_secs {
							fps_string = format!("FPS = {:.1}", fps);
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
