use std::{cell::Cell, rc::Rc};

use godot::{classes::AnimatedSprite2D, prelude::*};

use super::{Actor, ActorData};

#[derive(GodotClass)]
#[class(base=Node)]
pub struct LazerDetector {
	base: Base<Node>,
	actor: Rc<Cell<ActorData>>,
}

#[godot_api]
impl INode for LazerDetector {
	fn init(base: Base<Node>) -> Self {
		Self {
			base,
			actor: Default::default(),
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
		data.signal = data.beams > 0;
		self.actor.set(data);
	}
}