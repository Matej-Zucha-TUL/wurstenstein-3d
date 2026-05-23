use rand::RngExt as _;
use rand::rngs::ThreadRng;

use crate::assets::Assets;
use crate::model::Transform;
use crate::playfield::{Playfield, PlayfieldPiece};
use crate::shader::Program;
use crate::transparent::TransparentRenderer;

pub enum PowerupKind {
	Health,
	Energy,
	Speed
}

impl PowerupKind {
	pub fn get_color(&self) -> [f32; 3] {
		match self {
			PowerupKind::Health => [1.0, 0.0, 0.0],
			PowerupKind::Energy => [0.0, 0.0, 1.0],
			PowerupKind::Speed => [0.0, 1.0, 0.0],
		}
	}
}

#[derive(PartialEq, Eq)]
enum PowerupState {
	Spawn,
	Floating,
	PickedUp,
	Gone,
}

pub struct Powerup {
	kind: PowerupKind,
	base_y: f32,
	transform: Transform,
	timer: f32,
	state: PowerupState,
}

impl Powerup {
	pub fn new(kind: PowerupKind, transform: Transform) -> Self {
		Self {
			kind,
			base_y: transform.position[1],
			transform,
			timer: 0.0,
			state: PowerupState::Spawn
		}
	}

	pub fn update(&mut self, dt: f32) {
		self.timer += dt;

		match self.state {
			PowerupState::Spawn => {
				self.transform.rotation[0] += 4.0 * dt;
				let scale = (self.timer * 2.0).min(1.0);
				self.transform.scale[0] = scale;
				self.transform.scale[1] = scale;
				self.transform.scale[2] = scale;

				if self.timer >= 0.5 {
					self.timer = 0.0;
					self.state = PowerupState::Floating;
				}
			},
			PowerupState::Floating => {
				self.transform.rotation[0] += 1.0 * dt;
				self.transform.position[1] = self.base_y + (self.timer * 2.0).sin() * 0.3;
			},
			PowerupState::PickedUp => {
				self.transform.rotation[0] += 4.0 * dt;
				let scale = (1.0 - self.timer * 2.0).max(0.0);
				self.transform.scale[0] = scale;
				self.transform.scale[1] = scale;
				self.transform.scale[2] = scale;

				self.transform.position[1] += dt;

				if self.timer >= 0.5 {
					self.state = PowerupState::Gone;
				}
			},
			PowerupState::Gone => {
				self.transform.scale[0] = 0.0;
				self.transform.scale[1] = 0.0;
				self.transform.scale[2] = 0.0;
			}
		}
	}
}

pub struct PowerupManager {
	powerups: Vec<Option<Powerup>>,
	rng: ThreadRng,
	spawn_timer: f32,
}

impl PowerupManager {
	pub fn new() -> Self {
		Self {
			powerups: vec![],
			rng: rand::rng(),
			spawn_timer: 5.0,
		}
	}

	fn spawn_new_powerup<T: PlayfieldPiece>(&mut self, world: &Playfield<'_, T>) {
		let kind = self.rng.random_range(0..=2);

		let spawn_point_num = world.powerup_spawn_points.len();

		let idx = self.rng.random_range(0..spawn_point_num);

		// Find a free slot for a powerup

		for off in 0..spawn_point_num {
			let idx = (idx + off) % spawn_point_num;

			if self.powerups[idx].is_some() {
				continue
			}

			let pos = world.powerup_spawn_points[idx];
			let pos = pos.map(|x| x as f32 * world.scale + world.scale * 0.5);

			let kind = match kind {
				0 => PowerupKind::Health,
				1 => PowerupKind::Energy,
				2 => PowerupKind::Speed,
				_ => unreachable!()
			};

			self.powerups[idx] = Some(Powerup::new(
				kind,
				Transform::origin().with_position([pos[0], 1.5, pos[1]].into())
			));

			break
		}
	}

	pub fn update<T: PlayfieldPiece>(&mut self, world: &Playfield<'_, T>, dt: f32) {
		self.powerups.resize_with(world.powerup_spawn_points.len(), || None);

		self.spawn_timer -= dt;

		if self.spawn_timer < 0.0 {
			self.spawn_timer = 5.0;
			self.spawn_new_powerup(world);
		}

		for powerup in &mut self.powerups {
			let Some(power) = powerup else { continue };

			power.update(dt);

			if power.state == PowerupState::Gone {
				*powerup = None;
			}
		}
	}

	pub fn update_point_lights(&self, program: &Program) {
		// TODO - upload entire array at once

		for (idx, powerup) in self.powerups.iter().enumerate() {
			let enabled = format!("point_enabled[{idx}]");
			let position = format!("point_position[{idx}]");
			let diffuse = format!("point_diffuse[{idx}]");
			let specular = format!("point_specular[{idx}]");

			if let Some(powerup) = powerup {
				let color = powerup.kind.get_color().map(|x| x * powerup.transform.scale[0]);

				program.set_uniform_u32(&enabled, 1);
				program.set_uniform_f32_3(&position, &powerup.transform.position.as_slice().try_into().unwrap());
				program.set_uniform_f32_3(&diffuse, &color);
				program.set_uniform_f32_3(&specular, &color);
			} else {
				program.set_uniform_u32(&enabled, 0);
			}
		}
	}

	pub fn render<'a>(&'a self, assets: &'a Assets, transparent: &mut TransparentRenderer<'a>) {
		for powerup in &self.powerups {
			let Some(powerup) = powerup else { continue };

			let color = powerup.kind.get_color();

			let model = match powerup.kind {
				PowerupKind::Health => &assets.powerup_hp,
				PowerupKind::Energy => &assets.powerup_energy,
				PowerupKind::Speed => &assets.powerup_speed,
			};

			transparent.add_object(&powerup.transform, move || {
				assets.powerup_program.set_uniform_f32_3("base_color", &color);
				model.draw(&powerup.transform, &assets.powerup_program, "model");
			});
		}
	}
}

