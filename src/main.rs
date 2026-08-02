//! Native and WASM entrypoints for citizen-builder.

use citizen_builder::app::CitizenBuilderApp;
#[cfg(not(target_arch = "wasm32"))]
use eframe::egui;

#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result<()> {
    env_logger::init();
    eframe::run_native(
        "citizen-builder",
        eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_inner_size([1440.0, 900.0])
                .with_min_inner_size([900.0, 600.0]),
            ..Default::default()
        },
        Box::new(|creation_context| Ok(Box::new(CitizenBuilderApp::new(creation_context)))),
    )
}

#[cfg(target_arch = "wasm32")]
fn main() {
    use eframe::wasm_bindgen::JsCast as _;

    eframe::WebLogger::init(log::LevelFilter::Debug).ok();
    wasm_bindgen_futures::spawn_local(async {
        let document = web_sys::window()
            .expect("window is unavailable")
            .document()
            .expect("document is unavailable");
        let canvas = document
            .get_element_by_id("builder_canvas")
            .expect("builder_canvas is missing")
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .expect("builder_canvas is not a canvas");
        eframe::WebRunner::new()
            .start(
                canvas,
                eframe::WebOptions::default(),
                Box::new(|creation_context| Ok(Box::new(CitizenBuilderApp::new(creation_context)))),
            )
            .await
            .expect("failed to start citizen-builder");
        if let Some(loading) = document.get_element_by_id("loading") {
            loading.remove();
        }
    });
}
