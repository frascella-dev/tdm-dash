// tdm-dash enclosure for the Newhaven NHD-7.0-800480AF-ASXP (7", 800x480).
//
// A thin display unit with a small pod behind it. Three printed parts, chosen
// with -D part="...":
//   front    - bezel: face, window pocket, button wings (black ASA, face down)
//   cover    - thin back of the display unit: holds the display, carries the
//              Pi Zero 2 W + Kippah and the button cups (black ASA, face down)
//   rear     - pod for the perfboard, screwed behind the cover (black ASA,
//              open side up)
//   all      - assembly preview with the display and the boards (not for printing)
//   exploded - the same, pulled apart along the assembly order
//
//   openscad -D 'part="front"' -o front.stl enclosure.scad
//
// Coordinates: X across the bike, Y up, Z towards the rider. The front face
// of the bezel is at Z = 0, everything else extends to negative Z.
// Values marked [TO VERIFY] come from datasheets or estimates and must be
// checked on the parts in hand before printing (manual, chapter 07). The
// asserts at the end refuse parameters that make the parts collide.

part = "all";

/* [Display] */
panel_w = 165.0;      // module outline (datasheet)
panel_h = 104.0;
panel_t = 6.0;        // module thickness [TO VERIFY]
aa_w = 154.2;         // active area (datasheet)
aa_h = 85.6;
aa_dx = 0.0;          // active area center offset from the outline center [TO VERIFY]
aa_dy = 0.0;
ffc_w = 34.0;         // slot for the 40-pin FFC tail, at the bottom edge of the panel [TO VERIFY]
ffc_y = -46.4;        // slot center; the tail must reach the Kippah [TO VERIFY]

/* [Display unit] */
fit = 0.4;            // clearance around the panel
front_t = 2.0;        // face in front of the window
window_t = 2.0;       // polycarbonate / PMMA window
foam_t = 1.0;         // foam tape between window and panel
gasket_t = 1.0;       // flat EPDM gasket between bezel and cover
cover_t = 2.5;        // cover plate
wall_t = 2.5;         // outer wall of the bezel
pocket_wall = 2.0;    // wall around the panel pocket
rim_top = 8.0;        // bezel above the panel
rim_bottom = 18.0;    // bezel below the panel: light sensor and wires
wing = 24.0;          // bezel beside the panel, where the buttons sit
bottom_margin = 6.0;  // bottom edge beyond the panel, each side
chamfer_y = -26.0;    // where the sides turn into the lower chamfers
corner_r = 6.0;       // outline corner radius

/* [Buttons] */
button_d = 12.2;      // two IP67 12 mm panel buttons in the wings
button_y = -6.0;      // height of the buttons
button_body = 22.0;   // body and terminals behind the face [TO VERIFY]
button_collar = 9.0;  // radius of the nut and collar behind the face
cup_wall = 1.6;

/* [Pod] */
perf_w = 90.0;        // perfboard [TO VERIFY]
perf_h = 70.0;
perf_standoff = 5.0;
perf_parts = 20.0;    // tallest part on the perfboard (Pololu in its socket) [TO VERIFY]
pi_stack = 15.0;      // Pi Zero 2 W with the Kippah on top [TO VERIFY]
pi_standoff = 4.0;
stack_gap = 3.0;      // between the perfboard parts and the Pi stack
pod_wall = 2.5;
pod_floor = 2.5;
pod_r = 6.0;
boss_h = 4.0;         // insert bosses on the cover, inside the pod
pod_cy = -23.5;       // pod center, low behind the display where the FFC comes out

/* [Openings] */
gland_d = 12.5;       // PG7 cable gland, bottom of the pod
vent_d = 12.3;        // M12 breather membrane, bottom of the pod
gland_x = -25.0;
vent_x = 25.0;
port_z = 12.0;        // gland and vent: distance behind the cover, above the perfboard parts
light_d = 4.0;        // light pipe through the bottom of the bezel for the TEMT6000
light_x = 20.0;
wire_d = 6.0;         // cover hole that takes the button and sensor wires into the pod
mount_insert_d = 6.4; // M5 heat-set inserts in the mount lugs, for the bike bracket
mount_points = [[-62, -3.5], [62, -3.5], [0, 28]];  // [TO VERIFY] on the bike
lug_t = 10.0;

$fn = 64;

inner_w = panel_w + 2 * fit;
inner_h = panel_h + 2 * fit;
bezel_d = front_t + window_t + foam_t + panel_t;   // panel back flush with the bezel back
unit_d = bezel_d + gasket_t + cover_t;             // thickness of the display unit
half_w = inner_w / 2 + wing;                       // half width at the wings
bottom_half = inner_w / 2 + bottom_margin;         // half width of the bottom edge
top_y = inner_h / 2 + rim_top;
bottom_y = -(inner_h / 2 + rim_bottom);
outer_w = 2 * half_w;
outer_h = top_y - bottom_y;
button_x = inner_w / 2 + wing / 2;
cup_d = button_d + 4 + 2 * cup_wall;
cup_z = -(front_t + button_body + 1);              // back end of the button cups
pod_in_w = perf_w + 14;                            // room for the corner screw columns
pod_in_h = perf_h + 14;
pod_w = pod_in_w + 2 * pod_wall;
pod_h = pod_in_h + 2 * pod_wall;
pod_in_d = perf_standoff + 1.6 + perf_parts + stack_gap + pi_stack + pi_standoff;
pod_d = pod_in_d + pod_floor;
z_cover = -(bezel_d + gasket_t);                   // front face of the cover
z_back = -(unit_d + pod_d);                        // back of the pod
wire_y = -(inner_h / 2 + pocket_wall + 6);
rim_screw_y = -(inner_h / 2 + rim_bottom / 2);

// M3 screws from the back of the cover into inserts in the bezel
screws = [[0, top_y - rim_top / 2], [-60, top_y - rim_top / 2], [60, top_y - rim_top / 2],
          [-100, 40], [100, 40], [-97, -32], [97, -32], [-70, rim_screw_y], [70, rim_screw_y]];
// M3 screws from the back of the pod into inserts in the cover bosses
pod_screws = [for (sx = [-1, 1], sy = [-1, 1]) [sx * (pod_in_w / 2 - 3), pod_cy + sy * (pod_in_h / 2 - 3)]];
perf_holes = [for (x = [-42, 42], y = [-32, 32]) [x, y]];            // 90 x 70 perfboard [TO VERIFY]
pi_holes = [for (x = [-29, 29], y = [-11.5, 11.5]) [x, y]];          // Pi Zero 2 W, 58 x 23
pi_at = [0, pod_cy + 10];
perf_at = [0, pod_cy];

// ---------------------------------------------------------------- outline

outline_pts = [[-bottom_half, bottom_y], [bottom_half, bottom_y], [half_w, chamfer_y],
               [half_w, top_y], [-half_w, top_y], [-half_w, chamfer_y]];

module outline(grow = 0) {
    offset(delta = grow) offset(r = corner_r) offset(delta = -corner_r) polygon(outline_pts);
}

module rrect(w, h, r) {
    offset(r = r) square([w - 2 * r, h - 2 * r], center = true);
}

// ---------------------------------------------------------------- front

module front() {
    difference() {
        union() {
            difference() {
                translate([0, 0, -bezel_d]) linear_extrude(bezel_d) outline();
                // hollow ring behind the face, around the panel pocket
                translate([0, 0, -bezel_d - 1]) linear_extrude(bezel_d - front_t + 1) difference() {
                    outline(-wall_t);
                    square([inner_w + 2 * pocket_wall, inner_h + 2 * pocket_wall], center = true);
                }
            }
            // screw bosses, kept inside the outline
            intersection() {
                for (p = screws) translate([p[0], p[1], -bezel_d]) cylinder(d = 7, h = bezel_d);
                translate([0, 0, -bezel_d]) linear_extrude(bezel_d) outline();
            }
        }
        // panel, foam and window pocket
        translate([0, 0, -bezel_d - 1]) linear_extrude(bezel_d - front_t + 1)
            square([inner_w, inner_h], center = true);
        // display aperture, chamfered 45 degrees towards the rider so the
        // edge never shades the picture
        translate([aa_dx, aa_dy, 0])
            hull() {
                translate([0, 0, -front_t - 0.01]) linear_extrude(0.01)
                    square([aa_w + 1, aa_h + 1], center = true);
                linear_extrude(0.01) square([aa_w + 1 + 2 * front_t, aa_h + 1 + 2 * front_t], center = true);
            }
        // heat-set inserts M3 (4.0 mm hole, 6 mm deep) from the back
        for (p = screws) translate([p[0], p[1], -bezel_d - 0.01]) cylinder(d = 4.0, h = 6);
        // buttons and light pipe through the face
        for (sx = [-1, 1]) translate([sx * button_x, button_y, -front_t - 1]) cylinder(d = button_d, h = front_t + 2);
        translate([light_x, rim_screw_y, -front_t - 1]) cylinder(d = light_d, h = front_t + 2);
    }
}

// ---------------------------------------------------------------- cover

module cover() {
    z1 = z_cover - cover_t;
    difference() {
        union() {
            translate([0, 0, z1]) linear_extrude(cover_t) outline();
            // cups for the button bodies
            for (sx = [-1, 1]) translate([sx * button_x, button_y, cup_z]) cylinder(d = cup_d, h = z1 - cup_z + 0.01);
            // insert bosses for the pod screws and standoffs for the Pi
            for (p = pod_screws) translate([p[0], p[1], z1 - boss_h]) cylinder(d = 8, h = boss_h + 0.01);
            for (p = pi_holes) translate([pi_at[0] + p[0], pi_at[1] + p[1], z1 - pi_standoff]) cylinder(d = 6, h = pi_standoff + 0.01);
        }
        for (p = screws) translate([p[0], p[1], z1 - 1]) cylinder(d = 3.4, h = cover_t + 2);
        for (sx = [-1, 1]) translate([sx * button_x, button_y, cup_z + cup_wall]) cylinder(d = cup_d - 2 * cup_wall, h = z_cover - cup_z);
        translate([0, ffc_y, z1 - 1]) linear_extrude(cover_t + 2) square([ffc_w, 4], center = true);
        translate([0, wire_y, z1 - 1]) cylinder(d = wire_d, h = cover_t + 2);
        for (p = pod_screws) translate([p[0], p[1], z1 - boss_h - 0.01]) cylinder(d = 4.0, h = 6);
        for (p = pi_holes) translate([pi_at[0] + p[0], pi_at[1] + p[1], z1 - pi_standoff - 1]) cylinder(d = 2.2, h = pi_standoff + 1);
    }
}

// ---------------------------------------------------------------- rear pod

module rear() {
    z_top = -unit_d;                   // mates with the back of the cover
    difference() {
        union() {
            difference() {
                translate([0, pod_cy, z_back]) linear_extrude(pod_d) rrect(pod_w, pod_h, pod_r);
                translate([0, pod_cy, z_back + pod_floor]) linear_extrude(pod_d) rrect(pod_in_w, pod_in_h, pod_r - pod_wall);
            }
            // screw columns in the corners, ending on the cover bosses
            for (p = pod_screws) translate([p[0], p[1], z_back]) cylinder(d = 7, h = pod_d - boss_h - 0.3);
            // lugs for the bike mount, outside the pod
            difference() {
                for (m = mount_points) hull() {
                    translate([m[0], m[1], z_back]) cylinder(d = 14, h = lug_t);
                    translate([m[0] * 0.8, pod_cy + (m[1] - pod_cy) * 0.8, z_back]) cylinder(d = 14, h = lug_t);
                }
                translate([0, pod_cy, z_back + pod_floor]) linear_extrude(lug_t) rrect(pod_in_w, pod_in_h, pod_r - pod_wall);
            }
            // perfboard standoffs
            for (p = perf_holes) translate([perf_at[0] + p[0], perf_at[1] + p[1], z_back + pod_floor - 0.01])
                difference() { cylinder(d = 6, h = perf_standoff); cylinder(d = 2.2, h = perf_standoff + 1); }
        }
        // M3 screws from the back: 3.4 mm through, heads in a 6 mm counterbore
        // so that an M3 x 30 reaches 5.5 mm into the insert
        for (p = pod_screws) translate([p[0], p[1], z_back - 1]) {
            cylinder(d = 3.4, h = pod_d + 2);
            cylinder(d = 6.0, h = pod_d - boss_h - 24.5 + 1);
        }
        for (m = mount_points) translate([m[0], m[1], z_back - 1]) cylinder(d = mount_insert_d, h = lug_t);
        // bottom face: cable gland and breather, near the cover, above the perfboard parts
        for (g = [[gland_x, gland_d], [vent_x, vent_d]]) translate([g[0], pod_cy - pod_h / 2 - 1, z_top - port_z])
            rotate([-90, 0, 0]) cylinder(d = g[1], h = pod_wall + 2);
    }
}

// ---------------------------------------------------------------- previews

// The display, window, boards and buttons, for the previews only. The active
// area stands 0.05 mm proud of the module, so the faces never share a plane.
module panel_ghost() {
    color("#222") translate([0, 0, -bezel_d]) linear_extrude(panel_t) square([panel_w, panel_h], center = true);
    color("#2b4d6e") translate([aa_dx, aa_dy, -bezel_d + panel_t + 0.05]) linear_extrude(0.2) square([aa_w, aa_h], center = true);
}

module window_ghost() {
    color("#9cc4e4", 0.45) translate([0, 0, -front_t - window_t]) linear_extrude(window_t)
        square([inner_w - 0.4, inner_h - 0.4], center = true);
}

module boards_ghost(pi = true, perf = true) {
    if (pi) {
        color("#2f7d3b") translate([pi_at[0], pi_at[1], -unit_d - pi_standoff - 1.4]) linear_extrude(1.4) square([65, 30], center = true);
        color("#3a5f8f") translate([pi_at[0], pi_at[1], -unit_d - pi_standoff - pi_stack]) linear_extrude(pi_stack - 1.4) square([60, 28], center = true);
    }
    if (perf) {
        color("#c9b26b") translate([perf_at[0], perf_at[1], z_back + pod_floor + perf_standoff]) linear_extrude(1.6) square([perf_w, perf_h], center = true);
        color("#8a8f96", 0.7) translate([perf_at[0], perf_at[1], z_back + pod_floor + perf_standoff + 1.6]) linear_extrude(perf_parts) square([perf_w - 10, perf_h - 10], center = true);
    }
}

module buttons_ghost() {
    for (sx = [-1, 1]) translate([sx * button_x, button_y, 0]) {
        color("#9aa0a6") cylinder(d = button_d - 0.5, h = 1.5);
        color("#555") translate([0, 0, -front_t - button_body + 1]) cylinder(d = button_d - 0.4, h = button_body - 1);
    }
}

if (part == "front") rotate([180, 0, 0]) front();                                // face down on the bed
else if (part == "cover") rotate([180, 0, 0]) translate([0, 0, -z_cover]) cover();  // front face down
else if (part == "rear") color("#3a3d42") translate([0, 0, -z_back]) rear();       // open side up
else if (part == "exploded") {
    color("#2c2f33") translate([0, 0, 90]) front();
    translate([0, 0, 60]) window_ghost();
    translate([0, 0, 40]) panel_ghost();
    translate([0, 0, 75]) buttons_ghost();
    color("#3a3d42") cover();
    boards_ghost(perf = false);
    translate([0, 0, -45]) { color("#45494e") rear(); boards_ghost(pi = false); }
}
else {
    color("#2c2f33") front();
    color("#3a3d42") cover();
    color("#45494e") rear();
    panel_ghost();
    buttons_ghost();
}

// ---------------------------------------------------------------- checks

function dist_to_line(p, a, b) =   // signed: positive outside, for a line with the inside on its left
    ((b[0] - a[0]) * (p[1] - a[1]) - (b[1] - a[1]) * (p[0] - a[0])) / -norm(b - a);

assert(-dist_to_line([inner_w / 2, -inner_h / 2], [bottom_half, bottom_y], [half_w, chamfer_y]) >= 6,
       "the lower chamfer cuts into the panel corner");
assert(button_x - button_collar >= inner_w / 2 + pocket_wall + 0.5, "the buttons touch the panel pocket");
assert(button_x + button_collar <= half_w - wall_t, "the buttons touch the outer wall");
assert(button_y - button_collar >= chamfer_y && button_y + cup_d / 2 <= top_y - wall_t, "the buttons leave the wings");
assert(pod_w / 2 + 2 <= button_x - cup_d / 2, "the pod touches the button cups");
assert(pod_cy - pod_h / 2 >= bottom_y + 2 && pod_cy + pod_h / 2 <= top_y - 2, "the pod leaves the cover");
for (p = screws) assert(abs(p[0]) > pod_w / 2 + 4 || p[1] > pod_cy + pod_h / 2 + 4 || p[1] < pod_cy - pod_h / 2 - 4,
                        str("cover screw ", p, " lies under the pod"));
assert(abs(ffc_y - pod_cy) + 2 <= pod_in_h / 2 && ffc_w / 2 <= pod_in_w / 2, "the FFC slot is outside the pod");
assert(wire_y - wire_d / 2 >= pod_cy - pod_in_h / 2 && wire_y - wire_d / 2 >= bottom_y + wall_t, "the wire hole misses the pod or the bezel rim");
assert(norm([pod_in_w / 2 - 3 - perf_w / 2, pod_in_h / 2 - 3 - perf_h / 2]) >= 3.5 + 0.5,
       "the pod screw columns touch the perfboard corners");
echo(str("display unit ", outer_w, " x ", outer_h, " x ", unit_d, " mm; pod ", pod_w, " x ", pod_h, " x ", pod_d,
         " mm; total depth ", unit_d + pod_d, " mm"));
