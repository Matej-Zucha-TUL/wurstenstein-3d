use nalgebra_glm as glm;
use parry2d::math::Pose;
use parry2d::shape::Cuboid;

use crate::assets::BoundingBox;
use crate::model::Transform;
use crate::playfield::{Playfield, PlayfieldPiece};
use crate::powerup::PowerupKind;

pub struct PlayerController {
	pub move_forward: bool,
	pub move_backward: bool,
	pub move_left: bool,
	pub move_right: bool,
	pub jump: bool,
	pub has_contact_with_world: bool,
	health: usize,
	ammo: usize,
	powerup_speed_timer: f32,
	fall_jump_triggered: bool,
	force_jump: bool,
	damage_timeout: f32,
	death_from_damage: bool,
	already_dead: bool,
	bounding_box: BoundingBox,
	gravity: f32,
	xz_force: [f32; 2],
	transform: Transform
}

pub enum PlayerAction {
	Jumped,
	FellToDeath,
	DiedFromDamage,
}

pub struct PlayerStats {
	pub health: usize,
	pub ammo: usize,
	pub speed_timer: f32
}

pub const MAX_AMMO: usize = 10;
pub const MAX_HEALTH: usize = 10;
pub const MAX_SPEED_TIMER: f32 = 10.0;

impl PlayerController {
	pub fn new(spawn: Transform, bounding_box: BoundingBox) -> Self {
		Self {
			move_forward: false,
			move_backward: false,
			move_left: false,
			move_right: false,
			jump: false,
			has_contact_with_world: true,
			health: MAX_HEALTH,
			ammo: MAX_AMMO,
			powerup_speed_timer: 0.0,
			fall_jump_triggered: false,
			force_jump: false,
			damage_timeout: 0.0,
			death_from_damage: false,
			already_dead: false,
			bounding_box,
			gravity: 0.0,
			xz_force: [0.0, 0.0],
			transform: spawn
		}
	}

	pub fn update_yaw(&mut self, yaw: f32) {
		self.transform.rotation[0] = -(yaw + 90.0).to_radians();
	}

	pub fn update<T: PlayfieldPiece>(&mut self, world: &Playfield<'_, T>, dt: f32) -> Option<PlayerAction> {
		if self.already_dead {
			return None
		}

		if self.death_from_damage {
			self.death_from_damage = false;
			self.already_dead = true;
			return Some(PlayerAction::DiedFromDamage);
		}

		let mut action = None;

		let max_speed: f32 = if self.powerup_speed_timer > 0.0 { 10.0 } else { 5.0 };

		const ACCEL: f32 = 20.0;
		const BASE_GRAVITY: f32 = 10.0;
		const BASE_GRAVITY_ACCEL: f32 = 40.0;

		// Update XZ coordinates

		let accel = ACCEL * dt;

		let mut xz_force = self.xz_force;

		if self.move_left {
			xz_force[0] = f32::max(xz_force[0] - accel, -max_speed);
		} else if xz_force[0] < 0.0 {
			xz_force[0] = f32::min(xz_force[0] + accel, 0.0);
		}

		if self.move_right {
			xz_force[0] = f32::min(xz_force[0] + accel, max_speed);
		} else if xz_force[0] > 0.0 {
			xz_force[0] = f32::max(xz_force[0] - accel, 0.0);
		}

		if self.move_forward {
			xz_force[1] = f32::max(xz_force[1] - accel, -max_speed);
		} else if xz_force[1] < 0.0 {
			xz_force[1] = f32::min(xz_force[1] + accel, 0.0);
		}

		if self.move_backward {
			xz_force[1] = f32::min(xz_force[1] + accel, max_speed);
		} else if xz_force[1] > 0.0 {
			xz_force[1] = f32::max(xz_force[1] - accel, 0.0);
		}

		self.xz_force = xz_force;

		let rotated = glm::rotate_vec2(&xz_force.into(), -self.transform.rotation[0]);

		self.transform.position[0] += rotated[0] * dt;
		self.transform.position[2] += rotated[1] * dt;

		// Fall to death if the player is not touching the world, or if the player has fallen enough (we allow a little edgebug to make the game more fair)

		let floor = if !self.has_contact_with_world || self.transform.position[1] < -world.height {
			world.death_barrier
		} else {
			if self.transform.position[1] < -0.01 && !self.fall_jump_triggered {
				self.fall_jump_triggered = true;
				self.force_jump = true;
			}

			0.0
		};

		// Allow jumping if at (or very near) floor level

		if self.jump && self.transform.position[1] <= floor + 0.01 {
			self.force_jump = true;
		}

		self.jump = false;

		if self.force_jump {
			self.gravity = -BASE_GRAVITY;
			action = Some(PlayerAction::Jumped);
		}

		self.force_jump = false;

		// Update Y coordinate

		self.transform.position[1] = f32::max(floor, self.transform.position[1] - self.gravity * dt * 2.0);
		
		if self.transform.position[1] == floor && self.gravity >= 0.0 {
			// If at floor level, reset gravity
			self.gravity = 0.0;
			self.fall_jump_triggered = false;
		} else {
			self.gravity = f32::min(self.gravity + BASE_GRAVITY_ACCEL * dt, BASE_GRAVITY);
		}

		if self.transform.position[1] <= world.death_barrier + 0.01 && !self.already_dead {
			action = Some(PlayerAction::FellToDeath);
			self.already_dead = true;
		}

		self.damage_timeout -= dt;

		self.powerup_speed_timer -= dt;

		action
	}

	pub fn get_transform(&self) -> &Transform {
		&self.transform
	}

	pub fn get_collision_shape(&self) -> (Cuboid, Pose) {
		self.bounding_box.get_collision_shape()
	}

	pub fn pick_up_powerup(&mut self, kind: PowerupKind) {
		match kind {
			PowerupKind::Health => {
				self.health = (self.health + 5).min(MAX_HEALTH);
			},
			PowerupKind::Speed => {
				self.powerup_speed_timer = MAX_SPEED_TIMER;
			},
			PowerupKind::Energy => {
				self.ammo = MAX_AMMO;
			}
		}
	}

	pub fn get_stats(&self) -> PlayerStats {
		PlayerStats {
			health: self.health,
			ammo: self.ammo,
			speed_timer: self.powerup_speed_timer.max(0.0)
		}
	}

	pub fn decrease_hp(&mut self, amount: usize) -> bool {
		if self.damage_timeout > 0.0 {
			return false;
		}

		self.health = self.health.saturating_sub(amount);
		self.damage_timeout = 0.5;

		if self.health == 0 {
			self.death_from_damage = true;
		}

		true
	}
}

