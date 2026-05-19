use glow::*;
use glutin::{
	context::{PossiblyCurrentContext},
	surface::{GlSurface, Surface, SwapInterval, WindowSurface},
};
use image::ImageReader;
use log::*;
use nalgebra_glm as glm;
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

use crate::{background::Background, player::PlayerController, screenshot::take_screenshot, transparent::TransparentRenderer};
use crate::shader::{Program, ProgramBuilder, ShaderType};
use crate::{
	camera::{Camera, Directions},
	model::{Model, VertexAttributes},
};

pub struct App {
	window: Window,
	egui: egui_glow::EguiGlow,
	gl: Arc<Context>,
	gl_context: PossiblyCurrentContext,
	gl_surface: Surface<WindowSurface>,

	assets: Assets,
	screen_config: ScreenConfig,
	player: PlayerController,
	perf: Perf,
	world: World,
	state: State,
}

struct Assets {
	normal_program: Program,
	rizz_program: Program,
	background_program: Program,
	powerup_program: Program,
	background: Background,
	terrain: Model,
	player: Model,
	enemy: Model,
	powerup_hp: Model,
	powerup_energy: Model,
	powerup_speed: Model,
}

struct State {
	background_color: [f32; 4],
	ambient_color: [f32; 3],
	diffuse_color: [f32; 3],
	specular_color: [f32; 3],
	specular_shininess: f32,
	enable_background: bool,
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
			specular_shininess: 20.0,
			enable_background: true,
			rizz_mode: false,
			pov_camera: true,
		}
	}
}

struct World {
	camera: Camera,
}

impl Default for World {
	fn default() -> Self {
		let mut camera = Camera::new(glm::vec3(0.0, 0.0, 0.0));
		camera.set_pov(true);

		Self {
			camera
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

impl App {
	pub fn init(
		event_loop: &ActiveEventLoop,
		window: Window,
		gl: Arc<Context>,
		gl_context: PossiblyCurrentContext,
		gl_surface: Surface<WindowSurface>
	) -> Self {
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

		let powerup_program = ProgramBuilder::new(gl.clone())
			.add_shader(
				ShaderType::Vertex,
				include_str!("./../assets/shaders/vert/main.vert"),
			)
			.add_shader(
				ShaderType::Fragment,
				include_str!("./../assets/shaders/frag/powerup.frag"),
			)
			.link();

		let mut model_data = Cursor::new(include_bytes!("./../assets/objects/pastry/pastry.obj"));
		let (model, _material) =
			tobj::load_obj_buf(&mut model_data, &tobj::GPU_LOAD_OPTIONS, |_| {
				Err(tobj::LoadError::ReadError)
			})
			.unwrap();
		let model = model.into_iter().next().unwrap();
		let player_mesh = model.mesh;

		let mut model_data = Cursor::new(include_bytes!("./../assets/objects/apple/apple.obj"));
		let (model, _material) =
			tobj::load_obj_buf(&mut model_data, &tobj::GPU_LOAD_OPTIONS, |_| {
				Err(tobj::LoadError::ReadError)
			})
			.unwrap();
		let model = model.into_iter().next().unwrap();
		let enemy_mesh = model.mesh;

		let mut model_data = Cursor::new(include_bytes!("./../assets/objects/powerups/powerup-hp.obj"));
		let (model, _material) =
			tobj::load_obj_buf(&mut model_data, &tobj::GPU_LOAD_OPTIONS, |_| {
				Err(tobj::LoadError::ReadError)
			})
			.unwrap();
		let model = model.into_iter().next().unwrap();
		let powerup_hp_mesh = model.mesh;

		let mut model_data = Cursor::new(include_bytes!("./../assets/objects/powerups/powerup-energy.obj"));
		let (model, _material) =
			tobj::load_obj_buf(&mut model_data, &tobj::GPU_LOAD_OPTIONS, |_| {
				Err(tobj::LoadError::ReadError)
			})
			.unwrap();
		let model = model.into_iter().next().unwrap();
		let powerup_energy_mesh = model.mesh;

		let mut model_data = Cursor::new(include_bytes!("./../assets/objects/powerups/powerup-speed.obj"));
		let (model, _material) =
			tobj::load_obj_buf(&mut model_data, &tobj::GPU_LOAD_OPTIONS, |_| {
				Err(tobj::LoadError::ReadError)
			})
			.unwrap();
		let model = model.into_iter().next().unwrap();
		let powerup_speed_mesh = model.mesh;

		let terrain_tex = ImageReader::open("assets/textures/ferris.png")
			.unwrap()
			.decode()
			.unwrap();

		let player_tex = ImageReader::open("assets/objects/pastry/pastry_tex.png")
			.unwrap()
			.decode()
			.unwrap();

		let enemy_tex = ImageReader::open("assets/objects/apple/apple_tex.png")
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
		terrain.add_texture(&normal_program, terrain_tex, "tex_unit");
		terrain.scale = glm::vec3(1.0, 1.0, 1.0);
		terrain.position = glm::vec3(0.0, 0.0, 0.0);

		let mut player = Model::new(gl.clone());
		player.add_mesh(&normal_program, player_mesh, &vertex_attribs);
		player.add_texture(&normal_program, player_tex, "tex_unit");
		player.scale = glm::vec3(20.0, 20.0, 20.0);
		player.position = glm::vec3(7.5, 0.0, 7.5);

		let mut enemy = Model::new(gl.clone());
		enemy.add_mesh(&normal_program, enemy_mesh, &vertex_attribs);
		enemy.add_texture(&normal_program, enemy_tex, "tex_unit");
		enemy.scale = glm::vec3(30.0, 30.0, 30.0);
		enemy.position = glm::vec3(12.5, 0.0, 7.5);

		let mut powerup_hp = Model::new(gl.clone());
		powerup_hp.add_mesh(&normal_program, powerup_hp_mesh, &vertex_attribs);
		powerup_hp.scale = glm::vec3(2.0, 2.0, 2.0);
		powerup_hp.position = glm::vec3(22.5, 1.5, 27.5);

		let mut powerup_energy = Model::new(gl.clone());
		powerup_energy.add_mesh(&normal_program, powerup_energy_mesh, &vertex_attribs);
		powerup_energy.scale = glm::vec3(2.0, 2.0, 2.0);
		powerup_energy.position = glm::vec3(22.5, 1.5, 22.5);

		let mut powerup_speed = Model::new(gl.clone());
		powerup_speed.add_mesh(&normal_program, powerup_speed_mesh, &vertex_attribs);
		powerup_speed.scale = glm::vec3(2.0, 2.0, 2.0);
		powerup_speed.position = glm::vec3(22.5, 1.5, 17.5);

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
			powerup_program,
			background,
			terrain,
			player,
			powerup_hp,
			powerup_energy,
			powerup_speed,
			enemy
		};

		let player = PlayerController::new();
		let world = World::default();
		let perf = Perf::default();
		let state = State::default();

		let mut app = App {
			window,
			egui,
			gl,
			gl_context,
			gl_surface,

			assets,
			player,
			screen_config,
			perf,
			world,
			state,
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
					take_screenshot(&self.gl, self.window.inner_size());
				}
				Key::Character(x) if x == "l" => {
					self.screen_config.cursor_lock = !self.screen_config.cursor_lock;
					self.update_cursor_lock();
					info!("Cursor lock = {}", self.screen_config.cursor_lock);
				}
				_ => {}
			}

			if self.state.pov_camera {
				match event.physical_key {
					PhysicalKey::Code(KeyCode::KeyW) => {
						self.player.move_forward = true;
					}
					PhysicalKey::Code(KeyCode::KeyS) => {
						self.player.move_backward = true;
					}
					PhysicalKey::Code(KeyCode::KeyA) => {
						self.player.move_left = true;
					}
					PhysicalKey::Code(KeyCode::KeyD) => {
						self.player.move_right = true;
					}
					PhysicalKey::Code(KeyCode::Space) => {
						self.player.jump = true;
					}
					_ => {}
				}
			} else {
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
		}

		if event.state == ElementState::Released {
			if self.state.pov_camera {
				match event.physical_key {
					PhysicalKey::Code(KeyCode::KeyW) => {
						self.player.move_forward = false;
					}
					PhysicalKey::Code(KeyCode::KeyS) => {
						self.player.move_backward = false;
					}
					PhysicalKey::Code(KeyCode::KeyA) => {
						self.player.move_left = false;
					}
					PhysicalKey::Code(KeyCode::KeyD) => {
						self.player.move_right = false;
					}
					_ => {}
				}
			} else {
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
	}

	fn handle_mouse_motion_event(&mut self, delta: (f64, f64)) {
		self.world
			.camera
			.mouse_interact(delta.0 as f32, delta.1 as f32);
	}

	fn handle_mouse_wheel(&mut self, dy: f32) {
		self.world.camera.scroll_wheel_interact(dy / 5.0);
	}

	fn update_camera(&mut self, dt: f32) {
		let pitch_range = if self.state.pov_camera {
			-89.9..=-15.0
		} else {
			-89.9..=89.9
		};

		self.world.camera.set_pov(self.state.pov_camera);
		self.world.camera.set_pitch_range(pitch_range);
		self.world.camera.set_target(self.assets.player.position);
		self.world.camera.update_position(dt);
	}

	fn redraw_ui(&mut self, event_loop: &ActiveEventLoop) {
		self.egui.run(&self.window, |ctx| {
			egui::Window::new("Wokýnko")
				.resizable(false)
				.show(ctx, |ui| {
					ui.label(&self.perf.fps_string);

					ui.add_space(4.0);

					ui.add_space(4.0);

					let [x, y, z] = self
						.assets
						.player
						.position
						.as_slice()
						.try_into()
						.unwrap();
					let (yaw, pitch) = self.world.camera.get_yaw_pitch();

					ui.horizontal(|ui| {
						ui.vertical(|ui| {
							ui.label(format!("Player X: {:.3}", x));
							ui.label(format!("Player Y: {:.3}", y));
							ui.label(format!("Player Z: {:.3}", z));
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

					let mut scale = self.assets.player.scale[0];
					ui.add(egui::Slider::new(&mut scale, 0.0..=100.0));
					self.assets.player.scale = glm::vec3(scale, scale, scale);

					ui.checkbox(&mut self.screen_config.vsync, "Enable Vsync");

					ui.checkbox(&mut self.state.rizz_mode, "Rizz mode");

					ui.checkbox(&mut self.state.enable_background, "Enable background");

					ui.checkbox(&mut self.state.pov_camera, "POV camera");

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

		self.assets.player.position = self.player.update_position(self.assets.player.position, self.world.camera.get_yaw_pitch().0, dt);
		self.update_camera(dt);

		self.assets.player.rotation[1] = -(self.world.camera.get_yaw_pitch().0 - 90.0).to_radians();

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

		let camera_pos = self.world.camera.get_position();
		let view_mtx = self.world.camera.get_view_matrix();

		program.set_uniform_f32_3("camera_position", camera_pos.as_slice().try_into().unwrap());
		program.set_uniform_matrix_f32_4("view", view_mtx.as_slice().try_into().unwrap());
		program.set_uniform_matrix_f32_4("projection", projection_mtx.as_slice().try_into().unwrap());

		self.assets.powerup_program.set_uniform_f32_3("camera_position", camera_pos.as_slice().try_into().unwrap());
		self.assets.powerup_program.set_uniform_matrix_f32_4("view", view_mtx.as_slice().try_into().unwrap());
		self.assets.powerup_program.set_uniform_matrix_f32_4("projection", projection_mtx.as_slice().try_into().unwrap());

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

		if self.state.enable_background {
			self.assets.background.draw(&self.assets.background_program);
		}

		self.assets.terrain.draw(program, "model");
		self.assets.player.draw(program, "model");
		self.assets.enemy.draw(program, "model");

		let mut transparent = TransparentRenderer::new(self.gl.clone());

		transparent.add_object(self.assets.powerup_speed.position, || {
			self.assets.powerup_program.set_uniform_f32_3("base_color", &[0.0, 1.0, 0.0]);
			self.assets.powerup_speed.draw(&self.assets.powerup_program, "model");
		});

		transparent.add_object(self.assets.powerup_hp.position, || {
			self.assets.powerup_program.set_uniform_f32_3("base_color", &[1.0, 0.0, 0.0]);
			self.assets.powerup_hp.draw(&self.assets.powerup_program, "model");
		});

		transparent.add_object(self.assets.powerup_energy.position, || {
			self.assets.powerup_program.set_uniform_f32_3("base_color", &[0.0, 0.0, 1.0]);
			self.assets.powerup_energy.draw(&self.assets.powerup_program, "model");
		});

		transparent.render(view_mtx);

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
