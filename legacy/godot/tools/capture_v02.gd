extends SceneTree
## Reproducible UI captures; never writes the user's actual save.
const Table = preload("res://main.tscn")
var table

func _initialize() -> void:
	call_deferred("capture")

func extract(id: String) -> void:
	table.game.deck.erase(id)
	table.game.discard.erase(id)
	for p in table.game.players:
		p.hand.erase(id)
		for slot in ["weapon","armor"]:
			if p[slot] == id: p[slot] = ""

func hand(who: int, ids: Array) -> void:
	table.game.discard.append_array(table.game.players[who].hand)
	table.game.players[who].hand.clear()
	for id in ids:
		extract(id)
		table.game.players[who].hand.append(id)

func shot(name: String) -> void:
	table.screen="table"
	table.private_owner=table.game.owner()
	table.rebuild()
	await process_frame
	await process_frame
	await RenderingServer.frame_post_draw
	root.get_texture().get_image().save_png("res://output/v02-"+name+".png")
	print("Captured "+name)

func capture() -> void:
	table=Table.instantiate()
	root.add_child(table)
	table.game.start(["vityok","zina","stalker"],42)
	table.game.active=2
	hand(2,["rebar","chain","brine","stash"])
	hand(0,["coat"])
	hand(1,["bullet"])
	table.game.play("rebar")
	table.game.play("chain")
	await shot("preparation")
	extract("golem")
	table.game.deck.push_back("golem")
	table.game.explore()
	table.game.request_help()
	table.game.choose("0")
	await shot("help-consent")
	table.game.choose("coat")
	table.game.pass_reaction()
	table.game.pass_reaction()
	table.game.play("bullet")
	table.game.pass_reaction()
	table.game.play("brine")
	await shot("final-response")
	table.game.pass_reaction()
	table.game.play("stash")
	await shot("stash")
	table.free()
	await process_frame
	preload("res://scripts/card_view.gd").textures.clear()
	quit()
