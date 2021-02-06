use yew::{
    prelude::*,
    worker::{Bridge, Bridged},
};

use crate::{worker, xform, xform::Opts, xformsched};

pub struct OutputView {
    link: ComponentLink<Self>,
    text_cell_ref: NodeRef,
    worker: Box<dyn Bridge<worker::WorkerServer>>,
    xformer: xformsched::Transformer<TransformerWorkerClient>,
    opts: Option<Opts>,
}

#[derive(Properties, Clone)]
pub struct Props {
    pub opts: Option<Opts>,
}

pub enum Msg {
    GotValue(String),
    StartTransformerWork(u64, xform::WorkerRequest),
    CancelTransformerWork(u64),
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
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::GotValue(x) => {
                // Since the output text's amount can be enormous, it might be
                // inefficient to route it through VDOM
                if let Some(e) = self.text_cell_ref.cast::<web_sys::HtmlElement>() {
                    e.set_inner_text(&x);
                }
            }
            Msg::StartTransformerWork(token, request) => {
                self.worker
                    .send(worker::C2sMsg::StartTransformerWork(token, request));
                return false;
            }
            Msg::CancelTransformerWork(token) => {
                self.worker
                    .send(worker::C2sMsg::CancelTransformerWork(token));
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
        if props.opts != self.opts {
            self.opts = props.opts;

            if let Some(opts) = self.opts.clone() {
                // Start transformation
                let work = self.xformer.transform(opts);

                let link = self.link.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    let text = work.await.unwrap();
                    link.send_message(Msg::GotValue(text));
                });
            } else {
                // TODO: clear the result
            }
        }

        false
    }

    fn view(&self) -> Html {
        html! {
            <pre aria-label="Conversion result" role="image">
                <code ref=self.text_cell_ref.clone() />
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

    fn cancel(&self, token: u64) {
        self.link.send_message(Msg::CancelTransformerWork(token));
    }
}