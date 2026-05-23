use nalgebra_glm as glm;
use parry2d::math::{Pose, Rot2, Vec2};
use parry2d::query;
use parry2d::shape::Cuboid;

use crate::assets::BoundingBox;
use crate::model::Transform;
use crate::playfield::{Playfield, PlayfieldPiece};

pub struct PlayerController {
	pub move_forward: bool,
	pub move_backward: bool,
	pub move_left: bool,
	pub move_right: bool,
	pub jump: bool,
	fall_jump_triggered: bool,
	force_jump: bool,
	already_fell_to_death: bool,
	bounding_box: BoundingBox,
	gravity: f32,
	xz_force: [f32; 2],
	transform: Transform
}

pub enum PlayerAction {
	Jumped,
	FellToDeath
}

impl PlayerController {
	pub fn new(spawn: Transform, bounding_box: BoundingBox) -> Self {
		Self {
			move_forward: false,
			move_backward: false,
			move_left: false,
			move_right: false,
			jump: false,
			fall_jump_triggered: false,
			force_jump: false,
			already_fell_to_death: false,
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
		let mut action = None;

		const MAX_SPEED: f32 = 5.0;
		const ACCEL: f32 = 20.0;
		const BASE_GRAVITY: f32 = 10.0;
		const BASE_GRAVITY_ACCEL: f32 = 40.0;

		// Update XZ coordinates

		let accel = ACCEL * dt;

		let mut xz_force = self.xz_force;

		if self.move_left {
			xz_force[0] = f32::max(xz_force[0] - accel, -MAX_SPEED);
		} else if xz_force[0] < 0.0 {
			xz_force[0] = f32::min(xz_force[0] + accel, 0.0);
		}

		if self.move_right {
			xz_force[0] = f32::min(xz_force[0] + accel, MAX_SPEED);
		} else if xz_force[0] > 0.0 {
			xz_force[0] = f32::max(xz_force[0] - accel, 0.0);
		}

		if self.move_forward {
			xz_force[1] = f32::max(xz_force[1] - accel, -MAX_SPEED);
		} else if xz_force[1] < 0.0 {
			xz_force[1] = f32::min(xz_force[1] + accel, 0.0);
		}

		if self.move_backward {
			xz_force[1] = f32::min(xz_force[1] + accel, MAX_SPEED);
		} else if xz_force[1] > 0.0 {
			xz_force[1] = f32::max(xz_force[1] - accel, 0.0);
		}

		self.xz_force = xz_force;

		let rotated = glm::rotate_vec2(&xz_force.into(), -self.transform.rotation[0]);

		self.transform.position[0] += rotated[0] * dt;
		self.transform.position[2] += rotated[1] * dt;

		// Check collision with world

		let (player_shape, mut player_pose) = self.bounding_box.get_collision_shape();
		let (w, h) = world.dimensions();

		player_pose.rotation = Rot2::from_angle(-self.transform.rotation[0]);
		player_pose.translation += Vec2::new(self.transform.position[0], self.transform.position[2]);

		let nearest_world_x = (self.transform.position[0] / world.scale).round() as isize;
		let nearest_world_z = (self.transform.position[2] / world.scale).round() as isize;

		let world_shape = Cuboid::new(Vec2::new(world.scale / 2.0, world.scale / 2.0));

		let mut has_contact_with_world = false;

		'check_loop: for x in -1..=0 {
			for z in -1..=0 {
				let x = nearest_world_x + x;
				let z = nearest_world_z + z;

				if x < 0 || x >= w as isize {
					continue
				}

				if z < 0 || z >= h as isize {
					continue
				}

				let x = x as usize;
				let z = z as usize;

				if world.field[z][x].is_empty() { continue }

				let mut world_pose = Pose::translation(world.scale / 2.0, world.scale / 2.0); // Offset the center of the world piece

				world_pose.translation += Vec2::new(x as f32 * world.scale, z as f32 * world.scale);

				if query::intersection_test(&player_pose, &player_shape, &world_pose, &world_shape).unwrap() {
					has_contact_with_world = true;
					break 'check_loop;
				}
			}
		}

		// Fall to death if the player is not touching the world, or if the player has fallen enough (we allow a little edgebug to make the game more fair)

		let floor = if !has_contact_with_world || self.transform.position[1] < -world.height {
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

		if self.transform.position[1] <= world.death_barrier + 0.01 && !self.already_fell_to_death {
			action = Some(PlayerAction::FellToDeath);
			self.already_fell_to_death = true;
		}

		action
	}

	pub fn get_transform(&self) -> &Transform {
		&self.transform
	}
}


