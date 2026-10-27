// Параметрический исходник. Размеры в мм. Низ плоский.
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
