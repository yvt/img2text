use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use yew::{
    services::{timeout::TimeoutTask, TimeoutService},
    worker::{Agent, AgentLink, HandlerId, Public},
};

use crate::xform;

#[derive(Serialize, Deserialize)]
pub enum C2sMsg {
    StartTransformerWork(u64, xform::WorkerRequest),
    CancelTransformerWork(u64),
}

#[derive(Serialize, Deserialize)]
pub enum S2cMsg {
    DoneTransformerWork(u64, xform::WorkerResponse),
}

pub enum Msg {
    ProcessQueue,
}

pub struct WorkerServer {
    link: AgentLink<WorkerServer>,
    queue: VecDeque<TransformerWork>,
    timeout: Option<TimeoutTask>,
}

struct TransformerWork {
    token: u64,
    request: xform::WorkerRequest,
    id: HandlerId,
}

impl Agent for WorkerServer {
    type Reach = Public<Self>;
    type Message = Msg;
    type Input = C2sMsg;
    type Output = S2cMsg;

    fn create(link: AgentLink<Self>) -> Self {
        Self {
            link,
            queue: VecDeque::new(),
            timeout: None,
        }
    }

    fn update(&mut self, msg: Self::Message) {
        match msg {
            Msg::ProcessQueue => {
                // Process one work item
                if let Some(work) = self.queue.pop_front() {
                    let response = xform::worker_kernel(work.request);

                    self.link
                        .respond(work.id, S2cMsg::DoneTransformerWork(work.token, response));
                }

                // If there are remaining works to be done, schedule the next
                // tick.
                self.schedule_process_queue();
            }
        }
    }

    fn handle_input(&mut self, msg: Self::Input, id: HandlerId) {
        match msg {
            C2sMsg::StartTransformerWork(token, request) => {
                // Enqueue a work item
                self.queue.push_back(TransformerWork { token, request, id });
                self.schedule_process_queue();
            }

            C2sMsg::CancelTransformerWork(token) => {
                if let Some(i) = self.queue.iter().position(|e| e.token == token) {
                    self.queue.remove(i);
                }
            }
        }
    }

    fn name_of_resource() -> &'static str {
        "wasm.js"
    }
}

impl WorkerServer {
    fn schedule_process_queue(&mut self) {
        // If there are remaining works to be done, schedule the next tick.
        //
        // Do not try to process more than one of them at once! I don't know how
        // Yew's queueing system is implemented, but if it's implemented by
        // synchronously looping until the queue is empty, that would prevent us
        // from receiving cancellation requests.
        if self.queue.is_empty() {
            self.timeout = None;
        } else {
            self.timeout = Some(TimeoutService::spawn(
                std::time::Duration::default(),
                self.link.callback(move |_| Msg::ProcessQueue),
            ));
        }
    }
}
