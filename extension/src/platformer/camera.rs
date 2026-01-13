use std::{cell::Cell, rc::Rc};

use godot::{classes::{DisplayServer, Camera2D, ICamera2D}, prelude::*};

use super::actor::{Actor, ActorData, Vec};

#[derive(GodotClass)]
#[class(base=Camera2D)]
pub struct ScreenCamera {
	base: Base<Camera2D>,
	actor: Rc<Cell<ActorData>>,

	#[export]
	follow: Option<Gd<Actor>>,
	#[export]
	screen_size: Vector2i,
}

#[godot_api]
impl ICamera2D for ScreenCamera {
	fn init(base: Base<Camera2D>) -> Self {
		Self {
			base,
			actor: Default::default(),
			follow: None,
			screen_size: Vector2i { x: 21, y: 12 },
		}
	}

	fn ready(&mut self) {
		self.actor = self.follow.as_ref().unwrap().bind().data.clone();
	}

	fn process(&mut self, _: f64) {
		let screen_size : Vec = self.screen_size.cast_float().into();
		let follow = self.actor.get();
		let follow_point = follow.pos + follow.area_offset + follow.area_size.half();
		let screen = Vec {
			x: follow_point.x.div_euclid(screen_size.x),
			y: follow_point.y.div_euclid(screen_size.y),
		};
		let center = Vec {
			x: screen.x * screen_size.x,
			y: screen.y * screen_size.y
		} + screen_size.half();
		let center: Vector2 = center.into();
		let position = self.base().get_position();
		let speed = if center.distance_squared_to(position) < 0.0002 || self.base().get_zoom().x == 1000.0 { 1.0 } else { 0.1 };
		self.base_mut().set_position(position.lerp(center, speed));

		let target_resolution = DisplayServer::singleton().window_get_size().y as f32;
		let zoom = target_resolution / self.screen_size.y as f32;
		self.base_mut().set_zoom(Vector2 { x: zoom, y: zoom });
	}
}