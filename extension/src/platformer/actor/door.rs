use std::{cell::Cell, rc::Rc};

use godot::{classes::AnimatedSprite2D, prelude::*};

use super::{Actor, ActorData, SurfaceProperties};

const TICKS_PER_FRAME: u32 = 2;

#[derive(GodotClass)]
#[class(base=Node2D)]
pub struct Door {
	base: Base<Node2D>,
	actor: Rc<Cell<ActorData>>,
	sprite: Option<Gd<AnimatedSprite2D>>,
	ttnf: u32,
	input_actors: Box<[Rc<Cell<ActorData>>]>,

	#[export]
	inputs: Array<Gd<Actor>>,
}

#[godot_api]
impl INode2D for Door {
	fn init(base: Base<Node2D>) -> Self {
		Self {
			base,
			actor: Default::default(),
			sprite: None,
			ttnf: 0,
			input_actors: Box::from([]),
			inputs: Default::default(),
		}
	}

	fn ready(&mut self) {
		self.actor = Rc::clone(&self.base().get_child(0).unwrap().try_cast::<Actor>().unwrap().bind().data);
		self.sprite = self.base().find_child("Sprite").map(|c| c.try_cast().unwrap());
		self.input_actors = self.inputs.iter_shared().map(|input| Rc::clone(&input.bind().data)).collect();
	}

	fn physics_process(&mut self, _: f64) {
		let open = self.input_actors.iter().all(|i| i.get().signal);

		let mut data = self.actor.get();
		let property = match open {
			true => SurfaceProperties::empty(),
			false => SurfaceProperties::SOLID
		};
		let change = data.top != property;
		data.top = property;
		data.bottom = property;
		data.left = property;
		data.right = property;
		self.actor.set(data);

		if self.ttnf > 0 {
			self.ttnf -= 1;
			return;
		}

		if change {
			return;
		}

		let frame = self.sprite.as_ref().unwrap().get_frame();

		if open && frame < 8 {
			self.sprite.as_mut().unwrap().set_frame(frame + 1);
			self.ttnf = TICKS_PER_FRAME;
		} else if !open && frame > 0 {
			self.sprite.as_mut().unwrap().set_frame(frame - 1);
			self.ttnf = TICKS_PER_FRAME;
		}
	}
}