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

use crate::domain::signals::ChartSignals;

/// Escapes a string for safe use in HTML attribute context.
///
/// This escapes `<`, `>`, `&`, `"`, and `'` to their HTML entity equivalents.
fn escape_html_attr(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
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
/// The chart ID and height are HTML-escaped before interpolation.
/// The `data-signals` attribute embeds JSON which is also HTML-escaped.
pub fn echarts_chart(
    id: &str,
    signals: &ChartSignals,
    height: &str,
) -> Raw<String> {
    let signals_json = serde_json::to_string(signals).unwrap_or_else(|_| "{}".to_string());
    let escaped_id = escape_html_attr(id);
    let escaped_height = escape_html_attr(height);
    let escaped_signals = escape_html_attr(&signals_json);

    let html = format!(
        r##"<div id="{escaped_id}" class="chart-container" data-signals="{escaped_signals}">
    <div class="chart-loading" style="display: none;" data-show="$loading">
        <span class="loading-spinner"></span>
        Loading...
    </div>
    <div class="chart-error" style="display: none;" data-show="$error">
        <span data-text="$error"></span>
    </div>
    <ds-echarts
        data-ignore-morph="true"
        data-attr:option="JSON.stringify($chartOption)"
        data-on:chart-ready="console.log('Chart ready:', evt.detail)"
        data-on:chart-click="$selected = evt.detail"
        data-on:chart-error="$error = evt.detail.message"
        style="height: {escaped_height}; width: 100%; display: block;">
    </ds-echarts>
</div>"##
    );

    Raw::dangerously_create(html)
}

/// Renders a chart page with SSE connection for data streaming.
///
/// This template provides the outer page structure for chart-focused views,
/// establishing the SSE connection that will deliver chart data via signal
/// updates.
///
/// # Arguments
///
/// * `title` - Page title displayed as h1
/// * `chart_id` - ID for the chart container (used in DOM targeting)
/// * `sse_endpoint` - SSE endpoint path (e.g., "/api/charts/astronauts/data")
///
/// # SSE initialization
///
/// The `data-on-load` directive triggers a GET request to the SSE endpoint
/// when the page loads. The server responds with signal updates containing
/// the chart configuration.
pub fn chart_page(title: &str, chart_id: &str, sse_endpoint: &str) -> Raw<String> {
    let escaped_title = escape_html_attr(title);
    let escaped_chart_id = escape_html_attr(chart_id);
    let escaped_endpoint = escape_html_attr(sse_endpoint);

    let html = format!(
        r##"<main class="chart-page" style="max-width: var(--size-content-3); margin-inline: auto; padding: var(--size-4);" data-on-load="@get('{escaped_endpoint}')">
    <h1>{escaped_title}</h1>
    <div id="{escaped_chart_id}-container">
        <p class="loading-placeholder">Loading chart...</p>
    </div>
</main>"##
    );

    Raw::dangerously_create(html)
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
) -> Raw<String> {
    let signals_json = serde_json::to_string(signals).unwrap_or_else(|_| "{}".to_string());
    let escaped_id = escape_html_attr(id);
    let escaped_height = escape_html_attr(height);
    let escaped_signals = escape_html_attr(&signals_json);

    let html = format!(
        r##"<div id="{escaped_id}" class="chart-with-feedback" data-signals="{escaped_signals}">
    <div class="chart-loading" style="display: none;" data-show="$loading">
        <span class="loading-spinner"></span>
        Loading...
    </div>
    <div class="chart-error" style="display: none;" data-show="$error">
        <span data-text="$error"></span>
    </div>
    <ds-echarts
        data-ignore-morph="true"
        data-attr:option="JSON.stringify($chartOption)"
        data-on:chart-ready="console.log('Chart ready:', evt.detail)"
        data-on:chart-click="$selected = evt.detail"
        data-on:chart-error="$error = evt.detail.message"
        style="height: {escaped_height}; width: 100%; display: block;">
    </ds-echarts>
    <div class="selection-feedback" style="margin-top: var(--size-3); padding: var(--size-3); background: var(--surface-2); border-radius: var(--radius-2);" data-show="$selected">
        <p>
            <strong>Selected: </strong>
            <span data-text="$selected ? $selected.name + ': ' + $selected.value : ''"></span>
        </p>
    </div>
</div>"##
    );

    Raw::dangerously_create(html)
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

    #[test]
    fn chart_page_renders_with_sse_endpoint() {
        let raw = chart_page("Test Chart", "astronauts", "/api/charts/astronauts/data");
        let html = raw.render();
        let body = html.as_inner();

        assert!(body.contains(r#"@get('/api/charts/astronauts/data')"#));
        assert!(body.contains("data-on-load"));
    }

    #[test]
    fn chart_page_renders_title() {
        let raw = chart_page("Astronaut Statistics", "astronauts", "/api/data");
        let html = raw.render();
        let body = html.as_inner();

        assert!(body.contains("<h1>"));
        assert!(body.contains("Astronaut Statistics"));
    }

    #[test]
    fn chart_page_renders_container_with_id() {
        let raw = chart_page("Test", "my-chart", "/api/data");
        let html = raw.render();
        let body = html.as_inner();

        assert!(body.contains(r#"id="my-chart-container""#));
    }

    #[test]
    fn chart_page_escapes_xss_in_title() {
        let raw = chart_page("<script>alert(1)</script>", "test", "/api/data");
        let html = raw.render();
        let body = html.as_inner();

        // Script tag should be escaped in the title
        assert!(!body.contains("<script>alert(1)</script>"));
        assert!(body.contains("&lt;script&gt;"));
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

    #[test]
    fn escape_html_attr_escapes_all_dangerous_chars() {
        let dangerous = r#"<script>"'&</script>"#;
        let escaped = escape_html_attr(dangerous);

        assert!(!escaped.contains('<'));
        assert!(!escaped.contains('>'));
        assert!(!escaped.contains('"'));
        assert!(!escaped.contains('\''));
        // Original & should be escaped, but we shouldn't have raw &
        assert!(escaped.contains("&lt;"));
        assert!(escaped.contains("&gt;"));
        assert!(escaped.contains("&quot;"));
        assert!(escaped.contains("&#x27;"));
        assert!(escaped.contains("&amp;"));
    }
}
