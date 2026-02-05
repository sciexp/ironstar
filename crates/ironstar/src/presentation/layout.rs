//! Base layout template with Datastar initialization.
//!
//! Provides the HTML document structure for all pages, including:
//! - DOCTYPE and html element with lang attribute
//! - Head with charset, viewport, and asset links
//! - Body with optional hot reload div (debug builds only)
//! - Content slot for page-specific markup

use hypertext::prelude::*;

use crate::infrastructure::assets::AssetManifest;

/// Renders the base HTML layout with Datastar initialization.
///
/// This function produces a complete HTML document structure suitable for
/// server-side rendering. The layout includes:
///
/// - `<!DOCTYPE html>` declaration
/// - `<html lang="en">` root element
/// - Head section with charset, viewport meta, CSS bundle link, and Datastar script
/// - Body section with content slot
///
/// In debug builds, the body includes a hot reload div that polls `/reload`
/// for development workflow support.
///
/// # Arguments
///
/// * `manifest` - Asset manifest for resolving hashed filenames
/// * `content` - The page content to render inside the body
///
/// # Example
///
/// ```no_run
/// use hypertext::prelude::*;
/// use ironstar::presentation::layout::base_layout;
/// use ironstar::infrastructure::assets::AssetManifest;
///
/// let manifest = AssetManifest::load();
/// let page_content = maud! {
///     main {
///         h1 { "Welcome" }
///     }
/// };
/// let html = base_layout(&manifest, page_content);
/// let rendered = html.render();
/// ```
pub fn base_layout(manifest: &AssetManifest, content: impl Renderable) -> impl Renderable {
    let css_href = format!("/static/{}", manifest.resolve("bundle.css"));
    let datastar_src = format!("/static/{}", manifest.resolve("datastar.js"));

    maud! {
        !DOCTYPE
        html lang="en" {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1";
                meta name="color-scheme" content="light dark";
                link rel="stylesheet" href=(css_href);
                script defer type="module" src=(datastar_src) {}
            }
            body {
                @if cfg!(debug_assertions) {
                    div "data-init"="@get('/reload', {retryMaxCount: 1000, retryInterval:20, retryMaxWaitMs:200})" {}
                }
                (content)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base_layout_renders_doctype() {
        let manifest = AssetManifest::default();
        let content = maud! { main { "test" } };
        let html = base_layout(&manifest, content).render();
        let body = html.as_inner();

        assert!(body.starts_with("<!DOCTYPE html>"));
    }

    #[test]
    fn base_layout_renders_html_lang() {
        let manifest = AssetManifest::default();
        let content = maud! { main { "test" } };
        let html = base_layout(&manifest, content).render();
        let body = html.as_inner();

        assert!(body.contains(r#"<html lang="en">"#));
    }

    #[test]
    fn base_layout_renders_meta_charset() {
        let manifest = AssetManifest::default();
        let content = maud! { main { "test" } };
        let html = base_layout(&manifest, content).render();
        let body = html.as_inner();

        assert!(body.contains(r#"<meta charset="utf-8">"#));
    }

    #[test]
    fn base_layout_renders_viewport() {
        let manifest = AssetManifest::default();
        let content = maud! { main { "test" } };
        let html = base_layout(&manifest, content).render();
        let body = html.as_inner();

        assert!(body.contains(r#"name="viewport""#));
        assert!(body.contains(r#"content="width=device-width, initial-scale=1""#));
    }

    #[test]
    fn base_layout_renders_color_scheme() {
        let manifest = AssetManifest::default();
        let content = maud! { main { "test" } };
        let html = base_layout(&manifest, content).render();
        let body = html.as_inner();

        assert!(body.contains(r#"name="color-scheme""#));
        assert!(body.contains(r#"content="light dark""#));
    }

    #[test]
    fn base_layout_renders_css_link() {
        let manifest = AssetManifest::default();
        let content = maud! { main { "test" } };
        let html = base_layout(&manifest, content).render();
        let body = html.as_inner();

        // Default manifest returns input unchanged, so should have bundle.css
        assert!(body.contains("bundle.css"));
        assert!(body.contains(r#"rel="stylesheet""#));
    }

    #[test]
    fn base_layout_renders_datastar_script() {
        let manifest = AssetManifest::default();
        let content = maud! { main { "test" } };
        let html = base_layout(&manifest, content).render();
        let body = html.as_inner();

        assert!(body.contains("datastar.js"));
        assert!(body.contains(r#"type="module""#));
        assert!(body.contains("defer"));
    }

    #[test]
    fn base_layout_renders_content() {
        let manifest = AssetManifest::default();
        let content = maud! { main { "Hello World" } };
        let html = base_layout(&manifest, content).render();
        let body = html.as_inner();

        assert!(body.contains("<main>Hello World</main>"));
    }

    #[test]
    #[allow(clippy::expect_used)]
    fn base_layout_uses_manifest_resolved_css() {
        let json = r#"{"bundle.css": "bundle-abc12345.css"}"#;
        let manifest = AssetManifest::from_json(json).expect("valid JSON");
        let content = maud! { main { "test" } };
        let html = base_layout(&manifest, content).render();
        let body = html.as_inner();

        assert!(body.contains("bundle-abc12345.css"));
    }

    #[test]
    #[allow(clippy::expect_used)]
    fn base_layout_uses_manifest_resolved_datastar() {
        let json = r#"{"datastar.js": "datastar-def67890.js"}"#;
        let manifest = AssetManifest::from_json(json).expect("valid JSON");
        let content = maud! { main { "test" } };
        let html = base_layout(&manifest, content).render();
        let body = html.as_inner();

        assert!(body.contains("datastar-def67890.js"));
    }
}
