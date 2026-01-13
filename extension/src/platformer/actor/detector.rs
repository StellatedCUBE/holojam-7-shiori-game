use std::{cell::Cell, rc::Rc};

use godot::{classes::AnimatedSprite2D, prelude::*};

use super::{Actor, ActorData};

const CHARGE_MAX: u32 = 32;

#[derive(GodotClass)]
#[class(base=Node)]
pub struct LazerDetector {
	base: Base<Node>,
	actor: Rc<Cell<ActorData>>,
	charge: u32,
}

#[godot_api]
impl INode for LazerDetector {
	fn init(base: Base<Node>) -> Self {
		Self {
			base,
			actor: Default::default(),
			charge: 0,
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
		if data.beams > 0 && self.charge < CHARGE_MAX {
			self.charge += 1;
		} else if data.beams == 0 && self.charge > 0 {
			self.charge -= 1;
		}
		if self.charge == 0 {
			data.signal = false;
		} else if self.charge == CHARGE_MAX {
			data.signal = true;
		}
		self.actor.set(data);
	}
}