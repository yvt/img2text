use yew::{prelude::*, services::fetch};

pub struct InlineHtmlLoader {
    content: Content,
    content_placeholder_ref: NodeRef,
    _fetch_task: fetch::FetchTask,
}

pub enum Content {
    NotReady,
    Ready(String),
    Error,
}

#[derive(Properties, Clone)]
pub struct Props {
    pub src: String,
}

pub enum Msg {
    SetContent(Content),
}

impl Component for InlineHtmlLoader {
    type Message = Msg;
    type Properties = Props;
    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        // Start fetching the content
        let req = fetch::Request::get(&props.src)
            .body(yew::format::Nothing)
            .unwrap();
        let task = fetch::FetchService::fetch(
            req,
            link.callback(|response: fetch::Response<yew::format::Text>| {
                if response.status().is_success() {
                    if let Ok(x) = response.into_body() {
                        Msg::SetContent(Content::Ready(x))
                    } else {
                        Msg::SetContent(Content::Error)
                    }
                } else {
                    Msg::SetContent(Content::Error)
                }
            }),
        )
        .unwrap();

        Self {
            content: Content::NotReady,
            content_placeholder_ref: NodeRef::default(),
            _fetch_task: task,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::SetContent(c) => self.content = c,
        }
        true
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        // we don't support changing `src` dynamically
        false
    }

    fn rendered(&mut self, _first_render: bool) {
        if let Some(content_placeholder) =
            self.content_placeholder_ref.cast::<web_sys::HtmlElement>()
        {
            let content = match &self.content {
                Content::NotReady => return,
                Content::Ready(s) => s,
                Content::Error => "<p><i>Something went wrong while loading this region.</i></p>",
            };

            if !content_placeholder.class_list().contains("ready") {
                content_placeholder.class_list().add_1("ready").unwrap();
                content_placeholder.set_inner_html(content);
            }
        }
    }

    fn view(&self) -> Html {
        html! {
            <div ref=self.content_placeholder_ref.clone() />
        }
    }
}
