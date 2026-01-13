use std::{cell::Cell, rc::Rc};

use godot::prelude::*;
use godot::classes::Input;

use super::{player::Player, Actor, ActorData, Directions, SurfaceProperties};

const GRAB_DISTANCE: i32 = 4096;

#[derive(Clone, Copy, PartialEq, Eq)]
enum HoldSide {
	Left, Right
}

enum HoldStatus {
	None,
	CanHold(Rc<Cell<ActorData>>, HoldSide),
	Holding(Rc<Cell<ActorData>>, HoldSide),
}

#[derive(GodotClass)]
#[class(base=Node)]
pub struct Crate {
	base: Base<Node>,
	actor: Rc<Cell<ActorData>>,
	push: Directions,
	hold: HoldStatus,

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
			hold: HoldStatus::None,
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
		
		match &mut self.hold {
			HoldStatus::Holding(carrier_cell, side) => {
				let carrier = carrier_cell.get();
				data.pos = carrier.pos + carrier.area_offset + super::Vec { x: match *side {
					HoldSide::Left => data.area_size.x / -2,
					HoldSide::Right => carrier.area_size.x / 2,
				} - data.area_offset.x, y: -data.area_offset.y };
				if Input::singleton().is_action_just_pressed("hold") {
					data.pos.x += (carrier.area_size.x / 2 - data.area_size.x) * match *side {
						HoldSide::Left => -1,
						HoldSide::Right => 1,
					};
					data.vel = super::Vec {
						x: match *side {
							HoldSide::Left => carrier.vel.x.min(0) - data.area_size.x,
							HoldSide::Right => data.area_size.x + carrier.vel.x.max(0),
						},
						y: carrier.vel.y,
					};
					self.hold = HoldStatus::CanHold(carrier_cell.clone(), *side);
					data.top = SurfaceProperties::SOLID;
					data.bottom = SurfaceProperties::SOLID;
					data.left = SurfaceProperties::SOLID | SurfaceProperties::NOTIFY;
					data.right = SurfaceProperties::SOLID | SurfaceProperties::NOTIFY;
				} else if carrier.vel.x != 0 {
					*side = if carrier.vel.x < 0 { HoldSide::Left } else { HoldSide::Right };
				}
				self.actor.set(data);
				return;
			}
			HoldStatus::CanHold(by, side) => {
				let bydata = by.get();
				let my_tl = data.pos + data.area_offset;
				let my_br = my_tl + data.area_size;
				let by_tl = bydata.pos + bydata.area_offset;
				let by_br = by_tl + bydata.area_size;
				if my_tl.x > by_br.x + GRAB_DISTANCE || my_br.x < by_tl.x - GRAB_DISTANCE || my_br.y > by_br.y + GRAB_DISTANCE || my_tl.y < by_br.y - (1 << 17) {
					self.hold = HoldStatus::None;
				} else if Input::singleton().is_action_pressed("hold") {
					self.hold = HoldStatus::Holding(by.clone(), *side);
					data.top = SurfaceProperties::empty();
					data.bottom = SurfaceProperties::empty();
					data.left = SurfaceProperties::empty();
					data.right = SurfaceProperties::empty();
				}
			}
			HoldStatus::None => {}
		}

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
			self.hold = HoldStatus::CanHold(Rc::clone(&actor.bind().data), if Directions::from_bits_truncate(direction) == Directions::LEFT { HoldSide::Right } else { HoldSide::Left });
		}
	}
}