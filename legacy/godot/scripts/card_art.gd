extends Control
## Preserve objects in full; portrait subjects crop from the top so faces stay visible.
var texture: Texture2D
var category: String = "enemy"

func _ready() -> void:
	mouse_filter = Control.MOUSE_FILTER_IGNORE
	resized.connect(queue_redraw)

func _draw() -> void:
	if texture == null: return
	var image_size = texture.get_size()
	var target = Rect2(Vector2.ZERO,size)
	if category in ["equipment","consumable","chaos"]:
		var scale_factor = minf(size.x/image_size.x,size.y/image_size.y)
		var fitted = image_size*scale_factor
		draw_rect(target,Color("c4b68e"))
		draw_texture_rect(texture,Rect2((size-fitted)/2,fitted),false)
	else:
		var ratio = size.x/size.y
		var crop_size = image_size
		var origin = Vector2.ZERO
		if image_size.x/image_size.y < ratio:
			crop_size.y = image_size.x/ratio
			origin.y = minf(image_size.y-crop_size.y,image_size.y*0.02) if category in ["enemy","character"] else (image_size.y-crop_size.y)/2
		else:
			crop_size.x = image_size.y*ratio
			origin.x = (image_size.x-crop_size.x)/2
		draw_texture_rect_region(texture,target,Rect2(origin,crop_size))
