use polkit_agent_rs::Listener;
mod imp;
use crate::Message;
use futures::channel::mpsc::Sender;

use glib::subclass::prelude::*;
glib::wrapper! {
     pub struct MyPolkit(ObjectSubclass<imp::MyPolkit>)
         @extends Listener;
}

impl super::MyPolkit {
    pub fn new(sender: Sender<Message>) -> Self {
        let obj: Self = glib::Object::new::<MyPolkit>();

        // Set the sender field inside the impl
        let imp = imp::MyPolkit::from_obj(&obj);
        println!("relaced {:?}", imp.sender);
        imp.sender.replace(Some(sender));
        println!("relaced {:?}", imp.sender);
        obj
    }
}

// impl Default for MyPolkit {
//     fn default() -> Self {
//         glib::Object::new()
//     }
// }
