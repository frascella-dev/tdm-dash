/// Font sizes used in ui/dash.slint at scale 1. Glyphs are embedded as
/// signed distance fields at one resolution chosen from the smallest and
/// largest of these, and drawn at any size (docs/adr/0013); sizes passed
/// through component properties are invisible to Slint's analysis, so the
/// extremes are listed here.
const FONT_SIZES: &str = "12,13,14,15,16,17,18,19,20,21,22,24,26,28,30,34,42,64,72,152,200";

fn main() {
    // The binary needs neither fontconfig nor a font rasteriser at run time.
    // SAFETY: build scripts are single-threaded at this point.
    unsafe { std::env::set_var("SLINT_FONT_SIZES", FONT_SIZES) };
    println!("cargo:rerun-if-changed=ui");
    println!("cargo:rerun-if-changed=fonts");
    let cfg = slint_build::CompilerConfiguration::new()
        .embed_resources(slint_build::EmbedResourcesKind::EmbedForSoftwareRenderer)
        .with_sdf_fonts(true);
    slint_build::compile_with_config("ui/dash.slint", cfg).expect("compile ui/dash.slint");
}
