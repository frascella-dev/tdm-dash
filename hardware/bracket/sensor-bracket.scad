// Bracket for the LJ12A3-4-Z/BX inductive sensor on the front brake disc.
//
// A flat plate parallel to the disc, held by one brake caliper bolt and kept
// from turning by a second hole (the other caliper bolt) or by a tab against
// the fork leg. The sensor goes through a slot, so its radius can be set over
// the bolt heads of the disc; its two M12 nuts set the 1-2 mm gap.
//
//   openscad -D 'out="3d"' -o sensor-bracket.stl sensor-bracket.scad   # printed test piece
//   openscad -D 'out="2d"' -o sensor-bracket.dxf sensor-bracket.scad   # outline for aluminium
//
// SAFETY: under a brake caliper bolt only the 3 mm aluminium plate is
// allowed. A printed part creeps under the bolt load and the bolt loosens:
// print it only to check the fit. All dimensions marked [TO VERIFY] come
// from the measurements on the bike (docs/research/bike-measurements.md).

out = "3d";

t = 3.0;               // plate thickness (aluminium 5754/6082)
bolt_d = 10.5;         // caliper bolt clearance hole (M10) [TO VERIFY]
second_bolt = true;    // use the second caliper bolt against rotation
bolt_pitch = 83.0;     // distance between the caliper bolts [TO VERIFY]
sensor_d = 12.3;       // M12x1 sensor body
reach = 48.0;          // first bolt to sensor axis, at the middle of the slot [TO VERIFY]
angle = 35.0;          // direction of the sensor arm from the bolt line [TO VERIFY]
slot = 8.0;            // radial adjustment of the sensor (+/- slot/2)
margin = 7.0;          // material around the holes

$fn = 64;

function polar(r, a) = [r * cos(a), r * sin(a)];

sensor_at = polar(reach, angle);
bolt2_at = [bolt_pitch, 0];

module outline() {
    difference() {
        union() {
            hull() {
                circle(d = bolt_d + 2 * margin);
                translate(sensor_at) rotate(angle) hull()
                    for (s = [-1, 1]) translate([s * slot / 2, 0]) circle(d = sensor_d + 2 * margin);
            }
            if (second_bolt) hull() {
                circle(d = bolt_d + 2 * margin);
                translate(bolt2_at) circle(d = bolt_d + 2 * margin);
            }
        }
        circle(d = bolt_d);
        if (second_bolt) translate(bolt2_at) circle(d = bolt_d);
        translate(sensor_at) rotate(angle) hull()
            for (s = [-1, 1]) translate([s * slot / 2, 0]) circle(d = sensor_d);
    }
}

if (out == "2d") outline();
else linear_extrude(t) outline();
