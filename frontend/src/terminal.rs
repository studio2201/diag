use common::WsMessage;
use eframe::WebRunner;
use std::sync::{Arc, Mutex};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use web_sys::{MessageEvent, WebSocket};
use yew::prelude::*;

#[function_component(TerminalComponent)]
pub fn terminal_component() -> Html {
    use_effect_with((), move |_| {
        spawn_local(async {
            let runner = WebRunner::new();
            let web_options = eframe::WebOptions::default();
            let _ = runner
                .start(
                    "terminal_canvas",
                    web_options,
                    Box::new(|cc| Box::new(crate::terminal::DiagApp::new(cc)) as Box<dyn eframe::App>),
                )
                .await;
        });
        || {}
    });

    html! {
        <div style="width: 100%; height: 80vh; background: black;">
            <canvas id="terminal_canvas"></canvas>
        </div>
    }
}

pub struct DiagApp {
    output: Arc<Mutex<String>>,
    target: String,
    ctx: Option<egui::Context>,
}

impl DiagApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            output: Arc::new(Mutex::new(String::new())),
            target: "8.8.8.8".to_string(),
            ctx: Some(cc.egui_ctx.clone()),
        }
    }

    fn run_ping(&mut self) {
        if let Some(ctx) = &self.ctx {
            let ctx = ctx.clone();
            let output = self.output.clone();
            let target = self.target.clone();

            spawn_local(async move {
                if let Ok(ws) = WebSocket::new("ws://127.0.0.1:3000/ws") {
                    let ws_c = ws.clone();
                    let onopen = Closure::wrap(Box::new(move || {
                        let msg = WsMessage::RunPing(target.clone());
                        if let Ok(json) = serde_json::to_string(&msg) {
                            let _ = ws_c.send_with_str(&json);
                        }
                    }) as Box<dyn FnMut()>);
                    ws.set_onopen(Some(onopen.as_ref().unchecked_ref()));
                    onopen.forget();

                    let ctx_c = ctx.clone();
                    let out_c = output.clone();
                    let onmessage = Closure::wrap(Box::new(move |e: MessageEvent| {
                        if let Ok(txt) = e.data().dyn_into::<js_sys::JsString>() {
                            let s: String = txt.into();
                            if let Ok(msg) = serde_json::from_str::<WsMessage>(&s) {
                                match msg {
                                    WsMessage::Output(o) | WsMessage::Error(o) => {
                                        if let Ok(mut lock) = out_c.lock() {
                                            lock.push_str(&o);
                                        }
                                        ctx_c.request_repaint();
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }) as Box<dyn FnMut(MessageEvent)>);
                    ws.set_onmessage(Some(onmessage.as_ref().unchecked_ref()));
                    onmessage.forget();
                }
            });
        }
    }
}

impl eframe::App for DiagApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("WebGL Terminal Network Diagnostics");
            ui.horizontal(|ui| {
                ui.label("Target:");
                ui.text_edit_singleline(&mut self.target);
                if ui.button("Ping").clicked() {
                    if let Ok(mut out) = self.output.lock() {
                        out.clear();
                    }
                    self.run_ping();
                }
            });
            ui.separator();
            egui::ScrollArea::vertical()
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    if let Ok(out) = self.output.lock() {
                        ui.label(egui::RichText::new(out.clone()).monospace());
                    }
                });
        });
    }
}
