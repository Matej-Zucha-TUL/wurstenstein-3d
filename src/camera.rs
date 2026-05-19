use nalgebra_glm as glm;

pub enum Directions {
	Left,
	Right,
	Up,
	Down,
	Forward,
	Backward,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Camera {
	position: glm::Vec3,
	front: glm::Vec3,
	up: glm::Vec3,
	right: glm::Vec3,
	world_up: glm::Vec3,

	use_pov: bool,
	target: glm::Vec3,
	distance: f32,

	move_forward: bool,
	move_backward: bool,
	move_left: bool,
	move_right: bool,
	move_up: bool,
	move_down: bool,
	move_fast: bool,

	yaw: f32,
	pitch: f32,
	speed: f32,
	sensitivity: f32,
	zoom: f32,
}

impl Camera {
	pub fn new(position: glm::Vec3) -> Self {
		let yaw = 45.0f32;
		let pitch = -23.0f32;
		let world_up = glm::vec3(0.0, 1.0, 0.0);
		let front = Camera::calc_front(yaw, pitch);
		let right = Camera::calc_right(&front, &world_up);
		let up = Camera::calc_up(&right, &front);

		Self {
			position,
			front,
			up,
			right,
			world_up,
			use_pov: false,
			target: glm::vec3(0.0, 0.0, 0.0),
			distance: 20.0f32,
			move_forward: false,
			move_backward: false,
			move_left: false,
			move_right: false,
			move_up: false,
			move_down: false,
			move_fast: false,
			yaw,
			pitch,
			speed: 2.5f32,
			sensitivity: 0.1f32,
			zoom: 45.0f32,
		}
	}

	pub fn set_pov(&mut self, enabled: bool) {
		self.use_pov = enabled;
	}
	pub fn set_target(&mut self, target: glm::Vec3) {
		self.target = target;
	}
	pub fn get_zoom(&self) -> f32 {
		self.zoom
	}
	pub fn get_position(&self) -> &glm::Vec3 {
		&self.position
	}
	pub fn get_front(&self) -> &glm::Vec3 {
		&self.front
	}
	pub fn get_yaw_pitch(&self) -> (f32, f32) {
		(self.yaw, self.pitch)
	}

	pub fn get_view_matrix(&self) -> glm::Mat4 {
		if self.use_pov {
			let eye = self.target - self.front * self.distance;
			glm::look_at(&eye, &self.target, &self.up)
		} else {
			glm::look_at(&self.position, &(self.position + self.front), &self.up)
		}
	}

	pub fn move_fast(&mut self, fast: bool) {
		self.move_fast = fast;
	}

	pub fn key_interact(&mut self, direction: Directions, pressed: bool) {
		match direction {
			Directions::Left => self.move_left = pressed,
			Directions::Right => self.move_right = pressed,
			Directions::Up => self.move_up = pressed,
			Directions::Down => self.move_down = pressed,
			Directions::Forward => self.move_forward = pressed,
			Directions::Backward => self.move_backward = pressed,
		}
	}

	pub fn update_position(&mut self, dt: f32) {
		let dt = if self.move_fast { dt * 3.0 } else { dt };

		if self.use_pov {
			let orbit_speed = 67.0f32;

			if self.move_left {
				self.yaw -= orbit_speed * dt;
				self.update_vectors();
			}

			if self.move_right {
				self.yaw += orbit_speed * dt;
				self.update_vectors();
			}

			if self.move_forward {
				self.distance = (self.distance - self.speed * dt).max(0.5);
			}

			if self.move_backward {
				self.distance += self.speed * dt;
			}

			if self.move_up {
				self.pitch = (self.pitch + orbit_speed * dt).clamp(-89.0, 89.0);
				self.update_vectors();
			}

			if self.move_down {
				self.pitch = (self.pitch - orbit_speed * dt).clamp(-89.0, 89.0);
				self.update_vectors();
			}
		} else {
			if self.move_forward {
				self.position += self.front * self.speed * dt;
			}

			if self.move_left {
				self.position -= self.right * self.speed * dt;
			}

			if self.move_right {
				self.position += self.right * self.speed * dt;
			}

			if self.move_up {
				self.position += self.up * self.speed * dt;
			}

			if self.move_down {
				self.position -= self.up * self.speed * dt;
			}

			if self.move_backward {
				self.position -= self.front * self.speed * dt;
			}
		}
	}

	pub fn mouse_interact(&mut self, dx: f32, dy: f32) {
		self.yaw += dx * self.sensitivity;
		self.pitch = (self.pitch - dy * self.sensitivity).clamp(-89.0, 89.0);
		self.update_vectors();
	}

	pub fn scroll_wheel_interact(&mut self, delta: f32) {
		self.zoom = (self.zoom + delta).clamp(30.0, 150.0);
	}

	fn update_vectors(&mut self) {
		self.front = Camera::calc_front(self.yaw, self.pitch);
		self.right = Camera::calc_right(&self.front, &self.world_up);
		self.up = Camera::calc_up(&self.right, &self.front);
	}

	fn calc_front(yaw: f32, pitch: f32) -> glm::Vec3 {
		let ya = yaw.to_radians();
		let pa = pitch.to_radians();

		glm::vec3(ya.cos() * pa.cos(), pa.sin(), ya.sin() * pa.cos()).normalize()
	}

	fn calc_right(front: &glm::Vec3, world_up: &glm::Vec3) -> glm::Vec3 {
		glm::cross(front, world_up).normalize()
	}

	fn calc_up(right: &glm::Vec3, front: &glm::Vec3) -> glm::Vec3 {
		glm::cross(right, front).normalize()
	}
}
