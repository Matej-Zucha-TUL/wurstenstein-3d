use parry2d::math::Pose;
use parry2d::shape::Ball;
use rand::RngExt as _;
use rand::rngs::ThreadRng;

use crate::assets::Assets;
use crate::model::Transform;
use crate::playfield::{Playfield, PlayfieldPiece};
use crate::shader::Program;

pub enum EnemyKind {
	Apple
}

#[derive(PartialEq, Eq)]
enum EnemyState {
	Spawn,
	Idle,
	Despawn,
	Gone
}

pub struct Enemy {
	kind: EnemyKind,
	transform: Transform,
	timer: f32,
	state: EnemyState
}

impl Enemy {
	pub fn new(kind: EnemyKind, transform: Transform) -> Self {
		Self {
			kind,
			transform,
			timer: 0.0,
			state: EnemyState::Spawn
		}
	}

	fn update(&mut self, dt: f32) {
		self.timer += dt;

		match self.state {
			EnemyState::Spawn => {
				self.transform.rotation[0] += 4.0 * dt;
				let scale = (self.timer * 2.0).min(1.0);
				self.transform.scale[0] = scale;
				self.transform.scale[1] = scale;
				self.transform.scale[2] = scale;

				if self.timer >= 0.5 {
					self.timer = 0.0;
					self.state = EnemyState::Idle;
				}
			},
			EnemyState::Idle => {},
			EnemyState::Despawn => {
				self.transform.rotation[0] += 8.0 * dt;
				let scale = (1.0 - self.timer * 2.0).max(0.0);
				self.transform.scale[0] = scale;
				self.transform.scale[1] = scale;
				self.transform.scale[2] = scale;

				self.transform.position[1] += dt * 3.0;

				if self.timer >= 0.5 {
					self.state = EnemyState::Gone;
				}
			},
			EnemyState::Gone => {
				self.transform.scale[0] = 0.0;
				self.transform.scale[1] = 0.0;
				self.transform.scale[2] = 0.0;
			}
		}
	}
}

pub struct EnemyManager {
	enemies: Vec<Option<Enemy>>,
	rng: ThreadRng,
	spawn_timer: f32,
}

impl EnemyManager {
	pub fn new() -> Self {
		Self {
			enemies: vec![],
			rng: rand::rng(),
			spawn_timer: 3.0,
		}
	}

	fn spawn_new_enemy<T: PlayfieldPiece>(&mut self, world: &Playfield<'_, T>) {
		let kind = self.rng.random_range(0..=0);

		let spawn_point_num = world.enemy_spawn_points.len();

		let idx = self.rng.random_range(0..spawn_point_num);

		// Find a free slot for an enemy

		for off in 0..spawn_point_num {
			let idx = (idx + off) % spawn_point_num;

			if self.enemies[idx].is_some() {
				continue
			}

			let pos = world.enemy_spawn_points[idx];
			let pos = pos.map(|x| x as f32 * world.scale + world.scale * 0.5);

			let kind = match kind {
				0 => EnemyKind::Apple,
				_ => unreachable!()
			};

			self.enemies[idx] = Some(Enemy::new(
				kind,
				Transform::origin().with_position([pos[0], 0.0, pos[1]].into())
			));

			break
		}
	}

	pub fn update<T: PlayfieldPiece>(&mut self, world: &Playfield<'_, T>, dt: f32) {
		self.enemies.resize_with(world.enemy_spawn_points.len(), || None);

		self.spawn_timer -= dt;

		if self.spawn_timer < 0.0 {
			self.spawn_timer = 3.0;
			self.spawn_new_enemy(world);
		}

		for enemy in &mut self.enemies {
			let Some(enemak) = enemy else { continue };

			enemak.update(dt);

			if enemak.state == EnemyState::Gone {
				*enemy = None;
			}
		}
	}

	pub fn render(&self, assets: &Assets, program: &Program) {
		for enemy in &self.enemies {
			let Some(enemy) = enemy else { continue };

			let model = match enemy.kind {
				EnemyKind::Apple => &assets.enemy,
			};

			model.draw(&enemy.transform, program, "model");
		}
	}

	pub fn get_collision_shapes(&self) -> Vec<Option<(Ball, Pose)>> {
		self.enemies.iter()
			.map(|x| if let Some(x) = &x && x.state == EnemyState::Idle { Some(x) } else { None })
			.map(|x| x.map(|x| (Ball::new(1.0), Pose::translation(x.transform.position[0], x.transform.position[2]))))
			.collect::<Vec<_>>()
	}

	pub fn collide_with_player(&mut self, idx: usize) -> Option<usize> {
		self.enemies[idx].as_mut().map(|x| {
			match x.kind {
				EnemyKind::Apple => 3
			}
		})
	}
}

