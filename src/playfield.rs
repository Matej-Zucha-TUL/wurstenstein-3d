use tobj::Mesh;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TestPiece {
	__,
	Stone,
	Brick,
	Enemy,
	Ammo,
	Spawn
}

impl PlayfieldPiece for TestPiece {
	fn is_empty(&self) -> bool {
		*self == Self::__
	}

	fn vert_texture(&self) -> (f32, f32, f32, f32) {
		(0.0, 0.0, 1.0, 1.0)
	}

	fn horiz_texture(&self) -> (f32, f32, f32, f32) {
		(0.0, 0.0, 1.0, 1.0)
	}
}

pub trait PlayfieldPiece {
	// True if the playfield piece should not be a wall.
	fn is_empty(&self) -> bool;

	// Returns coordinates for the wall texture (or undefined if not a wall).
	// (X1, Y1, X2, Y2) - all in range 0.0..=1.0.
	fn vert_texture(&self) -> (f32, f32, f32, f32);

	// Returns coordinates for the floor texture (if not a wall), or the ceiling texture (if a wall).
	// (X1, Y1, X2, Y2) - all in range 0.0..=1.0.
	fn horiz_texture(&self) -> (f32, f32, f32, f32);
}

pub struct Playfield<'a, T: PlayfieldPiece> {
	// X/Z size of each wall piece.
	pub scale: f32,
	// Y size of each wall piece.
	pub height: f32,
	// Vector of rows, containing a vector of cells.
	pub field: &'a [&'a [T]]
}

impl<'a, T: PlayfieldPiece> Playfield<'a, T> {
	pub fn dimensions(&self) -> (usize, usize) {
		let width = self.field.iter()
			.map(|x| x.len())
			.min()
			.unwrap();

		let height = self.field.len();

		(width, height)
	}

	pub fn generate_mesh(&self) -> Mesh {
		let (w, h) = self.dimensions();
		let mut positions = vec![];
		let mut normals = vec![];
		let mut texcoords = vec![];
		let mut indices = vec![];

		for z in 0..h {
			for x in 0..w {
				let piece = &self.field[z][x];

				let elevated = !piece.is_empty();

				// Generate horizontal wall

				{
					let (tx1, ty1, tx2, ty2) = piece.horiz_texture();

					let y = if elevated { self.height } else { 0.0 };

					// Create 4 points

					let pos_start = positions.len() as u32 / 3;

					for inc_z in 0..=1 {
						for inc_x in 0..=1 {
							positions.push((x + inc_x) as f32 * self.scale);
							positions.push(y);
							positions.push((z + inc_z) as f32 * self.scale);

							// Normals will always point up
							normals.push(0.0);
							normals.push(1.0);
							normals.push(0.0);

							texcoords.push(if inc_x == 0 { tx1 } else { tx2 });
							texcoords.push(if inc_z == 0 { ty1 } else { ty2 });
						}
					}

					// Create 2 CCW polygons

					indices.push(pos_start + 2);
					indices.push(pos_start + 1);
					indices.push(pos_start);

					indices.push(pos_start + 2);
					indices.push(pos_start + 3);
					indices.push(pos_start + 1);
				}

				// Generate up to 4 vertical walls

				'vertical: {
					let (tx1, ty1, tx2, ty2) = piece.vert_texture();

					if !elevated { break 'vertical }

					let left_wall = match x {
						0 => true,
						x => self.field[z][x - 1].is_empty()
					};

					let right_wall = match x {
						x if x == w - 1 => true,
						x => self.field[z][x + 1].is_empty()
					};

					let front_wall = match z {
						0 => true,
						z => self.field[z - 1][x].is_empty()
					};

					let back_wall = match z {
						z if z == h - 1 => true,
						z => self.field[z + 1][x].is_empty()
					};
					
					let mut generate_wall = |base: [usize; 2], diff: [usize; 2], normal: [f32; 3], reverse: bool| {
						// Create 4 points

						let pos_start = positions.len() as u32 / 3;

						for inc_y in 0..=1 {
							for inc_xz in 0..=1 {
								positions.push((base[0] + inc_xz * diff[0]) as f32 * self.scale);
								positions.push(if inc_y > 0 { self.height } else { 0.0 });
								positions.push((base[1] + inc_xz * diff[1]) as f32 * self.scale);

								normals.extend_from_slice(&normal);

								texcoords.push(if inc_xz == 0 { tx1 } else { tx2 });
								texcoords.push(if inc_y == 0 { ty1 } else { ty2 });
							}
						}

						// Create 2 CCW polygons

						if reverse {
							indices.push(pos_start + 2);
							indices.push(pos_start);
							indices.push(pos_start + 1);

							indices.push(pos_start + 3);
							indices.push(pos_start + 2);
							indices.push(pos_start + 1);
						} else {
							indices.push(pos_start + 2);
							indices.push(pos_start + 1);
							indices.push(pos_start);

							indices.push(pos_start + 3);
							indices.push(pos_start + 1);
							indices.push(pos_start + 2);
						}
					};

					if left_wall {
						generate_wall([ x, z ], [ 0, 1 ], [ -1.0, 0.0, 0.0 ], true);
					}

					if right_wall {
						generate_wall([ x + 1, z ], [ 0, 1 ], [ 1.0, 0.0, 0.0 ], false);
					}

					if front_wall {
						generate_wall([ x, z ], [ 1, 0 ], [ 0.0, 0.0, -1.0 ], false);
					}

					if back_wall {
						generate_wall([ x, z + 1 ], [ 1, 0 ], [ 0.0, 0.0, 1.0 ], true);
					}
				}
			}
		}

		Mesh {
			positions,
			normals,
			texcoords,
			indices,
			vertex_color: vec![],
			face_arities: vec![],
			normal_indices: vec![],
			texcoord_indices: vec![],
			material_id: None,
		}
	}
}

use TestPiece::*;

pub const EXAMPLE_MAZE: Playfield<TestPiece> = Playfield {
	scale: 5.0,
	height: 2.7,
	field: &[
		&[Stone, Stone, Stone, Stone, Stone, Stone, Stone, Stone, Stone, Stone],
		&[Stone, __,    __,    __,    __,    __,    __,    __,    __,    Stone],
		&[Stone, __,    Brick, Brick, __,    __,    Brick, Brick, __,    Stone],
		&[Stone, __,    Brick, __,    __,    __,    __,    Brick, __,    Stone],
		&[Stone, __,    __,    __,    __,    __,    __,    __,    __,    Stone],
		&[Stone, __,    __,    __,    __,    __,    __,    __,    __,    Stone],
		&[Stone, __,    Brick, __,    __,    __,    __,    Brick, __,    Stone],
		&[Stone, __,    Brick, Brick, __,    __,    Brick, Brick, __,    Stone],
		&[Stone, __,    __,    __,    __,    __,    __,    __,    __,    Stone],
		&[Stone, Stone, Stone, Stone, Stone, Stone, Stone, Stone, Stone, Stone],
	]
};

