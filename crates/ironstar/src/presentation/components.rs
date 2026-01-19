//! Reusable hypertext component functions.
//!
//! Components are pure functions returning `impl Renderable` for lazy rendering.
//! Each component generates valid HTML with CSS classes from Open Props UI.

use hypertext::prelude::*;

/// Button component.
///
/// # Arguments
///
/// * `text` - Button label
/// * `variant` - CSS variant: "filled", "outlined", "tonal", "elevated"
/// * `size` - Optional: "small" or "large"
/// * `extra_attrs` - Raw HTML attributes string (e.g., `r#"data-on:click="@post('/api/save')""#`)
///
/// # Example
///
/// ```no_run
/// use ironstar::presentation::components::button;
/// let btn = button("Save", "filled", None, r#"data-on:click="@post('/api/save')""#);
/// ```
pub fn button(text: &str, variant: &str, size: Option<&str>, extra_attrs: &str) -> impl Renderable {
    let class = match size {
        Some(s) => format!("button {} {}", variant, s),
        None => format!("button {}", variant),
    };

    // Construct full HTML manually for attribute flexibility
    let html = if extra_attrs.is_empty() {
        format!(r#"<button class="{}">{}</button>"#, class, text)
    } else {
        format!(
            r#"<button class="{}" {}>{}</button>"#,
            class, extra_attrs, text
        )
    };

    maud! {
        (PreEscaped(html))
    }
}

/// Text field component with floating label.
///
/// # Arguments
///
/// * `name` - Input name attribute
/// * `label` - Floating label text
/// * `placeholder` - Placeholder text (use " " for floating label to work)
/// * `variant` - CSS variant: "filled", "outlined"
/// * `extra_attrs` - Raw HTML attributes string for the input element
///
/// # Example
///
/// ```no_run
/// use ironstar::presentation::components::text_field;
/// let field = text_field("email", "Email", " ", "outlined", r#"data-model="email""#);
/// ```
pub fn text_field(
    name: &str,
    label: &str,
    placeholder: &str,
    variant: &str,
    extra_attrs: &str,
) -> impl Renderable {
    let class = format!("field {}", variant);
    let placeholder = if placeholder.is_empty() {
        " "
    } else {
        placeholder
    };

    let input_html = if extra_attrs.is_empty() {
        format!(
            r#"<input type="text" name="{}" placeholder="{}">"#,
            name, placeholder
        )
    } else {
        format!(
            r#"<input type="text" name="{}" placeholder="{}" {}>"#,
            name, placeholder, extra_attrs
        )
    };

    maud! {
        div class=(class) {
            (PreEscaped(input_html))
            label class="label" { (label) }
        }
    }
}

/// Checkbox component.
///
/// # Arguments
///
/// * `name` - Input name attribute
/// * `checked` - Whether the checkbox is checked
/// * `extra_attrs` - Raw HTML attributes string
///
/// # Example
///
/// ```no_run
/// use ironstar::presentation::components::checkbox;
/// let cb = checkbox("agree", false, r#"data-model="agreed""#);
/// ```
pub fn checkbox(name: &str, checked: bool, extra_attrs: &str) -> impl Renderable {
    let checked_attr = if checked { "checked" } else { "" };

    let html = if extra_attrs.is_empty() {
        format!(
            r#"<input type="checkbox" name="{}" {}>"#,
            name, checked_attr
        )
    } else {
        format!(
            r#"<input type="checkbox" name="{}" {} {}>"#,
            name, checked_attr, extra_attrs
        )
    };

    maud! {
        (PreEscaped(html))
    }
}

/// Loading spinner component with signal-based visibility.
///
/// The spinner is shown/hidden via Datastar's `data-show` directive.
///
/// # Arguments
///
/// * `signal_name` - Name of the Datastar signal controlling visibility
///
/// # Example
///
/// ```no_run
/// use ironstar::presentation::components::loading_spinner;
/// let spinner = loading_spinner("isLoading");
/// // Renders: <span class="loading-spinner" data-show="$isLoading" ...>
/// ```
pub fn loading_spinner(signal_name: &str) -> impl Renderable {
    let data_show = format!("${}", signal_name);
    let svg = r#"<svg class="spinner" viewBox="0 0 24 24" width="20" height="20"><circle cx="12" cy="12" r="10" stroke="currentColor" stroke-width="3" fill="none" opacity="0.3"/><path d="M12 2a10 10 0 0 1 10 10" stroke="currentColor" stroke-width="3" fill="none" stroke-linecap="round"/></svg>"#;

    maud! {
        span
            class="loading-spinner"
            "data-show"=(data_show)
            "aria-label"="Loading"
        {
            (PreEscaped(svg))
        }
    }
}

/// Icon component for Lucide icons.
///
/// Icons are rendered at build time via Lucide integration.
/// This component provides the markup structure for icon placement.
///
/// # Arguments
///
/// * `name` - Icon name (e.g., "check", "trash-2", "plus")
/// * `size` - Optional: "small" or "large"
///
/// # Example
///
/// ```no_run
/// use ironstar::presentation::components::icon;
/// let icon_elem = icon("check", Some("small"));
/// ```
pub fn icon(name: &str, size: Option<&str>) -> impl Renderable {
    let class = match size {
        Some(s) => format!("icon {}", s),
        None => "icon".to_string(),
    };

    maud! {
        span class=(class) "data-icon"=(name) { (name) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn button_renders_with_variant() {
        let btn = button("Save", "filled", None, "").render();
        let html = btn.as_inner();

        assert!(html.contains(r#"class="button filled""#));
        assert!(html.contains(">Save</button>"));
    }

    #[test]
    fn button_renders_with_size() {
        let btn = button("Submit", "outlined", Some("large"), "").render();
        let html = btn.as_inner();

        assert!(html.contains(r#"class="button outlined large""#));
    }

    #[test]
    fn button_renders_with_extra_attrs() {
        let btn = button("Click", "tonal", None, r#"data-on:click="@post('/api')""#).render();
        let html = btn.as_inner();

        assert!(html.contains(r#"data-on:click="@post('/api')""#));
    }

    #[test]
    fn text_field_renders_structure() {
        let field = text_field("username", "Username", " ", "outlined", "").render();
        let html = field.as_inner();

        assert!(html.contains(r#"class="field outlined""#));
        assert!(html.contains(r#"name="username""#));
        assert!(html.contains(r#"<label class="label">"#));
        assert!(html.contains("Username</label>"));
    }

    #[test]
    fn text_field_renders_with_extra_attrs() {
        let field = text_field("email", "Email", " ", "filled", r#"data-model="email""#).render();
        let html = field.as_inner();

        assert!(html.contains(r#"data-model="email""#));
    }

    #[test]
    fn checkbox_renders_unchecked() {
        let cb = checkbox("agree", false, "").render();
        let html = cb.as_inner();

        assert!(html.contains(r#"type="checkbox""#));
        assert!(html.contains(r#"name="agree""#));
        assert!(!html.contains("checked"));
    }

    #[test]
    fn checkbox_renders_checked() {
        let cb = checkbox("terms", true, "").render();
        let html = cb.as_inner();

        assert!(html.contains("checked"));
    }

    #[test]
    fn checkbox_renders_with_extra_attrs() {
        let cb = checkbox("opt", false, r#"data-model="optIn""#).render();
        let html = cb.as_inner();

        assert!(html.contains(r#"data-model="optIn""#));
    }

    #[test]
    fn loading_spinner_renders_with_signal() {
        let spinner = loading_spinner("isLoading").render();
        let html = spinner.as_inner();

        assert!(html.contains(r#"class="loading-spinner""#));
        assert!(html.contains(r#"data-show="$isLoading""#));
        assert!(html.contains(r#"aria-label="Loading""#));
        assert!(html.contains("<svg"));
    }

    #[test]
    fn icon_renders_with_name() {
        let ic = icon("check", None).render();
        let html = ic.as_inner();

        assert!(html.contains(r#"class="icon""#));
        assert!(html.contains(r#"data-icon="check""#));
    }

    #[test]
    fn icon_renders_with_size() {
        let ic = icon("trash-2", Some("small")).render();
        let html = ic.as_inner();

        assert!(html.contains(r#"class="icon small""#));
    }
}
