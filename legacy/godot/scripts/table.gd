extends Control
const Rules = preload("res://scripts/game.gd")
const Card = preload("res://scripts/card_view.gd")
const SAVE_PATH = "user://table_save.json"
var game = Rules.new()
var screen: String = "menu"
var selected: String = ""
var private_owner: int = -1
var curtain: Control
var root: Control
var toast: String = ""
var chosen: Array = ["vityok","zina"]
var gallery_type: String = "all"

func _ready() -> void:
	get_window().title = "Орки с района · v0.2"
	var theme_base = Theme.new()
	theme_base.default_font_size = 16
	var normal = StyleBoxFlat.new()
	normal.bg_color = Color("34433c")
	normal.border_color = Color("677365")
	normal.set_border_width_all(1)
	normal.set_corner_radius_all(5)
	normal.content_margin_left = 16
	normal.content_margin_right = 16
	normal.content_margin_top = 10
	normal.content_margin_bottom = 10
	theme_base.set_stylebox("normal","Button",normal)
	var hover = normal.duplicate()
	hover.bg_color = Color("4c5f4b")
	hover.border_color = Color("ddbd7c")
	theme_base.set_stylebox("hover","Button",hover)
	var pressed = normal.duplicate()
	pressed.bg_color = Color("9b613b")
	theme_base.set_stylebox("pressed","Button",pressed)
	var disabled = normal.duplicate()
	disabled.bg_color = Color("26312d")
	theme_base.set_stylebox("disabled","Button",disabled)
	theme_base.set_color("font_color","Button",Color("eee0bc"))
	theme_base.set_color("font_color","Label",Color("e6dcbe"))
	theme_base.set_color("font_disabled_color","Button",Color("718072"))
	theme = theme_base
	RenderingServer.set_default_clear_color(Color("182321"))
	rebuild()
	if "--preview" in OS.get_cmdline_user_args():
		game.start(["vityok","zina","stalker"],42)
		game.explore()
		screen = "table"
		private_owner = game.owner()
		rebuild()
	if "--capture" in OS.get_cmdline_user_args():
		await get_tree().create_timer(3).timeout
		var capture_name = "table-preview" if "--preview" in OS.get_cmdline_user_args() else "menu-preview"
		get_viewport().get_texture().get_image().save_png("res://output/"+capture_name+".png")
		if "--keep-open" not in OS.get_cmdline_user_args(): get_tree().quit()

func _draw() -> void:
	draw_rect(Rect2(Vector2.ZERO,size),Color("182321"))
	for x in range(0,int(size.x),80):
		draw_line(Vector2(x,0),Vector2(x+260,size.y),Color(0.4,0.48,0.37,0.035),1)
	for y in range(0,int(size.y),80):
		draw_line(Vector2(0,y),Vector2(size.x,y),Color(0.4,0.48,0.37,0.025),1)

func clear_ui() -> void:
	if is_instance_valid(root):
		remove_child(root)
		root.queue_free()
	root = MarginContainer.new()
	root.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	for side in ["left","right","top","bottom"]: root.add_theme_constant_override("margin_"+side,22)
	add_child(root)

func label(text: String, font_size: int = 16, color: Color = Color("e6dcbe")) -> Label:
	var l = Label.new()
	l.text = text
	l.add_theme_font_size_override("font_size",font_size)
	l.add_theme_color_override("font_color",color)
	return l

func wrapped(text: String, font_size: int = 16) -> Label:
	var l = label(text,font_size)
	l.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	return l

func button(text: String, action: Callable, disabled: bool = false) -> Button:
	var b = Button.new()
	b.text = text
	b.disabled = disabled
	b.pressed.connect(action)
	b.mouse_default_cursor_shape = Control.CURSOR_POINTING_HAND
	return b

func panel(color: Color = Color("24322c")) -> PanelContainer:
	var p = PanelContainer.new()
	var s = StyleBoxFlat.new()
	s.bg_color = color
	s.border_color = Color("465349")
	s.set_border_width_all(1)
	s.set_corner_radius_all(10)
	for side in ["left","right","top","bottom"]: s.set("content_margin_"+side,16)
	p.add_theme_stylebox_override("panel",s)
	return p

func card_view(id: String, compact: bool = true, width: float = 162) -> Control:
	var c = Card.new()
	c.card = game.cards[id]
	c.compact = compact
	c.selected = selected == id
	c.custom_minimum_size = Vector2(width,242 if compact else 390)
	c.clicked = select_card
	c.draggable = screen == "table" and not game.players.is_empty() and game.choices.is_empty() and game.play_error(id) == ""
	return c

func select_card(id: String) -> void:
	selected = id
	toast = ""
	rebuild()

func rebuild() -> void:
	clear_ui()
	match screen:
		"menu": menu()
		"gallery": gallery()
		"rules": rules_screen()
		_: table_screen()
	queue_redraw()

func menu() -> void:
	var layout = HBoxContainer.new()
	layout.add_theme_constant_override("separation",36)
	root.add_child(layout)
	var art_panel = panel(Color("1c2924"))
	art_panel.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	layout.add_child(art_panel)
	var av = VBoxContainer.new()
	art_panel.add_child(av)
	av.add_child(label("БАЙКИ РЖАВОГО ДВОРА   /   КАРТОЧНАЯ RPG",15,Color("d99a60")))
	var pic = preload("res://scripts/card_art.gd").new()
	pic.category = "enemy"
	pic.texture = Card.art("assets/art/ork.png")
	if pic.texture == null: pic.texture = Card.art("assets/art/vityok.png")
	pic.size_flags_vertical = Control.SIZE_EXPAND_FILL
	av.add_child(pic)
	av.add_child(label("60 КАРТ    /    2–4 ИГРОКА    /    ОДИН СТОЛ",18))
	var info = VBoxContainer.new()
	info.custom_minimum_size.x = 590
	info.add_theme_constant_override("separation",16)
	layout.add_child(info)
	info.add_child(label("ОРКИ С РАЙОНА",76,Color("d8b87b")))
	info.add_child(label("С РАЙОНА",30,Color("d78b55")))
	info.add_child(wrapped("Мир рухнул. Соседи остались.\nСнаряжайся, побеждай чудовищ и подставляй корешей.\nПервый, кто возьмёт 10-й уровень в бою, станет легендой.",19))
	info.add_child(label("КТО САДИТСЯ ЗА СТОЛ?",17,Color("bdab80")))
	for id in ["vityok","stalker","goblin","zina"]:
		var box = CheckBox.new()
		box.text = game.cards[id].name
		box.button_pressed = chosen.has(id)
		box.tooltip_text = game.cards[id].text
		box.toggled.connect(func(on):
			if on and not chosen.has(id): chosen.append(id)
			elif not on: chosen.erase(id)
			call_deferred("rebuild"))
		info.add_child(box)
	info.add_child(wrapped("Локальная игра с передачей экрана. Перед просмотром чужой руки появляется закрывающая стол ширма. Ботов и сетевого режима в этой версии нет.",15))
	info.add_child(button("СЕСТЬ ЗА СТОЛ  →",func():
		game.start(chosen)
		selected = ""
		private_owner = -1
		screen = "table"
		save_game()
		rebuild(),chosen.size()<2 or chosen.size()>4))
	info.add_child(button("Продолжить сохранённую партию",load_game,not FileAccess.file_exists(SAVE_PATH)))
	var links = HBoxContainer.new()
	info.add_child(links)
	links.add_child(button("Вся колода · 60",func(): screen="gallery";rebuild()))
	links.add_child(button("Как играть",func(): screen="rules";rebuild()))
	if toast != "": info.add_child(wrapped(toast,14))

func header(col: VBoxContainer, subtitle: String) -> void:
	var row = HBoxContainer.new()
	col.add_child(row)
	row.add_child(label("ОРКИ С РАЙОНА",31,Color("d7ba81")))
	var sub = label(subtitle,16,Color("b2b79c"))
	sub.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	row.add_child(sub)
	row.add_child(button("Правила",func(): screen="rules";rebuild()))
	row.add_child(button("Колода",func(): screen="gallery";rebuild()))
	row.add_child(button("Меню",func(): screen="menu";rebuild()))

func table_screen() -> void:
	var col = VBoxContainer.new()
	col.add_theme_constant_override("separation",12)
	root.add_child(col)
	header(col,"      /    ХОД %02d" % game.turn)
	var body = HBoxContainer.new()
	body.size_flags_vertical = Control.SIZE_EXPAND_FILL
	body.add_theme_constant_override("separation",14)
	col.add_child(body)
	roster(body)
	center_table(body)
	inspector(body)
	if game.winner < 0:
		hand_area(col)
	else:
		var win = panel(Color("535132"))
		col.add_child(win)
		win.add_child(label("ЛЕГЕНДА РАЙОНА — %s! УРОВЕНЬ 10" % game.player_name(game.winner),30))
	if game.winner < 0 and private_owner != game.owner(): privacy_gate()

func roster(body: HBoxContainer) -> void:
	var wrapper = VBoxContainer.new()
	wrapper.custom_minimum_size.x = 245
	body.add_child(wrapper)
	var scroll = ScrollContainer.new()
	scroll.custom_minimum_size.x = 245
	scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	scroll.size_flags_vertical = Control.SIZE_EXPAND_FILL
	wrapper.add_child(scroll)
	var col = VBoxContainer.new()
	col.custom_minimum_size.x = 230
	col.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	col.add_theme_constant_override("separation",10)
	scroll.add_child(col)
	col.add_child(label("ЗА СТОЛОМ",14,Color("aaa985")))
	for i in range(game.players.size()):
		var p = game.players[i]
		var tile = panel(Color("3b4838") if i==game.active else Color("24312b"))
		col.add_child(tile)
		var v = VBoxContainer.new()
		tile.add_child(v)
		var name_label = wrapped(("▸ " if i==game.active else "")+game.player_name(i),19)
		v.add_child(name_label)
		var value = int(p.level)
		v.add_child(label("УРОВЕНЬ %d     РУКА %d / 5" % [value,p.hand.size()],14,Color("cfb77d")))
		var progress = ProgressBar.new()
		progress.min_value = 0
		progress.max_value = 10
		progress.value = value
		progress.show_percentage = false
		progress.custom_minimum_size.y = 5
		v.add_child(progress)
		for slot in ["weapon","armor"]:
			var id = p[slot]
			if id != "":
				var b = button(("⚔ " if slot=="weapon" else "◈ ")+game.cards[id].name,func(): select_card(id))
				b.add_theme_font_size_override("font_size",12)
				v.add_child(b)
		if p.silenced: v.add_child(label("Способность заблокирована",12,Color("cd876a")))
		if p.water < 0: v.add_child(label("Нет горячей воды: −1 силы",12,Color("cd876a")))
		if p.skip_now: v.add_child(label("Энергетик: без действий",12,Color("cd876a")))
		var hero_button = button("Способность / портрет",func(): select_card(p.hero))
		hero_button.add_theme_font_size_override("font_size",11)
		v.add_child(hero_button)
	wrapper.add_child(button("Закрыть руку ширмой",func(): private_owner=-1;rebuild()))

func center_table(body: HBoxContainer) -> void:
	var surface = panel(Color("253c32"))
	surface.set_script(preload("res://scripts/drop_zone.gd"))
	surface.receive_card = func(id): act(game.play(id))
	surface.tooltip_text = "Перетащите сюда карту с руки, чтобы разыграть или экипировать её."
	surface.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	body.add_child(surface)
	var v = VBoxContainer.new()
	v.add_theme_constant_override("separation",12)
	surface.add_child(v)
	var phase_text = {"explore":"01  /  ПОДГОТОВКА И ИССЛЕДОВАНИЕ","combat":"02  /  БОЙ И ВМЕШАТЕЛЬСТВО","actions":"03  /  СНАРЯЖЕНИЕ И ДЕЙСТВИЯ"}.get(game.phase,"")
	if game.final_response(): phase_text = "02  /  ФИНАЛЬНЫЙ ОТВЕТ ГЕРОЯ"
	v.add_child(label(phase_text,16,Color("c2bd94")))
	v.add_child(label("Перетащи карту с руки на стол, чтобы разыграть её",12,Color("a2ac91")))
	if not game.choices.is_empty():
		decision_area(v)
	else:
		var middle = HBoxContainer.new()
		middle.add_theme_constant_override("separation",16)
		middle.size_flags_vertical = Control.SIZE_EXPAND_FILL
		v.add_child(middle)
		var pilecol = VBoxContainer.new()
		pilecol.custom_minimum_size.x = 132
		middle.add_child(pilecol)
		var back = panel(Color("1a2a28"))
		back.custom_minimum_size = Vector2(132,197)
		pilecol.add_child(back)
		var bv = VBoxContainer.new()
		back.add_child(bv)
		bv.add_child(label("О Р К И",12,Color("cbaa70")))
		var mark = label("✦",72,Color("b98b51"))
		mark.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
		mark.size_flags_vertical = Control.SIZE_EXPAND_FILL
		var back_art = TextureRect.new()
		back_art.texture = Card.art("assets/art/card_back.png")
		back_art.expand_mode = TextureRect.EXPAND_IGNORE_SIZE
		back_art.stretch_mode = TextureRect.STRETCH_KEEP_ASPECT_COVERED
		back_art.custom_minimum_size.y = 140
		back_art.size_flags_vertical = Control.SIZE_EXPAND_FILL
		if back_art.texture != null:
			bv.add_child(back_art)
			mark.free()
		else:
			bv.add_child(mark)
			back_art.free()
		bv.add_child(label("ОБЩАЯ КОЛОДА",11,Color("b9aa83")))
		pilecol.add_child(label("Осталось: %d" % game.deck.size(),14))
		pilecol.add_child(button("Сброс · %d" % game.discard.size(),show_discard,game.discard.is_empty()))
		var shown = game.encounter if game.encounter != "" else game.last_card
		if shown != "":
			var face = card_view(shown,false,215)
			face.custom_minimum_size.y = 350
			middle.add_child(face)
		var status = VBoxContainer.new()
		status.custom_minimum_size.x = 200
		status.size_flags_horizontal = Control.SIZE_EXPAND_FILL
		status.add_theme_constant_override("separation",9)
		middle.add_child(status)
		if game.phase == "combat":
			status.add_child(label("ГЕРОЙ %d  :  %d ВРАГ" % [game.hero_strength(),game.enemy_strength()],25,Color("e5ba77")))
			status.add_child(wrapped(game.strength_text(),12))
			status.add_child(wrapped(("Последнее слово: " if game.final_response() else "Вмешательство: ")+game.player_name(game.reaction),18))
			status.add_child(wrapped(game.reaction_text(),13))
			status.add_child(wrapped("Хаос: %s. Расходники героя доступны и в финальном ответе." % ("уже сыгран" if game.chaos_used.has(game.owner()) else "ещё можно сыграть одну карту"),12))
			if game.helper >= 0: status.add_child(wrapped("Помощник: %s. Первая добыча — ему." % game.player_name(game.helper),13))
			if game.reaction_step == 0 and not game.help_requested:
				var help_button = button("ПРЕДЛОЖИТЬ ПОМОЩЬ",func(): act(game.request_help()),game.help_error() != "")
				help_button.tooltip_text = game.help_error() if game.help_error() != "" else "+2 силы за сброс карты помощником. Первая добыча — помощнику."
				status.add_child(help_button)
			if game.cards[game.encounter].effect == "dragon": status.add_child(wrapped("Дракон: нужно не менее 10 силы героя.",12))
			status.add_child(button("ПОДТВЕРДИТЬ БОЙ" if game.final_response() else "ПЕРЕДАТЬ ОЧЕРЕДЬ →",func(): act(game.pass_reaction())))
		elif game.phase == "explore":
			status.add_child(label("СОБЕРИСЬ В ДОРОГУ",25,Color("beae7e")))
			status.add_child(wrapped("Сначала надень снаряжение, предложи обмен или замени одну карту. Затем открой карту исследования.",18))
			status.add_child(wrapped("Замена и лимит обменов общие на весь ход. Враги, события и расходники с руки — после исследования.",14))
			if game.players[game.active].skip_now: status.add_child(wrapped("Энергетик: подготовка пропущена, но исследование и бой доступны.",14))
			status.add_child(button("ОТКРЫТЬ КАРТУ",func(): act(game.explore())))
		else:
			status.add_child(label("ПЕРЕДЫШКА",24,Color("beae7e")))
			status.add_child(wrapped("Снарядись, разыграй карты или предложи обмен. Лимит руки — 5 карт.",18))
			status.add_child(button("ЗАВЕРШИТЬ ХОД →",func(): act(game.end_turn()),game.winner>=0))
	var logbox = panel(Color("1d2e28"))
	v.add_child(logbox)
	var log_style = logbox.get_theme_stylebox("panel").duplicate()
	log_style.content_margin_top = 8
	log_style.content_margin_bottom = 8
	logbox.add_theme_stylebox_override("panel",log_style)
	var log_scroll = ScrollContainer.new()
	log_scroll.custom_minimum_size.y = 68
	log_scroll.horizontal_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	logbox.add_child(log_scroll)
	log_scroll.get_v_scroll_bar().changed.connect(func(): log_scroll.scroll_vertical = int(log_scroll.get_v_scroll_bar().max_value))
	var logs = VBoxContainer.new()
	logs.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	log_scroll.add_child(logs)
	logs.add_child(label("ХРОНИКА ДВОРА",11,Color("a2a183")))
	var start = maxi(0,game.journal.size()-3)
	for j in range(start,game.journal.size()): logs.add_child(wrapped(game.journal[j],12))

func inspector(body: HBoxContainer) -> void:
	var v = VBoxContainer.new()
	v.custom_minimum_size.x = 282
	v.add_theme_constant_override("separation",8)
	body.add_child(v)
	v.add_child(label("КАРТА КРУПНЫМ ПЛАНОМ",12,Color("aaa985")))
	var id = selected
	if id == "": id = game.encounter if game.encounter != "" else game.players[game.owner()].hero
	if game.cards.has(id):
		v.add_child(card_view(id,false,282))
		if game.cards[id].type == "enemy":
			v.add_child(wrapped("При поражении: "+{"card":"потеря случайной карты.","level":"−1 уровень.","armor":"сброс брони; без брони — потеря карты.","weapon":"сброс оружия; без оружия — потеря карты.","two_cards":"потеря двух карт.","level_card":"−1 уровень и потеря карты."}.get(game.cards[id].penalty,""),13))
	if selected in game.players[game.owner()].hand:
		var reason = game.play_error(selected)
		v.add_child(wrapped(reason if reason != "" else "Можно сыграть сейчас — кнопкой или перетаскиванием на стол.",13))
	if toast != "": v.add_child(wrapped(toast,14))

func hand_area(col: VBoxContainer) -> void:
	var who = game.owner()
	var row = HBoxContainer.new()
	col.add_child(row)
	var title = label("РУКА: %s   /   %d КАРТ" % [game.player_name(who),game.players[who].hand.size()],17,Color("d7bd83"))
	title.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	row.add_child(title)
	var blocked = not game.choices.is_empty()
	var mine = selected in game.players[who].hand
	var play_reason = game.play_error(selected)
	var play_button = button("Сыграть / экипировать",func(): act(game.play(selected)),play_reason != "")
	play_button.tooltip_text = play_reason if play_reason != "" else "Разыграть выбранную карту"
	row.add_child(play_button)
	row.add_child(button("Заменить",func(): act(game.recycle(selected)),not mine or blocked or game.phase not in ["explore","actions"] or game.players[who].recycled or game.players[who].skip_now))
	row.add_child(button("Обмен",trade_selected,not mine or blocked or game.phase not in ["explore","actions"] or game.players[who].skip_now or game.players[who].traded >= (2 if game.ability(who,"merchant") else 1)))
	row.add_child(button("Сбросить лишнюю",func(): act(game.discard_excess(selected)),not mine or blocked or game.phase!="actions" or game.players[who].hand.size()<=5))
	var scroll = ScrollContainer.new()
	scroll.custom_minimum_size.y = 258
	scroll.vertical_scroll_mode = ScrollContainer.SCROLL_MODE_DISABLED
	col.add_child(scroll)
	var hand = HBoxContainer.new()
	hand.add_theme_constant_override("separation",12)
	scroll.add_child(hand)
	for id in game.players[who].hand: hand.add_child(card_view(id))
	if game.players[who].hand.is_empty(): hand.add_child(label("Пустые карманы. Район бывает щедрым — но не сегодня.",17,Color("9fa48d")))

func decision_area(v: VBoxContainer) -> void:
	var q = game.choices[0]
	v.add_child(wrapped(q.title,23))
	if q.kind == "reveal":
		var names: Array = []
		for id in game.players[q.subject].hand: names.append(game.cards[id].name)
		v.add_child(wrapped("\n".join(names),18))
	var scroll = ScrollContainer.new()
	scroll.size_flags_vertical = Control.SIZE_EXPAND_FILL
	v.add_child(scroll)
	var options = VBoxContainer.new()
	options.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	scroll.add_child(options)
	for option in q.options:
		var text = str(option)
		if game.cards.has(option):
			text = game.cards[option].name+" — "+game.cards[option].text
		elif q.kind in ["silence","ambush","trade_target","help_target"] and option != "cancel": text = game.player_name(int(option))
		else: text = {"penalty":"Принять −1 силы","protect":"Сбросить плащ, сохранить руку","card":"Потерять случайную карту","random":"Обычная случайная потеря","decline":"Отказаться","cancel":"Не предлагать помощь","ok":"Готово / продолжить"}.get(option,text)
		var b = button(text,func(): act(game.choose(option)))
		b.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
		options.add_child(b)

func privacy_gate() -> void:
	selected = ""
	curtain = ColorRect.new()
	curtain.color = Color("15231f")
	curtain.z_index = 100
	curtain.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	root.add_child(curtain)
	var center = CenterContainer.new()
	center.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	curtain.add_child(center)
	var v = VBoxContainer.new()
	v.custom_minimum_size.x = 640
	v.add_theme_constant_override("separation",24)
	center.add_child(v)
	v.add_child(label("✦   ПЕРЕДАЙ ЭКРАН",22,Color("bfa77a")))
	v.add_child(label(game.player_name(game.owner()),44,Color("dfc18a")))
	v.add_child(wrapped("Остальные игроки отворачиваются.\nСледующий игрок открывает свою руку, когда будет готов.",22))
	v.add_child(button("Я готов — открыть мою руку",func(): private_owner=game.owner();rebuild()))

func act(ok: bool) -> void:
	toast = "" if ok else game.error
	if ok:
		selected = ""
		save_game()
	rebuild()

func trade_selected() -> void:
	if selected != "": act(game.trade(selected))

func save_game() -> void:
	var file = FileAccess.open(SAVE_PATH,FileAccess.WRITE)
	if file: file.store_string(JSON.stringify(game.export_state()))
	else: toast = "Не удалось записать сохранение."

func load_game() -> void:
	var state = JSON.parse_string(FileAccess.get_file_as_string(SAVE_PATH))
	if state is Dictionary and game.restore_state(state):
		screen = "table"
		private_owner = -1
		selected = ""
	else: toast = "Сохранение не удалось прочитать."
	rebuild()

func gallery() -> void:
	var col = VBoxContainer.new()
	root.add_child(col)
	var row = HBoxContainer.new()
	col.add_child(row)
	row.add_child(label("КОЛОДА РАЙОНА  /  60 КАРТ",30,Color("d4b67e")))
	row.add_child(button("← Назад",func(): screen="table" if not game.players.is_empty() else "menu";rebuild()))
	var filters = HBoxContainer.new()
	col.add_child(filters)
	for type in ["all","character","enemy","equipment","consumable","event","chaos"]:
		var text = "Все" if type=="all" else Card.TYPES[type]
		filters.add_child(button(text,func(): gallery_type=type;rebuild()))
	var scroll = ScrollContainer.new()
	scroll.size_flags_vertical = Control.SIZE_EXPAND_FILL
	col.add_child(scroll)
	var grid = GridContainer.new()
	grid.columns = 6
	grid.add_theme_constant_override("h_separation",15)
	grid.add_theme_constant_override("v_separation",18)
	scroll.add_child(grid)
	for id in game.cards:
		if gallery_type == "all" or game.cards[id].type == gallery_type:
			var c = card_view(id,false,240)
			c.clicked = func(card_id): show_card_popup(card_id)
			grid.add_child(c)

func show_card_popup(id: String) -> void:
	var popup = Window.new()
	popup.title = game.cards[id].name
	popup.size = Vector2i(400,680)
	popup.transient = true
	popup.exclusive = true
	add_child(popup)
	var c = card_view(id,false,360)
	c.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	popup.add_child(c)
	popup.close_requested.connect(popup.queue_free)
	popup.popup_centered()

func show_discard() -> void:
	var popup = AcceptDialog.new()
	popup.title = "Открытый сброс (верхняя карта — последняя)"
	var lines: Array = []
	for id in game.discard: lines.append(game.cards[id].name)
	popup.dialog_text = "\n".join(lines)
	popup.min_size = Vector2i(430,300)
	add_child(popup)
	popup.confirmed.connect(popup.queue_free)
	popup.canceled.connect(popup.queue_free)
	popup.popup_centered()

func rules_screen() -> void:
	var col = VBoxContainer.new()
	root.add_child(col)
	col.add_child(button("← К столу" if not game.players.is_empty() else "← В меню",func(): screen="table" if not game.players.is_empty() else "menu";rebuild()))
	var scroll = ScrollContainer.new()
	scroll.size_flags_vertical = Control.SIZE_EXPAND_FILL
	col.add_child(scroll)
	var text = RichTextLabel.new()
	text.bbcode_enabled = true
	text.fit_content = true
	text.custom_minimum_size.x = 1400
	text.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	text.text = FileAccess.get_file_as_string("res://rules/quickstart.txt")
	scroll.add_child(text)
