use wasm_bindgen::prelude::*;
use web_sys::HtmlInputElement;

/// A labelled range input. Rust has no named arguments, so the settings
/// come as a struct with named fields.
pub struct Slider<'a> {
    pub label: &'a str,
    pub min: f64,
    pub max: f64,
    pub step: f64,
    pub value: f64,
    /// Formats the value shown next to the slider.
    pub show: fn(f64) -> String,
}

/// Appends `slider` to the element with the given id. While it is
/// dragged, `on_change` gets each new value.
pub fn slider(id: &str, slider: Slider, on_change: impl Fn(f64) + 'static) -> Result<(), JsValue> {
    let document = web_sys::window()
        .and_then(|w| w.document())
        .ok_or("no document")?;
    let parent = document
        .get_element_by_id(id)
        .ok_or_else(|| format!("no element with id {id:?}"))?;

    let row = document.create_element("label")?;
    row.set_class_name("slider");
    let name = document.create_element("span")?;
    name.set_text_content(Some(slider.label));
    let input: HtmlInputElement = document.create_element("input")?.dyn_into()?;
    input.set_type("range");
    input.set_min(&slider.min.to_string());
    input.set_max(&slider.max.to_string());
    input.set_step(&slider.step.to_string());
    input.set_value_as_number(slider.value);
    let shown = document.create_element("span")?;
    shown.set_text_content(Some(&(slider.show)(slider.value)));

    let show = slider.show;
    // Cloning a web-sys value only copies a handle to the same JS object.
    let (source, label) = (input.clone(), shown.clone());
    let changed = Closure::<dyn Fn()>::new(move || {
        let v = source.value_as_number();
        label.set_text_content(Some(&show(v)));
        on_change(v);
    });
    input.add_event_listener_with_callback("input", changed.as_ref().unchecked_ref())?;
    // Like the buttons' callbacks, this one lives as long as the page.
    changed.forget();

    row.append_child(&name)?;
    row.append_child(&input)?;
    row.append_child(&shown)?;
    parent.append_child(&row)?;
    Ok(())
}
