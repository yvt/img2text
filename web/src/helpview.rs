use yew::prelude::*;

use crate::loader::InlineHtmlLoader;

pub struct HelpView {
    link: ComponentLink<Self>,
    on_dismiss: Callback<()>,
    visible: bool,
    load_contents: bool,
    dialog_ref: NodeRef,
}

#[derive(Properties, Clone)]
pub struct Props {
    pub visible: bool,
    pub on_dismiss: Callback<()>,
}

pub enum Msg {
    Dismiss,
    Nothing,
}

impl Component for HelpView {
    type Message = Msg;
    type Properties = Props;
    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            link,
            visible: props.visible,
            load_contents: false,
            on_dismiss: props.on_dismiss,
            dialog_ref: NodeRef::default(),
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Dismiss => {
                self.on_dismiss.emit(());
            }
            Msg::Nothing => {}
        }
        false
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        let should_render = props.visible != self.visible;
        self.visible = props.visible;
        self.on_dismiss = props.on_dismiss;
        self.load_contents |= props.visible;
        if props.visible {
            if let Some(e) = self.dialog_ref.cast::<web_sys::HtmlElement>() {
                let _ = e.focus();
            }
        }
        should_render
    }

    fn view(&self) -> Html {
        let dialog_class = ["helpView", "helpView show"][self.visible as usize];
        let dialog_on_close = self.link.callback(|_| Msg::Dismiss);
        let dialog_on_keydown = self.link.callback(|e: KeyboardEvent| {
            if e.key() == "Escape" {
                e.prevent_default();
                Msg::Dismiss
            } else {
                Msg::Nothing
            }
        });

        let contents = if self.load_contents {
            html! {
                <>
                    <InlineHtmlLoader src="help.html" />
                    <h2>{ "Third-Party Software Licenses" }</h2>
                    <InlineHtmlLoader src="license.html" />
                </>
            }
        } else {
            html! {}
        };

        html! {
            <div class=dialog_class
                role="dialog"
                onkeydown=dialog_on_keydown
                tabindex=["-1", "0"][!self.visible as usize]
                aria-hidden=["", "true"][!self.visible as usize]
            >
                <div class="background" onclick=dialog_on_close.clone() />
                <div class="frame" ref=self.dialog_ref.clone()>
                    { contents }
                </div>
                <button class="close" aria-label="Close" onclick=dialog_on_close />
            </div>
        }
    }
}
