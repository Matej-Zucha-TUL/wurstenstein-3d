use parry2d::{math::Pose, shape::Ball};
use nalgebra_glm as glm;

use crate::assets::Assets;
use crate::model::Transform;
use crate::shader::Program;

#[derive(PartialEq, Eq)]
enum BulletState {
	Flying,
	Despawn,
	Gone,
}

pub struct Bullet {
	transform: Transform,
	velocity: f32,
	timer: f32,
	state: BulletState
}

impl Bullet {
	fn update(&mut self, dt: f32) {
		let vector = glm::vec2(0.0, -1.0);
		let vector = glm::rotate_vec2(&vector, -self.transform.rotation[0]) * dt * self.velocity;

		self.transform.position[0] += vector[0];
		self.transform.position[2] += vector[1];

		self.timer += dt;

		match self.state {
			BulletState::Flying => {
				if self.timer >= 3.0 {
					self.timer = 0.0;
					self.state = BulletState::Despawn;
				}
			},
			BulletState::Despawn => {
				let scale = (1.0 - self.timer * 4.0).max(0.0);
				self.transform.scale[0] = scale;
				self.transform.scale[1] = scale;
				self.transform.scale[2] = scale;

				if self.timer >= 0.25 {
					self.state = BulletState::Gone;
				}
			},
			BulletState::Gone => {
				self.transform.scale[0] = 0.0;
				self.transform.scale[1] = 0.0;
				self.transform.scale[2] = 0.0;
			}
		}
	}
}

pub struct BulletManager {
	bullets: Vec<Option<Bullet>>
}

impl BulletManager {
	pub fn new() -> Self {
		Self {
			bullets: vec![]
		}
	}

	pub fn render(&self, assets: &Assets, program: &Program) {
		for bullet in &self.bullets {
			let Some(bullet) = bullet else { continue };

			assets.sausage_bullet.draw(&bullet.transform, program, "model");
		}
	}

	pub fn update(&mut self, dt: f32) {
		for bullet in &mut self.bullets {
			let Some(sausage) = bullet else { continue };

			sausage.update(dt);

			if sausage.state == BulletState::Gone {
				*bullet = None;
			}
		}
	}

	pub fn spawn_bullet(&mut self, transform: Transform, velocity: f32) {
		// Try to find an existing slot

		let new_bullet = Bullet {
			transform,
			velocity,
			timer: 0.0,
			state: BulletState::Flying
		};

		for bullet in &mut self.bullets {
			if bullet.is_some() { continue }

			*bullet = Some(new_bullet);
			return
		}

		self.bullets.push(Some(new_bullet));
	}

	pub fn get_collision_shapes(&self) -> Vec<Option<(Ball, Pose)>> {
		self.bullets.iter()
			.map(|x| if let Some(x) = &x && x.state == BulletState::Flying { Some(x) } else { None })
			.map(|x| x.map(|x| (Ball::new(1.0), Pose::translation(x.transform.position[0], x.transform.position[2]))))
			.collect::<Vec<_>>()
	}

	pub fn despawn_bullet(&mut self, idx: usize) {
		if let Some(x) = &mut self.bullets[idx] {
			x.timer = 0.0;
			x.state = BulletState::Despawn;
		}
	}
}

