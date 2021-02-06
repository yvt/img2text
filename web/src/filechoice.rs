use js_sys::global;
use once_cell::unsync::OnceCell;
use std::{cell::Cell, rc::Rc};
use wasm_bindgen::{closure::Closure, JsCast};
use web_sys::{File, HtmlInputElement};

pub struct FileChooser {
    inner: Rc<Inner>,
}

struct Inner {
    input: Cell<Option<HtmlInputElement>>,
    onchange: OnceCell<Closure<dyn FnMut()>>,
    callback: Cell<Option<Box<dyn FnMut(File)>>>,
}

impl FileChooser {
    pub fn new() -> Self {
        let inner = Rc::new(Inner {
            input: Cell::new(None),
            onchange: OnceCell::new(),
            callback: Cell::new(None),
        });

        // Create a closure to handle `change` events
        let inner2 = Rc::downgrade(&inner);
        inner
            .onchange
            .set(Closure::wrap(Box::new(move || {
                let inner = if let Some(x) = inner2.upgrade() {
                    x
                } else {
                    return;
                };

                if let Some(input) = inner.input.take() {
                    // Check the current selection
                    if let Some(file) = input.files().and_then(|f| f.get(0)) {
                        // Got a file; call the current callback function
                        if let Some(mut cb) = inner.callback.take() {
                            cb(file);
                        }
                    }

                    // Remove the input element (another one will be re-created
                    // the next time `choose_file` is called)
                    input.remove();
                }
            }) as Box<dyn FnMut()>))
            .unwrap();

        Self { inner }
    }

    /// Ask the user to choose a file.
    ///
    /// Note that if the user cancel the file choice, `cb` may live until this
    /// function is called again.
    pub fn choose_file(&self, cb: impl FnOnce(File) + 'static) {
        let mut cb = Some(cb);
        // The codegen of calling `Box<FnOnce>` is not really great, so...
        self.choose_file_inner(Box::new(move |file| {
            if let Some(cb) = cb.take() {
                cb(file);
            }
        }));
    }

    fn choose_file_inner(&self, cb: Box<dyn FnMut(File)>) {
        let input = if let Some(input) = self.inner.input.take() {
            input
        } else {
            let doc = global()
                .unchecked_into::<web_sys::Window>()
                .document()
                .unwrap();

            let input = doc
                .create_element("input")
                .unwrap()
                .unchecked_into::<HtmlInputElement>();
            input.set_attribute("type", "file").unwrap();
            input
                .set_attribute("style", "visibility:hidden;position:absolute")
                .unwrap();
            doc.document_element()
                .unwrap()
                .append_child(&input)
                .unwrap();

            input.set_onchange(Some(
                self.inner.onchange.get().unwrap().as_ref().unchecked_ref(),
            ));

            input
        };

        // Register the callback
        self.inner.callback.set(Some(cb));

        // Invoke the file chooser dialog
        input.click();

        self.inner.input.set(Some(input));
    }
}

impl Drop for Inner {
    fn drop(&mut self) {
        if let Some(input) = self.input.take() {
            input.remove();
        }
    }
}
