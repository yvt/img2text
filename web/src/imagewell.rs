use std::rc::Rc;
use wasm_bindgen::{closure::Closure, JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::{DataTransfer, Element, File, HtmlImageElement, Node, Url};
use yew::prelude::*;

use crate::filechoice::FileChooser;

pub struct ImageWell {
    link: ComponentLink<Self>,
    wrap_ref: NodeRef,
    image: Option<HtmlImageElement>,
    accepting: bool,
    cb_ondragover: Callback<DragEvent>,
    cb_ondragleave: Callback<DragEvent>,
    cb_ondrop: Callback<DragEvent>,
    cb_onclick: Callback<MouseEvent>,
    ondrop: Option<Callback<HtmlImageElement>>,
    chooser: FileChooser,
}

pub enum Msg {
    /// Because there doesn't seem to be a good way to return no messages from
    /// a callback
    DragOver(bool),
    DragLeave,
    DragEnd(Option<File>),
    ImageLoaded(HtmlImageElement),
    InvokeFileChooser,
}

#[derive(Properties, Clone)]
pub struct ImageWellProps {
    /// The image to display in the image well.
    ///
    /// Note that an HTML element can belong to only one parent at once. This
    /// means if the image is already added to somewhere else, it will be
    /// removed and reinserted to the image well.
    #[prop_or_default]
    pub image: Option<HtmlImageElement>,
    #[prop_or_default]
    pub ondrop: Option<Callback<HtmlImageElement>>,
}

impl Component for ImageWell {
    type Message = Msg;
    type Properties = ImageWellProps;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let cb_ondragover = link.callback(|e: DragEvent| {
            e.prevent_default();

            if let Some(dt) = e.data_transfer() {
                dt.set_effect_allowed("copy");
                // `files` is unavailable at this point, so only check the data types
                let accepting = dt.types().includes(&JsValue::from("Files"), 0);
                dt.set_drop_effect(["none", "copy"][accepting as usize]);
                Msg::DragOver(accepting)
            } else {
                Msg::DragOver(false)
            }
        });

        let cb_ondragleave = link.callback(|_: DragEvent| Msg::DragLeave);

        let cb_ondrop = link.callback(|e: DragEvent| {
            e.prevent_default();

            let file = e
                .data_transfer()
                .and_then(|t| extract_file_from_data_transfer(&t));

            Msg::DragEnd(file)
        });

        let cb_onclick = link.callback(|_: MouseEvent| Msg::InvokeFileChooser);

        Self {
            link,
            cb_ondragover,
            cb_ondragleave,
            cb_ondrop,
            cb_onclick,
            accepting: false,
            wrap_ref: NodeRef::default(),
            image: props.image,
            ondrop: props.ondrop,
            chooser: FileChooser::new(),
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::DragOver(accepting) => {
                if accepting != self.accepting {
                    self.accepting = accepting;
                    true
                } else {
                    false
                }
            }
            Msg::DragLeave => {
                self.accepting = false;
                true
            }
            Msg::DragEnd(file) => {
                self.accepting = false;
                if let Some(file) = file {
                    let link = self.link.clone();

                    wasm_bindgen_futures::spawn_local(async move {
                        let object_url = match ObjectUrl::new(&file) {
                            Ok(x) => x,
                            Err(x) => {
                                log::warn!(
                                    "Could not create an object URL for the \
                                    dropped file {:?}; ignoring the file. Error: {:?}",
                                    x,
                                    file.name(),
                                );
                                return;
                            }
                        };

                        log::trace!("object URL = {:?}", object_url.url);

                        if let Some(image) = load_image(&object_url.url).await {
                            log::debug!(
                                "got an image of size {}x{}",
                                image.width(),
                                image.height()
                            );
                            link.send_message(Msg::ImageLoaded(image));
                        } else {
                            log::warn!(
                                "Could not load the image for the dropped file {:?}",
                                file.name(),
                            );
                        }
                    });
                } else {
                    log::debug!("no file dropped");
                }
                true
            }
            Msg::ImageLoaded(image) => {
                if let Some(ondrop) = &self.ondrop {
                    ondrop.emit(image);
                }
                false
            }
            Msg::InvokeFileChooser => {
                let link = self.link.clone();
                self.chooser.choose_file(move |file| {
                    link.send_message(Msg::DragEnd(Some(file)));
                });
                false
            }
        }
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        let should_render = props.image != self.image;
        self.image = props.image;
        should_render
    }

    fn rendered(&mut self, _first_render: bool) {
        // Remove the existing image if it's not supposed to be there
        let wrap = self.wrap_ref.cast::<Element>().unwrap();
        if let Some(first_child) = wrap.first_child() {
            if Some(&first_child) != self.image.as_ref().map(|x| x as &Node) {
                wrap.remove_child(&first_child).unwrap();
            }
        }

        // Insert the specified image if it has't been yet
        if let Some(image) = &self.image {
            if image.parent_node().as_ref() != Some(&wrap as &Node) {
                wrap.append_child(image).unwrap();
            }
        }
    }

    fn view(&self) -> Html {
        html! {
            <div class=["imagewell", "imagewell imagewell accept"][self.accepting as usize]
                ondragover=self.cb_ondragover.clone()
                ondragleave=self.cb_ondragleave.clone()
                ondrop=self.cb_ondrop.clone()
                onclick=self.cb_onclick.clone()
                title="Click to choose an image file"
                ref=self.wrap_ref.clone() />
        }
    }
}

/// Extract a `File` object if it's the sole constituent of the given
/// `DataTransfer`. Otherwise (i.e., if it contains zero or multiple files),
/// return `None.`
fn extract_file_from_data_transfer(dt: &DataTransfer) -> Option<File> {
    let files = dt.files()?;
    if files.length() == 1 {
        files.get(0)
    } else {
        log::debug!(
            "DataTransfer includes {} files; we are not accepting this",
            files.length()
        );
        None
    }
}

async fn load_image(src: &String) -> Option<HtmlImageElement> {
    let doc = js_sys::global()
        .unchecked_ref::<web_sys::Window>()
        .document()
        .unwrap();
    let image = doc
        .create_element("img")
        .unwrap()
        .unchecked_into::<HtmlImageElement>();

    let (send, recv) = futures::channel::oneshot::channel();
    let send = Rc::new(std::cell::Cell::new(Some(send)));
    let send2 = Rc::clone(&send);
    let onload = Closure::wrap(Box::new(move || {
        if let Some(send) = send.take() {
            let _ = send.send(true);
        }
    }) as Box<dyn Fn()>);
    let onerror = Closure::wrap(Box::new(move || {
        if let Some(send) = send2.take() {
            let _ = send.send(false);
        }
    }) as Box<dyn Fn()>);
    image.set_onload(Some(onload.as_ref().unchecked_ref()));
    image.set_onerror(Some(onerror.as_ref().unchecked_ref()));
    image.set_src(src);

    if !recv.await.unwrap_or(false) {
        // Loading the image failed
        return None;
    }

    JsFuture::from(image.decode()).await.ok()?;

    Some(image)
}

/// An RAII guard for an object URL created by `URL#createObjectURL`.
struct ObjectUrl {
    url: String,
}

impl ObjectUrl {
    fn new(blob: &web_sys::Blob) -> Result<Self, JsValue> {
        let url = Url::create_object_url_with_blob(blob)?;
        Ok(Self { url })
    }
}

impl Drop for ObjectUrl {
    fn drop(&mut self) {
        Url::revoke_object_url(&self.url).unwrap();
    }
}
