// tdm-dash enclosure for the Newhaven NHD-7.0-800480AF-ASXP (7", 800x480).
//
// Three printed parts, chosen with -D part="...":
//   front  - bezel with the window pocket, printed face down (black ASA)
//   rear   - shell for Pi Zero 2 W + Kippah and the perfboard (light ASA)
//   visor  - sun visor screwed on top (black ASA)
//   all    - assembly preview (not for printing)
//
//   openscad -D 'part="front"' -o front.stl enclosure.scad
//
// Coordinates: X across the bike, Y up, Z towards the rider. The front face
// of the bezel is at Z = 0, the enclosure extends to negative Z.
// Values marked [DA VERIFICARE] come from datasheets or estimates and must be
// checked on the parts in hand before printing (manual, chapter 07).

part = "all";

/* [Display] */
panel_w = 165.0;      // module outline (datasheet)
panel_h = 104.0;
panel_t = 6.0;        // module thickness [DA VERIFICARE]
aa_w = 154.2;         // active area (datasheet)
aa_h = 85.6;
aa_dx = 0.0;          // active area centre offset from the outline centre [DA VERIFICARE]
aa_dy = 0.0;

/* [Enclosure] */
fit = 0.4;            // clearance around the panel
rim = 8.0;            // width of the rim around the panel (room for M3 inserts)
corner_r = 4.0;       // outer corner radius (larger breaks into the corner screws)
front_t = 2.0;        // front plate in front of the window
window_t = 2.0;       // polycarbonate / PMMA window
foam_t = 1.0;         // foam tape between window and panel
depth = 46.0;         // rear shell depth (Pi + Kippah stack and perfboard)
shell_t = 2.5;        // rear shell walls
gasket_t = 1.0;       // flat EPDM foam gasket between bezel and shell, holed at the screws

/* [Openings] */
gland_d = 12.5;       // PG7 cable gland, bottom face
vent_d = 12.3;        // M12 breather membrane, bottom face
button_d = 16.2;      // two IP67 push buttons, bottom face
button_pitch = 26;
light_d = 4.0;        // light pipe through the bottom rim for the TEMT6000
mount_insert_d = 6.4; // M5 heat-set inserts on the back, for the bike bracket
mount_points = [[-50, 44], [50, 44], [0, -44]];  // [DA VERIFICARE] on the bike

$fn = 64;

inner_w = panel_w + 2 * fit;
inner_h = panel_h + 2 * fit;
outer_w = inner_w + 2 * rim;
outer_h = inner_h + 2 * rim;
bezel_d = front_t + window_t + foam_t + panel_t;   // panel back flush with the bezel rim

// M3 screw positions, in the rim
screws = [
    for (sx = [-1, 1], sy = [-1, 1]) [sx * (inner_w / 2 + rim / 2), sy * (inner_h / 2 + rim / 2)],
    [0, inner_h / 2 + rim / 2], [0, -(inner_h / 2 + rim / 2)]
];

module rrect(w, h, r) {
    offset(r = r) square([w - 2 * r, h - 2 * r], center = true);
}

module slab(w, h, r, z0, z1) {
    translate([0, 0, z0]) linear_extrude(z1 - z0) rrect(w, h, r);
}

// ---------------------------------------------------------------- front

module front() {
    difference() {
        slab(outer_w, outer_h, corner_r, -bezel_d, 0);
        // cavity for window, foam and panel
        translate([0, 0, -bezel_d - 1]) linear_extrude(bezel_d - front_t + 1)
            square([inner_w, inner_h], center = true);
        // display aperture, chamfered 45 degrees towards the rider so the
        // edge never shades the picture
        translate([aa_dx, aa_dy, 0])
            hull() {
                translate([0, 0, -front_t - 0.01]) linear_extrude(0.01)
                    square([aa_w + 1, aa_h + 1], center = true);
                translate([0, 0, 0]) linear_extrude(0.01)
                    square([aa_w + 1 + 2 * front_t, aa_h + 1 + 2 * front_t], center = true);
            }
        // heat-set inserts M3 (4.0 mm hole, 6 mm deep) from the back
        for (p = screws) translate([p[0], p[1], -bezel_d - 0.01]) cylinder(d = 4.0, h = 6);
        // light pipe for the TEMT6000, through the bottom rim
        translate([outer_w / 4, -(inner_h / 2 + rim / 2), -bezel_d - 1]) cylinder(d = light_d, h = bezel_d + 2);
    }
}

// ---------------------------------------------------------------- rear

pi_holes = [for (x = [-29, 29], y = [-11.5, 11.5]) [x, y]];          // Pi Zero 2 W, 58 x 23
perf_holes = [for (x = [-42, 42], y = [-32, 32]) [x, y]];            // 90 x 70 perfboard [DA VERIFICARE]
pi_at = [inner_w / 2 - 40, 10];        // Pi + Kippah on the right (seen from the back: left)
perf_at = [-inner_w / 2 + 50, 0];      // perfboard on the left

module standoffs(at, holes, h, d_out = 6, d_hole = 2.2) {
    for (p = holes) translate([at[0] + p[0], at[1] + p[1], -depth - bezel_d + shell_t])
        difference() {
            cylinder(d = d_out, h = h);
            cylinder(d = d_hole, h = h + 1);
        }
}

module rear() {
    z_top = -bezel_d;                  // mates with the bezel rim
    z_back = -bezel_d - depth;
    difference() {
        union() {
            difference() {
                slab(outer_w, outer_h, corner_r, z_back, z_top);
                // hollow, leaving the rim full width at the top for the screws
                slab(outer_w - 2 * shell_t, outer_h - 2 * shell_t, corner_r - shell_t, z_back + shell_t, z_top - 6);
                translate([0, 0, z_top - 6.01]) linear_extrude(6.02) square([inner_w, inner_h], center = true);
            }
            // screw columns along the corners and the long sides, kept
            // inside the rounded outline
            intersection() {
                for (p = screws) translate([p[0], p[1], z_back]) cylinder(d = 7, h = depth);
                slab(outer_w, outer_h, corner_r, z_back, z_top);
            }
            // tabs that hold the panel against the window, outside the FFC area
            for (p = [[-inner_w / 2, 20], [inner_w / 2, 20], [-inner_w / 2, -20], [inner_w / 2, -20]])
                translate([p[0] - (p[0] < 0 ? 0 : 4), p[1] - 6, z_top - 3]) cube([4, 12, 3]);
            // bike mounting bosses
            for (m = mount_points) translate([m[0], m[1], z_back]) cylinder(d = 12, h = 12);
        }
        // M3 screws from the back: 3.4 mm through, head counterbore 6 mm
        for (p = screws) translate([p[0], p[1], z_back - 1]) {
            cylinder(d = 3.4, h = depth + 2);
            cylinder(d = 6.0, h = depth - 14 + 1);
        }
        // M5 inserts for the bike bracket
        for (m = mount_points) translate([m[0], m[1], z_back - 1]) cylinder(d = mount_insert_d, h = 11);
        // bottom face: cable gland, breather, buttons
        translate([-outer_w / 2 + 30, -outer_h / 2 - 1, z_back + depth / 2]) rotate([-90, 0, 0]) cylinder(d = gland_d, h = shell_t + 2);
        translate([-outer_w / 2 + 55, -outer_h / 2 - 1, z_back + depth / 2]) rotate([-90, 0, 0]) cylinder(d = vent_d, h = shell_t + 2);
        for (i = [0, 1]) translate([outer_w / 2 - 30 - i * button_pitch, -outer_h / 2 - 1, z_back + depth / 2 + 4])
            rotate([-90, 0, 0]) cylinder(d = button_d, h = shell_t + 2);
        // visor inserts on the top face
        for (x = [-outer_w / 4, outer_w / 4]) translate([x, outer_h / 2 - 6, z_back + depth / 2]) rotate([-90, 0, 0]) cylinder(d = 4.0, h = 7);
    }
    standoffs(pi_at, pi_holes, 5);
    standoffs(perf_at, perf_holes, 5);
}

// ---------------------------------------------------------------- visor

visor_len = 40;        // how far it reaches towards the rider
visor_t = 2.5;
visor_cheek = 0.35;    // side cheeks reach down this fraction of the height

module visor() {
    // flat plate screwed on the top face, reaching forward over the display,
    // with side cheeks that taper to nothing at the front face
    z_back = -bezel_d - depth;
    difference() {
        union() {
            translate([-outer_w / 2, outer_h / 2, z_back + 6]) cube([outer_w, visor_t, visor_len + depth + bezel_d - 6]);
            for (sx = [-1, 1]) translate([sx * (outer_w / 2 - visor_t / 2) - visor_t / 2, 0, 0])
                hull() {
                    translate([0, outer_h / 2, 0]) cube([visor_t, 0.01, visor_len]);
                    translate([0, outer_h / 2 * (1 - 2 * visor_cheek), 0]) cube([visor_t, 0.01, 0.01]);
                }
        }
        for (x = [-outer_w / 4, outer_w / 4]) translate([x, outer_h / 2 - 1, z_back + depth / 2]) rotate([-90, 0, 0]) cylinder(d = 3.4, h = 10);
    }
}

// ---------------------------------------------------------------- output

module panel_ghost() {
    color("#222") translate([0, 0, -bezel_d]) linear_extrude(panel_t) square([panel_w, panel_h], center = true);
    color("#4af", 0.6) translate([aa_dx, aa_dy, -bezel_d + panel_t]) linear_extrude(0.2) square([aa_w, aa_h], center = true);
}

if (part == "front") rotate([180, 0, 0]) front();               // face down on the bed
else if (part == "rear") translate([0, 0, bezel_d + depth]) rear();  // open side up
else if (part == "visor") rotate([90, 0, 0]) translate([0, -outer_h / 2 - visor_t, 0]) visor();  // top face down
else {
    color("#333") front();
    color("#ddd") rear();
    color("#333") visor();
    panel_ghost();
}
