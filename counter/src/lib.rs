use serde_json;
use state::{Counter, CounterTransition};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{Document, Element, ErrorEvent, HtmlElement, MessageEvent, WebSocket};

macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    // Retrieve the window and the document.
    let window = web_sys::window().expect("no global `window` exists");
    let document = window.document().expect("should have a document on window");

    // Connect to an the broadcast server.
    let ws = WebSocket::new("ws://localhost:8080")?;

    // Setup the websocket client with the associated callbacks to handle
    // the connection lifecycle.
    start_websocket(
        document
            .get_element_by_id("counter")
            .expect("should have a #counter on the page"),
        ws.clone(),
    );

    // Listen to onClick events for the Add, Substract and Reset buttons.
    setup_adder(&document, ws.clone());
    setup_subtractor(&document, ws.clone());
    setup_reseter(&document, ws.clone());

    Ok(())
}

fn start_websocket(element: Element, ws: WebSocket) {
    // Setup callback to handle all incoming messages.
    let onmessage_callback = Closure::wrap(Box::new(move |e: MessageEvent| {
        if let Ok(txt) = e.data().dyn_into::<js_sys::JsString>() {
            let txt_str: String = txt.clone().into();
            match serde_json::from_str::<Counter>(txt_str.as_str()) {
                Ok(counter) => {
                    // Setup the new counter value as broadcasted.
                    update_counter(&element, counter);
                }
                Err(_) => {}
            }
            console_log!("message event, received Text: {:?}", txt);
        } else {
            console_log!("message event, received Unknown: {:?}", e.data());
        }
    }) as Box<dyn FnMut(MessageEvent)>);

    // Set message event handler on WebSocket
    ws.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));

    // Forget the callback to keep it alive.
    onmessage_callback.forget();

    let onerror_callback = Closure::wrap(Box::new(move |e: ErrorEvent| {
        console_log!("error event: {:?}", e);
    }) as Box<dyn FnMut(ErrorEvent)>);
    ws.set_onerror(Some(onerror_callback.as_ref().unchecked_ref()));

    // Forget the callback to keep it alive.
    onerror_callback.forget();

    // Set on connection callback.
    let onopen_callback = Closure::wrap(Box::new(move |_| {
        // declare socket opened and wait for messages to be sent.
        console_log!("socket opened");
    }) as Box<dyn FnMut(JsValue)>);
    ws.set_onopen(Some(onopen_callback.as_ref().unchecked_ref()));

    // Forget the callback to keep it alive.
    onopen_callback.forget();
}

fn update_counter(counter_elem: &Element, state: state::Counter) {
    counter_elem
        .dyn_ref::<HtmlElement>()
        .expect("#counter be an `HtmlElement`")
        .set_text_content(Some(format!("Counter: {}", state.value()).as_str()));
}

fn setup_adder(document: &Document, ws: WebSocket) {
    // Callback function to trigger a message for the CounterTransition.
    let a = Closure::wrap(Box::new(move || {
        // Create CounterTransition.
        let add_transition = CounterTransition::Add(1);
        let transition_data = serde_json::to_string(&add_transition).unwrap();

        // Send the transition over the connection to the server.

        match ws.send_with_str(transition_data.as_str()) {
            Ok(_) => {
                console_log!("transition sent successfully!")
            }
            Err(err) => {
                console_log!("error sending transition: {:?}", err)
            }
        }
    }) as Box<dyn Fn()>);

    document
        .get_element_by_id("adder")
        .expect("should have an #adder on the page")
        .dyn_ref::<HtmlElement>()
        .expect("#adder be an `HtmlElement`")
        .set_onclick(Some(a.as_ref().unchecked_ref()));

    a.forget();
}

fn setup_subtractor(document: &Document, ws: WebSocket) {
    // Callback function to trigger a message for the CounterTransition.
    let a = Closure::wrap(Box::new(move || {
        // Create CounterTransition.
        let subtract_transition = CounterTransition::Subtract(1);
        let transition_data = serde_json::to_string(&subtract_transition).unwrap();

        // Send the transition over the connection to the server.

        match ws.send_with_str(transition_data.as_str()) {
            Ok(_) => {
                console_log!("transition sent successfully!")
            }
            Err(err) => {
                console_log!("error sending transition: {:?}", err)
            }
        }
    }) as Box<dyn Fn()>);

    document
        .get_element_by_id("subtractor")
        .expect("should have an #subtractor on the page")
        .dyn_ref::<HtmlElement>()
        .expect("#subtractor be an `HtmlElement`")
        .set_onclick(Some(a.as_ref().unchecked_ref()));

    a.forget();
}

fn setup_reseter(document: &Document, ws: WebSocket) {
    // Callback function to trigger a message for the CounterTransition.
    let a = Closure::wrap(Box::new(move || {
        // Create CounterTransition.
        let reset_transition = CounterTransition::Reset;
        let transition_data = serde_json::to_string(&reset_transition).unwrap();

        // Send the transition over the connection to the server.

        match ws.send_with_str(transition_data.as_str()) {
            Ok(_) => {
                console_log!("transition sent successfully!")
            }
            Err(err) => {
                console_log!("error sending transition: {:?}", err)
            }
        }
    }) as Box<dyn Fn()>);

    document
        .get_element_by_id("reseter")
        .expect("should have an #reseter on the page")
        .dyn_ref::<HtmlElement>()
        .expect("#reseter be an `HtmlElement`")
        .set_onclick(Some(a.as_ref().unchecked_ref()));

    a.forget();
}
