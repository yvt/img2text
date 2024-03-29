#![recursion_limit = "1024"]
use js_sys::global;
use std::unreachable;
use wasm_bindgen::{prelude::*, JsCast, JsValue};
use yew::prelude::*;

#[global_allocator]
static ALLOC: rlsf::SmallGlobalTlsf = rlsf::SmallGlobalTlsf::INIT;

mod filechoice;
mod helpview;
mod imagewell;
mod loader;
mod outputview;
mod worker;
mod xform;
mod xformsched;
use self::{helpview::HelpView, imagewell::ImageWell, outputview::OutputView};

struct Model {
    link: ComponentLink<Self>,
    image: Option<web_sys::HtmlImageElement>,
    font_size: u32,
    max_size: u32,
    input_ty: xform::InputTy,
    style: xform::Style,
    help_visible: bool,
}

enum Msg {
    SetImage(web_sys::HtmlImageElement),
    SetFontSize(u32),
    SetMaxSize(u32),
    SetInputTy(xform::InputTy),
    SetStyle(xform::Style),
    ToggleTheme,
    ToggleHelp(bool),
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();
    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            link,
            image: None,
            font_size: 14,
            max_size: 80,
            input_ty: xform::InputTy::Auto,
            style: xform::Style::Braille,
            help_visible: false,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::SetImage(x) => self.image = Some(x.clone()),
            Msg::SetFontSize(x) => self.font_size = x,
            Msg::SetMaxSize(x) => self.max_size = x,
            Msg::SetInputTy(x) => self.input_ty = x,
            Msg::SetStyle(x) => self.style = x,
            Msg::ToggleTheme => {
                global()
                    .unchecked_into::<web_sys::Window>()
                    .document()
                    .unwrap()
                    .document_element()
                    .unwrap()
                    .class_list()
                    .toggle("invert")
                    .unwrap();
                return false;
            }
            Msg::ToggleHelp(b) => self.help_visible = b,
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
        const INPUT_TY_TABLE: &[(xform::InputTy, &str)] = &[
            (xform::InputTy::Auto, "Auto"),
            (xform::InputTy::Wob, "White-on-black"),
            (xform::InputTy::Bow, "Black-on-white"),
            (xform::InputTy::EdgeCanny, "Detect edges"),
        ];
        const STYLE_TABLE: &[(xform::Style, &str)] = &[
            (xform::Style::Slc, "SLC best effort"),
            (xform::Style::Ms2x3, "SLC marching squares"),
            (xform::Style::_1x1, "Blocks 1x1"),
            (xform::Style::_1x2, "Blocks 1x2"),
            (xform::Style::_2x2, "Blocks 2x2"),
            (xform::Style::_2x3, "Blocks 2x3"),
            (xform::Style::Braille, "Braille patterns"),
        ];

        let ondrop = self.link.callback(|i| Msg::SetImage(i));
        let font_size_oninput = self
            .link
            .callback(|e: InputData| Msg::SetFontSize(e.value.parse().unwrap()));
        let max_size_oninput = self
            .link
            .callback(|e: InputData| Msg::SetMaxSize(e.value.parse().unwrap()));
        let input_ty_onchange = self.link.callback(|e: ChangeData| match e {
            ChangeData::Select(s) => Msg::SetInputTy(
                INPUT_TY_TABLE
                    .iter()
                    .find(|pair| pair.1 == s.value())
                    .unwrap()
                    .0,
            ),
            _ => unreachable!(),
        });
        let style_onchange = self.link.callback(|e: ChangeData| match e {
            ChangeData::Select(s) => Msg::SetStyle(
                STYLE_TABLE
                    .iter()
                    .find(|pair| pair.1 == s.value())
                    .unwrap()
                    .0,
            ),
            _ => unreachable!(),
        });
        let toggle_theme_onclick = self.link.callback(|_| Msg::ToggleTheme);
        let help_show = self.link.callback(|_| Msg::ToggleHelp(true));
        let help_hide = self.link.callback(|_| Msg::ToggleHelp(false));

        let opts = self.image.clone().map(|image| xform::Opts {
            image,
            max_size: self.max_size as _,
            input_ty: self.input_ty,
            style: self.style,
        });

        html! {
            <>
                <header class="appHeader" aria-hidden=["", "true"][self.help_visible as usize]>
                    <h1 class="branding">{ "img" }<span>{ "2" }</span>{ "text" }</h1>
                    <div class="chooseImage">
                        <span>{ "Choose an input image:" }</span>
                        <ImageWell
                            // TODO: `aria-labelled`
                            accept="image/*"
                            ondrop=ondrop image=self.image.clone() />
                    </div>
                    <label>
                        <span>{ "Font size:" }</span>
                        <input type="range" min="1" max="24"
                            oninput=font_size_oninput />
                    </label>
                    <label>
                        <span>{ "Image size:" }</span>
                        <input type="range" min="1" max="500"
                            oninput=max_size_oninput />
                    </label>
                    <select onchange=input_ty_onchange aria-label="Input interpretation">
                        {
                            for INPUT_TY_TABLE.iter()
                                .map(|&(x, label)| html! {
                                    <option value=label selected={x == self.input_ty}>{label}</option>
                                })
                        }
                    </select>
                    <select onchange=style_onchange aria-label="Output glyph set">
                        {
                            for STYLE_TABLE.iter()
                                .map(|&(x, label)| html! {
                                    <option value=label selected={x == self.style}>{label}</option>
                                })
                        }
                    </select>
                    <span class="right">
                        <button class="showHelp"
                            onclick=help_show
                            label="About this application"
                            aria-haspopup="dialog">{ "?" }</button>
                        <button class="switchTheme"
                            label="Switch theme"
                            onclick=toggle_theme_onclick>{ "☀️" }</button>
                    </span>
                </header>
                <main aria-hidden=["", "true"][self.help_visible as usize]>
                    <OutputView
                        opts=opts
                        font_size=self.font_size />
                </main>
                <HelpView
                    on_dismiss=help_hide
                    visible=self.help_visible />
            </>
        }
    }
}

#[wasm_bindgen(start)]
pub fn start() {
    use js_sys::Reflect;
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
