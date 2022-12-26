use wasm_bindgen::prelude::*;
use js_sys::Map;
use js_sys::Array;

const MAIL_VAR: &str = "WINDOW_MAILER_MAILBOX";

pub fn send_message(_channel_name: String, message: String) {
    let channel_name: JsValue = JsValue::from(_channel_name);
    let window = web_sys::window().unwrap();

    // Create the mail-storing map if needed
    if !js_sys::Reflect::has(&window, &JsValue::from(MAIL_VAR)).unwrap() {

        let map: Map = Map::new();
        js_sys::Reflect::set(&window, &JsValue::from(MAIL_VAR), &map);
    }

    let map: Map = js_sys::Reflect::get(&window, &JsValue::from(MAIL_VAR)).unwrap().into();

    // Create the channel message array if needed
    if !map.has(&channel_name) {
        let arr: Array = Array::new();
        map.set(&channel_name, &arr);
    }

    let arr: Array = map.get(&channel_name).into();
    arr.push(&JsValue::from(message));
}
