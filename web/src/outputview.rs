use wasm_bindgen::JsCast;
use yew::{
    prelude::*,
    services::{timeout::TimeoutTask, TimeoutService},
    worker::{Bridge, Bridged},
};

use crate::{worker, xform, xform::Opts, xformsched};

pub struct OutputView {
    link: ComponentLink<Self>,
    text_cell_ref: NodeRef,
    worker: Box<dyn Bridge<worker::WorkerServer>>,
    xformer: xformsched::Transformer<TransformerWorkerClient>,
    opts: Option<Opts>,
    pending_work: bool,
    busy: bool,
    font_size: u32,
    copy_button_animation_timeout: Option<TimeoutTask>,
    copy_button_result: Option<bool>,
}

#[derive(Properties, Clone)]
pub struct Props {
    pub opts: Option<Opts>,
    pub font_size: u32,
}

pub enum Msg {
    StartWorkIfDirty,
    GotValue(String),
    CopyToClipboard,
    CopyToClipboardResult(bool),
    StartTransformerWork(u64, xform::WorkerRequest),
    DoneTransformerWork(u64, xform::WorkerResponse),
}

impl Component for OutputView {
    type Message = Msg;
    type Properties = Props;
    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let worker = worker::WorkerServer::bridge(link.callback(|msg| match msg {
            worker::S2cMsg::DoneTransformerWork(token, response) => {
                Msg::DoneTransformerWork(token, response)
            }
        }));

        let xformer = xformsched::Transformer::new(TransformerWorkerClient { link: link.clone() });

        // Assume this to simplify the impl
        assert!(props.opts.is_none());

        Self {
            link,
            text_cell_ref: NodeRef::default(),
            worker,
            xformer,
            opts: props.opts,
            pending_work: false,
            busy: false,
            font_size: props.font_size,
            copy_button_animation_timeout: None,
            copy_button_result: None,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::StartWorkIfDirty => {
                if !self.busy && self.pending_work {
                    self.pending_work = false;
                    if let Some(opts) = self.opts.clone() {
                        // Start transformation
                        let work = self.xformer.transform(opts);
                        let link = self.link.clone();
                        self.busy = true;
                        wasm_bindgen_futures::spawn_local(async move {
                            let text = work.await.unwrap();
                            link.send_message(Msg::GotValue(text));
                        });
                    } else {
                        self.link.send_message(Msg::GotValue(String::new()));
                    }
                }
                return false;
            }
            Msg::GotValue(x) => {
                // Since the output text's amount can be enormous, it might be
                // inefficient to route it through VDOM
                if let Some(e) = self.text_cell_ref.cast::<web_sys::HtmlElement>() {
                    e.set_inner_text(&x);
                }

                self.busy = false;
                self.link.send_message(Msg::StartWorkIfDirty);
            }
            Msg::CopyToClipboard => {
                if let Some(e) = self.text_cell_ref.cast() {
                    let doc = js_sys::global()
                        .unchecked_into::<web_sys::Window>()
                        .document()
                        .unwrap()
                        .dyn_into::<web_sys::HtmlDocument>()
                        .unwrap();

                    let range = doc.create_range().unwrap();
                    range.select_node(&e).unwrap();

                    let selection = js_sys::global()
                        .unchecked_into::<web_sys::Window>()
                        .get_selection()
                        .unwrap()
                        .unwrap();
                    selection.empty().unwrap();
                    selection.add_range(&range).unwrap();

                    let ok = doc.exec_command("copy").is_ok();

                    // Before displaying the result, reset the button's CSS class
                    // so that the animation is restarted every time it's pressed
                    self.copy_button_result = None;
                    self.copy_button_animation_timeout = Some(TimeoutService::spawn(
                        std::time::Duration::from_millis(30),
                        self.link.callback(move |_| Msg::CopyToClipboardResult(ok)),
                    ));

                    selection.empty().unwrap();
                }
            }
            Msg::CopyToClipboardResult(ok) => {
                self.copy_button_result = Some(ok);
            }
            Msg::StartTransformerWork(token, request) => {
                self.worker
                    .send(worker::C2sMsg::StartTransformerWork(token, request));
                return false;
            }
            Msg::DoneTransformerWork(token, response) => {
                self.xformer.handle_worker_response(token, response);
                return false;
            }
        }
        true
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        let should_render = props.font_size != self.font_size;

        if props.opts != self.opts {
            self.opts = props.opts;
            self.pending_work = true;
            self.link.send_message(Msg::StartWorkIfDirty);
        }

        self.font_size = props.font_size;

        should_render
    }

    fn view(&self) -> Html {
        let copy_to_clipboard_onclick = self.link.callback(|_| Msg::CopyToClipboard);
        let copy_to_clipboard_class = match self.copy_button_result {
            None => "",
            Some(false) => "fail",
            Some(true) => "success",
        };

        html! {
            <pre aria-label="Conversion result" role="image"
                style=format!("font-size: {}px", self.font_size)>
                <code ref=self.text_cell_ref.clone() />
                <span class="copyToClipboard">
                    <button class=copy_to_clipboard_class
                        onclick=copy_to_clipboard_onclick>
                        { "Copy to Clipboard" }</button>
                </span>
            </pre>
        }
    }
}

struct TransformerWorkerClient {
    link: ComponentLink<OutputView>,
}

impl xformsched::WorkerClientInterface for TransformerWorkerClient {
    fn request(&self, token: u64, req: xform::WorkerRequest) {
        self.link
            .send_message(Msg::StartTransformerWork(token, req));
    }
}
