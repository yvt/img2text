use yew::prelude::*;

use crate::loader::InlineHtmlLoader;

pub struct HelpView {
    link: ComponentLink<Self>,
    on_dismiss: Callback<MouseEvent>,
    visible: bool,
}

#[derive(Properties, Clone)]
pub struct Props {
    pub visible: bool,
    pub on_dismiss: Callback<MouseEvent>,
}

pub enum Msg {}

impl Component for HelpView {
    type Message = Msg;
    type Properties = Props;
    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            link,
            visible: props.visible,
            on_dismiss: props.on_dismiss,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {}
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        let should_render = props.visible != self.visible;
        self.visible = props.visible;
        self.on_dismiss = props.on_dismiss;
        should_render
    }

    fn view(&self) -> Html {
        let dialog_class = ["helpView", "helpView show"][self.visible as usize];

        html! {
            <dialog class=dialog_class aria-hidden=["", "true"][!self.visible as usize]>
                <div class="background" onclick=self.on_dismiss.clone() />
                <div class="frame">
                    <InlineHtmlLoader src="help.html" />
                    <h2>{ "Third-Party Software License" }</h2>
                    <InlineHtmlLoader src="license.html" />
                </div>
            </dialog>
        }
    }
}
