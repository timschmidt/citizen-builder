//! Syntax highlighting for generated code using syntect.

use egui::Color32;
use syntect::easy::HighlightLines;
use syntect::highlighting::{Style, ThemeSet};
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;

/// Cached syntax highlighting resources.
pub struct Highlighter {
    syntax_set: SyntaxSet,
    theme_set: ThemeSet,
    theme_name: String,
}

impl Default for Highlighter {
    fn default() -> Self {
        Self::new()
    }
}

impl Highlighter {
    /// Creates a highlighter with syntect's default syntax and theme sets.
    ///
    /// The current theme is `base16-ocean.dark`, falling back to the first
    /// available syntect theme if that theme is unavailable.
    pub fn new() -> Self {
        Self {
            syntax_set: SyntaxSet::load_defaults_newlines(),
            theme_set: ThemeSet::load_defaults(),
            theme_name: "base16-ocean.dark".to_string(),
        }
    }

    /// Highlights Rust source and returns `(text, color)` spans.
    ///
    /// The returned text fragments preserve the original source text,
    /// including line endings, so callers can concatenate the span text to
    /// recover the input.
    pub fn highlight_rust(&self, code: &str) -> Vec<(String, Color32)> {
        let syntax = self
            .syntax_set
            .find_syntax_by_extension("rs")
            .unwrap_or_else(|| self.syntax_set.find_syntax_plain_text());

        let theme = self
            .theme_set
            .themes
            .get(&self.theme_name)
            .unwrap_or_else(|| {
                self.theme_set
                    .themes
                    .values()
                    .next()
                    .expect("No themes available")
            });

        let mut highlighter = HighlightLines::new(syntax, theme);
        let mut result = Vec::new();

        for line in LinesWithEndings::from(code) {
            match highlighter.highlight_line(line, &self.syntax_set) {
                Ok(ranges) => {
                    for (style, text) in ranges {
                        let color = style_to_color32(style);
                        result.push((text.to_string(), color));
                    }
                }
                Err(_) => {
                    // Fallback to plain text on error
                    result.push((line.to_string(), Color32::LIGHT_GRAY));
                }
            }
        }

        result
    }

    /// Renders highlighted Rust source as an [`egui::text::LayoutJob`].
    pub fn layout_job(&self, code: &str) -> egui::text::LayoutJob {
        let mut job = egui::text::LayoutJob::default();

        for (text, color) in self.highlight_rust(code) {
            job.append(
                &text,
                0.0,
                egui::TextFormat {
                    font_id: egui::FontId::monospace(12.0),
                    color,
                    ..Default::default()
                },
            );
        }

        job
    }
}

/// Convert syntect Style to egui Color32.
fn style_to_color32(style: Style) -> Color32 {
    Color32::from_rgb(style.foreground.r, style.foreground.g, style.foreground.b)
}

/// Shows a read-only, scrollable code viewer with Rust syntax highlighting.
#[allow(dead_code)]
pub fn code_viewer(ui: &mut egui::Ui, highlighter: &Highlighter, code: &str) {
    let job = highlighter.layout_job(code);

    egui::ScrollArea::vertical()
        .id_salt("highlighted_code_scroll")
        .max_height(280.0)
        .auto_shrink([false, false])
        .show(ui, |ui| {
            // Use a Label with the layout job for syntax-highlighted display
            ui.add(egui::Label::new(job).selectable(true));
        });
}

/// Shows an editable code editor styled for Rust source.
///
/// Returns `true` when the text was modified. This uses egui's standard code
/// editor styling rather than live syntax highlighting because highlighting
/// every edit can be expensive for large generated files.
#[allow(dead_code)]
pub fn code_editor_highlighted(
    ui: &mut egui::Ui,
    _highlighter: &Highlighter,
    code: &mut String,
) -> bool {
    let mut changed = false;

    egui::ScrollArea::vertical()
        .id_salt("code_editor_scroll")
        .max_height(280.0)
        .auto_shrink([false, false])
        .show(ui, |ui| {
            // For editing, we use a regular TextEdit with code_editor styling
            // Syntax highlighting on edit is expensive, so we show it read-only
            // The user can toggle between edit and view modes
            let response = ui.add(
                egui::TextEdit::multiline(code)
                    .id_salt("highlight_code_editor")
                    .code_editor()
                    .desired_rows(18)
                    .desired_width(f32::INFINITY),
            );
            changed = response.changed();
        });

    changed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_highlighter_creation() {
        let highlighter = Highlighter::new();
        // Should not panic
        let _ = highlighter.highlight_rust("fn main() {}");
    }

    #[test]
    fn test_highlight_rust_basic() {
        let highlighter = Highlighter::new();
        let code = "fn main() {\n    println!(\"Hello\");\n}\n";
        let spans = highlighter.highlight_rust(code);
        assert!(!spans.is_empty());
        let reconstructed: String = spans.iter().map(|(text, _)| text.as_str()).collect();
        assert_eq!(reconstructed, code);
    }

    #[test]
    fn test_layout_job() {
        let highlighter = Highlighter::new();
        let code = "let x = 42;";
        let job = highlighter.layout_job(code);
        assert!(!job.text.is_empty());
        assert_eq!(job.text, code);
    }
}
