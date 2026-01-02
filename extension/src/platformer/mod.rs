use std::{cell::Cell, rc::Rc};

use actor::{Actor, ActorData, Directions};
use godot::prelude::*;

mod actor;

#[derive(GodotClass)]
#[class(base=Node2D)]
struct PlatformerGame {
	base: Base<Node2D>,
	actors: Vec<Rc<Cell<ActorData>>>,
	actors_that_move: Vec<Rc<Cell<ActorData>>>,
}

#[godot_api]
impl INode2D for PlatformerGame {
	fn init(base: Base<Node2D>) -> Self {
		Self {
			base,
			actors: vec![],
			actors_that_move: vec![],
		}
	}

	fn ready(&mut self) {
		self.register_actors(self.to_gd().upcast());
	}

	fn physics_process(&mut self, _: f64) {
		for actor in &self.actors_that_move {
			let mut data = actor.get();
			data.collided = Directions::empty();

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
						data.vel.x = edge.pos.x + data2.vel.x - (data.pos.x + data.area_offset.x + data.area_size.x);
						data.collided |= Directions::RIGHT;
					}
				} else if rmov < 0 {
					let edge = actor2.get().right_edge();
					if edge.properties.any() &&
						edge.pos.y < data.pos.y + data.area_offset.y + data.area_size.y &&
						edge.pos.y + edge.length > data.pos.y + data.area_offset.y &&
						edge.pos.x <= data.pos.x + data.area_offset.x &&
						edge.pos.x > data.pos.x + data.area_offset.x + rmov
					{
						data.vel.x = edge.pos.x + data2.vel.x - (data.pos.x + data.area_offset.x);
						data.collided |= Directions::LEFT;
					}
				}
			}

			data.pos.x += data.vel.x;
			data.fall();
			actor.set(data);
		}

		for actor in &self.actors_that_move {
			let mut data = actor.get();
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
						data.vel.y = edge.pos.y + data2.vel.y - (data.pos.y + data.area_offset.y + data.area_size.y);
						data.collided |= Directions::DOWN;
					}
				} else if rmov < 0 {
					let edge = actor2.get().bottom_edge();
					if edge.properties.any() &&
						edge.pos.x < data.pos.x + data.area_offset.x + data.area_size.x &&
						edge.pos.x + edge.length > data.pos.x + data.area_offset.x &&
						edge.pos.y <= data.pos.y + data.area_offset.y &&
						edge.pos.y > data.pos.y + data.area_offset.y + rmov
					{
						data.vel.y = edge.pos.y + data2.vel.y - (data.pos.y + data.area_offset.y);
						data.collided |= Directions::UP;
					}
				}
			}
			data.pos.y += data.vel.y;
			actor.set(data);
		}
	}
}

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
}