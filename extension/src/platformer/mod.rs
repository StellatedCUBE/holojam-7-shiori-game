use std::{cell::{Cell, RefCell}, i32, rc::Rc};

use actor::{Actor, ActorData, Directions, Reflection, SurfaceProperties, SCENE_SCALE_INV};
use godot::{classes::TileMapLayer, prelude::*};
use lazer::{Beam, Direction, Lazer, SegmentData};

mod actor;
mod camera;
mod lazer;

#[derive(GodotClass)]
#[class(base=Node2D)]
struct PlatformerGame {
	base: Base<Node2D>,
	actors: Vec<Rc<Cell<ActorData>>>,
	actors_that_move: Vec<Rc<Cell<ActorData>>>,
	beams: Vec<Rc<RefCell<Beam>>>,

	#[export]
	tilemap: Option<Gd<TileMapLayer>>,
	#[export]
	beam_container: Option<Gd<Node>>,
}

#[godot_api]
impl INode2D for PlatformerGame {
	fn init(base: Base<Node2D>) -> Self {
		Self {
			base,
			actors: vec![],
			actors_that_move: vec![],
			beams: vec![],
			tilemap: None,
			beam_container: None,
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

		for beam in &self.beams {
			let mut beam = beam.borrow_mut();
			if beam.active == beam.segments.is_empty() || (beam.active && self.actors_that_move.iter().any(|actor| {
				let data = actor.get();
				(data.vel.x != 0 || data.vel.y != 0) /*&& {
					let pmin = data.pos + actor::Vec {
						x: data.vel.x.min(0) - data.vel.x,
						y: data.vel.y.min(0) - data.vel.y
					};
					let pmax = data.pos + actor::Vec {
						x: data.vel.x.max(0) - data.vel.x,
						y: data.vel.y.max(0) - data.vel.y
					};
					let tl = pmin + data.area_offset;
					let br = pmax + data.area_offset + data.area_size;
					br.x >= beam.left && br.y >= beam.top && tl.x <= beam.right && tl.y <= beam.bottom
				}*/
			})) {
				if let Some(actor) = beam.hit_actor.take() {
					let mut data = actor.get();
					data.beams -= 1;
					actor.set(data);
				}

				if beam.active {
					let mut hit: Option<Rc<Cell<ActorData>>> = None;
					let mut segment = SegmentData {
						start: beam.start_pos,
						direction: beam.start_direction,
						length: i32::MAX,
						end: false,
					};
					let mut segments = vec![];

					while !segment.end {
						match segment.direction {
							Direction::Left => for actor in &self.actors {
								let edge = actor.get().right_edge();
								if edge.properties.opaque() && edge.pos.x < segment.start.x && edge.pos.y < segment.start.y && edge.pos.y + edge.length > segment.start.y && segment.start.x - edge.pos.x < segment.length {
									segment.length = segment.start.x - edge.pos.x;
									hit = Some(Rc::clone(actor));
								}
							}
							Direction::Right => for actor in &self.actors {
								let edge = actor.get().left_edge();
								if edge.properties.opaque() && edge.pos.x > segment.start.x && edge.pos.y < segment.start.y && edge.pos.y + edge.length > segment.start.y && edge.pos.x - segment.start.x < segment.length {
									segment.length = edge.pos.x - segment.start.x;
									hit = Some(Rc::clone(actor));
								}
							}
							Direction::Up => for actor in &self.actors {
								let edge = actor.get().bottom_edge();
								if edge.properties.opaque() && edge.pos.y < segment.start.y && edge.pos.x < segment.start.x && edge.pos.x + edge.length > segment.start.x && segment.start.y - edge.pos.y < segment.length {
									segment.length = segment.start.y - edge.pos.y;
									hit = Some(Rc::clone(actor));
								}
							}
							Direction::Down => for actor in &self.actors {
								let edge = actor.get().top_edge();
								if edge.properties.opaque() && edge.pos.y > segment.start.y && edge.pos.x < segment.start.x && edge.pos.x + edge.length > segment.start.x && edge.pos.y - segment.start.y < segment.length {
									segment.length = edge.pos.y - segment.start.y;
									hit = Some(Rc::clone(actor));
								}
							}
						}

						let tile = self.tile_pos(segment.start);
						let mut tile = Vector2i { x: tile.x, y: tile.y };
						let tile_dir = segment.direction.tile_offset();
						let mut i = 0;
						while i <= segment.length >> 16 {
							if self.tilemap.as_ref().unwrap().get_cell_tile_data(tile).is_some_and(|t| t.get_custom_data("Solid").booleanize()) {
								segment.length = i << 16;
								hit = None;
								break;
							}
							i += 1;
							tile += tile_dir;
						}


						segment.end = hit.as_ref().is_none_or(|a| a.get().reflection == Reflection::None);

						if !segment.end {
							let hit = hit.as_ref().unwrap().get();
							let inverse = hit.reflection == Reflection::Inverse;
							let (offset, size) = match segment.direction {
								Direction::Up | Direction::Down => (segment.start.x - hit.pos.x - hit.area_offset.x, hit.area_size.x),
								Direction::Left | Direction::Right => (segment.start.y - hit.pos.y - hit.area_offset.y, hit.area_size.y),
							};
							segment.length += match (segment.direction, inverse) {
								(Direction::Up, true) |
								(Direction::Down, false) |
								(Direction::Left, true) |
								(Direction::Right, false) => offset,
								_ => size - offset,
							};
						}

						segments.push(segment);
						
						if !segment.end {
							let hit = hit.as_ref().unwrap().get();
							let inverse = hit.reflection == Reflection::Inverse;
							let offset = tile_dir * segment.length;
							segment = SegmentData {
								start: segment.start + actor::Vec {
									x: offset.x,
									y: offset.y
								},
								direction: if inverse {
									segment.direction.reflect_inv()
								} else {
									segment.direction.reflect_main()
								},
								length: i32::MAX,
								end: false,
							};
						}
					}

					if let Some(hit) = hit {
						let mut data = hit.get();
						data.beams += 1;
						hit.set(data);
						beam.hit_actor = Some(hit);
					}

					if segments.len() < beam.segments.len() {
						for mut segment in beam.segments.drain(segments.len()..) {
							segment.queue_free();
						}	
					} else {
						for _ in beam.segments.len()..segments.len() {
							let segment = beam.scene.instantiate().unwrap().try_cast().unwrap();
							self.beam_container.as_mut().unwrap().add_child(&segment);
							beam.segments.push(segment);
						}
					}

					let mut top = beam.start_pos.y;
					let mut bottom = beam.start_pos.y;
					let mut left = beam.start_pos.x;
					let mut right = beam.start_pos.x;

					for (segment, node) in segments.into_iter().zip(beam.segments.iter_mut()) {
						node.set_position(segment.start.into());
						node.set_scale(Vector2 { x: segment.length as f32 * SCENE_SCALE_INV, y: 1.0 });
						node.set_rotation(segment.direction.rot());

						match segment.direction {
							Direction::Down => bottom = bottom.max(segment.start.y + segment.length),
							Direction::Left => left = left.min(segment.start.x - segment.length),
							Direction::Right => right = right.max(segment.start.x + segment.length),
							Direction::Up => top = top.min(segment.start.y - segment.length),
						}
					}

					beam.top = top - 1;
					beam.bottom = bottom + 1;
					beam.left = left - 1;
					beam.right = right + 1;
				} else {
					for mut segment in beam.segments.drain(..) {
						segment.queue_free();
					}
				}
			}
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
			Err(_) => match from.clone().try_cast::<Lazer>() {
				Ok(lazer) => {
					self.beams.push(Rc::clone(lazer.bind().beam.as_ref().unwrap()));
				}
				Err(_) => for child in from.get_children().iter_shared() {
					self.register_actors(child);
				}
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