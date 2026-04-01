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
	position    : glm::Vec3,
	front       : glm::Vec3,
	up          : glm::Vec3,  
	right       : glm::Vec3,  
	world_up    : glm::Vec3,  
  
  use_pov     : bool,
  distance    : f32,
	
	yaw         : f32,
	pitch       : f32, 
	speed       : f32,
	sensitivity : f32,
	zoom        : f32,   
}

impl Camera {
	pub fn new( position: glm::Vec3 ) -> Self {

		let yaw = -90.0f32;
		let pitch = 0.0f32;
		let world_up = glm::vec3(0.0,1.0,0.0 );
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
      distance: 5.0f32,
			yaw,
			pitch, 
			speed       : 2.5f32,
			sensitivity : 0.1f32,
			zoom        : 45.0f32, 
		}
	}


  pub fn set_pov(&mut self, enabled: bool) {self.use_pov = enabled;}
	pub fn get_zoom(&self) -> f32 {self.zoom}
	pub fn get_position(&self) -> &glm::Vec3 {&self.position}
	pub fn get_front(&self) -> &glm::Vec3 {&self.front}
	pub fn get_yaw_pitch(&self) -> (f32, f32) { (self.yaw, self.pitch) }

	pub fn get_view_matrix(&self) -> glm::Mat4 {
    if self.use_pov {
      let eye = self.position - self.front * self.distance;
      glm::look_at(&eye, &self.position, &self.up)
    } else {
      glm::look_at(
        &self.position,
        &(self.position + self.front),
        &self.up,
      )
    }
	}

	pub fn key_interact(&mut self, direction: Directions, dt: f32) {
      if self.use_pov {
            let orbit_speed = 67.0f32;
            match direction {
                Directions::Left => {
                    self.yaw -= orbit_speed * dt;
                    self.update_vectors();
                }
                Directions::Right => {
                    self.yaw += orbit_speed * dt;
                    self.update_vectors();
                }
                Directions::Forward => {
                    self.distance =
                        (self.distance - self.speed * dt).max(0.5);
                }
                Directions::Backward => {
                    self.distance += self.speed * dt;
                }
                Directions::Up => {
                    self.pitch =
                        (self.pitch + orbit_speed * dt).clamp(-89.0, 89.0);
                    self.update_vectors();
                }
                Directions::Down => {
                    self.pitch =
                        (self.pitch - orbit_speed * dt).clamp(-89.0, 89.0);
                    self.update_vectors();
                }
            }
        } else {
        match direction {
          Directions::Forward => {
            self.position += self.front * self.speed * dt;
          },
          Directions::Left => {
            self.position -= self.right * self.speed * dt;
          },
          Directions::Right => {
            self.position += self.right * self.speed * dt;
          },
          Directions::Up => {
            self.position += self.up * self.speed * dt;
          },
          Directions::Down => {
            self.position -= self.up * self.speed * dt;
          },
          Directions::Backward => {
          self.position -= self.front * self.speed * dt;
        }
      }
    }
	}

	pub fn mouse_interact(&mut self,  dx: f32, dy : f32 ) {
		self.yaw += dx * self.sensitivity;
		self.pitch = (self.pitch - dy * self.sensitivity).clamp(-89.0, 89.0);
    self.update_vectors();
	}

	pub fn scroll_wheel_interact(&mut self, delta: f32) {
		self.zoom = (self.zoom + delta).clamp(1.0, 55.0);
	}

  fn update_vectors(&mut self) {
    self.front = Camera::calc_front(self.yaw, self.pitch);
    self.right = Camera::calc_right(&self.front, &self.world_up);
    self.up = Camera::calc_up(&self.right, &self.front);
  }

	fn calc_front(yaw:f32, pitch: f32) -> glm::Vec3 {
		let ya = yaw.to_radians();
		let pa = pitch.to_radians();

		glm::vec3(
			ya.cos() * pa.cos(),
			pa.sin(),
			ya.sin() * pa.cos()
		).normalize()
	}

	fn calc_right(front :&glm::Vec3, world_up: &glm::Vec3) -> glm::Vec3 {
		glm::cross(front, world_up).normalize()
	}

	fn calc_up(right :&glm::Vec3, front: &glm::Vec3) -> glm::Vec3 {
		glm::cross(right, front).normalize()
	}
}
