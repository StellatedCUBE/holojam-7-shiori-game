use std::{cell::Cell, rc::Rc};

use godot::prelude::*;

use super::{player::{self, Player}, Actor, ActorData};

#[derive(GodotClass)]
#[class(base=Node)]
pub struct UnlockBookTrigger {
	base: Base<Node>,
	actor: Rc<Cell<ActorData>>,
}

#[godot_api]
impl INode for UnlockBookTrigger {
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
}

#[godot_api]
impl UnlockBookTrigger {
	#[func]
	fn collide_notify(&mut self, actor: Gd<Actor>, _: u8) {
		if actor.get_child(0).and_then(|c| c.try_cast::<Player>().ok()).is_some() {
			player::HAS_BOOK.store(true, std::sync::atomic::Ordering::Relaxed);
		}
	}
}