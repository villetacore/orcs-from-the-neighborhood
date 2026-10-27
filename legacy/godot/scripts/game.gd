extends RefCounted
## Pure rules engine. No UI dependencies; all hidden decisions have an explicit owner.
const SAVE_VERSION = 2
var cards: Dictionary = {}
var players: Array = []
var deck: Array = []
var discard: Array = []
var active: int = 0
var phase: String = "explore"
var turn: int = 1
var encounter: String = ""
var last_card: String = ""
var reaction: int = 0
var reaction_step: int = 0
var chaos_used: Array = []
var boosts: Array = []
var enemy_mod: int = 0
var weapon_mod: int = 0
var battle_penalty: int = 0
var lucky: bool = false
var rusk: bool = false
var helper: int = -1
var help_requested: bool = false
var fought: bool = false
var choices: Array = []
var journal: Array = []
var winner: int = -1
var error: String = ""
var rng = RandomNumberGenerator.new()

func _init() -> void:
	var source = JSON.parse_string(FileAccess.get_file_as_string("res://data/cards.json"))
	for c in source:
		cards[c.id] = c

func start(hero_ids: Array, game_seed: int = 0) -> void:
	players.clear()
	deck.clear()
	discard.clear()
	choices.clear()
	journal.clear()
	winner = -1
	turn = 1
	phase = "explore"
	encounter = ""
	last_card = ""
	fought = false
	helper = -1
	help_requested = false
	reaction_step = 0
	boosts.clear()
	chaos_used.clear()
	weapon_mod = 0
	battle_penalty = 0
	if game_seed == 0:
		rng.randomize()
	else:
		rng.seed = game_seed
	for id in hero_ids:
		players.append({"hero":id,"level":1,"hand":[],"weapon":"","armor":"",
			"water":0,"next_penalty":0,"silenced":false,"witch_used":false,
			"skip_next":false,"skip_now":false,"recycled":false,"traded":0})
	for id in cards:
		if cards[id].type != "character":
			deck.append(id)
	shuffle(deck)
	for p in players:
		for j in range(3):
			var id = draw()
			if id != "": p.hand.append(id)
	active = rng.randi_range(0, players.size()-1)
	note("Партия началась. Первый ход: %s." % player_name(active))

func shuffle(a: Array) -> void:
	for i in range(a.size()-1, 0, -1):
		var j = rng.randi_range(0, i)
		var temp = a[i]
		a[i] = a[j]
		a[j] = temp

func draw() -> String:
	if deck.is_empty() and not discard.is_empty():
		deck = discard.duplicate()
		discard.clear()
		shuffle(deck)
		note("Сброс перемешан в новую колоду.")
	if deck.is_empty(): return ""
	return deck.pop_back()

func take(p: int, count: int = 1) -> void:
	for i in range(count):
		var id = draw()
		if id != "": players[p].hand.append(id)

func player_name(p: int) -> String:
	return cards[players[p].hero].name

func note(message: String) -> void:
	journal.append(message)
	if journal.size() > 120: journal.pop_front()

func reject(message: String) -> bool:
	error = message
	return false

func owner() -> int:
	if not choices.is_empty(): return int(choices[0].player)
	if phase == "combat": return reaction
	return active

func ability(p: int, effect: String) -> bool:
	return not players[p].silenced and cards[players[p].hero].effect == effect

func technical(p: int) -> bool:
	for slot in ["weapon","armor"]:
		var id = players[p][slot]
		if id != "" and cards[id].get("technical",false): return true
	return false

func strength_breakdown() -> Array:
	var p = players[active]
	var parts: Array = [{"label":"Уровень","amount":int(p.level)}]
	if p.water != 0: parts.append({"label":"Холодная вода","amount":int(p.water)})
	for slot in ["weapon","armor"]:
		var id = p[slot]
		if id != "":
			var bonus: int = cards[id].bonus
			if slot == "weapon": bonus = maxi(0,bonus+weapon_mod)
			parts.append({"label":"Оружие" if slot == "weapon" else "Броня","amount":bonus})
			if encounter != "" and cards[id].effect == "undead" and cards[encounter].tag == "undead":
				parts.append({"label":"Против нежити","amount":2})
	if ability(active,"mechanic") and technical(active): parts.append({"label":"Механик","amount":1})
	if encounter != "" and ability(active,"mutant_hunter") and cards[encounter].tag == "mutant": parts.append({"label":"Сталкер","amount":1})
	if battle_penalty != 0: parts.append({"label":"Штраф встречи","amount":battle_penalty})
	for boost in boosts:
		parts.append({"label":"Помощь" if boost.source == "help_contract" else cards[boost.source].name,"amount":int(boost.amount)})
	return parts

func hero_strength() -> int:
	var total = 0
	for part in strength_breakdown(): total += int(part.amount)
	return maxi(0,total)

func strength_text() -> String:
	var parts = PackedStringArray()
	for part in strength_breakdown(): parts.append("%s %+d" % [part.label,part.amount])
	return " · ".join(parts)

func enemy_strength() -> int:
	if encounter == "": return 0
	var c = cards[encounter]
	var total: int = c.strength + enemy_mod
	match c.effect:
		"unarmored":
			if players[active].armor == "": total += 1
		"armor":
			if players[active].armor != "": total += 1
		"technical":
			var id = players[active].weapon
			if id != "" and cards[id].get("technical",false): total += 1
	return maxi(0,total)

func can_win() -> bool:
	return hero_strength() >= enemy_strength() and (cards[encounter].effect != "dragon" or hero_strength() >= 10)

func explore() -> bool:
	if phase != "explore" or not choices.is_empty(): return reject("Сейчас нельзя исследовать.")
	phase = "actions"
	open_card()
	return true

func open_card() -> void:
	var id = draw()
	if id == "":
		note("Колода пуста. Можно перейти к действиям.")
		return
	last_card = id
	note("%s открывает «%s»." % [player_name(active),cards[id].name])
	match cards[id].type:
		"enemy":
			if fought:
				players[active].hand.append(id)
			else: begin_battle(id)
		"event":
			discard.append(id)
			event_effect(cards[id].effect)
		_: players[active].hand.append(id)

func begin_battle(id: String) -> void:
	encounter = id
	fought = true
	phase = "combat"
	reaction = active
	reaction_step = 0
	chaos_used.clear()
	boosts.clear()
	enemy_mod = 0
	weapon_mod = 0
	battle_penalty = -int(players[active].next_penalty)
	players[active].next_penalty = 0
	lucky = false
	rusk = false
	helper = -1
	help_requested = false
	if cards[id].effect == "pipe":
		var options: Array = []
		for held in players[active].hand:
			if cards[held].type == "consumable": options.append(held)
		options.append("penalty")
		choices.append({"kind":"pipe","player":active,"title":"Кикимора: сбрось расходник или получи −1 силы","options":options})

func event_effect(effect: String) -> void:
	match effect:
		"water":
			for offset in range(players.size()):
				var p = (active+offset)%players.size()
				var opts = players[p].hand.duplicate()
				opts.append("penalty")
				choices.append({"kind":"water","player":p,"title":"Нет горячей воды: сбрось карту или −1 силы до своего хода","options":opts})
		"portal":
			var opts: Array = []
			for i in range(2):
				var id = draw()
				if id != "": opts.append(id)
			if not opts.is_empty(): choices.append({"kind":"portal","player":active,"title":"Портал: какую карту оставить?","options":opts})
		"silence":
			var opts: Array = []
			for i in range(players.size()): opts.append(str(i))
			choices.append({"kind":"silence","player":active,"title":"Кто лишится способности до конца своего следующего боя?","options":opts})
		"lift":
			if players[active].hand.is_empty(): players[active].next_penalty += 1
			else: lose_card(active)
		"raid": open_card()
		"reveal": choices.append({"kind":"reveal","player":(active+1)%players.size(),"subject":active,"title":"Сосед показывает вам руку. Посмотрите и закройте.","options":["ok"]})
		"draw": take(active)
		"harvest": take(active,2)
		"rust": remove_gear(active,"armor")
		"tax": players[active].level = maxi(1,players[active].level-1)

func remove_gear(p: int, slot: String) -> void:
	var id = players[p][slot]
	if id != "":
		players[p][slot] = ""
		discard.append(id)
		note("%s теряет «%s»." % [player_name(p),cards[id].name])

func lose_card(p: int) -> void:
	if players[p].hand.is_empty(): return
	var armor = players[p].armor
	if armor != "" and cards[armor].effect == "protect":
		choices.append({"kind":"protect","player":p,"title":"Сбросить плащ вместо случайной карты?","options":["protect","card"]})
	elif ability(p,"witch") and not players[p].witch_used:
		var opts = players[p].hand.duplicate()
		opts.append("random")
		choices.append({"kind":"witch","player":p,"title":"Зина: выбери другую жертву или обычную случайную потерю","options":opts})
	else: random_loss(p)

func random_loss(p: int) -> void:
	if players[p].hand.is_empty(): return
	var index = rng.randi_range(0,players[p].hand.size()-1)
	var id = players[p].hand.pop_at(index)
	discard.append(id)
	note("%s теряет «%s»." % [player_name(p),cards[id].name])

func play_error(id: String) -> String:
	# The UI and drag-and-drop use the same legality check as the rules engine.
	if players.is_empty() or winner >= 0: return "Партия не идёт."
	if not choices.is_empty(): return "Сначала завершите текущий выбор."
	var who = owner()
	if not players[who].hand.has(id): return "Выберите карту из своей руки."
	var c = cards[id]
	if phase == "combat":
		if c.type == "consumable":
			if who != active: return "Расходники применяет только герой этого боя."
			if c.effect not in ["boost","tape","luck","rusk"]: return "Этот расходник используется после исследования."
			if c.effect == "tape" and players[active].weapon == "": return "Для изоленты нужно оружие."
			return ""
		if c.type != "chaos": return "В бой можно играть расходники и боевой хаос."
		if chaos_used.has(who): return "Вы уже сыграли свою карту хаоса в этом бою."
		match c.effect:
			"enemy_up", "noise", "enemy_down": return ""
			"weapon_down":
				if who == active or players[active].weapon == "" or int(cards[players[active].weapon].bonus)+weapon_mod <= 0: return "Нужно оружие соперника с положительным бонусом."
			"battery":
				if who == active or not technical(active): return "Нужна техническая экипировка соперника."
			"hero_up":
				if who != active: return "Только в свой бой."
			"friend":
				if who == active or helper >= 0: return "Нужен чужой бой без другого помощника."
			"cancel":
				for boost in boosts:
					if int(boost.amount)>0: return ""
				return "Нет положительного временного бонуса для отмены."
			_: return "Эта карта применяется после исследования."
		return ""
	if phase not in ["explore","actions"]: return "Эту карту сейчас нельзя сыграть."
	if players[active].skip_now: return "Энергетик: подготовка и действия в этот ход пропущены."
	if phase == "explore" and c.type != "equipment": return "В подготовку можно надевать снаряжение, обмениваться и заменять карту. Остальное — после исследования."
	match c.type:
		"equipment", "event": return ""
		"enemy":
			if fought: return "В этот ход уже был бой."
			return ""
		"consumable":
			if c.effect in ["energy","cycle"]: return ""
			return "Этот расходник используется во время боя."
		"chaos":
			if c.effect in ["ambush","stash"]: return ""
			return "Эта карта хаоса используется во время боя."
	return "Эту карту сейчас нельзя сыграть."

func play(id: String) -> bool:
	error = play_error(id)
	if error != "": return false
	var who = owner()
	var c = cards[id]
	if phase == "combat":
		if c.type == "consumable":
			match c.effect:
				"boost": boosts.append({"amount":2,"source":id})
				"tape": boosts.append({"amount":1,"source":id})
				"luck": lucky = true
				"rusk": rusk = true
		else:
			match c.effect:
				"enemy_up": enemy_mod += 2
				"noise": enemy_mod += 3
				"enemy_down": enemy_mod -= 2
				"weapon_down": weapon_mod -= 1
				"battery": boosts.append({"amount":-2,"source":id})
				"hero_up": boosts.append({"amount":1,"source":id})
				"friend":
					helper = who
					boosts.append({"amount":2,"source":id})
				"cancel":
					for i in range(boosts.size()-1,-1,-1):
						if int(boosts[i].amount)>0:
							if boosts[i].source in ["friend","help_contract"]: helper = -1
							boosts.remove_at(i)
							break
			chaos_used.append(who)
	else:
		match c.type:
			"equipment":
				remove_gear(active,c.slot)
				players[active][c.slot] = id
				players[active].hand.erase(id)
				note("%s экипирует «%s»." % [player_name(active),c.name])
				return true
			"enemy":
				players[active].hand.erase(id)
				note("Герой вызывает с руки «%s»." % c.name)
				begin_battle(id)
				return true
			"event":
				players[active].hand.erase(id)
				discard.append(id)
				note("%s разыгрывает «%s»." % [player_name(active),c.name])
				event_effect(c.effect)
				return true
			"consumable":
				if c.effect == "energy":
					players[active].level = mini(9,players[active].level+1)
					players[active].skip_next = true
				else:
					# Exclude the resolving cutlet from both hand and redraw until draws finish.
					players[active].hand.erase(id)
					take(active,2)
					discard.append(id)
					if not players[active].hand.is_empty(): choices.append({"kind":"discard","player":active,"title":"Котлета: возьми две, затем выбери одну карту для сброса","options":players[active].hand.duplicate()})
					note("Герой использует «%s»." % c.name)
					return true
			"chaos":
				if c.effect == "stash":
					players[active].hand.erase(id)
					var options: Array = []
					for i in range(2):
						var found = draw()
						if found != "": options.append(found)
					discard.append(id)
					if not options.is_empty(): choices.append({"kind":"stash","player":active,"title":"Тайник: возьми одну карту, вторую положи вниз колоды","options":options})
					note("%s открывает свой тайник." % player_name(active))
					return true
				var options: Array = []
				for i in range(players.size()):
					if i != active: options.append(str(i))
				choices.append({"kind":"ambush","player":active,"title":"Кому устроить подставу?","options":options})
	players[who].hand.erase(id)
	discard.append(id)
	note("%s играет «%s»." % [player_name(who),c.name])
	return true

func final_response() -> bool:
	return phase == "combat" and reaction_step == players.size()

func reaction_text() -> String:
	if final_response(): return "Финальный ответ героя. После подтверждения бой завершится; нового круга нет."
	var remaining = players.size()-reaction_step-1
	return "После вас вмешаются ещё %d. Затем герой получит финальный ответ." % remaining

func help_error() -> String:
	if phase != "combat" or winner >= 0 or not choices.is_empty(): return "Сначала завершите текущий выбор."
	if reaction_step != 0: return "Помощь предлагают до первого круга вмешательства."
	if helper >= 0: return "В этом бою уже есть помощник."
	if help_requested: return "Предложение помощи уже было сделано в этом бою."
	if not boosts.is_empty() or chaos_used.has(active) or lucky or rusk: return "Предложите помощь до розыгрыша боевых карт."
	for i in range(players.size()):
		if i != active and not players[i].hand.is_empty(): return ""
	return "У соперников нет карт для оплаты помощи."

func request_help() -> bool:
	var reason = help_error()
	if reason != "": return reject(reason)
	var options: Array = []
	for i in range(players.size()):
		if i != active and not players[i].hand.is_empty(): options.append(str(i))
	choices.append({"kind":"help_target","player":active,"title":"Кому предложить помощь? +2 силы за сброс карты; первая добыча помощнику","options":options+["cancel"]})
	return true

func pass_reaction() -> bool:
	if phase != "combat" or not choices.is_empty(): return reject("Сначала завершите текущий выбор.")
	if final_response():
		note("%s подтверждает результат боя." % player_name(active))
		resolve_battle()
		return true
	note("%s завершает вмешательство." % player_name(reaction))
	reaction_step += 1
	if reaction_step == players.size():
		reaction = active
		note("Последнее слово возвращается герою.")
	else: reaction = (active+reaction_step)%players.size()
	return true

func resolve_battle() -> void:
	var c = cards[encounter]
	var won = can_win()
	note("Бой: %d против %d. %s" % [hero_strength(),enemy_strength(),"Победа!" if won else "Поражение."])
	players[active].silenced = false
	if won:
		players[active].level = mini(10,players[active].level+int(c.level_reward))
		for j in range(int(c.loot_cards)):
			take(helper if helper >= 0 and j == 0 else active)
		var w = players[active].weapon
		if w != "" and cards[w].effect == "victory_discard" and not players[active].hand.is_empty():
			choices.append({"kind":"discard","player":active,"title":"Цена дубины: выбери карту для сброса","options":players[active].hand.duplicate()})
	else:
		if not lucky:
			if c.penalty in ["level","level_card"] and not rusk: players[active].level = maxi(1,players[active].level-1)
			if c.penalty in ["card","two_cards","level_card"]: lose_card(active)
			if c.penalty == "two_cards":
				# Resolve the second loss after the first decision to avoid stale options.
				choices.append({"kind":"second_loss","player":active,"title":"Ведьма очереди требует вторую карту","options":["ok"]})
			if c.penalty in ["armor","weapon"]:
				if players[active][c.penalty] != "": remove_gear(active,c.penalty)
				else: lose_card(active)
	discard.append(encounter)
	last_card = encounter
	encounter = ""
	phase = "actions"
	boosts.clear()
	weapon_mod = 0
	battle_penalty = 0
	finish_victory()

func finish_victory() -> void:
	if choices.is_empty() and phase == "actions" and players[active].level >= 10 and winner < 0:
		winner = active
		note("%s становится легендой района!" % player_name(active))

func choose(option: String) -> bool:
	if choices.is_empty(): return reject("Нет активного выбора.")
	var q = choices[0]
	if not q.options.has(option): return reject("Недопустимый выбор.")
	choices.pop_front()
	var p: int = q.player
	match q.kind:
		"discard", "water", "pipe":
			if option == "penalty":
				if q.kind == "water": players[p].water = -1
				else: battle_penalty -= 1
			else:
				players[p].hand.erase(option)
				discard.append(option)
				note("%s сбрасывает «%s»." % [player_name(p),cards[option].name])
		"portal", "stash":
			players[p].hand.append(option)
			for id in q.options:
				if id != option:
					if q.kind == "stash": deck.push_front(id)
					else: discard.append(id)
		"silence":
			players[int(option)].silenced = true
			note("%s временно лишается способности." % player_name(int(option)))
		"ambush": lose_card(int(option))
		"witch":
			if option == "random": random_loss(p)
			else:
				players[p].witch_used = true
				players[p].hand.erase(option)
				discard.append(option)
		"protect":
			if option == "protect": remove_gear(p,"armor")
			else: random_loss(p)
		"second_loss": lose_card(p)
		"help_target":
			if option != "cancel":
				help_requested = true
				choices.push_front({"kind":"help_accept","player":int(option),"title":"%s просит помощи: сбросьте карту за +2 силы и первую добычу при победе. Можно отказаться." % player_name(active),"options":players[int(option)].hand.duplicate()+["decline"]})
		"help_accept":
			if option != "decline":
				players[p].hand.erase(option)
				discard.append(option)
				helper = p
				boosts.append({"amount":2,"source":"help_contract"})
				note("%s помогает за «%s»: +2 силы, первая добыча помощнику." % [player_name(p),cards[option].name])
			else: note("%s отказывается помогать." % player_name(p))
		"trade_target":
			choices.push_front({"kind":"trade_accept","player":int(option),"from":p,"offered":q.offered,"title":"%s предлагает «%s». Выберите свою карту или откажитесь." % [player_name(p),cards[q.offered].name],"options":players[int(option)].hand.duplicate()+["decline"]})
		"trade_accept":
			if option != "decline":
				players[p].hand.erase(option)
				players[q.from].hand.erase(q.offered)
				players[p].hand.append(q.offered)
				players[q.from].hand.append(option)
				players[q.from].traded += 1
				note("Игроки обменялись картами.")
			else: note("Обмен отклонён.")
	finish_victory()
	return true

func recycle(id: String) -> bool:
	if winner >= 0 or phase not in ["explore","actions"] or not choices.is_empty() or players[active].skip_now or players[active].recycled: return reject("Замена доступна один раз за ход: в подготовку или после исследования.")
	if not players[active].hand.has(id): return reject("Выберите карту с руки.")
	players[active].hand.erase(id)
	discard.append(id)
	take(active)
	players[active].recycled = true
	note("%s заменяет одну карту." % player_name(active))
	return true

func trade(id: String) -> bool:
	if winner >= 0 or phase not in ["explore","actions"] or not choices.is_empty() or players[active].skip_now: return reject("Обмен доступен в подготовку или после исследования.")
	if players[active].traded >= (2 if ability(active,"merchant") else 1): return reject("Лимит обменов на этот ход исчерпан.")
	if not players[active].hand.has(id): return reject("Выберите предлагаемую карту с руки.")
	var opts: Array = []
	for i in range(players.size()):
		if i != active and not players[i].hand.is_empty(): opts.append(str(i))
	if opts.is_empty(): return reject("Ни у кого нет карт для обмена.")
	choices.append({"kind":"trade_target","player":active,"offered":id,"title":"Кому предложить «%s»?" % cards[id].name,"options":opts})
	return true

func discard_excess(id: String) -> bool:
	if phase != "actions" or not choices.is_empty() or players[active].hand.size() <= 5 or not players[active].hand.has(id): return reject("Свободный сброс доступен только при превышении лимита 5.")
	players[active].hand.erase(id)
	discard.append(id)
	return true

func end_turn() -> bool:
	if winner >= 0 or phase != "actions" or not choices.is_empty(): return reject("Сначала завершите исследование и все эффекты.")
	if players[active].hand.size() > 5: return reject("Сбросьте лишние карты до лимита 5.")
	active = (active+1)%players.size()
	turn += 1
	phase = "explore"
	fought = false
	last_card = ""
	for p in players: p.witch_used = false
	players[active].water = 0
	players[active].recycled = false
	players[active].traded = 0
	players[active].skip_now = players[active].skip_next
	players[active].skip_next = false
	note("Ход %d. %s." % [turn,player_name(active)])
	return true

func export_state() -> Dictionary:
	var result = {"version":SAVE_VERSION,"rng":str(rng.state)}
	for key in ["players","deck","discard","active","phase","turn","encounter","last_card","reaction","reaction_step","chaos_used","boosts","enemy_mod","weapon_mod","battle_penalty","lucky","rusk","helper","help_requested","fought","choices","journal","winner"]:
		result[key] = get(key)
	return result.duplicate(true)

func restore_state(state: Dictionary) -> bool:
	var version = int(state.get("version",0))
	if version not in [1,SAVE_VERSION] or not state.has("rng"): return false
	var incoming = state.duplicate(true)
	if version == 1:
		incoming.help_requested = int(incoming.get("helper",-1)) >= 0
		# Old pending trades no longer charge the stash card.
		for q in incoming.get("choices",[]): q.erase("stash")
	var keys = ["players","deck","discard","active","phase","turn","encounter","last_card","reaction","reaction_step","chaos_used","boosts","enemy_mod","weapon_mod","battle_penalty","lucky","rusk","helper","help_requested","fought","choices","journal","winner"]
	for key in keys:
		if not incoming.has(key): return false
	for key in keys: set(key,incoming[key])
	rng.state = int(incoming.rng)
	return true
