"""Монеты в миллиметрах: замкнутые STL с интегрированным рельефом и OpenSCAD."""
from pathlib import Path
from collections import Counter
import math, struct
ROOT=Path(__file__).resolve().parents[1]
OUT=ROOT/'output/coins'
OUT.mkdir(parents=True,exist_ok=True)
SEGMENTS=192
RINGS=64
DIGITS={'0':'ab cdef'.replace(' ',''),'1':'bc','5':'afgcd'}
# Координаты семи сегментов относительно центра цифры.
BARS={'a':(-1.4,1.6,1.4,2.3),'g':(-1.4,-.35,1.4,.35),'d':(-1.4,-2.3,1.4,-1.6),
      'f':(-1.4,.0,-.7,2.0),'b':(.7,.0,1.4,2.0),'e':(-1.4,-2.0,-.7,.0),'c':(.7,-2.0,1.4,.0)}

def raised(x,y,value):
    text=str(value)
    for k,digit in enumerate(text):
        cx=(k-(len(text)-1)/2)*3.8
        for bar in DIGITS[digit]:
            x0,y0,x1,y1=BARS[bar]
            if x0+cx<=x<=x1+cx and y0+5.0<=y<=y1+5.0:return .6
    return 0

def mesh(value,diameter):
    radius=diameter/2
    vertices=[]
    # Два слоя одинаковой сетки: поверхность и плоское дно.
    for side in range(2):
        for ring in range(RINGS+1):
            t=ring/RINGS
            for segment in range(SEGMENTS):
                angle=2*math.pi*segment/SEGMENTS
                co,si=math.cos(angle),math.sin(angle)
                inner=2.2/max(abs(co),abs(si)) # квадратное отверстие 4.4 мм
                r=inner+(radius-inner)*t
                x,y=r*co,r*si
                z=(3+raised(x,y,value)-(.35 if ring==RINGS else 0)) if side==0 else 0
                vertices.append((x,y,z))
    layer=(RINGS+1)*SEGMENTS
    faces=[]
    for ring in range(RINGS):
        for j in range(SEGMENTS):
            k=(j+1)%SEGMENTS
            a=ring*SEGMENTS+j;b=ring*SEGMENTS+k;c=(ring+1)*SEGMENTS+j;d=(ring+1)*SEGMENTS+k
            faces.extend([(a,c,d),(a,d,b),(a+layer,d+layer,c+layer),(a+layer,b+layer,d+layer)])
    for j in range(SEGMENTS):
        k=(j+1)%SEGMENTS
        for ring,reverse in [(0,True),(RINGS,False)]:
            a=ring*SEGMENTS+j;b=ring*SEGMENTS+k;c=a+layer;d=b+layer
            faces.extend([(a,b,d),(a,d,c)] if reverse else [(a,d,b),(a,c,d)])
    edges=Counter(tuple(sorted((f[i],f[(i+1)%3]))) for f in faces for i in range(3))
    assert all(n==2 for n in edges.values()),'mesh must be closed'
    directed=Counter((f[i],f[(i+1)%3]) for f in faces for i in range(3))
    assert all(n==directed[(b,a)] for (a,b),n in directed.items()),'normals must be consistent'
    path=OUT/f'kryshak-{value}.stl'
    volume=0.0
    records=[]
    for face in faces:
        a,b,c=[vertices[i] for i in face]
        u=[b[i]-a[i] for i in range(3)];v=[c[i]-a[i] for i in range(3)]
        normal=(u[1]*v[2]-u[2]*v[1],u[2]*v[0]-u[0]*v[2],u[0]*v[1]-u[1]*v[0])
        length=math.sqrt(sum(x*x for x in normal));assert length>0
        normal=tuple(x/length for x in normal)
        volume+=(a[0]*(b[1]*c[2]-b[2]*c[1])+a[1]*(b[2]*c[0]-b[0]*c[2])+a[2]*(b[0]*c[1]-b[1]*c[0]))/6
        records.append(struct.pack('<12fH',*normal,*a,*b,*c,0))
    assert volume>0
    with path.open('wb') as f:
        f.write(f'Orcs kryshak {value}; units mm; flat bottom; relief top'.encode().ljust(80,b'\0'))
        f.write(struct.pack('<I',len(faces)))
        f.writelines(records)
    print(f'{path.relative_to(ROOT)}: {len(faces)} triangles, closed, positive volume {volume:.1f} mm3')

for value,diameter in [(1,22),(5,27),(10,32)]:mesh(value,diameter)
(OUT/'kryshaki.scad').write_text('''// Параметрический исходник. Размеры в мм. Низ плоский.
nominal = 5; // 1, 5 или 10
$fn=192;
diameter = nominal==1 ? 22 : nominal==5 ? 27 : 32;
module base() {
 difference() {
  union() { cylinder(h=2.65,d=diameter); translate([0,0,2.65]) cylinder(h=.35,d1=diameter,d2=diameter-.7); }
  translate([0,0,-1]) linear_extrude(6) square([4.4,4.4],center=true);
 }
}
// В STL используется семисегментная цифра, встроенная в замкнутую сетку.
// OpenSCAD допускает замену шрифта для собственной печати.
union() { base(); translate([0,5,2.98]) linear_extrude(.62)
 text(str(nominal),size=5.4,halign="center",valign="center",font="Liberation Sans:style=Bold"); }
''')
(OUT/'README.md').write_text('''# Крышаки для 3D-печати

| Номинал | Диаметр | Количество в стартовом банке |
|---|---|---|
| 1 | 22 мм | 36 |
| 5 | 27 мм | 18 |
| 10 | 32 мм | 6 |

Толщина основы 3 мм, цифра сверху ещё 0.6 мм; квадратное отверстие 4.4 мм, скос верхнего края 0.35 мм. Низ плоский, поэтому поддержки не нужны по геометрии модели. Номиналы различаются размером и рельефной цифрой, цвет необязателен. Всего 60 монет на 186 крышаков. Банк в правилах не ограничен пластиком; при нехватке запишите сумму.

`kryshak-1.stl`, `kryshak-5.stl`, `kryshak-10.stl` — готовые замкнутые сетки в миллиметрах, без пересекающихся отдельных букв. Генератор проверяет, что каждое ребро принадлежит двум граням и объём положительный. `kryshaki.scad` — редактируемый параметрический вариант с обычным шрифтом; начертание отличается от семисегментных STL.

Сначала напечатайте по одной монете и проверьте читаемость цифр, размеры и проходимость отверстия. Физическая печать ещё не выполнялась; слой, материал и компенсацию отверстия подберите под свой принтер. Повторная сборка: `python3 tools/build_coins.py` из корня проекта.
''')
