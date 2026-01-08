use std::{cell::Cell, rc::Rc};

use godot::prelude::*;

use super::{player::Player, Actor, ActorData, Directions};

#[derive(GodotClass)]
#[class(base=Node)]
pub struct Crate {
	base: Base<Node>,
	actor: Rc<Cell<ActorData>>,
	push: Directions,

	#[export]
	push_speed: i32,
	#[export]
	carryable: bool,
}

#[godot_api]
impl INode for Crate {
	fn init(base: Base<Node>) -> Self {
		Self {
			base,
			actor: Default::default(),
			push: Directions::empty(),
			push_speed: 0,
			carryable: false,
		}
	}

	fn ready(&mut self) {
		self.actor = Rc::clone(&self.base().get_parent().unwrap().try_cast::<Actor>().unwrap().bind().data);
		let mut data = self.actor.get();
		data.notify_target = Some(self.base().instance_id());
		self.actor.set(data);
	}

	fn physics_process(&mut self, _: f64) {
		let mut data = self.actor.get();
		data.vel.x = match self.push {
			Directions::LEFT => self.push_speed,
			Directions::RIGHT => -self.push_speed,
			_ => 0
		};
		self.actor.set(data);
		self.push = Directions::empty();
	}
}

#[godot_api]
impl Crate {
	#[func]
	fn collide_notify(&mut self, actor: Gd<Actor>, direction: u8) {
		if actor.get_child(0).and_then(|c| c.try_cast::<Player>().ok()).is_some() && actor.bind().data.get().collided_old.contains(Directions::DOWN) {
			self.push |= Directions::from_bits_truncate(direction);
		}
	}
}