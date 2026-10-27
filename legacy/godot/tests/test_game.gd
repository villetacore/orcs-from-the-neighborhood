extends SceneTree
const Rules = preload("res://scripts/game.gd")
var failures: int = 0
var checks: int = 0

func check(condition: bool, message: String) -> void:
	checks += 1
	if not condition:
		failures += 1
		printerr("FAIL: "+message)

func fixture():
	var g = Rules.new()
	g.start(["vityok","zina"],91)
	g.active = 0
	g.phase = "actions"
	g.choices.clear()
	for p in g.players:
		p.hand.clear()
		p.weapon = ""
		p.armor = ""
	return g

func conserved(g) -> bool:
	var held: Array = g.deck.duplicate()+g.discard.duplicate()
	for p in g.players:
		held.append_array(p.hand)
		for slot in ["weapon","armor"]:
			if p[slot] != "": held.append(p[slot])
	if g.encounter != "": held.append(g.encounter)
	for q in g.choices:
		if q.kind in ["portal","stash"]: held.append_array(q.options)
	if held.size() != 56: return false
	var seen = {}
	for id in held:
		if seen.has(id) or not g.cards.has(id) or g.cards[id].type == "character": return false
		seen[id] = true
	return true

func _initialize() -> void:
	var g = Rules.new()
	g.start(["vityok","zina","stalker","goblin"],42)
	check(g.cards.size()==60,"60 cards")
	check(g.deck.size()==44,"56 non-hero cards minus 12 dealt")
	check(conserved(g),"initial conservation")
	var counts = {}
	for c in g.cards.values(): counts[c.type] = counts.get(c.type,0)+1
	check(counts=={"character":4,"enemy":18,"equipment":12,"consumable":6,"event":10,"chaos":10},"category counts")
	g = fixture()
	g.players[0].level = 2
	g.begin_battle("zombie")
	check(g.can_win(),"ties favor hero")
	g.players[0].hand = ["salt","bullet","brine"]
	check(g.play("salt"),"hero chaos")
	check(not g.play("bullet"),"one chaos per player")
	check(g.play("brine"),"consumable after chaos")
	g.pass_reaction()
	g.players[1].hand = ["magic"]
	check(g.play("magic"),"opponent cancellation")
	check(g.hero_strength()==3,"cancel latest bonus, leave earlier one")
	g = fixture()
	g.players[0].level = 9
	g.begin_battle("dragon")
	g.enemy_mod = -9
	check(not g.can_win(),"dragon minimum 10 even after debuff")
	g.boosts = [{"amount":1,"source":"salt"}]
	check(g.can_win(),"dragon threshold satisfied")
	g.resolve_battle()
	check(g.winner==0 and g.players[0].level==10,"combat victory at level 10")
	g = fixture()
	g.players[0].level = 9
	g.players[0].hand = ["energy"]
	check(g.play("energy") and g.players[0].level==9 and g.winner==-1,"energy cannot win")
	g.end_turn()
	g.phase = "actions"
	g.end_turn()
	g.phase = "actions"
	g.players[0].hand = ["rebar"]
	check(g.players[0].skip_now and not g.play("rebar"),"energy skips next actions")
	g = fixture()
	g.players[0].armor = "curtain"
	g.players[0].hand = ["rebar"]
	g.lose_card(0)
	check(g.choices[0].kind=="protect","cloak offers protection")
	g.choose("protect")
	check(g.players[0].hand==["rebar"] and g.players[0].armor=="","cloak consumed")
	g = fixture()
	g.players[1].hand = ["helmet","rusk"]
	g.lose_card(1)
	check(g.choices[0].kind=="witch","witch choice")
	g.choose("rusk")
	check(g.players[1].hand==["helmet"] and g.players[1].witch_used,"witch substitute")
	g = fixture()
	g.players[0].hand = ["rebar","stash"]
	g.players[1].hand = ["helmet"]
	check(g.trade("rebar"),"trade offered")
	g.choose("1")
	check(g.owner()==1,"trade recipient owns consent")
	g.choose("decline")
	check(g.players[0].hand==["rebar","stash"],"decline keeps stash")
	g.trade("rebar")
	g.choose("1")
	g.choose("helmet")
	check(g.players[0].hand==["stash","helmet"] and g.players[1].hand==["rebar"],"ordinary trade never consumes stash")
	g = fixture()
	g.players[0].hand = ["brine"]
	g.begin_battle("kikimora")
	g.choose("brine")
	check(g.players[0].hand.is_empty() and g.battle_penalty==0,"pipe discard instead of penalty")
	g = fixture()
	g.begin_battle("kikimora")
	g.choose("penalty")
	check(g.battle_penalty==-1,"pipe penalty")
	g = Rules.new()
	g.start(["vityok","zina"],123)
	g.phase = "actions"
	g.event_effect("portal")
	var h = Rules.new()
	check(h.restore_state(JSON.parse_string(JSON.stringify(g.export_state()))),"save restore")
	check(conserved(h) and h.choices[0].kind=="portal","pending cards survive save")
	check(h.choose(h.choices[0].options[0]) and conserved(h),"portal choice conservation")
	g = fixture()
	g.players[0].level = 9
	g.players[0].weapon = "club"
	g.players[0].hand = ["helmet"]
	g.begin_battle("zombie")
	g.resolve_battle()
	check(g.winner==-1 and not g.choices.is_empty(),"final victory waits for weapon cost")
	g.choose(g.choices[0].options[0])
	check(g.winner==0,"victory after final cost")
	g = fixture()
	g.players[0].weapon="wrench"
	g.begin_battle("zombie")
	check(g.hero_strength()==4,"mechanic technical bonus exactly once")
	g.players[0].silenced=true
	check(g.hero_strength()==3,"silence suppresses ability")
	g = fixture()
	g.players[0].hero="stalker"
	g.begin_battle("rat")
	check(g.hero_strength()==2,"stalker mutant bonus")
	g = fixture()
	g.players[0].weapon="slipper"
	g.begin_battle("zombie")
	check(g.hero_strength()==4,"slipper bonus against undead")
	g = fixture()
	g.players[0].water=-1
	g.end_turn()
	g.phase="actions"
	g.end_turn()
	check(g.players[0].water==0,"water penalty expires at next own turn")
	g = fixture()
	g.players[0].hand=["rebar","helmet","brine","salt","rumor","luck"]
	check(not g.end_turn(),"hand limit blocks end turn")
	check(g.discard_excess("luck") and g.end_turn(),"chosen excess discard allows end turn")
	g = fixture()
	g.players[0].hand=["helmet","chain"]
	g.play("helmet")
	g.play("chain")
	check(g.players[0].armor=="chain" and g.discard.has("helmet"),"equipment replaces same slot")
	g = fixture()
	g.players[0].hand=["friend","battery","magic"]
	g.begin_battle("zombie")
	check(not g.play("friend") and not g.play("battery") and not g.play("magic"),"conditional chaos rejected without consuming")
	check(g.players[0].hand.size()==3,"invalid plays keep cards")
	test_v02()
	for count in [2,3,4]:
		for seed_value in range(1,5): simulate(count,seed_value)
	print("Checks: %d; failures: %d" % [checks,failures])
	quit(0 if failures==0 else 1)

func simulate(count: int, seed_value: int) -> void:
	var g = Rules.new()
	g.start(["vityok","zina","stalker","goblin"].slice(0,count),seed_value)
	var steps = 0
	while g.winner < 0 and steps < 15000:
		steps += 1
		if not g.choices.is_empty():
			var q = g.choices[0]
			g.choose(q.options[0])
		elif g.phase=="explore":
			var did = false
			for id in g.players[g.active].hand.duplicate():
				var c = g.cards[id]
				if c.type=="equipment" and (g.players[g.active][c.slot]=="" or int(g.cards[g.players[g.active][c.slot]].bonus)<int(c.bonus)):
					if g.play(id): did=true;break
			if not did: g.explore()
		elif g.phase=="combat":
			var p: int = g.owner()
			var did = false
			if not g.can_win() and g.help_error() == "": did = g.request_help()
			for id in g.players[p].hand.duplicate():
				if did: break
				var c = g.cards[id]
				if p==g.active and c.type=="consumable" or c.type=="chaos" and (c.effect in ["enemy_down","hero_up","friend","noise","enemy_up"]):
					if g.play(id): did=true;break
			if not did: g.pass_reaction()
		else:
			var did = false
			for id in g.players[g.active].hand.duplicate():
				var c = g.cards[id]
				if c.type=="equipment" and (g.players[g.active][c.slot]=="" or int(g.cards[g.players[g.active][c.slot]].bonus)<int(c.bonus)) or c.type in ["enemy","event"] or c.effect in ["energy","cycle","ambush","stash"]:
					if g.play(id): did=true;break
			if not did:
				if g.players[g.active].hand.size()>5: g.discard_excess(g.players[g.active].hand[0])
				elif not g.players[g.active].skip_now and not g.players[g.active].recycled and not g.players[g.active].hand.is_empty(): g.recycle(g.players[g.active].hand[0])
				else: g.end_turn()
		if not conserved(g):
			check(false,"conservation at %d players seed %d step %d" % [count,seed_value,steps])
			return
	check(g.winner>=0,"simulation reaches victory: %d players seed %d, %d turns" % [count,seed_value,g.turn])
	print("Simulation: %d players, seed %d: %d turns, %d actions" % [count,seed_value,g.turn,steps])


func extract(g, id: String) -> void:
	g.deck.erase(id)
	g.discard.erase(id)
	for p in g.players:
		p.hand.erase(id)
		for slot in ["weapon","armor"]:
			if p[slot] == id: p[slot] = ""

func set_hand(g, who: int, ids: Array) -> void:
	g.discard.append_array(g.players[who].hand)
	g.players[who].hand.clear()
	for id in ids:
		extract(g,id)
		g.players[who].hand.append(id)

func valid_battle(id: String):
	var g = Rules.new()
	g.start(["vityok","zina"],76)
	g.active = 0
	extract(g,id)
	g.begin_battle(id)
	return g

func complete_battle(g) -> void:
	for i in range(g.players.size()+1): g.pass_reaction()

func test_v02() -> void:
	var g = fixture()
	g.phase = "explore"
	g.players[0].hand = ["rebar","salt","energy","zombie","portal"]
	check(g.play("rebar"),"preparation equips before exploration")
	check(not g.play("energy") and not g.play("zombie") and not g.play("portal"),"preparation disallows actions that could skip exploration")
	g.deck = ["zombie"]
	check(g.explore() and g.hero_strength()==3 and g.can_win(),"prepared equipment counts in first encounter")
	g = fixture()
	g.phase = "explore"
	g.players[0].hand = ["rebar","stash"]
	g.players[1].hand = ["helmet"]
	check(g.trade("rebar"),"preparation trade available")
	g.choose("1")
	g.choose("helmet")
	check(g.players[0].hand.has("stash"),"preparation trade retains stash")
	check(g.recycle("stash"),"preparation replacement available")
	g.phase = "actions"
	check(not g.trade("helmet") and not g.recycle("helmet"),"trade and recycle limits span both action windows")
	g = fixture()
	g.phase = "explore"
	g.players[0].skip_now = true
	g.players[0].hand = ["rebar"]
	check(not g.play("rebar") and not g.trade("rebar") and not g.recycle("rebar"),"energy also skips preparation")
	check(g.explore(),"energy still permits exploration")
	g = fixture()
	g.players[0].level = 3
	g.players[0].hand = ["brine","salt","bullet"]
	g.players[1].hand = ["noise"]
	g.begin_battle("zombie")
	g.pass_reaction()
	g.play("noise")
	g.pass_reaction()
	check(g.final_response() and g.owner()==0 and g.phase=="combat" and not g.can_win(),"last opponent returns control without resolving")
	check(g.play("brine") and g.can_win(),"hero can save battle with final consumable")
	check(g.play("salt") and not g.play("bullet"),"final response allows unused chaos but preserves one-card cap")
	check(g.pass_reaction() and g.phase=="actions" and g.players[0].level==4,"final confirmation resolves battle once")
	g = fixture()
	g.players[0].hand = ["salt","rumor"]
	g.begin_battle("zombie")
	g.play("salt")
	g.pass_reaction()
	g.pass_reaction()
	check(g.final_response() and not g.play("rumor"),"initial hero chaos cannot be repeated in final response")
	g = Rules.new()
	g.start(["vityok","zina","stalker","goblin"],44)
	g.active = 2
	extract(g,"zombie")
	g.begin_battle("zombie")
	for expected in [2,3,0,1]:
		check(g.owner()==expected,"clockwise combat order")
		g.pass_reaction()
	check(g.owner()==2 and g.final_response(),"four-player final response returns to active hero")
	check(conserved(g),"conservation before final response")
	g = valid_battle("zombie")
	set_hand(g,1,["helmet"])
	check(g.request_help() and g.owner()==0,"hero owns choice of helper")
	g.choose("cancel")
	check(not g.help_requested,"cancel without sending does not spend offer")
	g.request_help()
	g.choose("1")
	check(g.owner()==1 and g.choices[0].kind=="help_accept","recipient owns consent and payment")
	var saved = g.export_state()
	var h = Rules.new()
	check(h.restore_state(JSON.parse_string(JSON.stringify(saved))) and h.owner()==1 and conserved(h),"pending help consent survives save")
	g.choose("decline")
	check(g.players[1].hand==["helmet"] and g.helper==-1 and not g.request_help(),"declining costs no cards and ends offer opportunity")
	g = valid_battle("zombie")
	g.players[0].level = 5
	set_hand(g,1,["helmet","friend","noise"])
	g.request_help()
	g.choose("1")
	g.choose("helmet")
	check(g.helper==1 and g.discard.has("helmet") and not g.players[1].witch_used and not g.chaos_used.has(1),"help pays chosen card without spending chaos or triggering protection")
	check(g.hero_strength()==7 and conserved(g),"contract grants +2 and conserves cards")
	g.pass_reaction()
	check(not g.play("friend") and g.play("noise"),"helper slot excludes friend but helper can still play chaos")
	g.pass_reaction()
	var reward = g.deck.back()
	g.pass_reaction()
	check(g.players[1].hand.has(reward) and g.players[0].level==6 and g.players[1].level==1,"helper gets first loot; only hero gets level")
	check(conserved(g),"conservation after contract victory")
	g = valid_battle("elevator_boss")
	set_hand(g,1,["helmet","salt"])
	g.request_help()
	g.choose("1")
	g.choose("helmet")
	complete_battle(g)
	check(g.players[1].hand==["salt"] and g.players[1].level==1,"helper pays cost but avoids defeat penalty")
	check(conserved(g),"conservation after contract defeat with pending hero penalty")
	g = valid_battle("zombie")
	set_hand(g,1,["helmet","magic"])
	g.request_help()
	g.choose("1")
	g.choose("helmet")
	g.pass_reaction()
	check(g.play("magic") and g.helper==-1 and g.discard.has("helmet"),"cancelled contract loses bonus and loot right without refund")
	check(conserved(g),"conservation after contract cancellation")
	g = fixture()
	g.players[0].hand=["salt"]
	g.begin_battle("zombie")
	g.play("salt")
	check(not g.request_help(),"help must precede battle card plays")
	g = Rules.new()
	g.start(["vityok","zina"],55)
	g.active=0
	g.phase="actions"
	set_hand(g,0,["stash"])
	extract(g,"rebar")
	g.deck.push_back("rebar")
	extract(g,"helmet")
	g.deck.push_back("helmet")
	check(g.play("stash") and g.choices[0].options==["helmet","rebar"] and conserved(g),"stash holds top two cards outside deck")
	h = Rules.new()
	check(h.restore_state(JSON.parse_string(JSON.stringify(g.export_state()))) and conserved(h),"pending stash survives save")
	h.choose("rebar")
	check(h.players[0].hand==["rebar"] and h.deck.front()=="helmet" and not h.discard.has("helmet"),"unchosen stash card goes to bottom")
	check(conserved(h),"stash choice conserves cards")
	g = fixture()
	g.players[0].hand=["stash"]
	g.deck=[]
	g.discard=[]
	check(g.play("stash") and g.choices.is_empty() and g.discard==["stash"],"empty deck cannot redraw resolving stash")
	g = fixture()
	g.players[0].hand=["stash"]
	g.deck=["rebar"]
	g.discard=[]
	g.play("stash")
	check(g.choices[0].options==["rebar"],"stash handles single remaining card")
	g.choose("rebar")
	check(g.players[0].hand==["rebar"] and g.deck.is_empty(),"single stash card is kept")
	g = Rules.new()
	g.start(["vityok","zina"],88)
	g.active=0
	g.phase="actions"
	set_hand(g,0,["cutlet","salt"])
	check(g.play("cutlet") and g.players[0].hand.size()==3 and conserved(g),"cutlet draws two before choice")
	g.choose("salt")
	check(g.players[0].hand.size()==2 and conserved(g),"cutlet filters hand without increasing size")
	g = valid_battle("zombie")
	g.pass_reaction()
	g.pass_reaction()
	h = Rules.new()
	check(h.restore_state(JSON.parse_string(JSON.stringify(g.export_state()))) and h.final_response() and h.owner()==0 and conserved(h),"final response survives save")
	var legacy = g.export_state()
	legacy.version=1
	legacy.erase("help_requested")
	legacy.reaction_step=1
	legacy.reaction=1
	h = Rules.new()
	check(h.restore_state(legacy) and h.reaction_step==1 and conserved(h),"version-one combat save migrates without losing cards")
	legacy.choices=[{"kind":"trade_accept","player":1,"from":0,"offered":"rebar","stash":"stash","title":"Обмен","options":["decline"]}]
	h = Rules.new()
	check(h.restore_state(legacy) and not h.choices[0].has("stash"),"legacy pending trade drops obsolete stash charge")
	check(not h.restore_state({"version":2}),"incomplete save rejected")
