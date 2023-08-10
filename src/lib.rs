use std::hash::Hash;

use delimiters::{Delimiters, ARRAY_DELIMITERS, OBJECT_DELIMITERS};
use egui::{collapsing_header::CollapsingState, Id, Ui};
use serde_json::Value;

mod delimiters;

pub struct JsonTree {
    id: Id,
    prefix: Option<String>,
    default_open: bool,
}

impl JsonTree {
    pub fn new(id: impl Hash) -> Self {
        Self {
            id: Id::new(id),
            prefix: None,
            default_open: false,
        }
    }

    pub fn default_open(mut self, default_open: bool) -> Self {
        self.default_open = default_open;
        self
    }

    fn prefix(mut self, prefix: String) -> Self {
        self.prefix = Some(prefix);
        self
    }

    pub fn show(mut self, ui: &mut Ui, value: &Value) {
        self.show_inner(ui, &mut vec![], value);
    }

    fn show_inner(&mut self, ui: &mut Ui, path_segments: &mut Vec<String>, value: &Value) {
        let prefix = self
            .prefix
            .as_ref()
            .map(|p| p.to_string())
            .unwrap_or_default();

        match value {
            Value::Null => {
                ui.monospace(format!("{}: null", prefix));
            }
            Value::Bool(b) => {
                ui.monospace(format!("{}: {}", prefix, b));
            }
            Value::Number(n) => {
                ui.monospace(format!("{}: {}", prefix, n));
            }
            Value::String(s) => {
                ui.monospace(format!("{}: \"{}\"", prefix, s));
            }
            Value::Array(arr) => {
                let iter = arr.iter().enumerate();
                self.show_expandable(path_segments, ui, iter, &ARRAY_DELIMITERS, |prefix| {
                    prefix.to_string()
                });
            }
            Value::Object(obj) => {
                let iter = obj.iter();
                self.show_expandable(path_segments, ui, iter, &OBJECT_DELIMITERS, |prefix| {
                    format!("\"{prefix}\"")
                });
            }
        };
    }

    fn show_expandable<'a, K, I>(
        &self,
        path_segments: &mut Vec<String>,
        ui: &mut Ui,
        elem_iter: I,
        delimiters: &Delimiters,
        format_prefix: impl Fn(&K) -> String,
    ) where
        K: ToString,
        I: Iterator<Item = (K, &'a Value)>,
    {
        let id_source = ui.make_persistent_id(generate_id(self.id, path_segments));
        let state = CollapsingState::load_with_default_open(ui.ctx(), id_source, self.default_open);
        let is_expanded = state.is_open();

        state
            .show_header(ui, |ui| {
                ui.horizontal(|ui| {
                    if let Some(prefix) = &self.prefix {
                        ui.monospace(format!("{}:", prefix));
                    }
                    ui.label(if is_expanded {
                        delimiters.opening
                    } else {
                        delimiters.collapsed
                    });
                });
            })
            .body(|ui| {
                for (key, elem) in elem_iter {
                    path_segments.push(key.to_string());

                    let mut add_nested_tree = |ui: &mut Ui| {
                        ui.visuals_mut().indent_has_left_vline = true;

                        JsonTree::new(generate_id(self.id, path_segments))
                            .default_open(self.default_open)
                            .prefix(format_prefix(&key))
                            .show_inner(ui, path_segments, elem);
                    };

                    ui.visuals_mut().indent_has_left_vline = false;

                    if is_expandable(elem) {
                        add_nested_tree(ui);
                    } else {
                        let original_indent = ui.spacing().indent;

                        ui.spacing_mut().indent =
                            ui.spacing().icon_width + ui.spacing().icon_spacing;

                        ui.indent(id_source, |ui| add_nested_tree(ui));

                        ui.spacing_mut().indent = original_indent;
                    }

                    path_segments.pop();
                }
            });

        if is_expanded {
            ui.horizontal(|ui| {
                let indent = ui.spacing().icon_width / 2.0;
                ui.add_space(indent);

                ui.monospace(delimiters.closing);
            });
        }
    }
}

fn is_expandable(value: &Value) -> bool {
    matches!(value, Value::Array(_) | Value::Object(_))
}

fn generate_id(id: Id, path: &[String]) -> Id {
    Id::new(format!("{:?}-{}", id, path.join("/")))
}
