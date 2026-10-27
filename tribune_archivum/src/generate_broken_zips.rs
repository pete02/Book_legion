// tests/broken_epubs.rs
//
// Cargo.toml:
//   [dev-dependencies]
//   zip   = "2"
//   image = { version = "0.25", default-features = false, features = ["png"] }
//
// Run:  cargo test generate_broken_epubs -- --nocapture
// Output: target/broken_epubs/
//   valid.epub                 -> should pass fix -> validate
//   <requirement_function>.epub -> one violation each (see function names)
//   all_issues.epub            -> every (non-huge) breaker applied to one book
//
// Notes:
// - Some violations cascade in the data model (e.g. a stray mp3 is both R4 and R5/M6).
//   Each breaker triggers the named rule; the fixer/validator decides which code it reports first.
// - X3 (empty body) is WARN-only and R7 (external <a>) is allowed, so neither has a breaker.
// - The two `HUGE` cases write 100-540 MB of data, so they're excluded from all_issues.
#![allow(deprecated)]

use std::{fs, io::Cursor, io::Write, path::Path};
use zip::{unstable::write::FileOptionsExt, write::SimpleFileOptions, CompressionMethod, ZipWriter};

// ---------------------------------------------------------------- model

#[derive(Clone, PartialEq)]
enum Kind {
    Stored,
    Deflate,
    Symlink,
    Encrypted,
}

#[derive(Clone, PartialEq)]
struct F {
    name: String,
    data: Vec<u8>,
    kind: Kind,
    repeat: u64, // data is written `repeat` times (cheap way to make big entries)
}
type Files = Vec<F>;

const XML: &str = "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n";
const CONTAINER: &str = "META-INF/container.xml";
const OPF: &str = "OEBPS/content.opf";
const NAV: &str = "OEBPS/nav.xhtml";
const CH1: &str = "OEBPS/ch1.xhtml";
const CH2: &str = "OEBPS/ch2.xhtml";
const CSS: &str = "OEBPS/style.css";
const COVER: &str = "OEBPS/cover.png";

// ---------------------------------------------------------------- helpers

fn add_kind(fs: &mut Files, name: &str, data: impl Into<Vec<u8>>, kind: Kind) {
    fs.retain(|f| f.name != name);
    fs.push(F { name: name.into(), data: data.into(), kind, repeat: 1 });
}
fn add(fs: &mut Files, name: &str, data: impl Into<Vec<u8>>) {
    let kind = if name == "mimetype" { Kind::Stored } else { Kind::Deflate };
    add_kind(fs, name, data, kind);
}
fn remove(fs: &mut Files, name: &str) {
    fs.retain(|f| f.name != name);
}
/// Edit a file as text. Silently does nothing if the file is missing.
fn text(fs: &mut Files, name: &str, f: impl FnOnce(String) -> String) {
    if let Some(x) = fs.iter_mut().find(|x| x.name == name) {
        x.data = f(String::from_utf8_lossy(&x.data).into_owned()).into_bytes();
    }
}
/// Replace the first occurrence. Silently does nothing if not found.
fn replace(fs: &mut Files, name: &str, from: &str, to: &str) {
    text(fs, name, |s| s.replacen(from, to, 1));
}
/// Remove everything from the first `a` through the first `b` (inclusive).
fn cut(s: String, a: &str, b: &str) -> String {
    match (s.find(a), s.find(b)) {
        (Some(i), Some(j)) if j >= i => format!("{}{}", &s[..i], &s[j + b.len()..]),
        _ => s,
    }
}
fn add_item(fs: &mut Files, xml: &str) {
    replace(fs, OPF, "</manifest>", &format!("{xml}\n</manifest>"));
}
fn add_itemref(fs: &mut Files, xml: &str) {
    replace(fs, OPF, "</spine>", &format!("{xml}\n</spine>"));
}
fn in_body(fs: &mut Files, xml: &str) {
    replace(fs, CH1, "<p>Hello.</p>", &format!("<p>Hello.</p>{xml}"));
}
fn in_head(fs: &mut Files, xml: &str) {
    replace(fs, CH1, "</head>", &format!("{xml}</head>"));
}
fn add_css(fs: &mut Files, css: &str) {
    text(fs, CSS, |s| format!("{s}\n{css}\n"));
}
fn png(w: u32, h: u32) -> Vec<u8> {
    let mut out = Cursor::new(Vec::new());
    image::RgbImage::from_pixel(w, h, image::Rgb([180, 180, 220]))
        .write_to(&mut out, image::ImageFormat::Png)
        .unwrap();
    out.into_inner()
}

// ---------------------------------------------------------------- the valid book

fn chapter(title: &str, body: &str) -> String {
    format!(
        "{XML}<html xmlns=\"http://www.w3.org/1999/xhtml\" xmlns:epub=\"http://www.idpf.org/2007/ops\">\n\
         <head><title>{title}</title><link rel=\"stylesheet\" type=\"text/css\" href=\"style.css\"/></head>\n\
         <body><h1>{title}</h1>{body}</body>\n</html>\n"
    )
}

fn valid() -> Files {
    let mut fs = Files::new();
    add(&mut fs, "mimetype", "application/epub+zip");
    add(&mut fs, CONTAINER, format!(
r#"{XML}<container version="1.0" xmlns="urn:oasis:names:tc:opendocument:xmlns:container">
<rootfiles>
<rootfile full-path="OEBPS/content.opf" media-type="application/oebps-package+xml"/>
</rootfiles>
</container>
"#));
    add(&mut fs, OPF, format!(
r#"{XML}<package xmlns="http://www.idpf.org/2007/opf" version="3.0" unique-identifier="bookid">
<metadata xmlns:dc="http://purl.org/dc/elements/1.1/">
<dc:identifier id="bookid">urn:uuid:1234</dc:identifier>
<dc:title>Test Book</dc:title>
<dc:creator>Some Author</dc:creator>
<dc:language>en</dc:language>
</metadata>
<manifest>
<item id="nav" href="nav.xhtml" media-type="application/xhtml+xml" properties="nav"/>
<item id="cover" href="cover.png" media-type="image/png" properties="cover-image"/>
<item id="css" href="style.css" media-type="text/css"/>
<item id="ch1" href="ch1.xhtml" media-type="application/xhtml+xml"/>
<item id="ch2" href="ch2.xhtml" media-type="application/xhtml+xml"/>
</manifest>
<spine>
<itemref idref="ch1"/>
<itemref idref="ch2"/>
</spine>
</package>
"#));
    add(&mut fs, NAV, format!(
r#"{XML}<html xmlns="http://www.w3.org/1999/xhtml" xmlns:epub="http://www.idpf.org/2007/ops">
<head><title>Contents</title></head>
<body>
<nav epub:type="toc">
<ol>
<li><a href="ch1.xhtml">Chapter One</a></li>
<li><a href="ch2.xhtml">Chapter Two</a></li>
</ol>
</nav>
</body>
</html>
"#));
    add(&mut fs, CH1, chapter("One", "<p>Hello.</p>"));
    add(&mut fs, CH2, chapter("Two", "<p>World.</p>"));
    add(&mut fs, CSS, "p { color: black; }\n");
    add(&mut fs, COVER, png(200, 300));
    fs
}

// ---------------------------------------------------------------- the breakers
// One function per violation. Function name = requirement ID + what is wrong.
// Order matters only for all_issues.epub: destructive breakers are last.

macro_rules! breakers {
    ($( $name:ident ($fs:ident) $body:block )*) => {
        $( fn $name($fs: &mut Files) $body )*
        const BREAKERS: &[(&str, fn(&mut Files))] = &[ $( (stringify!($name), $name) ),* ];
    };
}

breakers! {
    // ---- Z: archive
    z2_backslash_name(fs)        { add(fs, "OEBPS\\extra.css", "x"); }
    z2_absolute_name(fs)         { add(fs, "/abs.txt", "x"); }
    z2_escapes_root(fs)          { add(fs, "../evil.txt", "x"); }
    z2_nul_byte_name(fs)         { add(fs, "OEBPS/nul\0.txt", "x"); }
    z3_symlink(fs)               { add_kind(fs, "OEBPS/link.xhtml", "ch1.xhtml", Kind::Symlink); }
    z3_encrypted_entry(fs)       { add_kind(fs, "OEBPS/secret.txt", "x", Kind::Encrypted); }
    z4_compression_ratio(fs)     { fs.push(F { name: "OEBPS/zeros.bin".into(), data: vec![0; 1 << 20], kind: Kind::Deflate, repeat: 50 }); }
    z4_too_many_entries(fs)      { for i in 0..10_001 { add(fs, &format!("OEBPS/pad{i}.txt"), "x"); } }
    z5_mimetype_not_first(fs)    { let m = fs.remove(0); fs.push(m); }
    z5_mimetype_compressed(fs)   { if let Some(m) = fs.iter_mut().find(|f| f.name == "mimetype") { m.kind = Kind::Deflate; } }
    z5_mimetype_wrong_content(fs){ add(fs, "mimetype", "application/zip"); }
    z6_control_char_name(fs)     { add(fs, "OEBPS/bad\u{1}name.txt", "x"); }

    // ---- C: container
    c2_two_rootfiles(fs)         { replace(fs, CONTAINER, "</rootfiles>", "<rootfile full-path=\"OEBPS/other.opf\" media-type=\"application/oebps-package+xml\"/></rootfiles>"); }
    c2_no_rootfile(fs)           { text(fs, CONTAINER, |s| cut(s, "<rootfile ", "/>")); }
    c2_wrong_media_type(fs)      { replace(fs, CONTAINER, "application/oebps-package+xml", "text/plain"); }
    c2_full_path_not_opf(fs)     { replace(fs, CONTAINER, "content.opf", "content.xml"); }
    c3_full_path_leading_slash(fs){ replace(fs, CONTAINER, "full-path=\"OEBPS", "full-path=\"/OEBPS"); }
    c3_full_path_percent_encoded(fs){ replace(fs, CONTAINER, "OEBPS/", "OEBPS%2F"); }
    c3_full_path_file_missing(fs){ replace(fs, CONTAINER, "OEBPS/content.opf", "OEBPS/missing.opf"); }

    // ---- O: package document
    o2_guide_present(fs)         { replace(fs, OPF, "</package>", "<guide><reference type=\"cover\" href=\"cover.png\"/></guide></package>"); }
    o3_two_manifests(fs)         { replace(fs, OPF, "</spine>", "</spine><manifest><item id=\"x\" href=\"style.css\" media-type=\"text/css\"/></manifest>"); }
    o4_two_spines(fs)            { replace(fs, OPF, "</spine>", "</spine><spine><itemref idref=\"ch1\"/></spine>"); }

    // ---- M: manifest
    m1_empty_id(fs)              { replace(fs, OPF, "id=\"css\"", "id=\"\""); }
    m1_duplicate_id(fs)          { add_item(fs, "<item id=\"ch1\" href=\"style.css\" media-type=\"text/css\"/>"); }
    m1_missing_href(fs)          { add_item(fs, "<item id=\"nohref\" media-type=\"text/css\"/>"); }
    m2_href_case_mismatch(fs)    { replace(fs, OPF, "href=\"ch2.xhtml\"", "href=\"CH2.xhtml\""); }
    m2_href_percent_encoded(fs)  { add(fs, "OEBPS/my file.css", "p{}"); add_item(fs, "<item id=\"enc\" href=\"my%20file.css\" media-type=\"text/css\"/>"); }
    m2_href_has_fragment(fs)     { add(fs, "OEBPS/frag.css", "p{}"); add_item(fs, "<item id=\"frag\" href=\"frag.css#x\" media-type=\"text/css\"/>"); }
    m2_href_escapes_root(fs)     { add_item(fs, "<item id=\"esc\" href=\"../../outside.css\" media-type=\"text/css\"/>"); }
    m3_file_missing(fs)          { add_item(fs, "<item id=\"gone\" href=\"gone.css\" media-type=\"text/css\"/>"); }
    m4_media_type_missing(fs)    { add(fs, "OEBPS/a.css", "p{}"); add_item(fs, "<item id=\"a\" href=\"a.css\"/>"); }
    m4_media_type_wrong(fs)      { add(fs, "OEBPS/b.css", "p{}"); add_item(fs, "<item id=\"b\" href=\"b.css\" media-type=\"text/plain\"/>"); }
    m4_media_type_disallowed(fs) { add(fs, "OEBPS/a.mp3", "ID3"); add_item(fs, "<item id=\"snd\" href=\"a.mp3\" media-type=\"audio/mpeg\"/>"); }
    m5_remote_href(fs)           { add_item(fs, "<item id=\"remote\" href=\"http://example.com/x.css\" media-type=\"text/css\"/>"); }
    m6_file_not_in_manifest(fs)  { add(fs, "OEBPS/stray.txt", "stray"); }

    // ---- S: spine
    s2_dangling_idref(fs)        { add_itemref(fs, "<itemref idref=\"nope\"/>"); }
    s3_linear_no(fs)             { replace(fs, OPF, "<itemref idref=\"ch2\"/>", "<itemref idref=\"ch2\" linear=\"no\"/>"); }
    s4_duplicate_idref(fs)       { add_itemref(fs, "<itemref idref=\"ch1\"/>"); }

    // ---- N: navigation
    n1_nav_id_missing(fs)        { replace(fs, OPF, "id=\"nav\"", "id=\"navigation\""); }
    n1_nav_id_duplicated(fs)     { add_item(fs, "<item id=\"nav\" href=\"ch2.xhtml\" media-type=\"application/xhtml+xml\"/>"); }
    n1_nav_wrong_media_type(fs)  { replace(fs, OPF, "application/xhtml+xml\" properties=\"nav\"", "text/html\" properties=\"nav\""); }
    n2_nav_in_spine(fs)          { add_itemref(fs, "<itemref idref=\"nav\"/>"); }
    n3_extra_landmarks_nav(fs)   { replace(fs, NAV, "</body>", "<nav epub:type=\"landmarks\"><ol><li><a href=\"ch1.xhtml\">Start</a></li></ol></nav></body>"); }
    n3_second_toc_nav(fs)        { replace(fs, NAV, "</body>", "<nav epub:type=\"toc\"><ol><li><a href=\"ch1.xhtml\">Again</a></li></ol></nav></body>"); }
    n4_span_heading(fs)          { replace(fs, NAV, "</ol>", "<li><span>Part I</span></li></ol>"); }
    n5_target_not_in_spine(fs)   { replace(fs, NAV, "</ol>", "<li><a href=\"style.css\">Styles</a></li></ol>"); }
    n6_empty_label(fs)           { replace(fs, NAV, ">Chapter Two</a>", "> </a>"); }
    n8_ncx_present(fs)           {
        add(fs, "OEBPS/toc.ncx", format!("{XML}<ncx xmlns=\"http://www.daisy.org/z3986/2005/ncx/\" version=\"2005-1\"><navMap/></ncx>"));
        add_item(fs, "<item id=\"ncx\" href=\"toc.ncx\" media-type=\"application/x-dtbncx+xml\"/>");
        replace(fs, OPF, "<spine>", "<spine toc=\"ncx\">");
    }

    // ---- V: cover
    v1_cover_id_missing(fs)      { replace(fs, OPF, "id=\"cover\"", "id=\"coverimg\""); }
    v1_cover_wrong_type(fs)      { replace(fs, OPF, "media-type=\"image/png\"", "media-type=\"image/gif\""); }
    v2_cover_property_missing(fs){ replace(fs, OPF, " properties=\"cover-image\"", ""); }
    v3_second_cover_image(fs)    { add_item(fs, "<item id=\"cover2\" href=\"cover.png\" media-type=\"image/png\" properties=\"cover-image\"/>"); }
    v3_meta_cover(fs)            { replace(fs, OPF, "</metadata>", "<meta name=\"cover\" content=\"cover\"/></metadata>"); }
    v5_cover_too_small(fs)       { add(fs, COVER, png(50, 50)); }
    v5_cover_undecodable(fs)     { add(fs, COVER, "not a png"); }

    // ---- L: library metadata
    l1_title_missing(fs)         { replace(fs, OPF, "<dc:title>Test Book</dc:title>", ""); }
    l3_language_invalid(fs)      { replace(fs, OPF, "<dc:language>en</dc:language>", "<dc:language>English??</dc:language>"); }
    l4_unique_identifier_dangling(fs) { replace(fs, OPF, "unique-identifier=\"bookid\"", "unique-identifier=\"missing\""); }

    // ---- X: content documents (injected into ch1 unless noted)
    x1_malformed_xml(fs)         { replace(fs, CH2, "<p>World.</p>", "<p>World."); }
    x2_wrong_namespace(fs)       { replace(fs, CH2, " xmlns=\"http://www.w3.org/1999/xhtml\"", ""); }
    x2_no_body(fs)               { replace(fs, CH2, "<body>", "<div>"); replace(fs, CH2, "</body>", "</div>"); }
    x4_script(fs)                { in_body(fs, "<script>alert(1)</script>"); }
    x4_iframe(fs)                { in_body(fs, "<iframe src=\"ch2.xhtml\"></iframe>"); }
    x4_object(fs)                { in_body(fs, "<object data=\"ch2.xhtml\"></object>"); }
    x4_embed(fs)                 { in_body(fs, "<embed src=\"ch2.xhtml\"/>"); }
    x4_applet(fs)                { in_body(fs, "<applet code=\"A.class\"></applet>"); }
    x4_form(fs)                  { in_body(fs, "<form action=\"x\"><input type=\"text\"/></form>"); }
    x4_meta_http_equiv(fs)       { in_head(fs, "<meta http-equiv=\"refresh\" content=\"0\"/>"); }
    x4_base(fs)                  { in_head(fs, "<base href=\"x/\"/>"); }
    x5_event_handler(fs)         { replace(fs, CH1, "<h1>One</h1>", "<h1 onclick=\"x()\">One</h1>"); }
    x5_javascript_url(fs)        { in_body(fs, "<a href=\"javascript:alert(1)\">x</a>"); }
    x5_srcdoc(fs)                { in_body(fs, "<p srcdoc=\"x\">y</p>"); }
    x6_style_tag_import(fs)      { in_head(fs, "<style>@import url(style.css);</style>"); }
    x6_style_attr_remote_url(fs) { in_body(fs, "<p style=\"background:url(http://example.com/a.png)\">x</p>"); }
    x6_style_attr_javascript_url(fs) { in_body(fs, "<p style=\"background:url(javascript:alert(1))\">x</p>"); }
    x6_style_attr_expression(fs) { in_body(fs, "<p style=\"width:expression(alert(1))\">x</p>"); }
    x6_style_attr_behavior(fs)   { in_body(fs, "<p style=\"behavior:url(x.htc)\">x</p>"); }
    x7_svg_script(fs)            { in_body(fs, "<svg xmlns=\"http://www.w3.org/2000/svg\"><script>1</script></svg>"); }
    x7_svg_foreign_object(fs)    { in_body(fs, "<svg xmlns=\"http://www.w3.org/2000/svg\"><foreignObject/></svg>"); }
    x7_svg_event_handler(fs)     { in_body(fs, "<svg xmlns=\"http://www.w3.org/2000/svg\" onload=\"x()\"></svg>"); }
    x7_svg_external_ref(fs)      { in_body(fs, "<svg xmlns=\"http://www.w3.org/2000/svg\" xmlns:xlink=\"http://www.w3.org/1999/xlink\"><image xlink:href=\"http://example.com/a.png\"/></svg>"); }
    x8_epub_switch(fs)           { in_body(fs, "<epub:switch><epub:case required-namespaces=\"http://www.w3.org/1998/Math/MathML\">m</epub:case><epub:default>d</epub:default></epub:switch>"); }
    x9_mathml(fs)                { in_body(fs, "<math xmlns=\"http://www.w3.org/1998/Math/MathML\"><mi>x</mi></math>"); }
    x10_audio(fs)                { in_body(fs, "<audio controls=\"controls\">fallback text</audio>"); }
    x10_video(fs)                { in_body(fs, "<video controls=\"controls\">fallback text</video>"); }

    // ---- R: resource references
    r1_data_uri_disallowed_type(fs) { in_body(fs, "<img src=\"data:text/html;base64,AAAA\" alt=\"a\"/>"); }
    r2_remote_img(fs)            { in_body(fs, "<img src=\"http://example.com/a.png\" alt=\"a\"/>"); }
    r2_protocol_relative_img(fs) { in_body(fs, "<img src=\"//example.com/a.png\" alt=\"a\"/>"); }
    r2_remote_link_css(fs)       { in_head(fs, "<link rel=\"stylesheet\" href=\"https://example.com/s.css\"/>"); }
    r3_dangling_img(fs)          { in_body(fs, "<img src=\"missing.png\" alt=\"a\"/>"); }
    r3_img_escapes_root(fs)      { in_body(fs, "<img src=\"../../x.png\" alt=\"a\"/>"); }
    r4_img_of_disallowed_type(fs){ add(fs, "OEBPS/clip.mp3", "ID3"); in_body(fs, "<img src=\"clip.mp3\" alt=\"a\"/>"); }
    r5_resource_not_in_manifest(fs) { add(fs, "OEBPS/extra.png", png(120, 120)); in_body(fs, "<img src=\"extra.png\" alt=\"a\"/>"); }
    r6_internal_link_missing(fs) { in_body(fs, "<a href=\"nope.xhtml#x\">x</a>"); }

    // ---- Y: css
    y1_unparseable_css(fs)       { add_css(fs, "p { color: ; {{{ }"); }
    y2_import(fs)                { add_css(fs, "@import url(\"other.css\");"); }
    y2_remote_url(fs)            { add_css(fs, "p { background: url(http://example.com/a.png); }"); }
    y2_expression(fs)            { add_css(fs, "p { width: expression(1); }"); }
    y2_behavior(fs)              { add_css(fs, "p { behavior: url(x.htc); }"); }
    y2_moz_binding(fs)           { add_css(fs, "p { -moz-binding: url(x.xml#y); }"); }
    y3_dangling_url(fs)          { add_css(fs, "p { background: url(missing.png); }"); }
    y4_disallowed_at_rule(fs)    { add_css(fs, "@keyframes k { from { color: red; } to { color: blue; } }"); }

    // ---- last: byte-level and destructive breakers (these mask others, so they go at the end)
    x1_latin1_not_utf8(fs)       { replace(fs, CH2, "World.", "Café"); if let Some(f) = fs.iter_mut().find(|f| f.name == CH2) { f.data = String::from_utf8_lossy(&f.data).chars().map(|c| c as u8).collect(); } }
    x1_bom(fs)                   { if let Some(f) = fs.iter_mut().find(|f| f.name == CH2) { f.data.splice(0..0, [0xEF, 0xBB, 0xBF]); } }
    v4_no_cover(fs)              { replace(fs, OPF, "<item id=\"cover\" href=\"cover.png\" media-type=\"image/png\" properties=\"cover-image\"/>", ""); remove(fs, COVER); }
    n7_no_entries(fs)            { text(fs, NAV, |s| cut(s, "<ol>", "</ol>").replacen("</nav>", "<ol></ol></nav>", 1)); }
    s1_empty_spine(fs)           { text(fs, OPF, |s| cut(s, "<spine", "</spine>").replacen("</manifest>", "</manifest>\n<spine></spine>", 1)); }
    o3_no_manifest(fs)           { text(fs, OPF, |s| cut(s, "<manifest>", "</manifest>")); }
    o4_no_spine(fs)              { text(fs, OPF, |s| cut(s, "<spine", "</spine>")); }
    o1_opf_malformed(fs)         { replace(fs, OPF, "</package>", ""); }
    o1_root_not_package(fs)      { replace(fs, OPF, "<package", "<book"); replace(fs, OPF, "</package>", "</book>"); }
    c1_container_missing(fs)     { remove(fs, CONTAINER); }
    c1_container_malformed(fs)   { replace(fs, CONTAINER, "</container>", ""); }
    z5_mimetype_missing(fs)      { remove(fs, "mimetype"); }
}

// Too big to include in all_issues.epub.
fn z4_entry_too_large(fs: &mut Files) {
    // stored (ratio 1:1) so only the per-entry limit (100 MB) is violated
    fs.push(F { name: "OEBPS/big.bin".into(), data: vec![7; 1 << 20], kind: Kind::Stored, repeat: 101 });
}
fn z4_total_too_large(fs: &mut Files) {
    // 6 x 90 MB of zeros, deflated: each entry is under 100 MB, total is 540 MB
    // (the ratio limit is also exceeded; zeros are the only way to do this without a 540 MB file)
    for i in 0..6 {
        fs.push(F { name: format!("OEBPS/big{i}.bin"), data: vec![0; 1 << 20], kind: Kind::Deflate, repeat: 90 });
    }
}
const HUGE: &[(&str, fn(&mut Files))] = &[
    ("z4_entry_too_large", z4_entry_too_large),
    ("z4_total_too_large", z4_total_too_large),
];

// ---------------------------------------------------------------- zip writer

fn write_zip(path: &Path, files: &Files) {
    let mut zw = ZipWriter::new(fs::File::create(path).unwrap());
    for f in files {
        let o = SimpleFileOptions::default();
        match f.kind {
            Kind::Symlink => {
                zw.add_symlink(f.name.as_str(), String::from_utf8(f.data.clone()).unwrap(), o).unwrap();
                continue;
            }
            Kind::Stored => zw.start_file(f.name.as_str(), o.compression_method(CompressionMethod::Stored)),
            Kind::Deflate => zw.start_file(f.name.as_str(), o.compression_method(CompressionMethod::Deflated)),
            Kind::Encrypted => zw.start_file(f.name.as_str(), o.with_deprecated_encryption(b"pw").unwrap()),
        }
        .unwrap();
        for _ in 0..f.repeat {
            zw.write_all(&f.data).unwrap();
        }
    }
    zw.finish().unwrap();
}

// ---------------------------------------------------------------- the test

#[test]
fn generate_broken_epubs() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("target/broken_epubs");
    fs::create_dir_all(&dir).unwrap();
    let out = |name: &str| dir.join(format!("{name}.epub"));

    // 1 zip that should pass
    write_zip(&out("valid"), &valid());

    // 1 error per zip
    for (name, breaker) in BREAKERS.iter().chain(HUGE) {
        let mut files = valid();
        breaker(&mut files);
        assert!(files != valid(), "{name} did not change the book (anchor text not found?)");
        write_zip(&out(name), &files);
    }

    // Z1: not a readable zip
    fs::write(out("z1_not_a_zip"), b"this is not a zip").unwrap();
    let bytes = fs::read(out("valid")).unwrap();
    fs::write(out("z1_truncated_zip"), &bytes[..bytes.len() / 2]).unwrap();

    // 1 zip with everything broken at once
    let mut all = valid();
    for (_, breaker) in BREAKERS {
        breaker(&mut all);
    }
    write_zip(&out("all_issues"), &all);
}