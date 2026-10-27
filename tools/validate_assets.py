"""Validate catalogue, intentional art reuse, printable faces and local documentation links."""
import hashlib, json, re, struct, xml.etree.ElementTree as ET
from collections import Counter
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
cards=json.loads((ROOT/'data/cards.json').read_text())
by_id={c['id']:c for c in cards}
assert len(cards)==len(by_id)==90
assert Counter(c['type'] for c in cards)==dict(character=8,enemy=19,equipment=12,consumable=6,event=10,chaos=10,perk=18,district=6,shop=1)
assert Counter(c['tier'] for c in cards if c['type']=='perk')=={3:6,5:6,7:6}
seen={}
for c in cards:
    assert c['text'] and c['flavor'],c['id']
    if c['type'] in ('equipment','consumable','chaos'): assert 2<=c['price']<=6,c['id']
    p=ROOT/c['image'];b=p.read_bytes()
    assert b[:8]==b'\x89PNG\r\n\x1a\n',p
    w,h=struct.unpack('>II',b[16:24]);assert w>=512 and h>=512,(p,w,h)
    digest=hashlib.sha256(b).hexdigest()
    if c.get('art_reuse'):
        original=by_id[c['art_reuse']]
        assert c['type']=='perk' and original['image']==c['image'],c['id']
    elif digest in seen: raise AssertionError(('unexplained duplicate art',p,seen[digest]))
    else: seen[digest]=p
    svg=ET.fromstring((ROOT/f"output/cards/{c['id']}.svg").read_text())
    assert svg.attrib['width']=='63mm' and svg.attrib['height']=='88mm'
    image=svg.find('.//{http://www.w3.org/2000/svg}image')
    assert (ROOT/'output/cards'/image.attrib['{http://www.w3.org/1999/xlink}href']).resolve()==p.resolve()
assert len(seen)==72,len(seen)
assert (ROOT/'assets/art/card_back.png').read_bytes().startswith(b'\x89PNG\r\n\x1a\n')
manifest=json.loads((ROOT/'prompts/manifest.json').read_text())
assert {p['id'] for p in manifest}==set(by_id)
generated=json.loads((ROOT/'prompts/generated.json').read_text())
assert len(generated)==len({entry["id"] for entry in generated})
assert len(generated)>=18
for record in generated:
    assert (ROOT/record['path']).is_file()
    assert next(p['prompt'] for p in manifest if p['id']==record['id'])==record['prompt']
for doc in [ROOT/'README.md',*sorted((ROOT/'docs').glob('*.md')),ROOT/'rules/quickstart.md',ROOT/'prompts/style-guide.md']:
    for href in re.findall(r'\]\(([^)]+)\)',doc.read_text()):
        if '://' not in href and not href.startswith('#'):
            assert (doc.parent/href.split('#')[0]).exists(),(doc,href)
# Переименованные карты: прежние названия не должны оставаться в видимых текстах.
base_names=dict(re.findall(r"^\('(\w+)','([^']+)'",(ROOT/'tools/build_deck.py').read_text(),re.M))
# Районы до московской привязки (docs/game.md) задавались в tools/content.py.
base_names.update(district_swamp='Панельные топи',district_depot='Подземное депо',district_market='Ночной рынок у портала',district_forest='Лесополоса бессмертных',district_mall='ТЦ «Лабиринт»',district_sanatorium='Санаторий последнего шанса')
stale={old:by_id[cid]['name'] for cid,old in base_names.items() if cid in by_id and by_id[cid]['name']!=old}
visible=[ROOT/'rules/quickstart.md',*sorted((ROOT/'crates').rglob('*.rs'))]
texts=[(f'{c["id"]}.text/flavor',c['text']+' '+c['flavor']) for c in cards]+[(str(f.relative_to(ROOT)),f.read_text()) for f in visible]
for where,text in texts:
    for old,new in stale.items():
        assert old not in text,(where,f'устаревшее имя «{old}», сейчас «{new}»')
print('Validated: 90 cards; 72 unique PNG illustrations + back; 18 explicit perk motifs; 90 SVG faces; current generated prompts; documentation links; no stale card names.')
