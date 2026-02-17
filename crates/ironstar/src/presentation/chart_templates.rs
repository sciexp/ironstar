//! ECharts hypertext templates for Datastar-driven chart UI.
//!
//! These templates render HTML fragments for the ds-echarts Lit web component,
//! integrating Apache ECharts with Datastar's signal-driven architecture.
//!
//! # Design principles
//!
//! ds-echarts forms a Moore machine coalgebra where the Lit `updated()` lifecycle
//! implements the transition function and `render()` produces output. The
//! `data-ignore-morph` attribute ensures bisimulation equivalence by preventing
//! Datastar from morphing ECharts' internal DOM state.
//!
//! # Datastar integration
//!
//! Chart state flows through Datastar signals:
//! - `$chartOption`: ECharts configuration object (server → browser)
//! - `$selected`: Currently selected data point (browser → server)
//! - `$loading`: Loading indicator state
//! - `$error`: Error message when chart operations fail
//!
//! # Example
//!
//! ```rust,ignore
//! use ironstar::presentation::chart_templates::echarts_chart;
//! use ironstar::domain::signals::ChartSignals;
//!
//! let signals = ChartSignals {
//!     chart_option: serde_json::json!({
//!         "xAxis": {"type": "category", "data": ["A", "B", "C"]},
//!         "yAxis": {"type": "value"},
//!         "series": [{"type": "bar", "data": [10, 20, 30]}]
//!     }),
//!     selected: None,
//!     loading: false,
//!     error: None,
//! };
//!
//! let html = echarts_chart("my-chart", &signals, "400px").render();
//! ```

use hypertext::Raw;
use hypertext::prelude::*;

use crate::domain::signals::ChartSignals;
use crate::infrastructure::assets::AssetManifest;
use crate::presentation::layout::base_layout;

/// Renders the ds-echarts custom element with Datastar attributes.
///
/// Custom elements with hyphens in their names cannot be expressed directly
/// in maud syntax, so we construct the element as raw HTML. The height
/// parameter must be pre-escaped by the caller.
fn ds_echarts_element(height: &str) -> Raw<String> {
    // XSS SAFETY: height is interpolated into a style attribute; callers must
    // ensure it contains only safe CSS values. The other attributes are static.
    let html = format!(
        r#"<ds-echarts data-ignore-morph="true" data-attr:option="JSON.stringify($chartOption)" data-on:chart-ready="console.log('Chart ready:', evt.detail)" data-on:chart-click="$selected = evt.detail" data-on:chart-error="$error = evt.detail.message" style="height: {height}; width: 100%; display: block;"></ds-echarts>"#
    );
    Raw::dangerously_create(html)
}

/// Renders a ds-echarts component with Datastar signal bindings.
///
/// # Arguments
///
/// * `id` - Unique element ID for the chart container
/// * `signals` - Initial chart signals (chartOption, selected, loading, error)
/// * `height` - CSS height value (e.g., "400px", "100%")
///
/// # Datastar attributes
///
/// - `data-signals`: JSON-stringified ChartSignals for initial state
/// - `data-attr:option`: Binds $chartOption signal to component
/// - `data-ignore-morph`: Prevents Datastar from morphing ECharts DOM
/// - Event handlers for chart-ready, chart-click, chart-error
///
/// # XSS safety
///
/// maud automatically escapes all interpolated values for XSS safety.
/// The ds-echarts custom element is rendered via a helper function since
/// custom element names with hyphens cannot be expressed in maud syntax.
pub fn echarts_chart(id: &str, signals: &ChartSignals, height: &str) -> impl Renderable {
    let signals_json = serde_json::to_string(signals).unwrap_or_else(|_| "{}".to_string());

    maud! {
        div
            id=(id)
            class="chart-container"
            "data-signals"=(signals_json)
        {
            div
                class="chart-loading"
                style="display: none;"
                "data-show"="$loading"
            {
                span class="loading-spinner" {}
                "Loading..."
            }
            div
                class="chart-error"
                style="display: none;"
                "data-show"="$error"
            {
                span "data-text"="$error" {}
            }
            (ds_echarts_element(height))
        }
    }
}

/// Renders a chart page with SSE connection for data streaming.
///
/// This template provides the outer page structure for chart-focused views,
/// establishing the SSE connection that will deliver chart data via signal
/// updates.
///
/// # Arguments
///
/// * `manifest` - Asset manifest for CSS/JS paths
/// * `title` - Page title displayed as h1
/// * `chart_id` - ID for the chart container (used in DOM targeting)
/// * `sse_endpoint` - SSE endpoint path (e.g., "/api/charts/astronauts/data")
///
/// # SSE initialization
///
/// The `data-init` directive triggers a GET request to the SSE endpoint
/// when the page loads. The server responds with signal updates containing
/// the chart configuration.
pub fn chart_page(
    manifest: &AssetManifest,
    title: &str,
    chart_id: &str,
    sse_endpoint: &str,
) -> impl Renderable {
    let on_load = format!(
        "@get('{}',{{requestCancellation:'disabled'}})",
        sse_endpoint
    );
    let container_id = format!("{}-container", chart_id);

    let content = maud! {
        main
            class="chart-page"
            style="max-width: var(--size-content-3); margin-inline: auto; padding: var(--size-4);"
            "data-init"=(on_load)
        {
            h1 { (title) }
            div id=(container_id) {
                p class="loading-placeholder" { "Loading chart..." }
            }
        }
    };

    base_layout(manifest, content)
}

/// Renders a chart with selection feedback section.
///
/// Extends the basic chart template with a reactive display showing the
/// currently selected data point. Useful for demonstrating chart interaction
/// and for dashboards where selection drives other UI updates.
///
/// # Arguments
///
/// * `id` - Unique element ID for the chart container
/// * `signals` - Initial chart signals
/// * `height` - CSS height value
///
/// # Selection feedback
///
/// When a user clicks a chart element, the `$selected` signal updates and
/// the feedback section displays the selected item's name and value.
pub fn echarts_chart_with_feedback(
    id: &str,
    signals: &ChartSignals,
    height: &str,
) -> impl Renderable {
    let signals_json = serde_json::to_string(signals).unwrap_or_else(|_| "{}".to_string());

    maud! {
        div
            id=(id)
            class="chart-with-feedback"
            "data-signals"=(signals_json)
        {
            div
                class="chart-loading"
                style="display: none;"
                "data-show"="$loading"
            {
                span class="loading-spinner" {}
                "Loading..."
            }
            div
                class="chart-error"
                style="display: none;"
                "data-show"="$error"
            {
                span "data-text"="$error" {}
            }
            (ds_echarts_element(height))
            div
                class="selection-feedback"
                style="margin-top: var(--size-3); padding: var(--size-3); background: var(--surface-2); border-radius: var(--radius-2);"
                "data-show"="$selected"
            {
                p {
                    strong { "Selected: " }
                    span "data-text"="$selected ? $selected.name + ': ' + $selected.value : ''" {}
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hypertext::Renderable;

    fn sample_signals() -> ChartSignals {
        ChartSignals {
            chart_option: serde_json::json!({
                "xAxis": {"type": "category", "data": ["A", "B", "C"]},
                "yAxis": {"type": "value"},
                "series": [{"type": "bar", "data": [10, 20, 30]}]
            }),
            selected: None,
            loading: false,
            error: None,
        }
    }

    #[test]
    fn echarts_chart_renders_with_data_signals() {
        let signals = sample_signals();
        let raw = echarts_chart("test-chart", &signals, "400px");
        let html = raw.render();
        let body = html.as_inner();

        assert!(body.contains("data-signals="));
        assert!(body.contains("chartOption"));
    }

    #[test]
    fn echarts_chart_has_data_ignore_morph() {
        let signals = sample_signals();
        let raw = echarts_chart("test-chart", &signals, "400px");
        let html = raw.render();
        let body = html.as_inner();

        assert!(body.contains(r#"data-ignore-morph="true""#));
    }

    #[test]
    fn echarts_chart_renders_container_id() {
        let signals = sample_signals();
        let raw = echarts_chart("my-unique-chart", &signals, "300px");
        let html = raw.render();
        let body = html.as_inner();

        assert!(body.contains(r#"id="my-unique-chart""#));
    }

    #[test]
    fn echarts_chart_renders_height_style() {
        let signals = sample_signals();
        let raw = echarts_chart("test-chart", &signals, "500px");
        let html = raw.render();
        let body = html.as_inner();

        assert!(body.contains("height: 500px"));
    }

    #[test]
    fn echarts_chart_has_loading_indicator() {
        let signals = sample_signals();
        let raw = echarts_chart("test-chart", &signals, "400px");
        let html = raw.render();
        let body = html.as_inner();

        assert!(body.contains(r#"data-show="$loading""#));
        assert!(body.contains("chart-loading"));
    }

    #[test]
    fn echarts_chart_has_error_display() {
        let signals = sample_signals();
        let raw = echarts_chart("test-chart", &signals, "400px");
        let html = raw.render();
        let body = html.as_inner();

        assert!(body.contains(r#"data-show="$error""#));
        assert!(body.contains(r#"data-text="$error""#));
    }

    #[test]
    fn echarts_chart_has_event_handlers() {
        let signals = sample_signals();
        let raw = echarts_chart("test-chart", &signals, "400px");
        let html = raw.render();
        let body = html.as_inner();

        assert!(body.contains("data-on:chart-ready"));
        assert!(body.contains("data-on:chart-click"));
        assert!(body.contains("data-on:chart-error"));
    }

    #[test]
    fn echarts_chart_escapes_xss_in_id() {
        let signals = sample_signals();
        let raw = echarts_chart("<script>alert(1)</script>", &signals, "400px");
        let html = raw.render();
        let body = html.as_inner();

        // Script tag should be escaped in the id attribute
        assert!(!body.contains("<script>"));
        assert!(body.contains("&lt;script&gt;"));
    }

    fn test_manifest() -> AssetManifest {
        AssetManifest::default()
    }

    #[test]
    fn chart_page_renders_with_sse_endpoint() {
        let manifest = test_manifest();
        let raw = chart_page(
            &manifest,
            "Test Chart",
            "astronauts",
            "/api/charts/astronauts/data",
        );
        let html = raw.render();
        let body = html.as_inner();

        assert!(body.contains(r#"@get('/api/charts/astronauts/data')"#));
        assert!(body.contains("data-init"));
    }

    #[test]
    fn chart_page_renders_title() {
        let manifest = test_manifest();
        let raw = chart_page(&manifest, "Astronaut Statistics", "astronauts", "/api/data");
        let html = raw.render();
        let body = html.as_inner();

        assert!(body.contains("<h1>"));
        assert!(body.contains("Astronaut Statistics"));
    }

    #[test]
    fn chart_page_renders_container_with_id() {
        let manifest = test_manifest();
        let raw = chart_page(&manifest, "Test", "my-chart", "/api/data");
        let html = raw.render();
        let body = html.as_inner();

        assert!(body.contains(r#"id="my-chart-container""#));
    }

    #[test]
    fn chart_page_escapes_xss_in_title() {
        let manifest = test_manifest();
        let raw = chart_page(&manifest, "<script>alert(1)</script>", "test", "/api/data");
        let html = raw.render();
        let body = html.as_inner();

        // Script tag should be escaped in the title
        assert!(!body.contains("<script>alert(1)</script>"));
        assert!(body.contains("&lt;script&gt;"));
    }

    #[test]
    fn chart_page_renders_full_html_document() {
        let manifest = test_manifest();
        let raw = chart_page(&manifest, "Test Chart", "test", "/api/data");
        let html = raw.render();
        let body = html.as_inner();

        // Should have full HTML document structure from base_layout
        assert!(body.starts_with("<!DOCTYPE html>"));
        assert!(body.contains("<html"));
        assert!(body.contains("<head>"));
        assert!(body.contains("<body>"));
    }

    #[test]
    fn echarts_chart_with_feedback_renders_selection_section() {
        let signals = sample_signals();
        let raw = echarts_chart_with_feedback("test-chart", &signals, "400px");
        let html = raw.render();
        let body = html.as_inner();

        assert!(body.contains("selection-feedback"));
        assert!(body.contains(r#"data-show="$selected""#));
        assert!(body.contains("Selected:"));
    }

    #[test]
    fn echarts_chart_with_feedback_has_data_text_binding() {
        let signals = sample_signals();
        let raw = echarts_chart_with_feedback("test-chart", &signals, "400px");
        let html = raw.render();
        let body = html.as_inner();

        assert!(body.contains(r#"data-text="$selected ? $selected.name"#));
    }

    #[test]
    fn echarts_chart_with_feedback_has_all_standard_features() {
        let signals = sample_signals();
        let raw = echarts_chart_with_feedback("test-chart", &signals, "400px");
        let html = raw.render();
        let body = html.as_inner();

        // Should have all features from basic echarts_chart
        assert!(body.contains("data-signals="));
        assert!(body.contains(r#"data-ignore-morph="true""#));
        assert!(body.contains(r#"data-show="$loading""#));
        assert!(body.contains(r#"data-show="$error""#));
    }

    #[test]
    fn data_signals_attribute_escapes_html_in_json() {
        // Verify that HTML characters in signal values are escaped
        // when rendered in the data-signals attribute
        let signals = ChartSignals {
            chart_option: serde_json::json!({
                "title": "<script>alert('xss')</script>"
            }),
            selected: None,
            loading: false,
            error: Some("Error with <html> tags".to_string()),
        };

        let raw = echarts_chart("test", &signals, "400px");
        let html = raw.render();
        let body = html.as_inner();

        // The output HTML should not contain unescaped script tags
        assert!(!body.contains("<script>alert"));
        // The angle brackets should be escaped as HTML entities
        assert!(body.contains("&lt;script&gt;"));
    }
}
