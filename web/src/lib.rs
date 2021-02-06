use wasm_bindgen::{prelude::*, JsValue};
use yew::{
    prelude::*,
    worker::{Bridge, Bridged},
};

mod imagewell;
mod worker;
use self::imagewell::ImageWell;

struct Model {
    link: ComponentLink<Self>,
    value: i64,
    worker: Box<dyn Bridge<worker::WorkerServer>>,
    image: Option<web_sys::HtmlImageElement>,
}

enum Msg {
    AddOne,
    GotValue(i64),
    SetImage(web_sys::HtmlImageElement),
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();
    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        let worker = worker::WorkerServer::bridge(link.callback(|msg| match msg {
            worker::S2cMsg::Value(x) => Msg::GotValue(x),
        }));
        Self {
            link,
            value: 0,
            worker,
            image: None,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::AddOne => {
                self.worker.send(worker::C2sMsg::AddOne);
            }
            Msg::GotValue(x) => {
                self.value = x;
            }
            Msg::SetImage(x) => {
                self.image = Some(x);
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
        html! {
            <div>
                <button onclick=self.link.callback(|_| Msg::AddOne)>{ "+1" }</button>
                <p>{ self.value }</p>
                <p>
                    <ImageWell ondrop=ondrop image=self.image.clone() />
                </p>
            </div>
        }
    }
}

#[wasm_bindgen(start)]
pub fn start() {
    use js_sys::{global, Reflect};
    #[cfg(debug_assertions)]
    wasm_logger::init(wasm_logger::Config::default());
    if Reflect::has(&global(), &JsValue::from_str("window")).unwrap() {
        App::<Model>::new().mount_to_body();
    } else {
        worker::WorkerServer::register();
    }
}
