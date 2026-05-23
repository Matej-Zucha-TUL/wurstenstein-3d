use crate::assets::Assets;
use crate::model::Transform;
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
	powerups: Vec<Powerup>
}

impl PowerupManager {
	pub fn new() -> Self {
		Self {
			powerups: vec![]
		}
	}

	pub fn update(&mut self, dt: f32) {
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

