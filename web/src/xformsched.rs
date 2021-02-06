//! Transformer scheduler
use futures::channel::oneshot::{channel, Sender};
use std::{
    cell::{Cell, RefCell},
    future::Future,
    marker::Unpin,
    rc::{Rc, Weak},
};

use crate::xform;

/// Hooks up the transformer with a concrete worker.
///
/// It produces "request" and "cancel" messages and handles "response" messages.
/// The user is responsible for providing concrete implementations of the
/// methods ([`WorkerClientInterface`]) to deliver these messages and for
/// calling [`Transformer::handle_worker_response`] in response to "response"
/// messages.
pub struct Transformer<TWorkerClientInterface> {
    inner: Rc<Inner<TWorkerClientInterface>>,
}

struct Inner<TWorkerClientInterface> {
    worker_client: TWorkerClientInterface,
    /// Outstanding requests
    requests: RefCell<Vec<Request>>,
    next_token: Cell<u64>,
}

struct Request {
    token: u64,
    send: Sender<xform::WorkerResponse>,
}

pub trait WorkerClientInterface {
    fn request(&self, token: u64, req: xform::WorkerRequest);
    fn cancel(&self, token: u64);
}

impl<TWorkerClientInterface: WorkerClientInterface> Transformer<TWorkerClientInterface> {
    pub fn new(worker_client: TWorkerClientInterface) -> Self {
        Self {
            inner: Rc::new(Inner {
                worker_client,
                requests: RefCell::new(Vec::new()),
                next_token: Cell::new(0),
            }),
        }
    }

    pub fn transform(
        &self,
        opts: xform::Opts,
    ) -> impl Future<Output = Result<String, TransformError>> {
        let token = {
            let next_token = &self.inner.next_token;
            next_token.set(next_token.get() + 1);
            next_token.get()
        };

        let inner_future = xform::transform(
            opts,
            InnerWorkerClientInterface {
                inner: Rc::downgrade(&self.inner),
                token,
            },
        );

        let inner = Rc::downgrade(&self.inner);
        HandleDrop(Box::pin(inner_future), move || {
            // This closure will be called when this future is dropped
            if let Some(inner) = inner.upgrade() {
                // The response is no longer needed, so forget any relevant
                // outstanding request
                let mut requests = inner.requests.borrow_mut();
                if let Some(i) = requests.iter().position(|r| r.token == token) {
                    requests.swap_remove(i);
                    drop(requests);

                    inner.worker_client.cancel(token);
                }
            }
        })
    }

    pub fn handle_worker_response(&self, token: u64, response: xform::WorkerResponse) {
        let request = {
            let mut requests = self.inner.requests.borrow_mut();
            if let Some(i) = requests.iter().position(|r| r.token == token) {
                requests.swap_remove(i)
            } else {
                // Already cancelled
                log::debug!(
                    "Ignoring a worker response with token {:?} (probably already cancelled)",
                    token
                );
                return;
            }
        };

        let _ = request.send.send(response);
    }
}

struct InnerWorkerClientInterface<TWorkerClientInterface> {
    inner: Weak<Inner<TWorkerClientInterface>>,
    token: u64,
}

#[derive(Debug)]
pub enum TransformError {
    TransformerDropped,
}

impl<TWorkerClientInterface: WorkerClientInterface> xform::WorkerClientInterface
    for InnerWorkerClientInterface<TWorkerClientInterface>
{
    type Error = TransformError;

    fn request(
        &mut self,
        req: xform::WorkerRequest,
    ) -> std::pin::Pin<Box<dyn Future<Output = Result<xform::WorkerResponse, Self::Error>> + '_>>
    {
        let token = self.token;

        Box::pin(async move {
            if let Some(inner) = self.inner.upgrade() {
                let (send, recv) = channel();

                // Tell the `Transformer` whom to call when a response is received
                inner.requests.borrow_mut().push(Request { token, send });

                // Issue a request
                inner.worker_client.request(token, req);

                // Wait until it sends a response
                let response = recv.await.map_err(|_| TransformError::TransformerDropped)?;

                Ok(response)
            } else {
                Err(TransformError::TransformerDropped)
            }
        })
    }
}

struct HandleDrop<T, DropHandler: FnMut()>(T, DropHandler);

impl<T: Future + Unpin, DropHandler: FnMut() + Unpin> Future for HandleDrop<T, DropHandler> {
    type Output = T::Output;

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        std::pin::Pin::new(&mut self.get_mut().0).poll(cx)
    }
}

impl<T, DropHandler: FnMut()> Drop for HandleDrop<T, DropHandler> {
    fn drop(&mut self) {
        (self.1)();
    }
}
