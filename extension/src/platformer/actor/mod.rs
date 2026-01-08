use std::{cell::Cell, ops::Add, rc::Rc};

use godot::{classes::{CollisionShape2D, RectangleShape2D}, prelude::*};
use bitflags::bitflags;

mod player;
mod cube;
mod plate;

const SCENE_SCALE: f32 = 65536.0;
const SCENE_SCALE_INV: f32 = 1.0 / SCENE_SCALE;

const GRAVITY: i32 = 3000;

bitflags! {
	#[derive(Default, Clone, Copy, PartialEq, Eq)]
	pub struct Directions: u8 {
		const UP = 1;
		const DOWN = 2;
		const LEFT = 4;
		const RIGHT = 8;
	}
}

bitflags! {
	#[derive(Default, Clone, Copy, PartialEq, Eq)]
	pub struct SurfaceProperties: u8 {
		const SOLID = 1;
		const NOTIFY = 2;
	}
}

impl SurfaceProperties {
	pub fn any(self) -> bool {
		self != Self::empty()
	}
}

#[derive(Default, Clone, Copy)]
pub struct Vec {
	pub x: i32,
	pub y: i32,
}

impl From<Vec> for Vector2 {
	fn from(value: Vec) -> Self {
		Self {
			x: value.x as f32 * SCENE_SCALE_INV,
			y: value.y as f32 * SCENE_SCALE_INV,
		}
	}
}

impl From<Vector2> for Vec {
	fn from(value: Vector2) -> Self {
		Self {
			x: (value.x * SCENE_SCALE) as i32,
			y: (value.y * SCENE_SCALE) as i32,
		}
	}
}

impl Add for Vec {
	type Output = Vec;

	fn add(self, rhs: Self) -> Self::Output {
		Self {
			x: self.x + rhs.x,
			y: self.y + rhs.y,
		}
	}
}

impl Vec {
	pub const fn half(self) -> Self {
		Self {
			x: self.x >> 1,
			y: self.y >> 1,
		}
	}
}

pub struct Edge {
	pub pos: Vec,
	pub length: i32,
	pub properties: SurfaceProperties
}

#[derive(Default, Clone, Copy)]
pub struct ActorData {
	pub moves: bool,
	pub pos: Vec,
	pub vel: Vec,
	pub next_vel: i32,
	pub area_offset: Vec,
	pub area_size: Vec,
	pub collided: Directions,
	pub collided_old: Directions,
	pub actor: Option<InstanceId>,
	pub notify_target: Option<InstanceId>,
	pub signal: bool,
	gravity: i32,
	terminal_velocity: i32,
	pub top: SurfaceProperties,
	left: SurfaceProperties,
	bottom: SurfaceProperties,
	right: SurfaceProperties,
}

#[derive(GodotClass)]
#[class(base=Node2D)]
pub struct Actor {
	pub data: Rc<Cell<ActorData>>,

	#[export]
	is_static: bool,
	#[export]
	terminal_velocity: u32,
	#[export]
	top_solid: bool,
	#[export]
	top_notify: bool,
	#[export]
	bottom_solid: bool,
	#[export]
	bottom_notify: bool,
	#[export]
	left_solid: bool,
	#[export]
	left_notify: bool,
	#[export]
	right_solid: bool,
	#[export]
	right_notify: bool,

	base: Base<Node2D>,
}

#[godot_api]
impl INode2D for Actor {
	fn init(base: Base<Node2D>) -> Self {
		Self {
			data: Default::default(),
			is_static: true,
			terminal_velocity: 0,
			top_solid: false,
			top_notify: false,
			left_solid: false,
			left_notify: false,
			bottom_solid: false,
			bottom_notify: false,
			right_solid: false,
			right_notify: false,
			base,
		}
	}

	fn ready(&mut self) {
		let mut data = self.data.get();

		data.moves = !self.is_static;
		data.pos = self.base().get_global_position().into();
		data.actor = Some(self.base().instance_id());
		if self.top_solid { data.top |= SurfaceProperties::SOLID; }
		if self.top_notify { data.top |= SurfaceProperties::NOTIFY; }
		if self.left_solid { data.left |= SurfaceProperties::SOLID; }
		if self.left_notify { data.left |= SurfaceProperties::NOTIFY; }
		if self.bottom_solid { data.bottom |= SurfaceProperties::SOLID; }
		if self.bottom_notify { data.bottom |= SurfaceProperties::NOTIFY; }
		if self.right_solid { data.right |= SurfaceProperties::SOLID; }
		if self.right_notify { data.right |= SurfaceProperties::NOTIFY; }

		if self.terminal_velocity > 0 {
			data.gravity = GRAVITY;
			data.terminal_velocity = self.terminal_velocity as i32;
		}

		for child in self.base().get_children().iter_shared() {
			if let Ok(mut shape) = child.try_cast::<CollisionShape2D>() {
				if let Some(rect) = shape.get_shape().and_then(|shape| shape.try_cast::<RectangleShape2D>().ok()) {
					let size = rect.get_size();
					let pos = shape.get_position() - size * 0.5;
					data.area_size = size.into();
					data.area_offset = pos.into();
					shape.queue_free();
					break;
				}
			}
		}

		self.data.set(data);
	}

	fn process(&mut self, _: f64) {
		let pos = self.data.get().pos.into();
		self.base_mut().set_global_position(pos);
	}
}

impl ActorData {
	pub fn top_edge(&self) -> Edge {
		Edge {
			pos: self.pos + self.area_offset,
			length: self.area_size.x,
			properties: self.top,
		}
	}

	pub fn left_edge(&self) -> Edge {
		Edge {
			pos: self.pos + self.area_offset,
			length: self.area_size.y,
			properties: self.left,
		}
	}

	pub fn bottom_edge(&self) -> Edge {
		let mut pos = self.pos + self.area_offset;
		pos.y += self.area_size.y;
		Edge {
			pos,
			length: self.area_size.x,
			properties: self.bottom,
		}
	}

	pub fn right_edge(&self) -> Edge {
		let mut pos = self.pos + self.area_offset;
		pos.x += self.area_size.x;
		Edge {
			pos,
			length: self.area_size.y,
			properties: self.right,
		}
	}

	pub fn fall(&mut self) {
		if self.gravity != 0 {
			if self.vel.y <= self.terminal_velocity {
				self.vel.y = self.terminal_velocity.min(self.vel.y + self.gravity);
			} else {
				self.vel.y = self.terminal_velocity + (self.vel.y - self.terminal_velocity) * 3 / 4;
			}
		}
	}
}