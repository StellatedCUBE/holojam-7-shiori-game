use std::{cell::Cell, rc::Rc};

use godot::{classes::{AnimatedSprite2D, Sprite2D}, prelude::*};

use super::{Actor, ActorData, SurfaceProperties};

const CHARGE_MAX: u32 = 32;

#[derive(GodotClass)]
#[class(base=Node)]
pub struct LazerDetector {
	base: Base<Node>,
	actor: Rc<Cell<ActorData>>,
	charge: u32,

	#[export]
	fx: Option<Gd<Sprite2D>>,
}

#[godot_api]
impl INode for LazerDetector {
	fn init(base: Base<Node>) -> Self {
		Self {
			base,
			actor: Default::default(),
			charge: 0,
			fx: None,
		}
	}

	fn ready(&mut self) {
		self.actor = Rc::clone(&self.base().get_parent().unwrap().try_cast::<Actor>().unwrap().bind().data);
		let mut data = self.actor.get();
		data.top |= SurfaceProperties::OPAQUE;
		data.left |= SurfaceProperties::OPAQUE;
		data.bottom |= SurfaceProperties::OPAQUE;
		data.right |= SurfaceProperties::OPAQUE;
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
		self.fx.as_mut().unwrap().set_modulate(Color { r: 1.0, g: 1.0, b: 1.0, a: self.charge as f32 / CHARGE_MAX as f32 });
	}
}