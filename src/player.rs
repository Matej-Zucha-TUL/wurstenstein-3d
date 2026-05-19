use nalgebra_glm as glm;

use crate::model::Transform;

pub struct PlayerController {
	pub move_forward: bool,
	pub move_backward: bool,
	pub move_left: bool,
	pub move_right: bool,
	pub jump: bool,
	gravity: f32,
	xz_force: [f32; 2],
	transform: Transform
}

impl PlayerController {
	pub fn new(spawn: Transform) -> Self {
		Self {
			move_forward: false,
			move_backward: false,
			move_left: false,
			move_right: false,
			jump: false,
			gravity: 0.0,
			xz_force: [0.0, 0.0],
			transform: spawn
		}
	}

	pub fn update_yaw(&mut self, yaw: f32) {
		self.transform.rotation[0] = -(yaw + 90.0).to_radians();
	}

	pub fn update_position(&mut self, dt: f32) {
		const MAX_SPEED: f32 = 5.0;
		const ACCEL: f32 = 20.0;
		const BASE_GRAVITY: f32 = 10.0;
		const BASE_GRAVITY_ACCEL: f32 = 40.0;

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

		if self.jump && self.gravity >= (BASE_GRAVITY - 0.1) {
			self.gravity = -BASE_GRAVITY;
		}

		self.jump = false;

		self.xz_force = xz_force;

		let rotated = glm::rotate_vec2(&xz_force.into(), -self.transform.rotation[0]);

		self.transform.position[0] += rotated[0] * dt;
		self.transform.position[1] = f32::max(0.0, self.transform.position[1] - self.gravity * dt * 2.0);
		self.transform.position[2] += rotated[1] * dt;

		self.gravity = f32::min(self.gravity + BASE_GRAVITY_ACCEL * dt, BASE_GRAVITY);
	}

	pub fn get_transform(&self) -> &Transform {
		&self.transform
	}
}


