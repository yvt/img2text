use serde::{Deserialize, Serialize};
use yew::{
    services::{timeout::TimeoutTask, TimeoutService},
    worker::{Agent, AgentLink, HandlerId, Public},
};

#[derive(Serialize, Deserialize)]
pub enum C2sMsg {
    AddOne,
}

#[derive(Serialize, Deserialize)]
pub enum S2cMsg {
    Value(i64),
}

pub enum Msg {
    AddOne(HandlerId),
}

pub struct WorkerServer {
    link: AgentLink<WorkerServer>,
    value: i64,
    timeout: Option<TimeoutTask>,
}

impl Agent for WorkerServer {
    type Reach = Public<Self>;
    type Message = Msg;
    type Input = C2sMsg;
    type Output = S2cMsg;

    fn create(link: AgentLink<Self>) -> Self {
        Self {
            link,
            value: 0,
            timeout: None,
        }
    }

    fn update(&mut self, msg: Self::Message) {
        match msg {
            Msg::AddOne(id) => {
                self.value += 1;
                self.link.respond(id, S2cMsg::Value(self.value));
            }
        }
    }

    fn handle_input(&mut self, msg: Self::Input, id: HandlerId) {
        match msg {
            C2sMsg::AddOne => {
                self.timeout = Some(TimeoutService::spawn(
                    std::time::Duration::from_secs(1),
                    self.link.callback(move |_| Msg::AddOne(id)),
                ));
            }
        }
    }

    fn name_of_resource() -> &'static str {
        "wasm.js"
    }
}
