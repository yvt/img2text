#![recursion_limit = "1024"]
use wasm_bindgen::{prelude::*, JsCast, JsValue};
use yew::{
    prelude::*,
    worker::{Bridge, Bridged},
};

mod filechoice;
mod imagewell;
mod worker;
mod xform;
mod xformsched;
use self::imagewell::ImageWell;

struct Model {
    link: ComponentLink<Self>,
    output_cell_ref: NodeRef,
    worker: Box<dyn Bridge<worker::WorkerServer>>,
    xformer: xformsched::Transformer<TransformerWorkerClient>,
    image: Option<web_sys::HtmlImageElement>,
}

enum Msg {
    GotValue(String),
    SetImage(web_sys::HtmlImageElement),
    StartTransformerWork(u64, xform::WorkerRequest),
    CancelTransformerWork(u64),
    DoneTransformerWork(u64, xform::WorkerResponse),
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();
    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        let worker = worker::WorkerServer::bridge(link.callback(|msg| match msg {
            worker::S2cMsg::DoneTransformerWork(token, response) => {
                Msg::DoneTransformerWork(token, response)
            }
        }));

        let xformer = xformsched::Transformer::new(TransformerWorkerClient { link: link.clone() });

        Self {
            link,
            output_cell_ref: NodeRef::default(),
            worker,
            xformer,
            image: None,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::GotValue(x) => {
                // Since the output text's amount can be enormous, it might be
                // inefficient to route it through VDOM
                if let Some(e) = self.output_cell_ref.cast::<web_sys::HtmlElement>() {
                    e.set_inner_text(&x);
                }
            }
            Msg::SetImage(x) => {
                self.image = Some(x.clone());

                // Start transformation
                let work = self.xformer.transform(xform::Opts { image: x });

                let link = self.link.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    let text = work.await.unwrap();
                    link.send_message(Msg::GotValue(text));
                });
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

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        // Should only return "true" if new properties are different to
        // previously received properties.
        // This component has no properties so we will always return "false".
        false
    }

    fn view(&self) -> Html {
        let ondrop = self.link.callback(|i| Msg::SetImage(i));

        let source_url = "https://github.com/yvt/img2text";

        html! {
            <>
                <header class="appHeader">
                    <h1>{ "img" }<span>{ "2" }</span>{ "text" }</h1>
                    <span>
                        { "[" }<a href=source_url>{ "Source Code" }</a>{ "]" }
                    </span>
                    <div class="chooseImage">
                        <span>{ "Choose an input image:" }</span>
                        <ImageWell
                            // TODO: `aria-labelled`
                            accept="image/*"
                            ondrop=ondrop image=self.image.clone() />
                    </div>
                </header>
                <main>
                    <pre aria-label="Conversion result" role="image">
                        <code ref=self.output_cell_ref.clone() />
                    </pre>
                </main>
            </>
        }
    }
}

struct TransformerWorkerClient {
    link: ComponentLink<Model>,
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

#[wasm_bindgen(start)]
pub fn start() {
    use js_sys::{global, Reflect};
    #[cfg(debug_assertions)]
    wasm_logger::init(wasm_logger::Config::default());
    if Reflect::has(&global(), &JsValue::from_str("window")).unwrap() {
        App::<Model>::new().mount_to_body();

        global()
            .unchecked_into::<web_sys::Window>()
            .document()
            .unwrap()
            .document_element()
            .unwrap()
            .class_list()
            .add_1("ready")
            .unwrap();
    } else {
        worker::WorkerServer::register();
    }
}
