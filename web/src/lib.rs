#![recursion_limit = "1024"]
use wasm_bindgen::{prelude::*, JsCast, JsValue};
use yew::prelude::*;

mod filechoice;
mod imagewell;
mod outputview;
mod worker;
mod xform;
mod xformsched;
use self::{imagewell::ImageWell, outputview::OutputView};

struct Model {
    link: ComponentLink<Self>,
    image: Option<web_sys::HtmlImageElement>,
}

enum Msg {
    SetImage(web_sys::HtmlImageElement),
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();
    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self { link, image: None }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::SetImage(x) => {
                self.image = Some(x.clone());
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

        let opts = self.image.clone().map(|image| xform::Opts { image });

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
                    <OutputView opts=opts />
                </main>
            </>
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
