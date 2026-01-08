use std::{cell::Cell, rc::Rc};

use actor::{Actor, ActorData, Directions, SurfaceProperties};
use godot::{classes::TileMapLayer, prelude::*};

mod actor;
mod camera;

#[derive(GodotClass)]
#[class(base=Node2D)]
struct PlatformerGame {
	base: Base<Node2D>,
	actors: Vec<Rc<Cell<ActorData>>>,
	actors_that_move: Vec<Rc<Cell<ActorData>>>,

	#[export]
	tilemap: Option<Gd<TileMapLayer>>,
}

#[godot_api]
impl INode2D for PlatformerGame {
	fn init(base: Base<Node2D>) -> Self {
		Self {
			base,
			actors: vec![],
			actors_that_move: vec![],
			tilemap: None,
		}
	}

	fn ready(&mut self) {
		self.register_actors(self.to_gd().upcast());
	}

	fn physics_process(&mut self, _: f64) {
		let tm = self.tilemap.as_ref().unwrap();

		for actor in &self.actors_that_move {
			let mut data = actor.get();
			data.collided_old = data.collided;
			data.collided = Directions::empty();
			data.fall();
			data.next_vel = data.vel.x;
			actor.set(data);
		}

		let mut dirty = true;

		while dirty {
			dirty = false;

			for actor in &self.actors_that_move {
				let mut data = actor.get();

				if data.vel.x > 0 {
					let br = data.pos + data.area_offset + data.area_size;
					let tr = br + actor::Vec {
						y: -data.area_size.y,
						x: -1
					};
					let mbr = br + data.vel;

					let tr = self.tile_pos(tr);
					let b = self.tile_pos(br + actor::Vec {x: 0, y: -1} ).y;
					let mr = self.tile_pos(mbr).x;

					'o: for x in (tr.x + 1)..=mr {
						for y in tr.y..=b {
							if let Some(tiledata) = tm.get_cell_tile_data(Vector2i { x, y }) {
								if tiledata.get_custom_data("Solid").booleanize() {
									data.next_vel = self.un_tile_pos(x) - (data.pos.x + data.area_offset.x + data.area_size.x);
									data.collided |= Directions::RIGHT;
									break 'o;
								}
							}
						}
					}
				} else if data.vel.x < 0 {
					let tl = data.pos + data.area_offset;
					let bl = tl + actor::Vec {
						y: data.area_size.y - 1,
						x: 1
					};
					let mtl = tl + data.vel;

					let bl = self.tile_pos(bl);
					let t = self.tile_pos(tl).y;
					let ml = self.tile_pos(mtl).x;

					for x in ml..bl.x {
						for y in t..=bl.y {
							if let Some(tiledata) = tm.get_cell_tile_data(Vector2i { x, y }) {
								if tiledata.get_custom_data("Solid").booleanize() {
									data.next_vel = self.un_tile_pos(x + 1) - (data.pos.x + data.area_offset.x);
									data.collided |= Directions::LEFT;
									break;
								}
							}
						}
					}
				}

				for actor2 in &self.actors {
					let data2 = actor2.get();
					let rmov = data.vel.x - data2.vel.x;
					if rmov > 0 {
						let edge = data2.left_edge();
						if edge.properties.any() &&
							edge.pos.y < data.pos.y + data.area_offset.y + data.area_size.y &&
							edge.pos.y + edge.length > data.pos.y + data.area_offset.y &&
							edge.pos.x >= data.pos.x + data.area_offset.x + data.area_size.x &&
							edge.pos.x < data.pos.x + data.area_offset.x + data.area_size.x + rmov
						{
							if edge.properties.contains(SurfaceProperties::SOLID) {
								data.next_vel = edge.pos.x + data2.vel.x - (data.pos.x + data.area_offset.x + data.area_size.x);
								data.collided |= Directions::RIGHT;
							}
							if edge.properties.contains(SurfaceProperties::NOTIFY) {
								Gd::<Node>::from_instance_id(data2.notify_target.unwrap()).call("collide_notify", &[
									Gd::<Actor>::from_instance_id(data.actor.unwrap()).to_variant(),
									Directions::LEFT.bits().to_variant(),
								]);
							}
						}
					} else if rmov < 0 {
						let edge = actor2.get().right_edge();
						if edge.properties.any() &&
							edge.pos.y < data.pos.y + data.area_offset.y + data.area_size.y &&
							edge.pos.y + edge.length > data.pos.y + data.area_offset.y &&
							edge.pos.x <= data.pos.x + data.area_offset.x &&
							edge.pos.x > data.pos.x + data.area_offset.x + rmov
						{
							if edge.properties.contains(SurfaceProperties::SOLID) {
								data.next_vel = edge.pos.x + data2.vel.x - (data.pos.x + data.area_offset.x);
								data.collided |= Directions::LEFT;
							}
							if edge.properties.contains(SurfaceProperties::NOTIFY) {
								Gd::<Node>::from_instance_id(data2.notify_target.unwrap()).call("collide_notify", &[
									Gd::<Actor>::from_instance_id(data.actor.unwrap()).to_variant(),
									Directions::RIGHT.bits().to_variant(),
								]);
							}
						}
					}
				}

				actor.set(data);
			}

			for actor in &self.actors_that_move {
				let mut data = actor.get();
				if data.next_vel != data.vel.x {
					data.vel.x = data.next_vel;
					actor.set(data);
					dirty = true;
				}
			}
		}

		for actor in &self.actors_that_move {
			let mut data = actor.get();
			data.pos.x += data.vel.x;
			data.next_vel = data.vel.y;
			actor.set(data);
		}

		dirty = true;

		while dirty {
			dirty = false;

			for actor in &self.actors_that_move {
				let mut data = actor.get();

				if data.vel.y > 0 {
					let br = data.pos + data.area_offset + data.area_size;
					let bl = br + actor::Vec {
						x: -data.area_size.x,
						y: -1
					};
					let mbr = br + data.vel;

					let bl = self.tile_pos(bl);
					let r = self.tile_pos(br + actor::Vec { x: -1, y: 0 }).x;
					let mb = self.tile_pos(mbr).y;

					'o: for y in (bl.y + 1)..=mb {
						for x in bl.x..=r {
							if let Some(tiledata) = tm.get_cell_tile_data(Vector2i { x, y }) {
								if tiledata.get_custom_data("Solid").booleanize() {
									data.next_vel = self.un_tile_pos(y) - (data.pos.y + data.area_offset.y + data.area_size.y);
									data.collided |= Directions::DOWN;
									break 'o;
								}
							}
						}
					}
				} else if data.vel.y < 0 {
					let tl = data.pos + data.area_offset;
					let tr = tl + actor::Vec {
						x: data.area_size.x - 1,
						y: 1
					};
					let mtl = tl + data.vel;

					let tr = self.tile_pos(tr);
					let l = self.tile_pos(tl).x;
					let mt = self.tile_pos(mtl).y;

					for y in mt..tr.y {
						for x in l..=tr.x {
							if let Some(tiledata) = tm.get_cell_tile_data(Vector2i { x, y }) {
								if tiledata.get_custom_data("Solid").booleanize() {
									data.next_vel = self.un_tile_pos(y + 1) - (data.pos.y + data.area_offset.y);
									data.collided |= Directions::UP;
									break;
								}
							}
						}
					}
				}

				for actor2 in &self.actors {
					let data2 = actor2.get();
					let rmov = data.vel.y - data2.vel.y;
					if rmov > 0 {
						let edge = data2.top_edge();
						if edge.properties.any() &&
							edge.pos.x < data.pos.x + data.area_offset.x + data.area_size.x &&
							edge.pos.x + edge.length > data.pos.x + data.area_offset.x &&
							edge.pos.y >= data.pos.y + data.area_offset.y + data.area_size.y &&
							edge.pos.y < data.pos.y + data.area_offset.y + data.area_size.y + rmov
						{
							if edge.properties.contains(SurfaceProperties::SOLID) {
								data.next_vel = edge.pos.y + data2.vel.y - (data.pos.y + data.area_offset.y + data.area_size.y);
								data.collided |= Directions::DOWN;
							}
							if edge.properties.contains(SurfaceProperties::NOTIFY) {
								Gd::<Node>::from_instance_id(data2.notify_target.unwrap()).call("collide_notify", &[
									Gd::<Actor>::from_instance_id(data.actor.unwrap()).to_variant(),
									Directions::UP.bits().to_variant(),
								]);
							}
						}
					} else if rmov < 0 {
						let edge = actor2.get().bottom_edge();
						if edge.properties.any() &&
							edge.pos.x < data.pos.x + data.area_offset.x + data.area_size.x &&
							edge.pos.x + edge.length > data.pos.x + data.area_offset.x &&
							edge.pos.y <= data.pos.y + data.area_offset.y &&
							edge.pos.y > data.pos.y + data.area_offset.y + rmov
						{
							if edge.properties.contains(SurfaceProperties::SOLID) && (data.top.contains(SurfaceProperties::SOLID) || !data.collided.contains(Directions::UP | Directions::DOWN)) {
								data.next_vel = edge.pos.y + data2.vel.y - (data.pos.y + data.area_offset.y);
								data.collided |= Directions::UP;
							}
							if edge.properties.contains(SurfaceProperties::NOTIFY) {
								Gd::<Node>::from_instance_id(data2.notify_target.unwrap()).call("collide_notify", &[
									Gd::<Actor>::from_instance_id(data.actor.unwrap()).to_variant(),
									Directions::DOWN.bits().to_variant(),
								]);
							}
						}
					}
				}

				actor.set(data);
			}

			for actor in &self.actors_that_move {
				let mut data = actor.get();
				if data.next_vel != data.vel.y {
					data.vel.y = data.next_vel;
					actor.set(data);
					dirty = true;
				}
			}
		}

		for actor in &self.actors_that_move {
			let mut data = actor.get();
			data.pos.y += data.vel.y;
			actor.set(data);
		}
	}
}

const TILEMAP_SCALE_LOG2 : u32 = 16;

impl PlatformerGame {
	fn register_actors(&mut self, from: Gd<Node>) {
		match from.clone().try_cast::<Actor>() {
			Ok(actor) => {
				let actor = Rc::clone(&actor.bind().data);
				if actor.get().moves {
					self.actors_that_move.push(actor.clone());
				}
				self.actors.push(actor);
			}
			Err(_) => for child in from.get_children().iter_shared() {
				self.register_actors(child);
			}
		}
	}

	fn tile_pos(&self, v: actor::Vec) -> actor::Vec {
		actor::Vec {
			x: v.x >> TILEMAP_SCALE_LOG2,
			y: v.y >> TILEMAP_SCALE_LOG2,
		}
	}

	fn un_tile_pos(&self, v: i32) -> i32 {
		v << TILEMAP_SCALE_LOG2
	}
}