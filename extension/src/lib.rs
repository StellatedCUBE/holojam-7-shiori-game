use godot::prelude::*;

mod platformer;

struct Extension;

#[gdextension]
unsafe impl ExtensionLibrary for Extension {}