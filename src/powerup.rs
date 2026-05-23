use rand::RngExt as _;
use rand::rngs::ThreadRng;

use crate::assets::Assets;
use crate::model::Transform;
use crate::playfield::{Playfield, PlayfieldPiece};
use crate::transparent::TransparentRenderer;

pub enum PowerupKind {
	Health,
	Energy,
	Speed
}

pub struct Powerup {
	kind: PowerupKind,
	base_y: f32,
	transform: Transform,
	timer: f32
}

impl Powerup {
	pub fn new(kind: PowerupKind, transform: Transform) -> Self {
		Self {
			kind,
			base_y: transform.position[1],
			transform,
			timer: 0.0
		}
	}

	pub fn update(&mut self, dt: f32) {
		self.transform.rotation[0] += 1.0 * dt;
		self.timer += dt;
		self.transform.position[1] = self.base_y + (self.timer * 2.0).sin() * 0.3;
	}
}

pub struct PowerupManager {
	powerups: Vec<Powerup>,
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

		let kind = match kind {
			0 => PowerupKind::Health,
			1 => PowerupKind::Energy,
			2 => PowerupKind::Speed,
			_ => unreachable!()
		};

		let pos = world.powerup_spawn_points[self.rng.random_range(0..=world.powerup_spawn_points.len())];

		let pos = pos.map(|x| x as f32 * world.scale + world.scale * 0.5);

		self.powerups.push(Powerup::new(
			kind,
			Transform::origin().with_position([pos[0], 1.5, pos[1]].into())
		));
	}

	pub fn update<T: PlayfieldPiece>(&mut self, world: &Playfield<'_, T>, dt: f32) {
		self.spawn_timer -= dt;

		if self.spawn_timer < 0.0 {
			self.spawn_timer = 5.0;
			self.spawn_new_powerup(world);
		}

		for powerup in &mut self.powerups {
			powerup.update(dt);
		}
	}

	pub fn render<'a>(&'a self, assets: &'a Assets, transparent: &mut TransparentRenderer<'a>) {
		for powerup in &self.powerups {
			let (model, color) = match powerup.kind {
				PowerupKind::Health => (&assets.powerup_hp, &[1.0, 0.0, 0.0]),
				PowerupKind::Energy => (&assets.powerup_energy, &[0.0, 0.0, 1.0]),
				PowerupKind::Speed => (&assets.powerup_speed, &[0.0, 1.0, 0.0]),
			};

			transparent.add_object(&powerup.transform, || {
				assets.powerup_program.set_uniform_f32_3("base_color", color);
				model.draw(&powerup.transform, &assets.powerup_program, "model");
			});
		}
	}
}

