use js_sys::Function;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

pub fn comet_connect() -> Result<(), JsValue> {
    let global = js_sys::global();

    let chrome = js_sys::Reflect::get(&global, &JsValue::from_str("chrome"))
        .map_err(|_| JsValue::from_str("no chrome global"))?;

    if !js_sys::Reflect::has(&chrome, &JsValue::from_str("runtime"))? {
        return Err(JsValue::from_str("no chrome.runtime"));
    }

    let runtime = js_sys::Reflect::get(&chrome, &JsValue::from_str("runtime"))?;
    let connect = js_sys::Reflect::get(&runtime, &JsValue::from_str("connect"))?;

    if connect.is_undefined() || connect.is_null() {
        return Err(JsValue::from_str("chrome.runtime.connect not available"));
    }

    let args = js_sys::Array::new();
    let port = Function::from(connect).apply(&runtime, &args)?;

    let on_msg_closure = Closure::<dyn Fn(JsValue)>::new(|msg: JsValue| {
        if let Ok(data) = js_sys::Reflect::get(&msg, &JsValue::from_str("data")) {
            if let Ok(text) = data.dyn_into::<js_sys::JsString>() {
                let s: String = text.into();
                crate::dom::append_message("agent", &format!("[Comet] {}", s));
            }
        }
    });

    let on_disconnect_closure = Closure::<dyn Fn()>::new(|| {
        crate::dom::set_status("Comet bridge disconnected").ok();
    });

    if js_sys::Reflect::has(&port, &JsValue::from_str("onMessage"))? {
        let on_msg = js_sys::Reflect::get(&port, &JsValue::from_str("onMessage"))?;
        let add_listener = js_sys::Reflect::get(&on_msg, &JsValue::from_str("addListener"))?;
        let args = js_sys::Array::new();
        args.push(on_msg_closure.as_ref());
        let _ = Function::from(add_listener).apply(&on_msg, &args);
    }

    if js_sys::Reflect::has(&port, &JsValue::from_str("onDisconnect"))? {
        let on_disc = js_sys::Reflect::get(&port, &JsValue::from_str("onDisconnect"))?;
        let add_listener = js_sys::Reflect::get(&on_disc, &JsValue::from_str("addListener"))?;
        let args = js_sys::Array::new();
        args.push(on_disconnect_closure.as_ref());
        let _ = Function::from(add_listener).apply(&on_disc, &args);
    }

    if js_sys::Reflect::has(&port, &JsValue::from_str("postMessage"))? {
        let post_msg = js_sys::Reflect::get(&port, &JsValue::from_str("postMessage"))?;
        let msg = js_sys::Object::new();
        js_sys::Reflect::set(&msg, &JsValue::from_str("type"), &JsValue::from_str("ping"))?;
        let args = js_sys::Array::new();
        args.push(&msg);
        let _ = Function::from(post_msg).apply(&port, &args);
    }

    on_msg_closure.forget();
    on_disconnect_closure.forget();

    crate::dom::set_status("Comet bridge connected")?;
    Ok(())
}

#[wasm_bindgen]
pub fn comet_send(message: &str) -> Result<(), JsValue> {
    let global = js_sys::global();
    let chrome = js_sys::Reflect::get(&global, &JsValue::from_str("chrome"))?;
    let runtime = js_sys::Reflect::get(&chrome, &JsValue::from_str("runtime"))?;
    let connect = js_sys::Reflect::get(&runtime, &JsValue::from_str("connect"))?;

    let args = js_sys::Array::new();
    let port = Function::from(connect).apply(&runtime, &args)?;

    let msg = js_sys::Object::new();
    js_sys::Reflect::set(
        &msg,
        &JsValue::from_str("type"),
        &JsValue::from_str("comet"),
    )?;
    js_sys::Reflect::set(
        &msg,
        &JsValue::from_str("data"),
        &JsValue::from_str(message),
    )?;

    let post_msg = js_sys::Reflect::get(&port, &JsValue::from_str("postMessage"))?;
    let args = js_sys::Array::new();
    args.push(&msg);
    Function::from(post_msg).apply(&port, &args)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn comet_module_compiles() {}
}
