use std::{cell::Cell, rc::Rc};

use godot::prelude::*;

use super::{Actor, ActorData, Directions, GRAVITY};

#[derive(GodotClass)]
#[class(base=Node)]
pub struct Player {
	base: Base<Node>,
	actor: Rc<Cell<ActorData>>,

	#[export]
	speed: i32,
	#[export]
	jump_power: i32,
	#[export]
	jump_gravity: i32,
	#[export]
	jump_gravity_cutoff: i32,
}

#[godot_api]
impl INode for Player {
	fn init(base: Base<Node>) -> Self {
		Self {
			base,
			actor: Default::default(),
			speed: 16000,
			jump_power: 20000,
			jump_gravity: 800,
			jump_gravity_cutoff: 0,
		}
	}

	fn ready(&mut self) {
		self.actor = Rc::clone(&self.base().get_parent().unwrap().try_cast::<Actor>().unwrap().bind().data);
	}

	fn physics_process(&mut self, _: f64) {
		let input = Input::singleton();
		let mut data = self.actor.get();

		data.vel.x = 0;
		if input.is_action_pressed("ui_left") { data.vel.x -= self.speed; }
		if input.is_action_pressed("ui_right") { data.vel.x += self.speed; }

		if data.gravity == self.jump_gravity && (
			data.vel.y > self.jump_gravity_cutoff ||
			data.collided.contains(Directions::DOWN) ||
			!input.is_action_pressed("ui_accept")
		) {
			data.gravity = GRAVITY;
		}

		if input.is_action_just_pressed("ui_accept") && data.collided.contains(Directions::DOWN) {
			data.vel.y -= self.jump_power;
			data.gravity = self.jump_gravity;
		}

		self.actor.set(data);
	}
}