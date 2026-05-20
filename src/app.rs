use glow::*;
use glutin::{
	context::{PossiblyCurrentContext},
	surface::{GlSurface, Surface, SwapInterval, WindowSurface},
};
use image::{DynamicImage, ImageReader};
use log::*;
use nalgebra_glm as glm;
use tobj::Mesh;
use winit::{
	dpi::PhysicalSize,
	event::{DeviceEvent, ElementState, KeyEvent, MouseScrollDelta, WindowEvent},
	event_loop::ActiveEventLoop,
	keyboard::{Key, KeyCode, NamedKey, PhysicalKey},
	window::{CursorGrabMode, Fullscreen, Window},
};

use std::{io::Cursor, time::Instant};
use std::num::NonZeroU32;
use std::sync::Arc;
use std::time::{Duration, SystemTime};

use crate::{background::Background, model::Transform, player::PlayerController, screenshot::take_screenshot, transparent::TransparentRenderer};
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
	perf: Perf,
	scene: Scene,
	params: Parameters,
}

enum PowerupKind {
	Health,
	Energy,
	Speed
}

struct Powerup {
	kind: PowerupKind,
	transform: Transform
}

enum EnemyKind {
	Apple
}

struct Enemy {
	kind: EnemyKind,
	transform: Transform
}

struct Scene {
	camera: Camera,
	player: PlayerController,
	enemies: Vec<Enemy>,
	powerups: Vec<Powerup>
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

struct Parameters {
	background_color: [f32; 4],
	ambient_color: [f32; 3],
	diffuse_color: [f32; 3],
	specular_color: [f32; 3],
	specular_shininess: f32,
	enable_background: bool,
	rizz_mode: bool,
	pov_camera: bool,
	cursor_lock: bool,
	fullscreen: bool,
	vsync: bool,
}

impl Default for Parameters {
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
			cursor_lock: true,
			fullscreen: false,
			vsync: true,
		}
	}
}

struct Perf {
	start_time: Instant,
	last_time: Instant,
	last_update: Instant,
	fps_update_interval: Duration,
	fps_string: String,
}

impl Default for Perf {
	fn default() -> Self {
		let start_time = Instant::now();

		Self {
			start_time,
			last_time: start_time,
			last_update: start_time,
			fps_update_interval: Duration::from_millis(500),
			fps_string: "FPS = ???".into(),
		}
	}
}

fn load_mesh(bytes: &[u8]) -> Mesh {
	let mut model_data = Cursor::new(bytes);
	let (model, _material) =
		tobj::load_obj_buf(&mut model_data, &tobj::GPU_LOAD_OPTIONS, |_| {
			Err(tobj::LoadError::ReadError)
		})
		.unwrap();
	let model = model.into_iter().next().unwrap();
	model.mesh
}

fn load_texture(bytes: &[u8]) -> DynamicImage {
	ImageReader::new(Cursor::new(bytes))
		.with_guessed_format()
		.unwrap()
		.decode()
		.unwrap()
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

		let assets = {
			// Load shaders

			let normal_program = ProgramBuilder::new(gl.clone())
				.add_shader(ShaderType::Vertex, include_str!("./../assets/shaders/vert/main.vert"))
				.add_shader(ShaderType::Fragment, include_str!("./../assets/shaders/frag/main.frag"))
				.link();

			let rizz_program = ProgramBuilder::new(gl.clone())
				.add_shader(ShaderType::Vertex, include_str!("./../assets/shaders/vert/main.vert"))
				.add_shader(ShaderType::Fragment, include_str!("./../assets/shaders/frag/rizz.frag"))
				.link();

			let powerup_program = ProgramBuilder::new(gl.clone())
				.add_shader(ShaderType::Vertex, include_str!("./../assets/shaders/vert/main.vert"))
				.add_shader(ShaderType::Fragment, include_str!("./../assets/shaders/frag/powerup.frag"))
				.link();

			let background_program = ProgramBuilder::new(gl.clone())
				.add_shader(ShaderType::Vertex, include_str!("./../assets/shaders/vert/screen.vert"))
				.add_shader(ShaderType::Fragment, include_str!("./../assets/shaders/frag/starfield.frag"))
				.link();

			// Load background effect

			let mut background = Background::new(gl.clone());
			background.register(&background_program, "aPos");

			// Load models

			let player_mesh = load_mesh(include_bytes!("../assets/objects/pastry/pastry.obj"));
			let enemy_mesh = load_mesh(include_bytes!("../assets/objects/apple/apple.obj"));
			let powerup_hp_mesh = load_mesh(include_bytes!("../assets/objects/powerups/powerup-hp.obj"));
			let powerup_energy_mesh = load_mesh(include_bytes!("../assets/objects/powerups/powerup-energy.obj"));
			let powerup_speed_mesh = load_mesh(include_bytes!("../assets/objects/powerups/powerup-speed.obj"));

			let terrain_tex = load_texture(include_bytes!("../assets/textures/ferris.png"));
			let player_tex = load_texture(include_bytes!("../assets/objects/pastry/pastry_tex.png"));
			let enemy_tex = load_texture(include_bytes!("../assets/objects/apple/apple_tex.png"));

			let vertex_attribs = VertexAttributes {
				position: Some("aPos".into()),
				normal: Some("aNormal".into()),
				texcoord: Some("aTexCoord".into()),
			};

			let terrain = Model::new(gl.clone())
				.with_mesh(&normal_program, crate::playfield::EXAMPLE_MAZE.generate_mesh(), &vertex_attribs)
				.with_texture(&normal_program, terrain_tex, "tex_unit");

			let player = Model::new(gl.clone())
				.with_mesh(&normal_program, player_mesh, &vertex_attribs)
				.with_texture(&normal_program, player_tex, "tex_unit")
				.with_scale(glm::vec3(20.0, 20.0, 20.0));

			let enemy = Model::new(gl.clone())
				.with_mesh(&normal_program, enemy_mesh, &vertex_attribs)
				.with_texture(&normal_program, enemy_tex, "tex_unit")
				.with_scale(glm::vec3(30.0, 30.0, 30.0));

			let powerup_hp = Model::new(gl.clone())
				.with_mesh(&normal_program, powerup_hp_mesh, &vertex_attribs)
				.with_scale(glm::vec3(2.0, 2.0, 2.0));

			let powerup_energy = Model::new(gl.clone())
				.with_mesh(&normal_program, powerup_energy_mesh, &vertex_attribs)
				.with_scale(glm::vec3(2.0, 2.0, 2.0));

			let powerup_speed = Model::new(gl.clone())
				.with_mesh(&normal_program, powerup_speed_mesh, &vertex_attribs)
				.with_scale(glm::vec3(2.0, 2.0, 2.0));

			Assets {
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
			}
		};

		let scene = {
			let player = PlayerController::new(Transform::origin().with_position(glm::vec3(7.5, 0.0, 7.5)));

			let mut camera = Camera::new(glm::vec3(0.0, 0.0, 0.0));
			camera.set_pov(true);

			let enemies = vec![
				Enemy {
					kind: EnemyKind::Apple,
					transform: Transform::origin().with_position(glm::vec3(12.5, 0.0, 7.5))
				}
			];

			let powerups = vec![
				Powerup {
					kind: PowerupKind::Speed,
					transform: Transform::origin().with_position(glm::vec3(22.5, 1.5, 17.5))
				},
				Powerup {
					kind: PowerupKind::Energy,
					transform: Transform::origin().with_position(glm::vec3(22.5, 1.5, 22.5))
				},
				Powerup {
					kind: PowerupKind::Health,
					transform: Transform::origin().with_position(glm::vec3(22.5, 1.5, 27.5))
				},
			];

			Scene {
				camera,
				player,
				enemies,
				powerups
			}
		};

		let egui = egui_glow::EguiGlow::new(event_loop, gl.clone(), None, None, true);
		let perf = Perf::default();
		let params = Parameters::default();

		window.set_visible(true);

		let mut app = App {
			window,
			egui,
			gl,
			gl_context,
			gl_surface,

			assets,
			scene,
			perf,
			params,
		};

		app.update_cursor_lock();

		app
	}

	fn handle_key_event(&mut self, event_loop: &ActiveEventLoop, event: KeyEvent) {
		info!("{:?} key: {:?}", event.state, event.physical_key);

		let enable = event.state == ElementState::Pressed;

		if self.params.pov_camera {
			match event.physical_key {
				PhysicalKey::Code(KeyCode::KeyW) => {
					self.scene.player.move_forward = enable;
				}
				PhysicalKey::Code(KeyCode::KeyS) => {
					self.scene.player.move_backward = enable;
				}
				PhysicalKey::Code(KeyCode::KeyA) => {
					self.scene.player.move_left = enable;
				}
				PhysicalKey::Code(KeyCode::KeyD) => {
					self.scene.player.move_right = enable;
				}
				PhysicalKey::Code(KeyCode::Space) => {
					self.scene.player.jump = enable;
				}
				_ => {}
			}
		} else {
			match event.physical_key {
				PhysicalKey::Code(KeyCode::ShiftLeft) => self.scene.camera.move_fast(enable),
				PhysicalKey::Code(KeyCode::KeyW) => {
					self.scene.camera.key_interact(Directions::Forward, enable)
				}
				PhysicalKey::Code(KeyCode::KeyS) => {
					self.scene.camera.key_interact(Directions::Backward, enable)
				}
				PhysicalKey::Code(KeyCode::KeyA) => {
					self.scene.camera.key_interact(Directions::Left, enable)
				}
				PhysicalKey::Code(KeyCode::KeyD) => {
					self.scene.camera.key_interact(Directions::Right, enable)
				}
				PhysicalKey::Code(KeyCode::ControlLeft) => {
					self.scene.camera.key_interact(Directions::Down, enable)
				}
				PhysicalKey::Code(KeyCode::Space) => {
					self.scene.camera.key_interact(Directions::Up, enable)
				}
				_ => {}
			}
		}

		if event.state == ElementState::Pressed {
			match event.logical_key {
				Key::Named(NamedKey::Escape) => event_loop.exit(),
				Key::Character(x) if x == "v" => {
					self.params.vsync = !self.params.vsync;
					info!("VSync = {}", self.params.vsync);
				}
				Key::Character(x) if x == "p" => {
					self.params.pov_camera = !self.params.pov_camera;
					info!("POV camera = {}", self.params.pov_camera);
				}
				Key::Character(x) if x == "f" => {
					self.params.fullscreen = !self.params.fullscreen;
					info!("Fullscreen = {}", self.params.fullscreen);
				}
				Key::Character(x) if x == "o" => {
					take_screenshot(&self.gl, self.window.inner_size());
				}
				Key::Character(x) if x == "l" => {
					self.params.cursor_lock = !self.params.cursor_lock;
					self.update_cursor_lock();
					info!("Cursor lock = {}", self.params.cursor_lock);
				}
				_ => {}
			}
		}
	}

	fn handle_mouse_motion_event(&mut self, delta: (f64, f64)) {
		self.scene.camera.mouse_interact(delta.0 as f32, delta.1 as f32);
	}

	fn handle_mouse_wheel(&mut self, dy: f32) {
		self.scene.camera.scroll_wheel_interact(dy / 5.0);
	}

	fn update_camera(&mut self, dt: f32) {
		let pitch_range = if self.params.pov_camera {
			-89.9..=-15.0
		} else {
			-89.9..=89.9
		};

		self.scene.camera.set_pov(self.params.pov_camera);
		self.scene.camera.set_pitch_range(pitch_range);
		self.scene.camera.set_target(self.scene.player.get_transform().position);
		self.scene.camera.update_position(dt);
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
						.scene
						.player
						.get_transform()
						.position
						.as_slice()
						.try_into()
						.unwrap();
					let (yaw, pitch) = self.scene.camera.get_yaw_pitch();

					ui.horizontal(|ui| {
						ui.vertical(|ui| {
							ui.label(format!("Player X: {:.3}", x));
							ui.label(format!("Player Y: {:.3}", y));
							ui.label(format!("Player Z: {:.3}", z));
						});

						ui.vertical(|ui| {
							ui.label(format!("Camera yaw: {:.3}", yaw));
							ui.label(format!("Camera pitch: {:.3}", pitch));
							ui.label(format!("Camera FOV: {:.3}", self.scene.camera.get_zoom()));
						})
					});

					ui.add_space(4.0);

					if self.params.cursor_lock {
						ui.label("Cursor is locked.");
						return;
					}

					ui.checkbox(&mut self.params.vsync, "Enable Vsync");

					ui.checkbox(&mut self.params.rizz_mode, "Rizz mode");

					ui.checkbox(&mut self.params.enable_background, "Enable background");

					ui.checkbox(&mut self.params.pov_camera, "POV camera");

					ui.horizontal(|ui| {
						ui.color_edit_button_rgb(&mut self.params.ambient_color);
						ui.label("Ambient color");
					});

					ui.horizontal(|ui| {
						ui.color_edit_button_rgb(&mut self.params.diffuse_color);
						ui.label("Diffuse color");
					});

					ui.horizontal(|ui| {
						ui.color_edit_button_rgb(&mut self.params.specular_color);
						ui.label("Specular color");
					});

					ui.label("Specular shininess");
					ui.add(egui::Slider::new(
						&mut self.params.specular_shininess,
						1.0..=100.0,
					));

					ui.horizontal(|ui| {
						ui.color_edit_button_rgb(
							(&mut self.params.background_color[..3]).try_into().unwrap(),
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

		if self.perf.last_update.elapsed() >= self.perf.fps_update_interval {
			self.perf.fps_string = format!("FPS = {:.1}", fps);
			let vsync_string = format!(
				"VSync = {}",
				if self.params.vsync {
					"on"
				} else {
					"off"
				}
			);
			let cursor_lock_string = format!(
				"Cursor lock = {}",
				if self.params.cursor_lock {
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
			self.perf.last_update = Instant::now();
		}
	}

	fn update_cursor_lock(&mut self) {
		if self.params.cursor_lock {
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

		if self.params.cursor_lock {
			self.window.set_cursor_visible(false);
			let _ = self.window.set_cursor_position(middle_point);
		} else {
			self.window.set_cursor_visible(true);
		}
	}

	fn enforce_vsync(&self) {
		if self.params.vsync {
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
			self.params
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

			let [r, g, b, a] = self.params.background_color;
			self.gl.clear_color(r, g, b, a);
			self.gl.clear(COLOR_BUFFER_BIT | DEPTH_BUFFER_BIT);
		}
	}

	fn end_drawing(&self) {
		self.gl_surface.swap_buffers(&self.gl_context).unwrap();
		self.window.request_redraw();
	}

	fn redraw(&mut self, event_loop: &ActiveEventLoop) {
		let new_time = Instant::now();
		let dt = new_time
			.duration_since(self.perf.last_time)
			.as_secs_f32();
		self.perf.last_time = new_time;

		self.scene.player.update_yaw(self.scene.camera.get_yaw_pitch().0);
		self.scene.player.update_position(dt);
		self.update_camera(dt);

		let aspect = self.window.inner_size().width as f32 / self.window.inner_size().height as f32;
		let projection_mtx = glm::perspective(
			aspect,
			self.scene.camera.get_zoom().to_radians(),
			0.1f32,
			100.0f32,
		);

		let program = match self.params.rizz_mode {
			false => &self.assets.normal_program,
			true => &self.assets.rizz_program,
		};

		let camera_pos = self.scene.camera.get_position();
		let view_mtx = self.scene.camera.get_view_matrix();

		program.set_uniform_f32_3("camera_position", camera_pos.as_slice().try_into().unwrap());
		program.set_uniform_matrix_f32_4("view", view_mtx.as_slice().try_into().unwrap());
		program.set_uniform_matrix_f32_4("projection", projection_mtx.as_slice().try_into().unwrap());

		self.assets.powerup_program.set_uniform_f32_3("camera_position", camera_pos.as_slice().try_into().unwrap());
		self.assets.powerup_program.set_uniform_matrix_f32_4("view", view_mtx.as_slice().try_into().unwrap());
		self.assets.powerup_program.set_uniform_matrix_f32_4("projection", projection_mtx.as_slice().try_into().unwrap());

		let time = self.perf
			.last_time
			.duration_since(self.perf.start_time)
			.as_secs_f32();

		program.set_uniform_f32("screen_w", self.window.inner_size().width as f32);
		program.set_uniform_f32("screen_h", self.window.inner_size().height as f32);
		program.set_uniform_f32("time", time);
		program.set_uniform_f32("specular_shininess", self.params.specular_shininess);
		program.set_uniform_f32_3("ambient_material", &self.params.ambient_color);
		program.set_uniform_f32_3("directional_diffuse", &self.params.diffuse_color);
		program.set_uniform_f32_3("directional_specular", &self.params.specular_color);

		self.assets.background_program.set_uniform_f32("time", time);
		self.assets.background_program.set_uniform_f32("screen_w", self.window.inner_size().width as f32);

		// program.set_uniform_u32("point_enabled[0]", 1);
		// program.set_uniform_f32_3("point_position[0]", self.camera.get_position().as_slice().try_into().unwrap());
		// program.set_uniform_f32_3("point_diffuse[0]", &[0.0, 0.5, 0.0]);
		// program.set_uniform_f32_3("point_specular[0]", &[0.5, 0.0, 0.0]);

		// program.set_uniform_u32("spot_enabled", 1);
		// program.set_uniform_f32_3("spot_position", &[0.0, 0.5, 10.0]);
		// program.set_uniform_f32_3("spot_direction", &[0.0, 0.0, -1.0]);
		// program.set_uniform_f32("spot_cos_cutoff", 1.0f32.to_radians().cos());
		// program.set_uniform_f32_3("spot_diffuse", &[0.0, 0.5, 0.0]);
		// program.set_uniform_f32_3("spot_specular", &[0.5, 0.0, 0.0]);

		self.init_drawing();

		if self.params.enable_background {
			self.assets.background.draw(&self.assets.background_program);
		}

		self.assets.terrain.draw(&Transform::origin(), program, "model");
		self.assets.player.draw(self.scene.player.get_transform(), program, "model");

		for enemy in &self.scene.enemies {
			self.assets.enemy.draw(&enemy.transform, program, "model");
		}

		let mut transparent = TransparentRenderer::new(self.gl.clone());

		for powerup in &self.scene.powerups {
			let (model, color) = match powerup.kind {
				PowerupKind::Health => (&self.assets.powerup_hp, &[1.0, 0.0, 0.0]),
				PowerupKind::Energy => (&self.assets.powerup_energy, &[0.0, 0.0, 1.0]),
				PowerupKind::Speed => (&self.assets.powerup_speed, &[0.0, 1.0, 0.0]),
			};

			transparent.add_object(&powerup.transform, || {
				self.assets.powerup_program.set_uniform_f32_3("base_color", color);
				model.draw(&powerup.transform, &self.assets.powerup_program, "model");
			});
		}

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
				if self.params.cursor_lock {
					self.handle_mouse_motion_event(delta);
				}
			}
			DeviceEvent::MouseWheel { delta } => {
				if !self.params.cursor_lock {
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
