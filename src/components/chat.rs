use serde::{Deserialize, Serialize};
use web_sys::HtmlInputElement;
use yew::prelude::*;
use yew_agent::{Bridge, Bridged};

use crate::services::event_bus::EventBus;
use crate::{services::websocket::WebsocketService, User};

pub enum Msg {
    HandleMsg(String),
    SubmitMessage,
    ToggleDarkMode,
}

#[derive(Deserialize)]
struct MessageData {
    from: String,
    message: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum MsgTypes {
    Users,
    Register,
    Message,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WebSocketMessage {
    message_type: MsgTypes,
    data_array: Option<Vec<String>>,
    data: Option<String>,
}

#[derive(Clone)]
struct UserProfile {
    name: String,
    avatar: String,
}

pub struct Chat {
    users: Vec<UserProfile>,
    chat_input: NodeRef,
    wss: WebsocketService,
    messages: Vec<MessageData>,
    _producer: Box<dyn Bridge<EventBus>>,
    dark_mode: bool,
}

impl Component for Chat {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let (user, _) = ctx
            .link()
            .context::<User>(Callback::noop())
            .expect("context to be set");
        let wss = WebsocketService::new();
        let username = user.username.borrow().clone();

        let message = WebSocketMessage {
            message_type: MsgTypes::Register,
            data: Some(username.to_string()),
            data_array: None,
        };

        if let Ok(_) = wss
            .tx
            .clone()
            .try_send(serde_json::to_string(&message).unwrap())
        {
            log::debug!("message sent successfully");
        }

        Self {
            users: vec![],
            messages: vec![],
            chat_input: NodeRef::default(),
            wss,
            _producer: EventBus::bridge(ctx.link().callback(Msg::HandleMsg)),
            dark_mode: false,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::HandleMsg(s) => {
                let msg: WebSocketMessage = serde_json::from_str(&s).unwrap();
                match msg.message_type {
                    MsgTypes::Users => {
                        let users_from_message = msg.data_array.unwrap_or_default();
                        self.users = users_from_message
                            .iter()
                            .map(|u| UserProfile {
                                name: u.into(),
                                avatar: format!(
                                    "https://avatars.dicebear.com/api/adventurer-neutral/{}.svg",
                                    u
                                )
                                .into(),
                            })
                            .collect();
                        return true;
                    }
                    MsgTypes::Message => {
                        let message_data: MessageData =
                            serde_json::from_str(&msg.data.unwrap()).unwrap();
                        self.messages.push(message_data);
                        return true;
                    }
                    _ => {
                        return false;
                    }
                }
            }
            Msg::SubmitMessage => {
                let input = self.chat_input.cast::<HtmlInputElement>();
                if let Some(input) = input {
                    //log::debug!("got input: {:?}", input.value());
                    let message = WebSocketMessage {
                        message_type: MsgTypes::Message,
                        data: Some(input.value()),
                        data_array: None,
                    };
                    if let Err(e) = self
                        .wss
                        .tx
                        .clone()
                        .try_send(serde_json::to_string(&message).unwrap())
                    {
                        log::debug!("error sending to channel: {:?}", e);
                    }
                    input.set_value("");
                };
                false
            }
            Msg::ToggleDarkMode => {
                self.dark_mode = !self.dark_mode;
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let submit = ctx.link().callback(|_| Msg::SubmitMessage);
        let toggle = ctx.link().callback(|_| Msg::ToggleDarkMode);
        let root_class = if self.dark_mode {
            "flex w-screen bg-gray-900 text-white"
        } else {
            "flex w-screen bg-white text-black"
        };
        let sidebar_class = if self.dark_mode {
            "flex-none w-56 h-screen bg-gray-800"
        } else {
            "flex-none w-56 h-screen bg-gray-100"
        };
        let message_bg = if self.dark_mode {
            "bg-gray-700"
        } else {
            "bg-gray-100"
        };
        let input_bg = if self.dark_mode {
            "bg-gray-700 text-white"
        } else {
            "bg-gray-100 text-black"
        };

        html! {
            <div class={root_class}>
                <div class={sidebar_class}>
                    <div class="text-xl p-3 flex justify-between items-center">
                        {"Users"}
                        <button onclick={toggle} class="text-sm bg-gray-300 dark:bg-gray-700 p-1 rounded">
                            { if self.dark_mode { "☀️" } else { "🌙" } }
                        </button>
                    </div>
                    {
                        self.users.clone().iter().map(|u| {
                            html!{
                                <div class="flex m-3 bg-white dark:bg-gray-700 rounded-lg p-2">
                                    <div>
                                        <img class="w-12 h-12 rounded-full" src={u.avatar.clone()} alt="avatar"/>
                                    </div>
                                    <div class="flex-grow p-3">
                                        <div class="flex text-xs justify-between">
                                            <div>{u.name.clone()}</div>
                                        </div>
                                        <div class="text-xs text-gray-400">
                                            {"Hi there!"}
                                        </div>
                                    </div>
                                </div>
                            }
                        }).collect::<Html>()
                    }
                </div>
                <div class="grow h-screen flex flex-col">
                    <div class="w-full h-14 border-b-2 border-gray-300"><div class="text-xl p-3">{"💬 Chat!"}</div></div>
                    <div class="w-full grow overflow-auto border-b-2 border-gray-300">
                        {
                            self.messages.iter().map(|m| {
                                let user = self.users.iter().find(|u| u.name == m.from).unwrap();
                                html!{
                                    <div class={classes!("flex", "items-end", "w-3/6", message_bg, "m-8", "rounded-tl-lg", "rounded-tr-lg", "rounded-br-lg")}>
                                        <img class="w-8 h-8 rounded-full m-3" src={user.avatar.clone()} alt="avatar"/>
                                        <div class="p-3">
                                            <div class="text-sm">
                                                {m.from.clone()}
                                            </div>
                                            <div class="text-xs text-gray-400">
                                                {
                                                    if m.message.ends_with(".gif") {
                                                        html! { <img class="mt-3" src={m.message.clone()} /> }
                                                    } else {
                                                        html! { {m.message.clone()} }
                                                    }
                                                }
                                            </div>
                                        </div>
                                    </div>
                                }
                            }).collect::<Html>()
                        }

                    </div>
                    <div class="w-full h-14 flex px-3 items-center">
                        <input ref={self.chat_input.clone()} type="text" placeholder="Message" class={classes!("block", "w-full", "py-2", "pl-4", "mx-3", input_bg, "rounded-full", "outline-none")}
                            name="message"
                            required=true
                        />
                        <button onclick={submit} class="p-3 shadow-sm bg-blue-600 w-10 h-10 rounded-full flex justify-center items-center text-white">
                            <svg fill="#ffffff" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg" class="fill-white">
                                <path d="M0 0h24v24H0z" fill="none"></path>
                                <path d="M2.01 21L23 12 2.01 3 2 10l15 2-15 2z"></path>
                            </svg>
                        </button>
                    </div>
                </div>
            </div>
        }
    }
}