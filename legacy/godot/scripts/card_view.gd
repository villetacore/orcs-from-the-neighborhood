extends PanelContainer
## Card face: generated art is separate from deterministic, readable text.
const COLORS = {"character":Color("648897"),"enemy":Color("a95640"),"equipment":Color("af944b"),"consumable":Color("879951"),"event":Color("8c7294"),"chaos":Color("da8245")}
const TYPES = {"character":"ПЕРСОНАЖ","enemy":"ВРАГ","equipment":"СНАРЯЖЕНИЕ","consumable":"РАСХОДНИК","event":"СОБЫТИЕ","chaos":"ХАОС"}
static var textures: Dictionary = {}
var card: Dictionary = {}
var compact: bool = false
var selected: bool = false
var clicked: Callable
var draggable: bool = false
var hover_tween: Tween

static func art(path: String) -> Texture2D:
	if not textures.has(path):
		textures[path] = load("res://"+path) if ResourceLoader.exists("res://"+path) else null
	return textures[path]

func _ready() -> void:
	mouse_default_cursor_shape = Control.CURSOR_POINTING_HAND
	var color: Color = COLORS.get(card.get("type","enemy"),Color.GRAY)
	var box = StyleBoxFlat.new()
	box.bg_color = Color("ded0aa")
	box.border_color = Color("f5c780") if selected else color.darkened(0.25)
	box.set_border_width_all(3 if selected else 2)
	box.set_corner_radius_all(9)
	box.content_margin_left = 9
	box.content_margin_right = 9
	box.content_margin_top = 8
	box.content_margin_bottom = 8
	box.shadow_color = Color(0,0,0,0.4)
	box.shadow_size = 7
	box.shadow_offset = Vector2(2,4)
	add_theme_stylebox_override("panel",box)
	var col = VBoxContainer.new()
	col.add_theme_constant_override("separation",5)
	col.mouse_filter = Control.MOUSE_FILTER_IGNORE
	add_child(col)
	var tag = Label.new()
	tag.text = TYPES.get(card.type,"")+"   /   "+str(card.id).to_upper()
	tag.add_theme_color_override("font_color",color.darkened(0.35))
	tag.add_theme_font_size_override("font_size",10 if compact else 12)
	col.add_child(tag)
	var title = Label.new()
	title.text = card.name
	title.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	if compact:
		title.max_lines_visible = 2
		title.text_overrun_behavior = TextServer.OVERRUN_TRIM_ELLIPSIS
	title.add_theme_color_override("font_color",Color("26332f"))
	title.add_theme_font_size_override("font_size",15 if compact else 21)
	title.custom_minimum_size.y = 35 if compact else 53
	col.add_child(title)
	var pic = preload("res://scripts/card_art.gd").new()
	pic.texture = art(card.image)
	pic.category = card.type
	pic.custom_minimum_size.y = 95 if compact else 140
	pic.size_flags_vertical = Control.SIZE_EXPAND_FILL
	pic.clip_contents = true
	pic.mouse_filter = Control.MOUSE_FILTER_IGNORE
	col.add_child(pic)
	if pic.texture == null:
		var fallback = Label.new()
		fallback.text = "✦"
		fallback.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
		fallback.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
		fallback.vertical_alignment = VERTICAL_ALIGNMENT_CENTER
		fallback.add_theme_font_size_override("font_size",50)
		pic.add_child(fallback)
	var stats = Label.new()
	stats.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	stats.add_theme_color_override("font_color",Color("4c432e"))
	stats.add_theme_font_size_override("font_size",13 if compact else 17)
	if card.type == "enemy": stats.text = "СИЛА %d   •   +%d УР.   /   %d ДОБЫЧА" % [card.strength,card.level_reward,card.loot_cards]
	elif card.type == "equipment": stats.text = "+%d СИЛЫ   /   %s" % [card.bonus,"ОРУЖИЕ" if card.slot == "weapon" else "БРОНЯ"]
	else: stats.text = "✦  ОРКИ С РАЙОНА"
	col.add_child(stats)
	var desc = Label.new()
	desc.text = card.text
	desc.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	desc.add_theme_color_override("font_color",Color("34372d"))
	desc.add_theme_font_size_override("font_size",12 if compact else 15)
	if compact:
		desc.max_lines_visible = 3
		desc.text_overrun_behavior = TextServer.OVERRUN_TRIM_ELLIPSIS
	else: desc.custom_minimum_size.y = 65
	col.add_child(desc)
	mouse_entered.connect(func():
		if is_instance_valid(hover_tween): hover_tween.kill()
		pivot_offset = size/2
		z_index = 10
		hover_tween = create_tween()
		hover_tween.tween_property(self,"scale",Vector2(1.025,1.025),0.12))
	mouse_exited.connect(func():
		if is_instance_valid(hover_tween): hover_tween.kill()
		z_index = 0
		hover_tween = create_tween()
		hover_tween.tween_property(self,"scale",Vector2.ONE,0.12))
	tooltip_text = card.name+"\n"+card.text
	if card.type == "enemy": tooltip_text += "\nПоражение: "+{"card":"случайная карта с руки","level":"−1 уровень","armor":"сброс брони, иначе карты","weapon":"сброс оружия, иначе карты","two_cards":"потеря двух карт","level_card":"−1 уровень и потеря карты"}.get(card.penalty,"")
	gui_input.connect(func(event):
		if event is InputEventMouseButton and event.pressed and event.button_index == MOUSE_BUTTON_LEFT and clicked.is_valid(): clicked.call(card.id))

func _get_drag_data(_at_position: Vector2) -> Variant:
	if not draggable: return null
	var preview = PanelContainer.new()
	var caption = Label.new()
	caption.text = "  "+card.name+"  "
	caption.add_theme_font_size_override("font_size",20)
	preview.add_child(caption)
	preview.modulate = Color(1,0.9,0.7,0.9)
	set_drag_preview(preview)
	return {"card_id":card.id}
