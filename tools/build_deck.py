"""Base definitions + content.py → canonical catalogue and art manifest."""
import json
from pathlib import Path
ROOT = Path(__file__).resolve().parents[1]
cards=[]
def add(id,name,type,text,subject,**kw):
    cards.append(dict(id=id,name=name,type=type,text=text,image=kw.pop("image",f"assets/art/{id}.png"),subject=subject,**kw))
heroes=[
('vityok','Витёк «Сварщик»','Техническое снаряжение даёт +1 силы (один раз за бой).','stocky mechanic with lifted welding mask and rune welding torch','mechanic'),
('stalker','Лысый Сталкер','+1 силы против мутантов.','bald scarred wanderer in faded blue coat holding a strange compass','mutant_hunter'),
('goblin','Гоблин Скупыч','За свой ход может провести два обмена вместо одного. Только по согласию.','shrewd goblin merchant with a sack of strange junk at a flea market','merchant'),
('zina','Бабка Зина','При потере карты можно выбрать другую карту для сброса. Один раз за ход.','stern elderly witch in headscarf with keys and glowing broom at an apartment entrance','witch')]
for id,n,t,s,e in heroes:add(id,n,'character',t,s,effect=e)
enemies=[
('zombie','Алкаш-Зомби',2,'undead','Особых свойств нет.','confused shabby zombie with glowing green bottles in a bag',1,1,'none','card'),
('ork','Орк с района',4,'orc','+1 силы, если у героя нет брони.','bulky orc in faded tracksuit with rebar club outside concrete flats',1,1,'unarmored','card'),
('kikimora','Кикимора ЖКХ',5,'spirit','Перед боем сбрось расходник или получи −1 силы.','swamp spirit made of leaking pipes holding blank utility bills',1,1,'pipe','card'),
('boyar','Боярин-Мутант',7,'mutant','+1 силы за броню героя.','pompous mutant noble with fur collar and extra arm in ruined cottage',1,2,'armor','card'),
('dragon','Дракон Гаражного Кооператива',9,'beast','Для победы нужно не менее 10 силы.','dragon assembled from scales rusty car doors and exhaust pipes atop garages',2,2,'dragon','level'),
('rat','Крыса размером с ЖЭК',2,'mutant','Особых свойств нет.','oversized mutant rat dragging a rusty radiator',1,1,'none','card'),
('skeleton','Скелет-вахтёр',3,'undead','Особых свойств нет.','skeletal gatekeeper with peaked cap and oversized ring of keys',1,1,'none','card'),
('mushroom','Грибной участковый',3,'mutant','Особых свойств нет.','stern mushroom humanoid with battered coat in concrete courtyard',1,1,'none','card'),
('dog','Трёхголовый Тузик',4,'mutant','Особых свойств нет.','three headed scruffy guard dog beside a rusted kennel',1,1,'none','level'),
('knight','Рыцарь просроченного ТО',5,'human','+1 силы против технического оружия.','knight in car panel armor with wheel hub shield',1,1,'technical','card'),
('druid','Дачный друид',4,'human','Особых свойств нет.','wild garden druid with enormous zucchini staff and weeds in beard',1,1,'none','card'),
('conductor','Кондуктор Последнего Маршрута',6,'undead','Особых свойств нет.','spectral tram conductor with punched blank tickets and lantern',1,2,'none','level'),
('moth','Моль из ковра',3,'beast','Особых свойств нет.','giant dusty moth emerging from patterned hanging carpet',1,1,'none','card'),
('golem','Бетонный дворник',6,'construct','Особых свойств нет.','cracked concrete golem sweeping rubble with huge birch broom',1,2,'none','armor'),
('witch','Ведьма очереди',5,'human','Особых свойств нет.','irate witch guarding market queue with bags and glowing walking stick',1,1,'none','two_cards'),
('samovar','Самовар-пожиратель',7,'construct','Особых свойств нет.','monstrous antique samovar with metal jaws and steam tendrils',1,2,'none','weapon'),
('leshy','Леший с промзоны',8,'mutant','Особых свойств нет.','forest giant with branches and factory pipes growing from shoulders',2,2,'none','level'),
('elevator_boss','Хозяин тринадцатого этажа',10,'spirit','Особых свойств нет.','sinister tall spirit in coat emerging from broken apartment elevator',2,2,'none','level_card')]
for id,n,p,tag,t,s,l,loot,e,bad in enemies:add(id,n,'enemy',t,s,strength=p,tag=tag,level_reward=l,loot_cards=loot,effect=e,penalty=bad)
equipment=[
('rebar','Арматурный аргумент',2,'weapon','Оружие. Без особых свойств.','heavy bent rebar with a taped grip','none',False),
('slipper','Священная шлёпанца',1,'weapon','Оружие. +2 дополнительных силы против нежити.','worn rubber slipper with sacred magical sparks','undead',False),
('club','Последнее предупреждение',3,'weapon','Оружие. После победы сбрось карту с руки.','huge wooden club studded with rusty nails','victory_discard',False),
('helmet','Каска прораба',1,'armor','Броня. Технический предмет.','dented yellow construction helmet with rune welds','none',True),
('chain','Кольчуга из цепей',2,'armor','Броня. Без особых свойств.','chain mail assembled from bicycle chains','none',True),
('curtain','Плащ из занавески',1,'armor','Броня. При случайной потере карты можно сбросить этот плащ вместо неё.','worn floral curtain fashioned into heroic cloak','protect',False),
('wrench','Ключ на тридцать судеб',2,'weapon','Оружие. Технический предмет.','enormous rusty wrench with glowing rune on jaw','none',True),
('antenna','Антенна громовержца',2,'weapon','Оружие. Технический предмет.','television antenna staff conducting small magical lightning','none',True),
('lid','Щит из крышки люка',2,'armor','Броня. Технический предмет.','round manhole cover shield with leather straps','none',True),
('coat','Телогрейка странника',1,'armor','Броня. Без особых свойств.','patched padded blue winter jacket','none',False),
('shovel','Лопата земных забот',2,'weapon','Оружие. Без особых свойств.','battle worn shovel with strange roots on handle','none',False),
('vest','Жилет из дорожных знаков',3,'armor','Броня. Технический предмет.','improvised armor of rusty blank road signs','none',True)]
for id,n,b,slot,t,s,e,tech in equipment:add(id,n,'equipment',t,s,bonus=b,slot=slot,effect=e,technical=tech)
cons=[
('brine','Святая рассольная настойка','В свой бой: +2 силы до конца боя.','glowing green pickle brine bottle with cork','boost'),
('cutlet','Бабушкина котлета','После исследования: возьми 2 Барахла, затем сбрось одну карту с руки по выбору.','homemade cutlet wrapped in parchment emitting warm magic','cycle'),
('tape','Изолента судьбы','В свой бой при наличии оружия: +1 силы до конца боя.','blue insulating tape roll glowing with fate threads','tape'),
('luck','Талон на удачу','В свой бой: отмени всё последствие поражения. Сыграй до сравнения сил.','old blank coupon with lucky clover and magical gold dust','luck'),
('energy','Последний энергетик','После исследования: +1 уровень (не выше 9). В следующий ход пропусти подготовку и действия.','battered unbranded energy drink can with electric sparks','energy'),
('rusk','Сухарь бессмертия','В свой бой: предотврати потерю одного уровня при поражении.','rock hard bread rusk with faint protective halo','rusk')]
for id,n,t,s,e in cons:add(id,n,'consumable',t,s,effect=e)
events=[
('water','Отключили горячую воду','Каждый сбрасывает карту по выбору или получает −1 силы до начала своего хода.','cold ruined bathroom with frozen pipe and tiny water spirit','water'),
('portal','Портал в гараже','Открой 2 Барахла: одно возьми в руку, другое сбрось.','swirling magical portal inside cluttered garage','portal'),
('meeting','Собрание жильцов','Выбранный игрок теряет способность до конца своего следующего боя.','elderly witch leading tense gathering of orc skeleton and scrap knight in stairwell','silence'),
('lift','Проклятый лифт','Потеряй случайную карту с руки; если рука пуста — −1 силы в следующем бою.','haunted apartment elevator with chains and purple mist','lift'),
('raid','Налёт орков-гопников','Открой ещё карту и выполни её как при исследовании.','orc gang in patched tracksuits storming concrete courtyard','raid'),
('saw','Бабка всё видела','Покажи руку соседу слева, затем закрой её.','suspicious old witch peeking from behind lace curtain','reveal'),
('market','Барахолка чудес','Возьми 1 Барахло.','fantasy flea market with strange glowing household junk','draw'),
('rain','Кислотный четверг','Сбрось броню; если брони нет — ничего не происходит.','acid green rain falling on rusty umbrella and armor','rust'),
('garden','Урожай кабачков','Возьми 2 Барахла.','enormous magical zucchini overflowing an abandoned garden','harvest'),
('tax','Налог на колдовство','Потеряй один уровень (минимум 1).','goblin tax collector weighing glowing magic in antique scales','tax')]
for id,n,t,s,e in events:add(id,n,'event',t,s,effect=e)
chaos=[
('bullet','Ржавый патрон','Во время боя: +2 силы врагу.','rusty cartridge with malicious orange sparks','enemy_up'),
('unfair','Не по понятиям','В чужой бой: −1 к бонусу оружия героя, если он больше нуля.','orc pointing accusingly at an improvised weapon','weapon_down'),
('ambush','Подстава у подъезда','После исследования: выбранный соперник теряет случайную карту с руки.','tripwire of shoelaces across ruined apartment entrance','ambush'),
('friend','Внезапный кореш','В чужой бой: +2 силы герою; при победе первая карта добычи твоя.','friendly burly scavenger emerging with helping hand from garage','friend'),
('stash','Да это мой тайник!','После исследования: открой 2 Барахла. Одно возьми, другое положи вниз колоды.','two scavengers arguing over hidden rusty treasure chest','stash'),
('magic','Магия из ларька','Во время боя: отмени последний положительный временный бонус силы героя.','dubious market stall dispensing bottled fake magic','cancel'),
('battery','Севший аккумулятор','В чужой бой: −2 силы герою с технической экипировкой.','dead car battery with dim magical sparks','battery'),
('rumor','Слух о проверке','Во время боя: −2 силы врагу.','frightened monsters fleeing a shadow of inspector','enemy_down'),
('salt','Соль через плечо','В свой бой: +1 силы до конца боя.','hand tossing coarse salt into a ghostly shadow','hero_up'),
('noise','Сосед с перфоратором','Во время боя: +3 силы врагу.','furious neighbor drilling concrete wall with rune powered drill','noise')]
for id,n,t,s,e in chaos:add(id,n,'chaos',t,s,effect=e)
from content import enrich
enrich(cards, add)
assert len(cards)==90
(ROOT/'data/cards.json').write_text(json.dumps(cards,ensure_ascii=False,indent=2)+'\n')
style='Portrait 3:4 single card illustration. Style Tales of the Rusty Yard: visibly HAND DRAWN adventure graphic novel, expressive variable weight black ink contours, cross hatching, dry brush, matte gouache and faded watercolor on grainy aged paper. Angular expressive faces, moderately grotesque believable anatomy. Rust orange, swamp green, faded blue, concrete gray, mustard. Humorous post-apocalyptic Slavic urban fantasy. Strong readable central subject. NO photorealism, realistic rendering, 3D, glossy surfaces, chibi, huge cartoon eyes. No text, letters, digits, logos, frame, border or UI. Original designs.'
generated_path=ROOT/'prompts/generated.json'
generated={p['id']:p for p in json.loads(generated_path.read_text())} if generated_path.exists() else {}
manifest=[]
for c in cards:
    art_id=c.get('art_reuse',c['id'])
    exact=generated.get(art_id)
    prompt=exact['prompt'] if exact else ((ROOT/'prompts/vityok-actual.txt').read_text().strip() if art_id=='vityok' else f"Use case: stylized-concept. {style} Subject: {next(x['subject'] for x in cards if x['id']==art_id)}.")
    manifest.append(dict(id=c['id'],prompt=prompt,image=c['image'],art_reuse=c.get('art_reuse'),production='built-in image_gen'))
(ROOT/'prompts/manifest.json').write_text(json.dumps(manifest,ensure_ascii=False,indent=2)+'\n')
print(f'{len(cards)} cards written')
