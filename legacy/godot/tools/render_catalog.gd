extends SceneTree
## Render QA contact sheets with the same card renderer used by the game.
const Card = preload("res://scripts/card_view.gd")
const Rules = preload("res://scripts/game.gd")
func _initialize() -> void:
	call_deferred("render")
func render() -> void:
	var g = Rules.new()
	for type in ["character","enemy","equipment","consumable","event","chaos"]:
		var ids: Array = []
		for c in g.cards.values():
			if c.type==type: ids.append(c.id)
		var cols = mini(6,ids.size())
		var rows = ceili(float(ids.size())/cols)
		var viewport = SubViewport.new()
		viewport.size = Vector2i(cols*260+20,rows*430+20)
		viewport.render_target_update_mode = SubViewport.UPDATE_ALWAYS
		root.add_child(viewport)
		var bg = ColorRect.new()
		bg.color = Color("182321")
		bg.size = viewport.size
		viewport.add_child(bg)
		for i in range(ids.size()):
			var face = Card.new()
			face.card = g.cards[ids[i]]
			face.compact = false
			face.position = Vector2(20+(i%cols)*260,20+floori(float(i)/cols)*430)
			face.size = Vector2(240,410)
			face.custom_minimum_size = face.size
			viewport.add_child(face)
		await process_frame
		await RenderingServer.frame_post_draw
		var image = viewport.get_texture().get_image()
		image.save_png("res://output/preview-"+type+".png")
		viewport.queue_free()
		await process_frame
		print("Rendered "+type+": "+str(ids.size())+" cards")
	Card.textures.clear()
	quit()
