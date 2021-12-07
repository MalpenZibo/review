use bom::callback;
use bom::component;
use bom::create_component;
use bom::create_element;
use bom::create_text;
use bom::use_state;
use bom::ComponentProvider;
use bom::EventType;
use bom::Tag;
use bom::VNode;
use wasm_bindgen::prelude::*;

// When the `wee_alloc` feature is enabled, this uses `wee_alloc` as the global
// allocator.
//
// If you don't want to use `wee_alloc`, you can safely delete this.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[component(App)]
fn app() -> VNode {
    let counter = use_state(0);

    let stuff = {
        let counter = counter.clone();
        callback!(move || {
            counter.set(*(counter.value) + 1);
        })
    };

    create_element(Tag::Div)
        .with_child(create_text(&format!("test {}", counter.value)))
        .with_child(
            create_element(Tag::Button)
                .with_child(create_text("Inrement Counter"))
                .with_event(EventType::OnClick, stuff)
                .build(),
        )
        .build()
}

// This is like the `main` function, except for JavaScript.
#[wasm_bindgen(start)]
pub fn main_js() -> Result<(), JsValue> {
    // This provides better error messages in debug mode.
    // It should be disabled in release mode so it doesn't bloat up the file size.
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();

    bom::init_logger();

    bom::render(
        create_component::<App>(()),
        web_sys::window()
            .unwrap()
            .document()
            .unwrap()
            .get_element_by_id("root")
            .unwrap(),
    );

    Ok(())
}
