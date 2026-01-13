use std::{cell::{Cell, RefCell}, f32::consts::PI, rc::Rc};

use godot::{prelude::*};

use super::actor::{self, Actor, ActorData};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Direction {
	Up, Down, Left, Right
}

impl Direction {
	pub const fn reflect_main(self) -> Self {
		match self {
			Self::Up => Self::Left,
			Self::Left => Self::Up,
			Self::Down => Self::Right,
			Self::Right => Self::Down,
		}
	}

	pub const fn reflect_inv(self) -> Self {
		match self {
			Self::Up => Self::Right,
			Self::Left => Self::Down,
			Self::Down => Self::Left,
			Self::Right => Self::Up,
		}
	}

	pub const fn rot(self) -> f32 {
		match self {
			Self::Right => 0.0,
			Self::Down => PI * 0.5,
			Self::Left => PI,
			Self::Up => PI * 1.5,
		}
	}
}

#[derive(Clone, Copy)]
pub struct SegmentData {
	pub start: actor::Vec,
	pub length: i32,
	pub direction: Direction,
	pub end: bool,
}

pub struct Beam {
	pub active: bool,
	pub start_direction: Direction,
	pub start_pos: actor::Vec,
	pub hit_actor: Option<Rc<Cell<ActorData>>>,
	pub scene: Gd<PackedScene>,
	pub segments: Vec<Gd<Node2D>>,
	pub top: i32,
	pub left: i32,
	pub bottom: i32,
	pub right: i32,
}

#[derive(GodotClass)]
#[class(base=Node2D)]
pub struct Lazer {
	base: Base<Node2D>,
	input_actors: Box<[Rc<Cell<ActorData>>]>,
	pub beam: Option<Rc<RefCell<Beam>>>,

	#[export]
	inputs: Array<Gd<Actor>>,
	#[export]
	beam_type: Option<Gd<PackedScene>>,
}

#[godot_api]
impl INode2D for Lazer {
	fn init(base: Base<Node2D>) -> Self {
		Self {
			base,
			input_actors: Box::from([]),
			beam: None,
			inputs: Default::default(),
			beam_type: None,
		}
	}

	fn ready(&mut self) {
		self.input_actors = self.inputs.iter_shared().map(|input| Rc::clone(&input.bind().data)).collect();
		self.beam = Some(Rc::from(RefCell::new(Beam {
			active: self.input_actors.is_empty(),
			start_direction: match (self.base().get_rotation_degrees() as i32).rem_euclid(360) {
				_ => Direction::Left,
			},
			start_pos: self.base().get_child(0).unwrap().try_cast::<Node2D>().unwrap().get_global_position().into(),
			hit_actor: None,
			scene: self.beam_type.take().unwrap(),
			segments: vec![],
			top: 0,
			left: 0,
			bottom: 0,
			right: 0,
		})));
	}

	fn physics_process(&mut self, _: f64) {
		let open = self.input_actors.iter().all(|i| i.get().signal);
		self.beam.as_ref().unwrap().borrow_mut().active = open;
	}
}