use serde::{Deserialize, Serialize};
use yew::worker::{Agent, AgentLink, HandlerId, Public};

use crate::xform;

#[derive(Serialize, Deserialize)]
pub enum C2sMsg {
    StartTransformerWork(u64, xform::WorkerRequest),
}

#[derive(Serialize, Deserialize)]
pub enum S2cMsg {
    DoneTransformerWork(u64, xform::WorkerResponse),
}

pub enum Msg {}

pub struct WorkerServer {
    link: AgentLink<WorkerServer>,
}

impl Agent for WorkerServer {
    type Reach = Public<Self>;
    type Message = Msg;
    type Input = C2sMsg;
    type Output = S2cMsg;

    fn create(link: AgentLink<Self>) -> Self {
        Self { link }
    }

    fn update(&mut self, msg: Self::Message) {
        match msg {}
    }

    fn handle_input(&mut self, msg: Self::Input, id: HandlerId) {
        match msg {
            C2sMsg::StartTransformerWork(token, request) => {
                let response = xform::worker_kernel(request);

                self.link
                    .respond(id, S2cMsg::DoneTransformerWork(token, response));
            }
        }
    }

    fn name_of_resource() -> &'static str {
        "wasm.js"
    }
}
