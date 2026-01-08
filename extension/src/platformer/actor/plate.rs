use std::{cell::Cell, rc::Rc};

use godot::{classes::AnimatedSprite2D, prelude::*};

use super::{Actor, ActorData};

#[derive(GodotClass)]
#[class(base=Node)]
pub struct Plate {
	base: Base<Node>,
	actor: Rc<Cell<ActorData>>,
	pushing: Vec<Rc<Cell<ActorData>>>,
	
	#[export]
	sprite: Option<Gd<AnimatedSprite2D>>,
}

#[godot_api]
impl INode for Plate {
	fn init(base: Base<Node>) -> Self {
		Self {
			base,
			actor: Default::default(),
			pushing: vec![],
			sprite: None,
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
		let plate_tl = data.pos + data.area_offset;
		let plate_r = plate_tl.x + data.area_size.x;

		self.pushing.retain(|weight| {
			let data = weight.get();
			let tl = data.pos + data.area_offset;
			let br = tl + data.area_size;
			br.x > plate_tl.x && tl.x < plate_r && br.y > plate_tl.y
		});

		if data.signal && self.pushing.is_empty() {
			self.sprite.as_mut().unwrap().set_animation("NotPressed");
			data.signal = false;
			self.actor.set(data);
		} else if !data.signal && !self.pushing.is_empty() {
			self.sprite.as_mut().unwrap().set_animation("Pressed");
			data.signal = true;
			self.actor.set(data);
		}
	}
}

#[godot_api]
impl Plate {
	#[func]
	fn collide_notify(&mut self, actor: Gd<Actor>, _: u8) {
		let actor = Rc::clone(&actor.bind().data);
		if !self.pushing.iter().any(|a| Rc::ptr_eq(a, &actor)) {
			self.pushing.push(actor);
		}
	}
}