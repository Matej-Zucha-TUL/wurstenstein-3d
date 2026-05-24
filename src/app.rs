use glow::*;
use glutin::{
	context::{PossiblyCurrentContext},
	surface::{GlSurface, Surface, SwapInterval, WindowSurface},
};
use log::*;
use nalgebra_glm as glm;
use winit::{
	dpi::PhysicalSize,
	event::{DeviceEvent, ElementState, KeyEvent, MouseButton, MouseScrollDelta, WindowEvent},
	event_loop::ActiveEventLoop,
	keyboard::{Key, KeyCode, NamedKey, PhysicalKey},
	window::{CursorGrabMode, Fullscreen, Window},
};

use std::num::NonZeroU32;
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::assets::Assets;
use crate::audio::{Audio, MusicRequest, SoundRequest};
use crate::bullet::BulletManager;
use crate::camera::{Camera, Directions};
use crate::collision;
use crate::enemy::EnemyManager;
use crate::explosion::ExplosionManager;
use crate::model::Transform;
use crate::player::{PlayerAction, PlayerController, MAX_AMMO, MAX_HEALTH};
use crate::playfield::EXAMPLE_MAZE;
use crate::powerup::{PowerupManager, PowerupKind};
use crate::screenshot::take_screenshot;
use crate::transparent::TransparentRenderer;

pub struct App {
	window: Window,
	egui: egui_glow::EguiGlow,
	gl: Arc<Context>,
	gl_context: PossiblyCurrentContext,
	gl_surface: Surface<WindowSurface>,

	assets: Assets,
	audio: Audio,
	perf: Perf,
	scene: Scene,
	params: Parameters,
}

#[derive(PartialEq)]
enum SceneState {
	Title,
	InGame { timer: f32, kills: usize },
	Dead,
	YoureWinner { kills: usize }
}

struct Scene {
	state: SceneState,
	camera: Camera,
	player: PlayerController,
	bullets: BulletManager,
	enemies: EnemyManager,
	powerups: PowerupManager,
	explosions: ExplosionManager,
}

impl Scene {
	pub fn new(gl: Arc<Context>, assets: &Assets) -> Self {
		let spawn = EXAMPLE_MAZE.player_spawn_point.map(|x| (x as f32 + 0.5) * EXAMPLE_MAZE.scale);
		let player = PlayerController::new(Transform::origin().with_position(glm::vec3(spawn[0], 0.0, spawn[1])), assets.player_bounding_box.clone());

		let mut camera = Camera::new(glm::vec3(0.0, 0.0, 0.0));
		camera.set_pov(true);

		let bullets = BulletManager::new();

		let enemies = EnemyManager::new();

		let powerups = PowerupManager::new();

		let explosions = ExplosionManager::new(gl, assets);

		Self {
			state: SceneState::InGame { timer: 120.0, kills: 0 },
			camera,
			player,
			bullets,
			enemies,
			powerups,
			explosions,
		}
	}
}

struct Parameters {
	background_color: [f32; 4],
	ambient_color: [f32; 3],
	diffuse_color: [f32; 3],
	specular_color: [f32; 3],
	specular_shininess: f32,
	enable_background: bool,
	debug_window_visible: bool,
	flashlight_enabled: bool,
	rizz_mode: bool,
	pov_camera: bool,
	cursor_lock: bool,
	fullscreen: bool,
	vsync: bool,
	window_focused: bool,
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
			debug_window_visible: false,
			flashlight_enabled: false,
			rizz_mode: false,
			pov_camera: true,
			cursor_lock: true,
			fullscreen: false,
			vsync: true,
			window_focused: true,
		}
	}
}

struct Perf {
	start_time: Instant,
	last_time: Instant,
	last_update: Instant,
	fps_update_interval: Duration,
	fps_string: String,
	accumulated_dt: f32,
	accumulated_count: usize
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
			accumulated_dt: 0.0,
			accumulated_count: 0
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

		info!("Loading assets archive...");

		let files = assets::Assets::parse_from_data(&std::fs::read("assets.bin").unwrap()).unwrap();

		info!("Loading music and sound effects...");

		let audio = Audio::init(&files.music, &files.sounds);

		let assets = Assets::init(gl.clone(), &files.shader_programs, &files.models, &files.textures);

		let mut scene = Scene::new(gl.clone(), &assets);
		scene.state = SceneState::Title;

		let egui = egui_glow::EguiGlow::new(event_loop, gl.clone(), None, None, true);
		let perf = Perf::default();
		let params = Parameters::default();

		// Start with the default fonts (we will be adding to them rather than replacing them).
		let mut fonts = egui::FontDefinitions::default();

		// Install my own font (maybe supporting non-latin characters).
		// .ttf and .otf files supported.
		fonts.font_data.insert(
			"mono".to_owned(),
			std::sync::Arc::new(egui::FontData::from_static(include_bytes!(
				"../static_assets/notosansmono-ascii.ttf"
			))),
		);

		// Put my font first (highest priority) for proportional text:
		fonts
			.families
			.entry(egui::FontFamily::Proportional)
			.or_default()
			.insert(0, "mono".to_owned());

		// Put my font as last fallback for monospace:
		fonts
			.families
			.entry(egui::FontFamily::Monospace)
			.or_default()
			.push("mono".to_owned());

		// Tell egui to use these fonts:
		egui.egui_ctx.set_fonts(fonts);

		window.set_visible(true);

		audio.play_music(MusicRequest::Title);

		let mut app = App {
			window,
			egui,
			gl,
			gl_context,
			gl_surface,

			assets,
			audio,
			scene,
			perf,
			params,
		};

		app.update_cursor_lock();

		app
	}

	fn respawn(&mut self) {
		self.scene = Scene::new(self.gl.clone(), &self.assets);
		self.audio.play_music(MusicRequest::InGame);
	}

	fn handle_key_event(&mut self, event_loop: &ActiveEventLoop, event: KeyEvent) {
		info!("{:?} key: {:?}", event.state, event.physical_key);

		let enable = event.state == ElementState::Pressed;

		match self.scene.state {
			SceneState::InGame { .. } => {
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
			},
			SceneState::Dead => {
				if event.physical_key == PhysicalKey::Code(KeyCode::KeyR) && enable {
					self.respawn();
				}
			},
			SceneState::YoureWinner { .. } => {
				if event.physical_key == PhysicalKey::Code(KeyCode::KeyR) && enable {
					self.respawn();
				}
			},
			SceneState::Title => {},
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

	fn fire_bullet_from_player(&mut self) {
		match self.scene.state {
			SceneState::Title => {
				self.respawn();
			}
			SceneState::InGame { .. } => {
				if self.scene.player.fire_bullet() {
					self.audio.play_sound(SoundRequest::PlayerShoot, None, 1.0);
					self.scene.bullets.spawn_bullet(self.scene.player.get_transform().clone(), 15.0);
				}
			},
			_ => {}
		}
	}

	fn redraw_ui(&mut self, event_loop: &ActiveEventLoop) {
		self.egui.run(&self.window, |ctx| {
			if self.params.debug_window_visible || !self.params.cursor_lock {
				egui::Window::new("Debug window")
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

						ui.checkbox(&mut self.params.debug_window_visible, "Debug window always visible");

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

			}

			match self.scene.state {
				SceneState::Title => {
					let color = egui::Color32::from_rgba_unmultiplied(0, 0, 0, 0);
					let frame = egui::Frame::new().fill(color);
					egui::CentralPanel::default().frame(frame).show(ctx, |ui| {
						let height = ui.available_size().y;
						ui.vertical_centered(|ui| {
							ui.add_space((height - 80.0) / 2.0);
							ui.label(egui::RichText::new("Wurstenstein 3D").size(40.0).color(egui::Color32::WHITE));
							ui.add_space(20.0);
							ui.label(egui::RichText::new("Shoot to start").size(20.0).color(egui::Color32::WHITE));
						});
					});
				},
				SceneState::InGame { timer, kills } => {
					egui::TopBottomPanel::new(egui::panel::TopBottomSide::Bottom, egui::Id::new("hud")).show(ctx, |ui| {
						ui.horizontal(|ui| {
							let stats = self.scene.player.get_stats();

							let health = format!("Health: {}/{}", stats.health, MAX_HEALTH);
							let speed = if stats.speed_timer > 0.0 {
								format!("Speed: turbo {:.1}s", stats.speed_timer)
							} else {
								"Speed: normal".to_string()
							};
							let ammo = format!("Ammo: {}/{}", stats.ammo, MAX_AMMO);

							let health = egui::RichText::new(health).size(20.0).color(egui::Color32::RED);
							let speed = egui::RichText::new(speed).size(20.0).color(egui::Color32::GREEN);
							let ammo = egui::RichText::new(ammo).size(20.0).color(egui::Color32::BLUE);

							ui.label(health);
							ui.separator();
							ui.label(speed);
							ui.separator();
							ui.label(ammo);

							let remaining = timer as u32;
							let timer = egui::RichText::new(format!("{:02}:{:02}", remaining / 60, remaining % 60)).size(20.0).strong();

							let kills = egui::RichText::new(format!("Kills: {}", kills)).size(20.0).strong();

							ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
								ui.label(timer);
								ui.separator();
								ui.label(kills);
							});
						});
					});
				},
				SceneState::Dead => {
					let color = egui::Color32::from_rgba_unmultiplied(255, 0, 0, 160);
					let frame = egui::Frame::new().fill(color);
					egui::CentralPanel::default().frame(frame).show(ctx, |ui| {
						let height = ui.available_size().y;
						ui.vertical_centered(|ui| {
							ui.add_space((height - 80.0) / 2.0);
							ui.label(egui::RichText::new("You are dead").size(40.0).color(egui::Color32::WHITE));
							ui.add_space(20.0);
							ui.label(egui::RichText::new("Press R to respawn").size(20.0).color(egui::Color32::WHITE));
						});
					});
				},
				SceneState::YoureWinner { kills } => {
					let color = egui::Color32::from_rgba_unmultiplied(0, 255, 0, 160);
					let frame = egui::Frame::new().fill(color);
					egui::CentralPanel::default().frame(frame).show(ctx, |ui| {
						let height = ui.available_size().y;
						ui.vertical_centered(|ui| {
							ui.add_space((height - 120.0) / 2.0);
							ui.label(egui::RichText::new("You win!").size(40.0).color(egui::Color32::WHITE));
							ui.add_space(20.0);
							ui.label(egui::RichText::new(format!("{} bad guys were eliminated.", kills)).size(20.0).color(egui::Color32::WHITE));
							ui.add_space(20.0);
							ui.label(egui::RichText::new("Press R to play again.").size(20.0).color(egui::Color32::WHITE));
						});
					});
				}
			}
		});

		self.egui.paint(&self.window);
	}

	fn update_perf_data(&mut self, dt: f32) {
		self.perf.accumulated_dt += dt;
		self.perf.accumulated_count += 1;

		if self.perf.last_update.elapsed() >= self.perf.fps_update_interval {
			let fps = (1.0 / self.perf.accumulated_dt) * self.perf.accumulated_count as f32;

			self.perf.accumulated_count = 0;
			self.perf.accumulated_dt = 0.0;

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
				"Wurstenstein 3D - {}, {}, {}",
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

		if self.params.cursor_lock && self.params.window_focused {
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

		if let SceneState::InGame { timer, kills } = &mut self.scene.state {
			if !self.scene.player.is_dead() {
				self.scene.player.has_contact_with_world = collision::check_with_ground(&self.scene.player, &EXAMPLE_MAZE);

				if let Some(idx) = collision::check_with_powerups(&self.scene.player, &self.scene.powerups) && let Some(kind) = self.scene.powerups.pick_up(idx) {
					self.scene.player.pick_up_powerup(kind);

					match kind {
						PowerupKind::Health => self.audio.play_sound(SoundRequest::PowerupHpPickup, None, 1.0),
						PowerupKind::Energy => self.audio.play_sound(SoundRequest::PowerupEnergyPickup, None, 1.0),
						PowerupKind::Speed => self.audio.play_sound(SoundRequest::PowerupSpeedPickup, None, 1.0),
					}
				}

				if let Some(idx) = collision::check_with_enemies(&self.scene.player, &self.scene.enemies) && let Some(damage) = self.scene.enemies.collide_with_player(idx) {
					if self.scene.player.decrease_hp(damage) {
						self.audio.play_sound(SoundRequest::EnemyHit, None, 1.0);
					}
				}

				for (enemy_idx, bullet_idx) in collision::check_enemies_with_bullets(&self.scene.enemies, &self.scene.bullets) {
					*kills += 1;
					if let Some(transform) = self.scene.enemies.get_transform(enemy_idx) {
						self.audio.play_sound(SoundRequest::EnemyDeath, Some(transform.position.into()), 10.0);
					}

					self.scene.enemies.collide_with_bullet(enemy_idx);
					self.scene.bullets.despawn_bullet(bullet_idx);
				}

				for bullet_idx in collision::check_player_with_bullet(&self.scene.player, &self.scene.bullets) {
					if self.scene.player.decrease_hp(2) {
						self.audio.play_sound(SoundRequest::EnemyHit, None, 1.0);
					}
					self.scene.bullets.despawn_bullet(bullet_idx);
				}

				self.scene.player.update_yaw(self.scene.camera.get_yaw_pitch().0);

				*timer = (*timer - dt).max(0.0);

				if *timer == 0.0 {
					self.scene.state = SceneState::YoureWinner { kills: *kills };
					self.audio.play_music(MusicRequest::Win);
				}
			}

			if let Some(action) = self.scene.player.update(&EXAMPLE_MAZE, dt) {
				match action {
					PlayerAction::Jumped => {
						self.audio.play_sound(SoundRequest::PlayerJump, None, 1.0);
					},
					PlayerAction::StartedDying => {
						self.audio.play_sound(SoundRequest::PlayerDeath, None, 1.0);
						self.audio.play_music(MusicRequest::Stop);
					},
					PlayerAction::Dead => {
						self.audio.play_sound(SoundRequest::PlayerExplosion, None, 1.0);
						self.scene.explosions.add_explosion(self.scene.player.get_transform().position.into());
						self.audio.play_music(MusicRequest::Death);
						self.scene.state = SceneState::Dead;
					}
				}
			}

			self.update_camera(dt);

			let pos = self.scene.player.get_transform().position;
			let rot = self.scene.player.get_transform().rotation[0];

			self.audio.update_position(pos.into(), rot);

			self.scene.powerups.update(&EXAMPLE_MAZE, dt);
			let enemy_updates = self.scene.enemies.update(&EXAMPLE_MAZE, dt);
			for bullet_pos in enemy_updates.shot_bullets {
				let angle = {
					let bullet_pos = glm::vec2(bullet_pos[0], bullet_pos[2]);

					let player_pos = self.scene.player.get_transform().position;
					let player_pos = glm::vec2(player_pos[0], player_pos[2]);

					let diff = player_pos - bullet_pos;

					let base = glm::vec2(0.0, 1.0);

					let mul = if glm::cross2d(&base, &diff) >= 0.0 {
						1.0
					} else {
						-1.0
					};

					glm::angle(&glm::vec2(0.0, 1.0), &diff) * mul
				};

				let transform = Transform::origin().with_position(bullet_pos.into()).with_rotation([std::f32::consts::PI - angle, 0.0, 0.0].into());

				self.scene.bullets.spawn_bullet(transform, 15.0);
			}
			for gone_pos in enemy_updates.enemies_gone {
				self.audio.play_sound(SoundRequest::EnemyExplosion, Some(gone_pos), 10.0);
				self.scene.explosions.add_explosion(gone_pos);
			}
			self.scene.bullets.update(dt);
		}

		self.scene.explosions.update(dt);

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

		self.assets.explosion_program.set_uniform_matrix_f32_4("view", view_mtx.as_slice().try_into().unwrap());
		self.assets.explosion_program.set_uniform_matrix_f32_4("projection", projection_mtx.as_slice().try_into().unwrap());

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

		self.scene.powerups.update_point_lights(program);

		if self.params.flashlight_enabled {
			let mut transform = self.scene.player.get_transform().clone();
			transform.position[1] += 1.0;

			let rotated = glm::rotate_vec2(&glm::vec2(0.0, 1.0), std::f32::consts::PI - transform.rotation[0]);

			program.set_uniform_u32("spot_enabled", 1);
			program.set_uniform_f32_3("spot_position", &transform.position.as_slice().try_into().unwrap());
			program.set_uniform_f32_3("spot_direction", &[rotated[0], 0.0, rotated[1]]);
			program.set_uniform_f32("spot_cos_cutoff", 1.5f32.to_radians().cos());
			program.set_uniform_f32_3("spot_diffuse", &[2.0, 2.0, 2.0]);
			program.set_uniform_f32_3("spot_specular", &[0.0, 0.0, 0.0]);
		} else {
			program.set_uniform_u32("spot_enabled", 0);
		}

		self.init_drawing();

		if self.params.enable_background {
			self.assets.background.draw(&self.assets.background_program);
		}

		if self.scene.state != SceneState::Title {
			self.assets.terrain.draw(&Transform::origin(), program, "model");

			self.assets.player.draw(self.scene.player.get_transform(), program, "model");
			if self.scene.player.get_stats().ammo > 0 {
				self.assets.sausage_tip.draw(self.scene.player.get_transform(), program, "model");
			}

			self.scene.enemies.render(&self.assets, program);
			self.scene.bullets.render(&self.assets, program);
			self.scene.explosions.render(&self.assets);

			let mut transparent = TransparentRenderer::new(self.gl.clone());

			self.scene.powerups.render(&self.assets, &mut transparent);

			transparent.render(view_mtx);
		}

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
			},
			DeviceEvent::MouseWheel { delta } => {
				if !self.params.cursor_lock {
					return;
				};

				if let MouseScrollDelta::LineDelta(_, y) = delta {
					self.handle_mouse_wheel(y);
				}
			},
			_ => {}
		}
	}

	pub fn handle_window_event(&mut self, event_loop: &ActiveEventLoop, event: WindowEvent) {
		let _ = self.egui.on_window_event(&self.window, &event);

		match event {
			WindowEvent::KeyboardInput { event, .. } if !event.repeat => {
				self.handle_key_event(event_loop, event);
			}
			WindowEvent::MouseInput { state, button, .. } => {
				if button == MouseButton::Left && state == ElementState::Pressed {
					self.fire_bullet_from_player();
				}

				if button == MouseButton::Right {
					self.params.flashlight_enabled = state == ElementState::Pressed;
				}
			},
			WindowEvent::CloseRequested => {
				event_loop.exit();
			}
			WindowEvent::Resized(new_size) => {
				self.handle_resize_event(new_size);
			}
			WindowEvent::RedrawRequested => {
				self.redraw(event_loop);
			}
			WindowEvent::Focused(focused) => {
				self.params.window_focused = focused;
			}
			_ => {}
		}
	}

	pub fn get_window(&self) -> &Window {
		&self.window
	}
}
