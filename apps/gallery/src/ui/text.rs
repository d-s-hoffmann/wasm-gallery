use wasm_bindgen::prelude::*;
use web_sys::Element;

/// An element whose text the client sets, e.g. a status line.
pub struct Text(Element);

impl Text {
    /// Returns a Text for the element with the given id.
    pub fn new(id: &str) -> Result<Self, JsValue> {
        let el = web_sys::window()
            .and_then(|w| w.document())
            .ok_or("no document")?
            .get_element_by_id(id)
            .ok_or_else(|| format!("no element with id {id:?}"))?;
        Ok(Self(el))
    }

    pub fn set(&self, text: &str) {
        self.0.set_text_content(Some(text));
    }
}
