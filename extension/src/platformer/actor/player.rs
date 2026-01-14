use std::{cell::Cell, rc::Rc, sync::atomic::AtomicBool};

use godot::prelude::*;
use godot::classes::Input;

use super::{Actor, ActorData, Directions, GRAVITY};

pub static HAS_BOOK: AtomicBool = AtomicBool::new(false); 

#[derive(GodotClass)]
#[class(base=Node)]
pub struct Player {
	base: Base<Node>,
	actor: Rc<Cell<ActorData>>,
	book: Rc<Cell<ActorData>>,

	#[export]
	speed: i32,
	#[export]
	jump_power: i32,
	#[export]
	jump_gravity: i32,
	#[export]
	jump_gravity_cutoff: i32,
	#[export]
	book_bounce: i32,
}

#[godot_api]
impl INode for Player {
	fn init(base: Base<Node>) -> Self {
		Self {
			base,
			actor: Default::default(),
			book: Default::default(),
			speed: 16000,
			jump_power: 50000,
			jump_gravity: 1500,
			jump_gravity_cutoff: 10000,
			book_bounce: 0,
		}
	}

	fn ready(&mut self) {
		self.actor = Rc::clone(&self.base().get_parent().unwrap().try_cast::<Actor>().unwrap().bind().data);
		self.book = Rc::clone(&self.base().get_parent().unwrap().get_parent().unwrap().find_child("Book").unwrap().try_cast::<Actor>().unwrap().bind().data);
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

		if input.is_action_just_pressed("ui_accept") {
			if data.collided.contains(Directions::DOWN) {
				data.vel.y -= self.jump_power;
				data.gravity = self.jump_gravity;
			} else if HAS_BOOK.load(std::sync::atomic::Ordering::Relaxed) {
				let mut book = self.book.get();
				book.pos = data.pos + data.area_offset + super::Vec {
					x: data.area_size.x / 2,
					y: data.area_size.y
				};
				self.book.set(book);
				data.vel.y = -self.book_bounce;
			}
		}

		//godot_print!("{}", Into::<Vector2>::into(data.pos));

		self.actor.set(data);
	}
}