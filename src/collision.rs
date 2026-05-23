use parry2d::math::{Pose, Rot2, Vec2};
use parry2d::query;
use parry2d::shape::Cuboid;

use crate::enemy::EnemyManager;
use crate::playfield::{Playfield, PlayfieldPiece};
use crate::player::PlayerController;
use crate::powerup::PowerupManager;

pub fn check_with_ground<T: PlayfieldPiece>(player: &PlayerController, world: &Playfield<'_, T>) -> bool {
	let transform = player.get_transform();
	let (player_shape, mut player_pose) = player.get_collision_shape();
	let (w, h) = world.dimensions();

	player_pose.rotation = Rot2::from_angle(-transform.rotation[0]);
	player_pose.translation += Vec2::new(transform.position[0], transform.position[2]);

	let nearest_world_x = (transform.position[0] / world.scale).round() as isize;
	let nearest_world_z = (transform.position[2] / world.scale).round() as isize;

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

	has_contact_with_world
}

pub fn check_with_powerups(player: &PlayerController, powerups: &PowerupManager) -> Option<usize> {
	let transform = player.get_transform();
	let (player_shape, mut player_pose) = player.get_collision_shape();

	player_pose.rotation = Rot2::from_angle(-transform.rotation[0]);
	player_pose.translation += Vec2::new(transform.position[0], transform.position[2]);

	let powerups = powerups.get_collision_shapes();

	for (idx, powerup) in powerups.iter().enumerate() {
		let Some((powerup_shape, powerup_pose)) = powerup else { continue };

		if query::intersection_test(&player_pose, &player_shape, powerup_pose, powerup_shape).unwrap() {
			return Some(idx);
		}
	}

	None
}

pub fn check_with_enemies(player: &PlayerController, enemies: &EnemyManager) -> Option<usize> {
	let transform = player.get_transform();
	let (player_shape, mut player_pose) = player.get_collision_shape();

	player_pose.rotation = Rot2::from_angle(-transform.rotation[0]);
	player_pose.translation += Vec2::new(transform.position[0], transform.position[2]);

	let enemies = enemies.get_collision_shapes();

	for (idx, enemy) in enemies.iter().enumerate() {
		let Some((enemy_shape, enemy_pose)) = enemy else { continue };

		if query::intersection_test(&player_pose, &player_shape, enemy_pose, enemy_shape).unwrap() {
			return Some(idx);
		}
	}

	None
}
