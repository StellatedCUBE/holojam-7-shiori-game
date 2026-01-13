extends CanvasLayer


# Called when the node enters the scene tree for the first time.
func _ready() -> void:
	visible = false
	get_tree().paused = false


# Called every frame. 'delta' is the elapsed time since the previous frame.
func _input(event: InputEvent) -> void:
	if Input.is_action_just_pressed("ui_cancel"):
		if get_tree().paused:
			visible = false
			get_tree().paused = false
		else:
			visible = true
			get_tree().paused = true



func _on_resume_button_pressed() -> void:
	visible = false
	get_tree().paused = false # Replace with function body.


func _on_quit_button_pressed() -> void:
	get_tree().quit() # Replace with function body.
