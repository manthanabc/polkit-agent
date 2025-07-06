pub use crate::Message;
use futures::channel::mpsc::Sender;
use glib::object::Cast;
use glib::subclass::prelude::*;
use polkit_agent_rs::Listener;
use polkit_agent_rs::gio;
use polkit_agent_rs::polkit;
use polkit_agent_rs::polkit::UnixUser;
use polkit_agent_rs::subclass::ListenerImpl;
use std::sync::{Arc, Mutex};

#[derive(Default)]
pub struct MyPolkitImpl {
    pub sender: Arc<Mutex<Option<Sender<Message>>>>,
}

#[glib::object_subclass]
impl ObjectSubclass for MyPolkitImpl {
    const NAME: &'static str = "MyPolkit";
    type Type = MyPolkit;
    type ParentType = Listener;
}

impl ObjectImpl for MyPolkitImpl {}

glib::wrapper! {
     pub struct MyPolkit(ObjectSubclass<MyPolkitImpl>)
         @extends Listener;
}

impl ListenerImpl for MyPolkitImpl {
    type Message = String;
    fn initiate_authentication(
        &self,
        _action_id: &str,
        message: &str,
        icon_name: &str,
        _details: &polkit::Details,
        cookie: &str,
        identities: Vec<polkit::Identity>,
        _cancellable: gio::Cancellable,
        task: gio::Task<Self::Message>,
    ) {
        let users: Vec<UnixUser> = identities
            .into_iter()
            .flat_map(|idenifier| idenifier.dynamic_cast())
            .collect();

        if let Ok(mut guard) = self.sender.lock() {
            if let Some(sender) = guard.as_mut() {
                let _ = sender.try_send(Message::NewSession(
                    cookie.to_string(),
                    users
                        .iter()
                        .map(|user| user.name().unwrap().to_string())
                        .collect(),
                    task,
                    message.to_string(),
                    icon_name.to_string(),
                ));
            }
        }
    }

    fn initiate_authentication_finish(
        &self,
        gio_result: Result<gio::Task<Self::Message>, glib::Error>,
    ) -> bool {
        gio_result.is_ok()
    }
}

impl MyPolkit {
    pub fn new(sender: Arc<Mutex<Sender<Message>>>) -> Self {
        let obj: Self = glib::Object::new();
        let imp = obj.imp();
        let sender_clone = sender.lock().unwrap().clone();
        *imp.sender.lock().unwrap() = Some(sender_clone);
        obj
    }
}
