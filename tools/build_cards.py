"""Читаемые SVG 63×88 мм и печатная колода; размер набора берётся из данных."""
from pathlib import Path
import html, json, math, textwrap
ROOT=Path(__file__).resolve().parents[1]
cards=json.loads((ROOT/'data/cards.json').read_text())
COLORS={'character':'#648897','enemy':'#a95640','equipment':'#af944b','consumable':'#879951','event':'#8c7294','chaos':'#da8245','perk':'#718b5d','district':'#638291','shop':'#b98b47'}
NAMES={'character':'ПЕРСОНАЖ','enemy':'ВРАГ','equipment':'СНАРЯЖЕНИЕ','consumable':'РАСХОДНИК','event':'СОБЫТИЕ','chaos':'ПОДСТАВА','perk':'ПЕРК','district':'РАЙОН','shop':'МАГАЗИН'}
# Совпадает с Penalty::description в crates/orcs-core/src/cards.rs.
PENALTIES={'card':'потеря случайной карты.','level':'−1 уровень.','armor':'сброс брони; без брони — потеря карты.','weapon':'сброс оружия; без оружия — потеря карты.','two_cards':'потеря двух карт.','level_card':'−1 уровень и потеря карты.','level_coins':'−1 уровень и до 2 крышаков.','coins':'потеря до 2 крышаков.'}
def cards_word(n):
    return f"{n} "+('КАРТ' if n%10 in (0,5,6,7,8,9) or 11<=n%100<=14 else 'КАРТА' if n%10==1 else 'КАРТЫ')
def esc(s):return html.escape(str(s),quote=True)
def wrap(text,width):return textwrap.wrap(text,width=width,break_long_words=False)
def lines(text,x,y,width,size,line,limit=None):
    parts=wrap(text,width)
    if limit is not None: assert len(parts)<=limit,(text,len(parts),limit)
    return ''.join(f'<text x="{x}" y="{y+i*line}" font-size="{size}">{esc(t)}</text>' for i,t in enumerate(parts))
def pile(c):
    if c['type'] in ('enemy','event') and c['id']!='boss':return 'РАЗБОРКИ'
    if c['type'] in ('equipment','consumable','chaos'):return 'БАРАХЛО'
    return 'КОМПОНЕНТ СТОЛА'
def svg(c,prefix):
    color=COLORS[c['type']]
    stats='КОМПОНЕНТ СТОЛА'
    if c['type']=='enemy':stats=f"СИЛА {c['strength']} · +{c['level_reward']} УР. · {cards_word(c['loot_cards'])} · {c.get('coins',0)} КР."
    if c['type']=='equipment':stats=f"+{c['bonus']} СИЛЫ · "+('ОРУЖИЕ' if c['slot']=='weapon' else 'БРОНЯ')+f" · {c['price']} КР."
    if c['type'] in ('consumable','chaos'):stats=f"ЦЕНА {c['price']} КР. · ПРОДАЖА 1 КР."
    if c['type']=='perk':stats=f"СТУПЕНЬ {c['tier']} · ПОСТОЯННЫЙ ПЕРК"
    aspect='xMidYMin slice' if c['type'] in ('character','enemy') else ('xMidYMid meet' if c['type'] in ('equipment','consumable','chaos','perk') else 'xMidYMid slice')
    body=c['text']
    if c['type']=='enemy':body+=' При поражении: '+PENALTIES[c['penalty']]
    # Длинное правило получает меньший кегль; за пределы текстового блока не выходит.
    for size,width,leading in [(25,38,29),(23,42,27),(21,46,25),(19,51,23)]:
        if len(wrap(body,width))*leading<=185:break
    assert len(wrap(body,width))*leading<=185,c['id']
    flavor=lines(c.get('flavor',''),38,791,52,19,22,limit=2)
    return f'''<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" width="63mm" height="88mm" viewBox="0 0 630 880">
<defs><clipPath id="art-{c['id']}"><rect x="27" y="151" width="576" height="355" rx="5"/></clipPath></defs>
<rect x="4" y="4" width="622" height="872" rx="19" fill="#ded0aa" stroke="{color}" stroke-width="8"/>
<g font-family="DejaVu Sans,Arial,sans-serif" fill="#26332f">
<text x="38" y="39" font-size="17" fill="{color}">{NAMES[c['type']]} · {pile(c)}</text>
<g font-weight="bold">{lines(c['name'],38,83,29,30,35,limit=2)}</g>
<image x="27" y="151" width="576" height="355" preserveAspectRatio="{aspect}" clip-path="url(#art-{c['id']})" xlink:href="{prefix}{c['image']}"/>
<path d="M28 522H602" stroke="{color}" stroke-width="3"/>
<text x="38" y="551" font-size="21" font-weight="bold">{esc(stats)}</text>
{lines(body,38,590,width,size,leading)}
<g fill="#746045" font-style="italic">{flavor}</g>
<text x="38" y="851" font-size="14" fill="#69644e">{esc(c['id'].upper())} · ОРКИ С РАЙОНА</text>
</g></svg>'''

def back(c):
    label=pile(c); color=COLORS[c['type']] if label=='КОМПОНЕНТ СТОЛА' else ('#a95640' if label=='РАЗБОРКИ' else '#af944b')
    return f'''<svg xmlns="http://www.w3.org/2000/svg" width="63mm" height="88mm" viewBox="0 0 630 880"><rect width="630" height="880" rx="18" fill="{color}"/><image href="../assets/art/card_back.png" x="12" y="12" width="606" height="856" preserveAspectRatio="xMidYMid slice"/><rect x="30" y="710" width="570" height="105" rx="12" fill="#182321"/><text x="315" y="775" text-anchor="middle" font-family="Arial,sans-serif" font-size="34" fill="#ded0aa">{label}</text></svg>'''

(ROOT/'output/cards').mkdir(parents=True,exist_ok=True)
for c in cards:(ROOT/f"output/cards/{c['id']}.svg").write_text(svg(c,'../../'))
count=len(cards);pages=math.ceil(count/9)
html_text=f'''<!doctype html><html lang="ru"><meta charset="utf-8"><title>Орки с района — {count} карт</title>
<style>@page{{size:A4;margin:10mm}}*{{box-sizing:border-box}}body{{background:#182321;color:#e6dcbe;font:16px Arial;margin:24px}}header{{max-width:900px;margin:0 auto 24px}}.page{{width:189mm;min-height:264mm;display:grid;grid-template-columns:repeat(3,63mm);grid-auto-rows:88mm;background:white;margin:20px auto;page-break-after:always}}.page svg{{display:block;outline:.15mm dashed #999}}button{{padding:12px 20px;cursor:pointer}}.back{{display:none}}.showbacks .front{{display:none}}.showbacks .back{{display:grid}}@media print{{body{{margin:0;background:white}}.page{{margin:0;break-inside:avoid}}header{{display:none}}*{{print-color-adjust:exact;-webkit-print-color-adjust:exact}}}}</style>
<header><h1>Орки с района</h1><p>{count} карт · 63 × 88 мм · {pages} листов A4. Масштаб 100%, поля 10 мм, без колонтитулов, фоновая графика включена. Сохраните структуру проекта: рисунки из assets/art.</p><p>Разборки и Барахло различаются надписями и цветом рубашек. Перки, районы, герои, магазин и Хозяин района лежат отдельно.</p><button onclick="document.body.classList.remove('showbacks');window.print()">Печатать лица</button> <button onclick="document.body.classList.toggle('showbacks')">Лица / рубашки</button><p>Рубашки печатать отдельно: автоматическая двусторонняя раскладка не предусмотрена. Для непрозрачных протекторов достаточно лиц. Сначала проверьте одну карту линейкой.</p></header>'''
for i in range(0,count,9):html_text+='<section class="page front">'+''.join(svg(c,'../') for c in cards[i:i+9])+'</section>'
for i in range(0,count,9):html_text+='<section class="page back">'+''.join(back(c) for c in cards[i:i+9])+'</section>'
(ROOT/'output/deck.html').write_text(html_text+'</html>')
print(f'{count} SVG faces + printable output/deck.html ({pages} front pages) written')
