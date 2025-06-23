use polkit_agent_rs::Listener;
use std::sync::{Arc, Mutex};
pub mod imp;
use crate::Message;
use futures::channel::mpsc::Sender;

use glib::subclass::prelude::*;
glib::wrapper! {
     pub struct MyPolkit(ObjectSubclass<imp::MyPolkit>)
         @extends Listener;
}

impl super::MyPolkit {
    pub fn new(sender: Arc<Mutex<Sender<Message>>>) -> Self {
        let obj: Self = glib::Object::new::<MyPolkit>();

        // Set the sender field inside the impl
        let imp = imp::MyPolkit::from_obj(&obj);
        // // Properly set the sender using RefCell's borrow_mut
        // if let Some(sener) = imp.sender {
        //     *sener.borrow_mut() = Some(sender);
        // }
        //
        // if let Ok(mut sender) = sender.lock() {
        //     println!("GOTCA");
        //     let _ = sender.try_send(Message::NewWindow);
        // } else {
        //     println!("NOPE");
        //     eprintln!("No sender available");
        // }

        let sender_clone = sender.lock().unwrap().clone(); // clone the inner Sender
        *imp.sender.lock().unwrap() = Some(sender_clone);
        println!("{:?}", obj);
        obj
    }
}

impl Default for MyPolkit {
    fn default() -> Self {
        glib::Object::new()
    }
}
