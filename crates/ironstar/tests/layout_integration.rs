//! Integration tests for base layout template.
//!
//! These tests verify the complete HTML document structure produced by
//! `base_layout()`, ensuring all required elements are present and properly
//! formatted for browser consumption.

use hypertext::prelude::*;
use ironstar::infrastructure::assets::AssetManifest;
use ironstar::presentation::layout::base_layout;

#[test]
fn base_layout_produces_valid_html_document() {
    let manifest = AssetManifest::default();
    let content = maud! {
        main {
            h1 { "Test Page" }
            p { "Test content" }
        }
    };

    let html = base_layout(&manifest, content).render();
    let body = html.as_inner();

    // Verify DOCTYPE is present and at the start
    assert!(
        body.contains("<!DOCTYPE html>"),
        "Missing DOCTYPE declaration"
    );

    // Verify html element with lang attribute
    assert!(
        body.contains(r#"<html lang="en">"#),
        "Missing html element with lang attribute"
    );

    // Verify closing html tag
    assert!(body.contains("</html>"), "Missing closing html tag");
}

#[test]
fn base_layout_head_contains_required_meta() {
    let manifest = AssetManifest::default();
    let content = maud! { main { "test" } };

    let html = base_layout(&manifest, content).render();
    let body = html.as_inner();

    // Charset meta
    assert!(
        body.contains(r#"<meta charset="utf-8">"#),
        "Missing charset meta"
    );

    // Viewport meta
    assert!(
        body.contains(r#"name="viewport""#),
        "Missing viewport meta name"
    );
    assert!(
        body.contains(r#"content="width=device-width, initial-scale=1""#),
        "Missing viewport content"
    );
}

#[test]
fn base_layout_includes_css_bundle() {
    let manifest = AssetManifest::default();
    let content = maud! { main { "test" } };

    let html = base_layout(&manifest, content).render();
    let body = html.as_inner();

    // CSS link element
    assert!(body.contains(r#"<link"#), "Missing link element");
    assert!(
        body.contains(r#"rel="stylesheet""#),
        "Missing stylesheet rel"
    );
    // With default manifest, filename is unchanged
    assert!(
        body.contains("bundle.css") || body.contains(".css"),
        "Missing CSS file reference"
    );
}

#[test]
fn base_layout_includes_datastar_script() {
    let manifest = AssetManifest::default();
    let content = maud! { main { "test" } };

    let html = base_layout(&manifest, content).render();
    let body = html.as_inner();

    // Datastar script
    assert!(body.contains("datastar.js"), "Missing datastar.js script");
    assert!(
        body.contains(r#"type="module""#),
        "Missing module type on script"
    );
    assert!(body.contains("defer"), "Missing defer attribute on script");
}

#[test]
fn base_layout_renders_content_in_body() {
    let manifest = AssetManifest::default();
    let content = maud! {
        main #content {
            h1 { "Page Title" }
            p { "Page paragraph content" }
        }
    };

    let html = base_layout(&manifest, content).render();
    let body = html.as_inner();

    // Content should be inside body
    assert!(
        body.contains("<body>") || body.contains("<body "),
        "Missing body tag"
    );
    assert!(body.contains("</body>"), "Missing closing body tag");

    // Specific content elements
    assert!(
        body.contains(r#"<main id="content">"#),
        "Missing main element with id"
    );
    assert!(body.contains("<h1>Page Title</h1>"), "Missing h1 content");
    assert!(
        body.contains("<p>Page paragraph content</p>"),
        "Missing paragraph content"
    );
}

#[test]
#[allow(clippy::expect_used)]
fn base_layout_resolves_hashed_css_filename() {
    // Simulate production manifest with hashed filenames
    let json = r#"{"bundle.css": "bundle-a1b2c3d4.css", "bundle.js": "bundle-e5f6g7h8.js"}"#;
    let manifest = AssetManifest::from_json(json).expect("valid JSON");
    let content = maud! { main { "test" } };

    let html = base_layout(&manifest, content).render();
    let body = html.as_inner();

    // Should use hashed filename from manifest
    assert!(
        body.contains("bundle-a1b2c3d4.css"),
        "Should use hashed CSS filename from manifest"
    );
    // Should NOT contain unhashed filename
    assert!(
        !body.contains(r#"href="/static/bundle.css""#),
        "Should not use unhashed filename when manifest provides hash"
    );
}

#[test]
#[allow(clippy::expect_used)]
fn base_layout_structure_order() {
    let manifest = AssetManifest::default();
    let content = maud! { main { "test" } };

    let html = base_layout(&manifest, content).render();
    let body = html.as_inner();

    // Verify order: DOCTYPE -> html -> head -> body
    let doctype_pos = body.find("<!DOCTYPE").expect("DOCTYPE not found");
    let html_pos = body.find("<html").expect("html not found");
    let head_pos = body.find("<head>").expect("head not found");
    let body_pos = body.find("<body").expect("body not found");

    assert!(
        doctype_pos < html_pos,
        "DOCTYPE should come before html element"
    );
    assert!(html_pos < head_pos, "html should come before head");
    assert!(head_pos < body_pos, "head should come before body");
}

/// In debug builds, hotreload div should be present.
/// In release builds, it should not be present.
#[test]
fn base_layout_hotreload_conditional() {
    let manifest = AssetManifest::default();
    let content = maud! { main { "test" } };

    let html = base_layout(&manifest, content).render();
    let body = html.as_inner();

    // The presence of hotreload div depends on debug_assertions
    #[cfg(debug_assertions)]
    {
        assert!(
            body.contains("data-init"),
            "Debug build should include hotreload data-init"
        );
        assert!(
            body.contains("@get('/reload'"),
            "Debug build should include reload endpoint"
        );
    }

    #[cfg(not(debug_assertions))]
    {
        assert!(
            !body.contains("data-init"),
            "Release build should not include hotreload data-init"
        );
    }
}
