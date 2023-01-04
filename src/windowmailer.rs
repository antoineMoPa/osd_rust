/// Send messages through the window object.

use wasm_bindgen::prelude::*;
use js_sys::Map;
use js_sys::Array;

const MAIL_VAR: &str = "WINDOW_MAILER_MAILBOX";


// The code here is not used in native builds
#[allow(dead_code)]

/// Writes a message in the specified channel.
///
/// Example:
///
/// ```
/// windowmailer::send_message(
///     String::from("ROAD_NETWORK_LOADED"),
///     String::from("ARE_ROADS_LOADED")
/// );
/// ```
pub fn send_message(_channel_name: String, message: String) {
    let channel_name: JsValue = JsValue::from(_channel_name);
    let window = web_sys::window().unwrap();

    // Create the mail-storing map if needed
    if !js_sys::Reflect::has(&window, &JsValue::from(MAIL_VAR)).unwrap() {
        let map: Map = Map::new();
        js_sys::Reflect::set(&window, &JsValue::from(MAIL_VAR), &map).unwrap();
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

// The code here is not used in native builds
#[allow(dead_code)]

/// Get the amount of message in a given channel.
pub fn message_count(_channel_name: String) -> u32 {
    let channel_name: JsValue = JsValue::from(_channel_name);
    let window = web_sys::window().unwrap();

    if !js_sys::Reflect::has(&window, &JsValue::from(MAIL_VAR)).unwrap() {
        return 0;
    }

    let map: Map = js_sys::Reflect::get(&window, &JsValue::from(MAIL_VAR)).unwrap().into();

    if !map.has(&channel_name) {
        return 0;
    }

    let arr: Array = map.get(&channel_name).into();

    return arr.length();
}

// The code here is not used in native builds
#[allow(dead_code)]

/// Read the first inserted message.
pub fn read_message(_channel_name: String) -> String {
    let channel_name: JsValue = JsValue::from(_channel_name);
    let window = web_sys::window().unwrap();

    if !js_sys::Reflect::has(&window, &JsValue::from(MAIL_VAR)).unwrap() {
        panic!("No message available");
    }

    let map: Map = js_sys::Reflect::get(&window, &JsValue::from(MAIL_VAR)).unwrap().into();

    if !map.has(&channel_name) {
        panic!("No message available");
    }

    let arr: Array = map.get(&channel_name).into();
    let value: JsValue = arr.shift();

    return value.as_string().unwrap();
}
