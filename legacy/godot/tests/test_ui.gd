extends SceneTree
const Table = preload("res://main.tscn")
var failures: int = 0
var checks: int = 0
func check(ok: bool, message: String) -> void:
	checks += 1
	if not ok:
		failures += 1
		printerr("FAIL UI: "+message)

func _initialize() -> void:
	call_deferred("run")

func run() -> void:
	root.size = Vector2i(1600,1000)
	var table = Table.instantiate()
	root.add_child(table)
	await process_frame
	await process_frame
	check(table.screen=="menu","menu opens")
	for type in ["all","character","enemy","equipment","consumable","event","chaos"]:
		table.gallery_type = type
		table.screen = "gallery"
		table.rebuild()
		await process_frame
	check(true,"all gallery filters render")
	table.screen = "rules"
	table.rebuild()
	await process_frame
	check(table.root.size.x<=1600,"rules fit viewport")
	table.game.start(["vityok","zina","stalker","goblin"],42)
	table.screen = "table"
	table.private_owner = -1
	table.rebuild()
	await process_frame
	check(is_instance_valid(table.curtain) and table.curtain.is_inside_tree(),"initial hand concealed")
	table.private_owner = table.game.owner()
	table.rebuild()
	await process_frame
	check(table.root.size.y<=1000 and table.root.size.x<=1600,"four-player table fits viewport")
	table.game.explore()
	table.private_owner = table.game.owner()
	table.rebuild()
	await process_frame
	check(table.root.size.y<=1000 and table.root.size.x<=1600,"encounter fits viewport")
	# Force an owner change and verify that a full opaque curtain is recreated.
	table.game.choices.push_front({"kind":"reveal","player":(table.game.owner()+1)%4,"subject":table.game.active,"title":"Скрытый выбор","options":["ok"]})
	table.rebuild()
	await process_frame
	check(table.curtain.is_inside_tree() and table.curtain.color.a==1.0,"owner change hides all hands")
	check(table.curtain.get_global_rect().encloses(table.root.get_global_rect().grow(-23)),"privacy curtain covers table")
	# All equipment visible at once must scroll the roster rather than stretch the table.
	table.game.choices.clear()
	for p in table.game.players:
		p.weapon="rebar"
		p.armor="chain"
	table.private_owner=table.game.owner()
	table.rebuild()
	await process_frame
	check(table.root.size.y<=1000 and table.root.size.x<=1600,"fully equipped roster fits")
	if table.game.phase != "combat":
		table.game.deck.erase("zombie")
		table.game.discard.erase("zombie")
		for p in table.game.players: p.hand.erase("zombie")
		table.game.begin_battle("zombie")
	# Return to the active hero for a final response without skipping any opponent.
	if table.game.phase == "combat":
		while not table.game.final_response(): table.game.pass_reaction()
		table.private_owner=table.game.owner()
		table.rebuild()
		await process_frame
		check(table.root.size.y<=1000 and table.root.size.x<=1600,"final response fits viewport: %s" % str(table.root.size))
		check(has_button(table,"ПОДТВЕРДИТЬ БОЙ"),"final confirmation is explicit")
		check(not has_button(table,"ПРЕДЛОЖИТЬ ПОМОЩЬ"),"cannot offer help after first round")
	# Test a real consent screen, with an opponent's hand behind the privacy curtain.
	table.game.start(["vityok","zina","stalker","goblin"],42)
	table.game.active=0
	for p in table.game.players:
		p.hand.erase("zombie")
	table.game.deck.erase("zombie")
	table.game.begin_battle("zombie")
	table.private_owner=0
	table.rebuild()
	await process_frame
	check(has_button(table,"ПРЕДЛОЖИТЬ ПОМОЩЬ"),"opening battle exposes help action")
	table.game.request_help()
	table.game.choose(table.game.choices[0].options[0])
	table.rebuild()
	await process_frame
	check(table.game.owner()!=0 and table.curtain.is_inside_tree(),"help consent switches privacy owner")
	table.private_owner=table.game.owner()
	table.rebuild()
	await process_frame
	check(table.root.size.y<=1000 and table.root.size.x<=1600,"help consent fits viewport")
	check(has_button(table,"Отказаться"),"help recipient can decline")
	table.free()
	await process_frame
	await process_frame
	preload("res://scripts/card_view.gd").textures.clear()
	print("UI checks: %d; failures: %d" % [checks,failures])
	quit(0 if failures==0 else 1)


func has_button(node: Node, text: String) -> bool:
	if node is Button and node.text == text: return true
	for child in node.get_children():
		if has_button(child,text): return true
	return false
