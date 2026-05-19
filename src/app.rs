use egui_file_dialog::FileDialog;
use glow::*;
use glutin::{
	config::{ConfigTemplateBuilder, GlConfig},
	context::{ContextApi, ContextAttributesBuilder, NotCurrentGlContext, PossiblyCurrentContext},
	display::{GetGlDisplay, GlDisplay},
	surface::{GlSurface, Surface, SwapInterval, WindowSurface},
};
use glutin_winit::{DisplayBuilder, GlWindow};
use image::{ExtendedColorType, ImageEncoder, ImageReader, codecs::png::PngEncoder};
use log::*;
use nalgebra_glm as glm;
use raw_window_handle::HasWindowHandle;
use winit::{
	dpi::PhysicalSize,
	event::{DeviceEvent, ElementState, KeyEvent, MouseScrollDelta, WindowEvent},
	event_loop::ActiveEventLoop,
	keyboard::{Key, KeyCode, NamedKey, PhysicalKey},
	window::{CursorGrabMode, Fullscreen, Window},
};

use std::io::Cursor;
use std::num::NonZeroU32;
use std::sync::Arc;
use std::time::{Duration, SystemTime};

use crate::{background::Background, config::Config};
use crate::shader::{Program, ProgramBuilder, ShaderType};
use crate::{
	camera::{Camera, Directions},
	model::{Model, VertexAttributes},
};

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

pub struct App {
	window: Window,
	egui: egui_glow::EguiGlow,
	gl: Arc<Context>,
	gl_context: PossiblyCurrentContext,
	gl_surface: Surface<WindowSurface>,
	file_dialog: FileDialog,

	assets: Assets,
	screen_config: ScreenConfig,
	perf: Perf,
	world: World,
	state: State,
	debug: DebugStuff,
}

struct Assets {
	normal_program: Program,
	rizz_program: Program,
	background_program: Program,
	background: Background,
	terrain: Model,
	model: Model,
}

struct State {
	background_color: [f32; 4],
	ambient_color: [f32; 3],
	diffuse_color: [f32; 3],
	specular_color: [f32; 3],
	specular_shininess: f32,
	rizz_mode: bool,
	pov_camera: bool,
}

impl Default for State {
	fn default() -> Self {
		Self {
			background_color: [0.1, 0.2, 0.3, 1.0],
			ambient_color: [0.5, 0.5, 0.5],
			diffuse_color: [0.5, 0.5, 0.5],
			specular_color: [0.5, 0.5, 0.5],
			specular_shininess: 5.0,
			rizz_mode: false,
			pov_camera: false,
		}
	}
}

struct World {
	camera: Camera,
}

impl Default for World {
	fn default() -> Self {
		Self {
			camera: Camera::new(glm::vec3(0.0, 0.0, 0.0)),
		}
	}
}

struct Perf {
	start_time: SystemTime,
	last_time: SystemTime,
	last_update: SystemTime,
	fps_update_interval: Duration,
	fps_string: String,
}

impl Default for Perf {
	fn default() -> Self {
		let start_time = SystemTime::now();

		Self {
			start_time,
			last_time: start_time,
			last_update: start_time,
			fps_update_interval: Duration::from_millis(500),
			fps_string: "FPS = ???".into(),
		}
	}
}

struct ScreenConfig {
	cursor_lock: bool,
	fullscreen: bool,
	vsync: bool,
}

impl Default for ScreenConfig {
	fn default() -> Self {
		Self {
			cursor_lock: true,
			fullscreen: false,
			vsync: true,
		}
	}
}

struct DebugStuff {
	cursor_x: f64,
	cursor_y: f64,
}

impl Default for DebugStuff {
	fn default() -> Self {
		Self {
			cursor_x: 0.0,
			cursor_y: 0.0,
		}
	}
}

impl App {
	#[allow(unsafe_op_in_unsafe_fn)]
	pub unsafe fn init(
		event_loop: &ActiveEventLoop,
		config: Config,
		display_builder: DisplayBuilder,
		template: ConfigTemplateBuilder,
	) -> Self {
		let (window, gl_config) = display_builder
			.build(event_loop, template, |configs| {
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

		let not_current_gl_context = gl_display
			.create_context(&gl_config, &context_attributes)
			.unwrap();

		let window = window.unwrap();

		window.set_title("Triangle");
		window.set_visible(false);

		let attrs = window.build_surface_attributes(Default::default()).unwrap();
		let gl_surface = gl_display
			.create_window_surface(&gl_config, &attrs)
			.unwrap();

		let gl_context = not_current_gl_context.make_current(&gl_surface).unwrap();

		let mut gl = Context::from_loader_function_cstr(|s| gl_display.get_proc_address(s));

		gl.debug_message_callback(opengl_callback);
		gl.enable(DEBUG_OUTPUT);

		let gl = Arc::new(gl);

		gl_surface
			.set_swap_interval(&gl_context, SwapInterval::Wait(NonZeroU32::new(1).unwrap()))
			.unwrap();

		// Load shaders

		let normal_program = ProgramBuilder::new(gl.clone())
			.add_shader(
				ShaderType::Vertex,
				include_str!("./../assets/shaders/vert/main.vert"),
			)
			.add_shader(
				ShaderType::Fragment,
				include_str!("./../assets/shaders/frag/main.frag"),
			)
			.link();

		let rizz_program = ProgramBuilder::new(gl.clone())
			.add_shader(
				ShaderType::Vertex,
				include_str!("./../assets/shaders/vert/main.vert"),
			)
			.add_shader(
				ShaderType::Fragment,
				include_str!("./../assets/shaders/frag/rizz.frag"),
			)
			.link();

		let mut model_data = Cursor::new(include_bytes!("./../assets/objects/pastry/pastry.obj"));
		let (model, _material) =
			tobj::load_obj_buf(&mut model_data, &tobj::GPU_LOAD_OPTIONS, |_| {
				Err(tobj::LoadError::ReadError)
			})
			.unwrap();
		let model = model.into_iter().next().unwrap();
		let mesh = model.mesh;

		let terrain_image = ImageReader::open("assets/textures/ferris.png")
			.unwrap()
			.decode()
			.unwrap();

		let image = ImageReader::open("assets/objects/pastry/pastry_tex.png")
			.unwrap()
			.decode()
			.unwrap();

		let vertex_attribs = VertexAttributes {
			position: Some("aPos".into()),
			normal: Some("aNormal".into()),
			texcoord: Some("aTexCoord".into()),
		};

		let mut terrain = Model::new(gl.clone());
		terrain.add_mesh(&normal_program, crate::playfield::EXAMPLE_MAZE.generate_mesh(), &vertex_attribs);
		terrain.add_texture(&normal_program, terrain_image, "tex_unit");
		terrain.scale = glm::vec3(1.0, 1.0, 1.0);
		terrain.position = glm::vec3(0.0, 0.0, 0.0);

		let mut model = Model::new(gl.clone());
		model.add_mesh(&normal_program, mesh, &vertex_attribs);
		model.add_texture(&normal_program, image, "tex_unit");
		model.scale = glm::vec3(20.0, 20.0, 20.0);
		model.position = glm::vec3(7.5, 0.0, 7.5);

		let file_dialog = egui_file_dialog::FileDialog::new()
			.movable(false)
			.resizable(false)
			.anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0));

		let screen_config = ScreenConfig::default();

		window.set_visible(true);

		let egui = egui_glow::EguiGlow::new(event_loop, gl.clone(), None, None, true);

		let background_program = ProgramBuilder::new(gl.clone())
			.add_shader(
				ShaderType::Vertex,
				include_str!("./../assets/shaders/vert/screen.vert"),
			)
			.add_shader(
				ShaderType::Fragment,
				include_str!("./../assets/shaders/frag/starfield.frag"),
			)
			.link();

		let mut background = Background::new(gl.clone());
		background.register(&background_program, "aPos");

		let assets = Assets {
			normal_program,
			rizz_program,
			background_program,
			background,
			terrain,
			model,
		};

		let world = World::default();
		let perf = Perf::default();
		let debug = DebugStuff::default();
		let state = State::default();

		let mut app = App {
			window,
			egui,
			gl,
			gl_context,
			gl_surface,
			file_dialog,

			assets,
			screen_config,
			perf,
			world,
			state,
			debug,
		};

		app.update_cursor_lock();

		app
	}

	fn handle_key_event(&mut self, event_loop: &ActiveEventLoop, event: KeyEvent) {
		info!("{:?} key: {:?}", event.state, event.physical_key);

		if event.state == ElementState::Pressed {
			match event.logical_key {
				Key::Named(NamedKey::Escape) => event_loop.exit(),
				Key::Character(x) if x == "v" => {
					self.screen_config.vsync = !self.screen_config.vsync;
					info!("VSync = {}", self.screen_config.vsync);
				}
				Key::Character(x) if x == "p" => {
					self.state.pov_camera = !self.state.pov_camera;
					info!("POV camera = {}", self.state.pov_camera);
				}
				Key::Character(x) if x == "f" => {
					self.screen_config.fullscreen = !self.screen_config.fullscreen;
					info!("Fullscreen = {}", self.screen_config.fullscreen);
				}
				Key::Character(x) if x == "o" => {
					self.take_screenshot();
				}
				Key::Character(x) if x == "l" => {
					self.screen_config.cursor_lock = !self.screen_config.cursor_lock;
					self.update_cursor_lock();
					info!("Cursor lock = {}", self.screen_config.cursor_lock);
				}
				_ => {}
			}

			match event.physical_key {
				PhysicalKey::Code(KeyCode::ShiftLeft) => self.world.camera.move_fast(true),
				PhysicalKey::Code(KeyCode::KeyW) => {
					self.world.camera.key_interact(Directions::Forward, true)
				}
				PhysicalKey::Code(KeyCode::KeyS) => {
					self.world.camera.key_interact(Directions::Backward, true)
				}
				PhysicalKey::Code(KeyCode::KeyA) => {
					self.world.camera.key_interact(Directions::Left, true)
				}
				PhysicalKey::Code(KeyCode::KeyD) => {
					self.world.camera.key_interact(Directions::Right, true)
				}
				PhysicalKey::Code(KeyCode::ControlLeft) => {
					self.world.camera.key_interact(Directions::Down, true)
				}
				PhysicalKey::Code(KeyCode::Space) => {
					self.world.camera.key_interact(Directions::Up, true)
				}
				_ => {}
			}
		}

		if event.state == ElementState::Released {
			match event.physical_key {
				PhysicalKey::Code(KeyCode::ShiftLeft) => self.world.camera.move_fast(false),
				PhysicalKey::Code(KeyCode::KeyW) => {
					self.world.camera.key_interact(Directions::Forward, false)
				}
				PhysicalKey::Code(KeyCode::KeyS) => {
					self.world.camera.key_interact(Directions::Backward, false)
				}
				PhysicalKey::Code(KeyCode::KeyA) => {
					self.world.camera.key_interact(Directions::Left, false)
				}
				PhysicalKey::Code(KeyCode::KeyD) => {
					self.world.camera.key_interact(Directions::Right, false)
				}
				PhysicalKey::Code(KeyCode::ControlLeft) => {
					self.world.camera.key_interact(Directions::Down, false)
				}
				PhysicalKey::Code(KeyCode::Space) => {
					self.world.camera.key_interact(Directions::Up, false)
				}
				_ => {}
			}
		}
	}

	fn handle_mouse_motion_event(&mut self, delta: (f64, f64)) {
		self.world
			.camera
			.mouse_interact(delta.0 as f32, delta.1 as f32);
		self.debug.cursor_x += delta.0;
		self.debug.cursor_y += delta.1;
	}

	fn handle_mouse_wheel(&mut self, dy: f32) {
		self.world.camera.scroll_wheel_interact(dy / 5.0);
	}

	fn update_camera(&mut self, dt: f32) {
		self.world.camera.set_pov(self.state.pov_camera);
		self.world.camera.set_target(glm::vec3(0.0, 0.0, -10.0));
		self.world.camera.update_position(dt);
	}

	fn redraw_ui(&mut self, event_loop: &ActiveEventLoop) {
		self.egui.run(&self.window, |ctx| {
			egui::Window::new("Wokýnko")
				.resizable(false)
				.show(ctx, |ui| {
					ui.label(&self.perf.fps_string);

					ui.add_space(4.0);

					ui.label(format!("Cursor X: {:.3}", self.debug.cursor_x));
					ui.label(format!("Cursor Y: {:.3}", self.debug.cursor_y));

					ui.add_space(4.0);

					let [x, y, z] = self
						.world
						.camera
						.get_position()
						.as_slice()
						.try_into()
						.unwrap();
					let (yaw, pitch) = self.world.camera.get_yaw_pitch();

					ui.horizontal(|ui| {
						ui.vertical(|ui| {
							ui.label(format!("Camera X: {:.3}", x));
							ui.label(format!("Camera Y: {:.3}", y));
							ui.label(format!("Camera Z: {:.3}", z));
						});

						ui.vertical(|ui| {
							ui.label(format!("Camera yaw: {:.3}", yaw));
							ui.label(format!("Camera pitch: {:.3}", pitch));
							ui.label(format!("Camera FOV: {:.3}", self.world.camera.get_zoom()));
						})
					});

					ui.add_space(4.0);

					if self.screen_config.cursor_lock {
						ui.label("Cursor is locked.");
						return;
					}

					let mut scale = self.assets.model.scale[0];
					ui.add(egui::Slider::new(&mut scale, 0.0..=100.0));
					self.assets.model.scale = glm::vec3(scale, scale, scale);

					ui.checkbox(&mut self.screen_config.vsync, "Enable Vsync");

					ui.checkbox(&mut self.state.rizz_mode, "Rizz mode");

					ui.checkbox(&mut self.state.pov_camera, "POV camera");

					if ui.button("Pick model").clicked() {
						self.file_dialog.pick_file();
					}

					ui.horizontal(|ui| {
						ui.color_edit_button_rgb(&mut self.state.ambient_color);
						ui.label("Ambient color");
					});

					ui.horizontal(|ui| {
						ui.color_edit_button_rgb(&mut self.state.diffuse_color);
						ui.label("Diffuse color");
					});

					ui.horizontal(|ui| {
						ui.color_edit_button_rgb(&mut self.state.specular_color);
						ui.label("Specular color");
					});

					ui.label("Specular shininess");
					ui.add(egui::Slider::new(
						&mut self.state.specular_shininess,
						1.0..=100.0,
					));

					ui.horizontal(|ui| {
						ui.color_edit_button_rgb(
							(&mut self.state.background_color[..3]).try_into().unwrap(),
						);
						ui.label("Background");
					});

					if ui.button("Quit").clicked() {
						event_loop.exit();
					}
				});

			self.file_dialog.update(ctx);
		});

		self.egui.paint(&self.window);
	}

	fn update_perf_data(&mut self, dt: f32) {
		let fps = 1.0 / dt;

		if self.perf.last_update.elapsed().unwrap() >= self.perf.fps_update_interval {
			self.perf.fps_string = format!("FPS = {:.1}", fps);
			let vsync_string = format!(
				"VSync = {}",
				if self.screen_config.vsync {
					"on"
				} else {
					"off"
				}
			);
			let cursor_lock_string = format!(
				"Cursor lock = {}",
				if self.screen_config.cursor_lock {
					"on"
				} else {
					"off"
				}
			);
			self.window.set_title(&format!(
				"Triangle - {}, {}, {}",
				self.perf.fps_string, vsync_string, cursor_lock_string
			));
			info!(
				"{}, {}, {}",
				self.perf.fps_string, vsync_string, cursor_lock_string
			);
			self.perf.last_update = SystemTime::now();
		}
	}

	fn take_screenshot(&self) {
		let width = self.window.inner_size().width as usize;
		let height = self.window.inner_size().height as usize;

		let mut buf = vec![0; width * height * 4];

		unsafe {
			self.gl.read_pixels(
				0,
				0,
				width as i32,
				height as i32,
				RGBA,
				UNSIGNED_BYTE,
				PixelPackData::Slice(Some(buf.as_mut_slice())),
			);
		}

		std::thread::spawn(move || {
			let mut out_file = std::fs::File::create("screenshot.png").unwrap();

			// Reverse the rows so that the image is not upside down

			let buf = buf
				.chunks(width * 4)
				.rev()
				.flatten()
				.copied()
				.collect::<Vec<_>>();

			PngEncoder::new(&mut out_file)
				.write_image(
					buf.as_slice(),
					width as u32,
					height as u32,
					ExtendedColorType::Rgba8,
				)
				.unwrap();

			info!("Screenshot saved!");
		});
	}

	fn update_cursor_lock(&mut self) {
		if self.screen_config.cursor_lock {
			if let Err(err) = self
				.window
				.set_cursor_grab(CursorGrabMode::Confined)
				.or_else(|_| self.window.set_cursor_grab(CursorGrabMode::Locked))
			{
				error!("Could not enable cursor grab: {}", err);
			}
		} else {
			if let Err(err) = self.window.set_cursor_grab(CursorGrabMode::None) {
				error!("Could not disable cursor grab: {}", err);
			}
		}
	}

	fn enforce_cursor_lock(&self) {
		let middle_point = winit::dpi::LogicalPosition::new(
			self.window.inner_size().width / 2,
			self.window.inner_size().height / 2,
		);

		if self.screen_config.cursor_lock {
			self.window.set_cursor_visible(false);
			let _ = self.window.set_cursor_position(middle_point);
		} else {
			self.window.set_cursor_visible(true);
		}
	}

	fn enforce_vsync(&self) {
		if self.screen_config.vsync {
			self.gl_surface
				.set_swap_interval(
					&self.gl_context,
					SwapInterval::Wait(NonZeroU32::new(1).unwrap()),
				)
				.unwrap();
		} else {
			self.gl_surface
				.set_swap_interval(&self.gl_context, SwapInterval::DontWait)
				.unwrap();
		}
	}

	fn enforce_fullscreen(&self) {
		self.window.set_fullscreen(
			self.screen_config
				.fullscreen
				.then_some(Fullscreen::Borderless(self.window.current_monitor())),
		);
	}

	fn init_drawing(&self) {
		unsafe {
			self.gl.enable(CULL_FACE);
			self.gl.cull_face(FRONT);
			self.gl.front_face(CW);

			self.gl.enable(DEPTH_TEST);

			let [r, g, b, a] = self.state.background_color;
			self.gl.clear_color(r, g, b, a);
			self.gl.clear(COLOR_BUFFER_BIT | DEPTH_BUFFER_BIT);
		}
	}

	fn end_drawing(&self) {
		self.gl_surface.swap_buffers(&self.gl_context).unwrap();
		self.window.request_redraw();
	}

	fn redraw(&mut self, event_loop: &ActiveEventLoop) {
		let new_time = SystemTime::now();
		let dt = new_time
			.duration_since(self.perf.last_time)
			.unwrap()
			.as_secs_f32();
		self.perf.last_time = new_time;

		self.update_camera(dt);

		// self.assets.model.position[2] = -10.0;
		// self.assets.model.rotation[1] += (dt * 50.0).to_radians();

		let aspect = self.window.inner_size().width as f32 / self.window.inner_size().height as f32;
		let projection_mtx = glm::perspective(
			aspect,
			self.world.camera.get_zoom().to_radians(),
			0.1f32,
			100.0f32,
		);

		let program = match self.state.rizz_mode {
			false => &self.assets.normal_program,
			true => &self.assets.rizz_program,
		};

		program.set_uniform_f32_3("camera_position", self.world.camera.get_position().as_slice().try_into().unwrap());
		program.set_uniform_matrix_f32_4(
			"view",
			self.world
				.camera
				.get_view_matrix()
				.as_slice()
				.try_into()
				.unwrap(),
		);
		program
			.set_uniform_matrix_f32_4("projection", projection_mtx.as_slice().try_into().unwrap());

		let time = self.perf
			.last_time
			.duration_since(self.perf.start_time)
			.unwrap()
			.as_secs_f32();

		program.set_uniform_f32("screen_w", self.window.inner_size().width as f32);
		program.set_uniform_f32("screen_h", self.window.inner_size().height as f32);
		program.set_uniform_f32("time", time);
		program.set_uniform_f32("specular_shininess", self.state.specular_shininess);
		program.set_uniform_f32_3("ambient_material", &self.state.ambient_color);
		program.set_uniform_f32_3("directional_diffuse", &self.state.diffuse_color);
		program.set_uniform_f32_3("directional_specular", &self.state.specular_color);

		self.assets.background_program.set_uniform_f32("time", time);
		self.assets.background_program.set_uniform_f32("screen_w", self.window.inner_size().width as f32);

		// program.set_uniform_u32("point_enabled[0]", 1);
		// program.set_uniform_f32_3("point_position[0]", self.world.camera.get_position().as_slice().try_into().unwrap());
		// program.set_uniform_f32_3("point_diffuse[0]", &[0.0, 0.5, 0.0]);
		// program.set_uniform_f32_3("point_specular[0]", &[0.5, 0.0, 0.0]);

		// program.set_uniform_u32("spot_enabled", 1);
		// program.set_uniform_f32_3("spot_position", &[0.0, 0.5, 10.0]);
		// program.set_uniform_f32_3("spot_direction", &[0.0, 0.0, -1.0]);
		// program.set_uniform_f32("spot_cos_cutoff", 1.0f32.to_radians().cos());
		// program.set_uniform_f32_3("spot_diffuse", &[0.0, 0.5, 0.0]);
		// program.set_uniform_f32_3("spot_specular", &[0.5, 0.0, 0.0]);

		self.init_drawing();

		self.assets.background.draw(&self.assets.background_program);

		self.assets.terrain.draw(program, "model");
		self.assets.model.draw(program, "model");

		self.redraw_ui(event_loop);

		self.end_drawing();

		self.update_perf_data(dt);
		self.enforce_fullscreen();
		self.enforce_vsync();
		self.enforce_cursor_lock();
	}

	fn handle_resize_event(&mut self, new_size: PhysicalSize<u32>) {
		self.gl_surface.resize(
			&self.gl_context,
			new_size.width.try_into().unwrap(),
			new_size.height.try_into().unwrap(),
		);
	}

	pub fn handle_device_event(&mut self, _event_loop: &ActiveEventLoop, event: DeviceEvent) {
		match event {
			DeviceEvent::MouseMotion { delta } => {
				if self.screen_config.cursor_lock {
					self.handle_mouse_motion_event(delta);
				}
			}
			DeviceEvent::MouseWheel { delta } => {
				if !self.screen_config.cursor_lock {
					return;
				};

				if let MouseScrollDelta::LineDelta(_, y) = delta {
					self.handle_mouse_wheel(y);
				}
			}
			_ => {}
		}
	}

	pub fn handle_window_event(&mut self, event_loop: &ActiveEventLoop, event: WindowEvent) {
		let _ = self.egui.on_window_event(&self.window, &event);

		match event {
			WindowEvent::KeyboardInput { event, .. } if !event.repeat => {
				self.handle_key_event(event_loop, event);
			}
			WindowEvent::CloseRequested => {
				event_loop.exit();
			}
			WindowEvent::Resized(new_size) => {
				self.handle_resize_event(new_size);
			}
			WindowEvent::RedrawRequested => {
				self.redraw(event_loop);
			}
			_ => {}
		}
	}
}
