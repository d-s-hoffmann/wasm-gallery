use std::rc::Rc;

use wasm_bindgen::prelude::*;

/// Appends one button per label to the element with the given id.
/// Clicking the i-th button calls `on_select(i)`.
pub fn buttons(
    id: &str,
    labels: &[&str],
    on_select: impl Fn(usize) + 'static,
) -> Result<(), JsValue> {
    let document = web_sys::window()
        .and_then(|w| w.document())
        .ok_or("no document")?;
    let bar = document
        .get_element_by_id(id)
        .ok_or_else(|| format!("no element with id {id:?}"))?;

    // Every button's callback calls the same function, so they share it.
    let on_select = Rc::new(on_select);
    for (i, label) in labels.iter().enumerate() {
        let btn = document.create_element("button")?;
        btn.set_text_content(Some(label));
        btn.set_class_name("btn");

        let on_select = Rc::clone(&on_select);
        let click = Closure::<dyn Fn()>::new(move || on_select(i));
        btn.add_event_listener_with_callback("click", click.as_ref().unchecked_ref())?;
        // The button needs the callback for the page's life, so hand it
        // over to JS instead of freeing it at the end of this iteration.
        click.forget();

        bar.append_child(&btn)?;
    }
    Ok(())
}
