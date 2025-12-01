//! DOM operation types and batch processing.
//!
//! This module defines all supported DOM operations and provides a high-performance
//! batch processor that generates optimized JavaScript code.

/// Represents a single DOM operation.
///
/// Each variant maps to a specific DOM manipulation that will be
/// converted to JavaScript code.
#[derive(Debug, Clone, PartialEq)]
pub enum DomOp {
    // === Text & Content ===
    /// Set element's text content
    SetText { selector: String, text: String },
    /// Set element's innerHTML
    SetHtml { selector: String, html: String },

    // === Attributes ===
    /// Set an attribute value
    SetAttribute {
        selector: String,
        name: String,
        value: String,
    },
    /// Remove an attribute
    RemoveAttribute { selector: String, name: String },

    // === Classes ===
    /// Add a CSS class
    AddClass { selector: String, class: String },
    /// Remove a CSS class
    RemoveClass { selector: String, class: String },
    /// Toggle a CSS class
    ToggleClass { selector: String, class: String },

    // === Styles ===
    /// Set a CSS style property
    SetStyle {
        selector: String,
        property: String,
        value: String,
    },
    /// Set multiple CSS styles at once
    SetStyles {
        selector: String,
        styles: Vec<(String, String)>,
    },

    // === Visibility ===
    /// Show element (display: '')
    Show { selector: String },
    /// Hide element (display: none)
    Hide { selector: String },

    // === Forms ===
    /// Set input/textarea value
    SetValue { selector: String, value: String },
    /// Set checkbox/radio checked state
    SetChecked { selector: String, checked: bool },
    /// Set element disabled state
    SetDisabled { selector: String, disabled: bool },
    /// Select an option by value
    SelectOption { selector: String, value: String },

    // === Interactions ===
    /// Click an element
    Click { selector: String },
    /// Double-click an element
    DoubleClick { selector: String },
    /// Focus an element
    Focus { selector: String },
    /// Blur (unfocus) an element
    Blur { selector: String },
    /// Scroll element into view
    ScrollIntoView { selector: String, smooth: bool },

    // === Input ===
    /// Type text into an input (simulates keystrokes)
    TypeText {
        selector: String,
        text: String,
        clear: bool,
    },
    /// Clear input value
    Clear { selector: String },
    /// Submit a form
    Submit { selector: String },

    // === DOM Manipulation ===
    /// Append HTML inside element
    AppendHtml { selector: String, html: String },
    /// Prepend HTML inside element
    PrependHtml { selector: String, html: String },
    /// Remove element from DOM
    Remove { selector: String },
    /// Empty element's content
    Empty { selector: String },

    // === Custom ===
    /// Execute raw JavaScript on element
    Raw { selector: String, script: String },
    /// Execute global JavaScript (no element selection)
    RawGlobal { script: String },
}

/// A batch of DOM operations for high-performance execution.
#[derive(Debug, Clone, Default)]
pub struct DomBatch {
    operations: Vec<DomOp>,
}

impl DomBatch {
    /// Create a new empty batch.
    pub fn new() -> Self {
        Self {
            operations: Vec::new(),
        }
    }

    /// Create a batch with pre-allocated capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            operations: Vec::with_capacity(capacity),
        }
    }

    /// Add an operation to the batch.
    pub fn push(&mut self, op: DomOp) {
        self.operations.push(op);
    }

    /// Get the number of operations in the batch.
    pub fn len(&self) -> usize {
        self.operations.len()
    }

    /// Check if the batch is empty.
    pub fn is_empty(&self) -> bool {
        self.operations.is_empty()
    }

    /// Clear all operations from the batch.
    pub fn clear(&mut self) {
        self.operations.clear();
    }

    /// Get operations slice
    pub fn operations(&self) -> &[DomOp] {
        &self.operations
    }

    /// Escape a CSS selector for use in JavaScript.
    pub fn escape_selector(selector: &str) -> String {
        selector
            .replace('\\', "\\\\")
            .replace('\'', "\\'")
            .replace('\n', "\\n")
            .replace('\r', "\\r")
    }

    /// Escape a string value for JavaScript.
    pub fn escape_string(value: &str) -> String {
        let mut result = String::with_capacity(value.len() + 10);
        for ch in value.chars() {
            match ch {
                '\\' => result.push_str("\\\\"),
                '"' => result.push_str("\\\""),
                '\n' => result.push_str("\\n"),
                '\r' => result.push_str("\\r"),
                '\t' => result.push_str("\\t"),
                '\'' => result.push_str("\\'"),
                _ => result.push(ch),
            }
        }
        result
    }

    /// Generate JavaScript code for a single operation.
    pub fn op_to_js(op: &DomOp) -> String {
        match op {
            DomOp::SetText { selector, text } => {
                format!(
                    "var e=document.querySelector('{}');if(e)e.textContent=\"{}\";",
                    Self::escape_selector(selector),
                    Self::escape_string(text)
                )
            }
            DomOp::SetHtml { selector, html } => {
                format!(
                    "var e=document.querySelector('{}');if(e)e.innerHTML=\"{}\";",
                    Self::escape_selector(selector),
                    Self::escape_string(html)
                )
            }
            DomOp::SetAttribute {
                selector,
                name,
                value,
            } => {
                format!(
                    "var e=document.querySelector('{}');if(e)e.setAttribute('{}',\"{}\");",
                    Self::escape_selector(selector),
                    Self::escape_string(name),
                    Self::escape_string(value)
                )
            }
            DomOp::RemoveAttribute { selector, name } => {
                format!(
                    "var e=document.querySelector('{}');if(e)e.removeAttribute('{}');",
                    Self::escape_selector(selector),
                    Self::escape_string(name)
                )
            }
            DomOp::AddClass { selector, class } => {
                format!(
                    "var e=document.querySelector('{}');if(e)e.classList.add('{}');",
                    Self::escape_selector(selector),
                    Self::escape_string(class)
                )
            }
            DomOp::RemoveClass { selector, class } => {
                format!(
                    "var e=document.querySelector('{}');if(e)e.classList.remove('{}');",
                    Self::escape_selector(selector),
                    Self::escape_string(class)
                )
            }
            DomOp::ToggleClass { selector, class } => {
                format!(
                    "var e=document.querySelector('{}');if(e)e.classList.toggle('{}');",
                    Self::escape_selector(selector),
                    Self::escape_string(class)
                )
            }
            DomOp::SetStyle {
                selector,
                property,
                value,
            } => {
                format!(
                    "var e=document.querySelector('{}');if(e)e.style['{}']=\"{}\";",
                    Self::escape_selector(selector),
                    Self::escape_string(property),
                    Self::escape_string(value)
                )
            }
            DomOp::SetStyles { selector, styles } => {
                let style_assignments: String = styles
                    .iter()
                    .map(|(prop, val)| {
                        format!(
                            "e.style['{}']=\"{}\";",
                            Self::escape_string(prop),
                            Self::escape_string(val)
                        )
                    })
                    .collect();
                format!(
                    "var e=document.querySelector('{}');if(e){{{}}}",
                    Self::escape_selector(selector),
                    style_assignments
                )
            }
            DomOp::Show { selector } => {
                format!(
                    "var e=document.querySelector('{}');if(e)e.style.display='';",
                    Self::escape_selector(selector)
                )
            }
            DomOp::Hide { selector } => {
                format!(
                    "var e=document.querySelector('{}');if(e)e.style.display='none';",
                    Self::escape_selector(selector)
                )
            }
            DomOp::SetValue { selector, value } => {
                format!(
                    "var e=document.querySelector('{}');if(e)e.value=\"{}\";",
                    Self::escape_selector(selector),
                    Self::escape_string(value)
                )
            }
            DomOp::SetChecked { selector, checked } => {
                format!(
                    "var e=document.querySelector('{}');if(e)e.checked={};",
                    Self::escape_selector(selector),
                    checked
                )
            }
            DomOp::SetDisabled { selector, disabled } => {
                format!(
                    "var e=document.querySelector('{}');if(e)e.disabled={};",
                    Self::escape_selector(selector),
                    disabled
                )
            }
            DomOp::SelectOption { selector, value } => {
                format!(
                    "var e=document.querySelector('{}');if(e)e.value=\"{}\";",
                    Self::escape_selector(selector),
                    Self::escape_string(value)
                )
            }
            DomOp::Click { selector } => {
                format!(
                    "var e=document.querySelector('{}');if(e)e.click();",
                    Self::escape_selector(selector)
                )
            }
            DomOp::DoubleClick { selector } => {
                format!(
                    "var e=document.querySelector('{}');if(e){{var ev=new MouseEvent('dblclick',{{bubbles:true}});e.dispatchEvent(ev);}}",
                    Self::escape_selector(selector)
                )
            }
            DomOp::Focus { selector } => {
                format!(
                    "var e=document.querySelector('{}');if(e)e.focus();",
                    Self::escape_selector(selector)
                )
            }
            DomOp::Blur { selector } => {
                format!(
                    "var e=document.querySelector('{}');if(e)e.blur();",
                    Self::escape_selector(selector)
                )
            }
            DomOp::ScrollIntoView { selector, smooth } => {
                let behavior = if *smooth { "smooth" } else { "auto" };
                format!(
                    "var e=document.querySelector('{}');if(e)e.scrollIntoView({{behavior:'{}'}});",
                    Self::escape_selector(selector),
                    behavior
                )
            }
            DomOp::TypeText {
                selector,
                text,
                clear,
            } => {
                let clear_code = if *clear { "e.value='';" } else { "" };
                format!(
                    "var e=document.querySelector('{}');if(e){{{}\"{}\".split('').forEach(function(c){{e.value+=c;e.dispatchEvent(new Event('input',{{bubbles:true}}));}});}}",
                    Self::escape_selector(selector),
                    clear_code,
                    Self::escape_string(text)
                )
            }
            DomOp::Clear { selector } => {
                format!(
                    "var e=document.querySelector('{}');if(e){{e.value='';e.dispatchEvent(new Event('input',{{bubbles:true}}));}}",
                    Self::escape_selector(selector)
                )
            }
            DomOp::Submit { selector } => {
                format!(
                    "var e=document.querySelector('{}');if(e){{var f=e.closest('form');if(f)f.submit();else if(e.tagName==='FORM')e.submit();}}",
                    Self::escape_selector(selector)
                )
            }
            DomOp::AppendHtml { selector, html } => {
                format!(
                    "var e=document.querySelector('{}');if(e)e.insertAdjacentHTML('beforeend',\"{}\");",
                    Self::escape_selector(selector),
                    Self::escape_string(html)
                )
            }
            DomOp::PrependHtml { selector, html } => {
                format!(
                    "var e=document.querySelector('{}');if(e)e.insertAdjacentHTML('afterbegin',\"{}\");",
                    Self::escape_selector(selector),
                    Self::escape_string(html)
                )
            }
            DomOp::Remove { selector } => {
                format!(
                    "var e=document.querySelector('{}');if(e)e.remove();",
                    Self::escape_selector(selector)
                )
            }
            DomOp::Empty { selector } => {
                format!(
                    "var e=document.querySelector('{}');if(e)e.innerHTML='';",
                    Self::escape_selector(selector)
                )
            }
            DomOp::Raw { selector, script } => {
                format!(
                    "var e=document.querySelector('{}');if(e){{{}}}",
                    Self::escape_selector(selector),
                    script
                )
            }
            DomOp::RawGlobal { script } => script.clone(),
        }
    }

    /// Generate optimized JavaScript for all operations.
    pub fn to_js(&self) -> String {
        if self.operations.is_empty() {
            return String::from("(function(){})()");
        }
        let mut js = String::with_capacity(self.operations.len() * 100);
        js.push_str("(function(){");
        for op in &self.operations {
            js.push_str(&Self::op_to_js(op));
        }
        js.push_str("})()");
        js
    }
}

// === Convenience methods ===
impl DomBatch {
    pub fn set_text(&mut self, selector: &str, text: &str) -> &mut Self {
        self.push(DomOp::SetText {
            selector: selector.to_string(),
            text: text.to_string(),
        });
        self
    }
    pub fn set_html(&mut self, selector: &str, html: &str) -> &mut Self {
        self.push(DomOp::SetHtml {
            selector: selector.to_string(),
            html: html.to_string(),
        });
        self
    }
    pub fn set_attribute(&mut self, selector: &str, name: &str, value: &str) -> &mut Self {
        self.push(DomOp::SetAttribute {
            selector: selector.to_string(),
            name: name.to_string(),
            value: value.to_string(),
        });
        self
    }
    pub fn remove_attribute(&mut self, selector: &str, name: &str) -> &mut Self {
        self.push(DomOp::RemoveAttribute {
            selector: selector.to_string(),
            name: name.to_string(),
        });
        self
    }
    pub fn add_class(&mut self, selector: &str, class: &str) -> &mut Self {
        self.push(DomOp::AddClass {
            selector: selector.to_string(),
            class: class.to_string(),
        });
        self
    }
    pub fn remove_class(&mut self, selector: &str, class: &str) -> &mut Self {
        self.push(DomOp::RemoveClass {
            selector: selector.to_string(),
            class: class.to_string(),
        });
        self
    }
    pub fn toggle_class(&mut self, selector: &str, class: &str) -> &mut Self {
        self.push(DomOp::ToggleClass {
            selector: selector.to_string(),
            class: class.to_string(),
        });
        self
    }
    pub fn set_style(&mut self, selector: &str, property: &str, value: &str) -> &mut Self {
        self.push(DomOp::SetStyle {
            selector: selector.to_string(),
            property: property.to_string(),
            value: value.to_string(),
        });
        self
    }
    pub fn show(&mut self, selector: &str) -> &mut Self {
        self.push(DomOp::Show {
            selector: selector.to_string(),
        });
        self
    }
    pub fn hide(&mut self, selector: &str) -> &mut Self {
        self.push(DomOp::Hide {
            selector: selector.to_string(),
        });
        self
    }
    pub fn set_value(&mut self, selector: &str, value: &str) -> &mut Self {
        self.push(DomOp::SetValue {
            selector: selector.to_string(),
            value: value.to_string(),
        });
        self
    }
    pub fn set_checked(&mut self, selector: &str, checked: bool) -> &mut Self {
        self.push(DomOp::SetChecked {
            selector: selector.to_string(),
            checked,
        });
        self
    }
    pub fn set_disabled(&mut self, selector: &str, disabled: bool) -> &mut Self {
        self.push(DomOp::SetDisabled {
            selector: selector.to_string(),
            disabled,
        });
        self
    }
    pub fn click(&mut self, selector: &str) -> &mut Self {
        self.push(DomOp::Click {
            selector: selector.to_string(),
        });
        self
    }
    pub fn double_click(&mut self, selector: &str) -> &mut Self {
        self.push(DomOp::DoubleClick {
            selector: selector.to_string(),
        });
        self
    }
    pub fn focus(&mut self, selector: &str) -> &mut Self {
        self.push(DomOp::Focus {
            selector: selector.to_string(),
        });
        self
    }
    pub fn blur(&mut self, selector: &str) -> &mut Self {
        self.push(DomOp::Blur {
            selector: selector.to_string(),
        });
        self
    }
    pub fn scroll_into_view(&mut self, selector: &str, smooth: bool) -> &mut Self {
        self.push(DomOp::ScrollIntoView {
            selector: selector.to_string(),
            smooth,
        });
        self
    }
    pub fn type_text(&mut self, selector: &str, text: &str, clear: bool) -> &mut Self {
        self.push(DomOp::TypeText {
            selector: selector.to_string(),
            text: text.to_string(),
            clear,
        });
        self
    }
    pub fn clear_input(&mut self, selector: &str) -> &mut Self {
        self.push(DomOp::Clear {
            selector: selector.to_string(),
        });
        self
    }
    pub fn submit(&mut self, selector: &str) -> &mut Self {
        self.push(DomOp::Submit {
            selector: selector.to_string(),
        });
        self
    }
    pub fn append_html(&mut self, selector: &str, html: &str) -> &mut Self {
        self.push(DomOp::AppendHtml {
            selector: selector.to_string(),
            html: html.to_string(),
        });
        self
    }
    pub fn prepend_html(&mut self, selector: &str, html: &str) -> &mut Self {
        self.push(DomOp::PrependHtml {
            selector: selector.to_string(),
            html: html.to_string(),
        });
        self
    }
    pub fn remove(&mut self, selector: &str) -> &mut Self {
        self.push(DomOp::Remove {
            selector: selector.to_string(),
        });
        self
    }
    pub fn empty(&mut self, selector: &str) -> &mut Self {
        self.push(DomOp::Empty {
            selector: selector.to_string(),
        });
        self
    }
    pub fn raw(&mut self, selector: &str, script: &str) -> &mut Self {
        self.push(DomOp::Raw {
            selector: selector.to_string(),
            script: script.to_string(),
        });
        self
    }
    pub fn raw_global(&mut self, script: &str) -> &mut Self {
        self.push(DomOp::RawGlobal {
            script: script.to_string(),
        });
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_batch() {
        let batch = DomBatch::new();
        assert!(batch.is_empty());
        assert_eq!(batch.to_js(), "(function(){})()");
    }

    #[test]
    fn test_set_text() {
        let mut batch = DomBatch::new();
        batch.set_text("#title", "Hello World");
        assert_eq!(batch.len(), 1);
        let js = batch.to_js();
        assert!(js.contains("textContent"));
        assert!(js.contains("Hello World"));
    }

    #[test]
    fn test_escape_string() {
        assert_eq!(DomBatch::escape_string("hello"), "hello");
        assert_eq!(DomBatch::escape_string("hello\"world"), "hello\\\"world");
        assert_eq!(DomBatch::escape_string("hello\nworld"), "hello\\nworld");
    }
}
