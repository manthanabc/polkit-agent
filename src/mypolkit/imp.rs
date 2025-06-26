use crate::Message;
use dialoguer::FuzzySelect;
use dialoguer::theme::ColorfulTheme;
use futures::channel::mpsc::Sender;
use glib::error::ErrorDomain;
use glib::object::Cast;
use glib::subclass::prelude::*;
use polkit_agent_rs::Session as AgentSession;
use polkit_agent_rs::gio;
use polkit_agent_rs::gio::prelude::CancellableExt;
use polkit_agent_rs::polkit;
use polkit_agent_rs::polkit::UnixUser;
use polkit_agent_rs::subclass::ListenerImpl;
use rpassword::prompt_password;
use std::sync::Mutex;

fn choose_user(users: &[UnixUser]) -> Option<(String, usize)> {
    let names: Vec<String> = users
        .iter()
        .map(|user| user.name().unwrap().to_string())
        .collect();

    let index = FuzzySelect::with_theme(&ColorfulTheme::default())
        .with_prompt("Which user?")
        .default(0)
        .items(&names)
        .interact()
        .ok()?;
    Some((names[index].clone(), index))
}

pub struct MyPolkit {
    pub sender: Arc<Mutex<Option<Sender<Message>>>>,
}

use std::sync::Arc;
use std::sync::atomic::AtomicU8;

#[derive(Debug, Clone, Copy)]
struct SessionError;

impl ErrorDomain for SessionError {
    fn domain() -> glib::Quark {
        glib::Quark::from_str("session_error")
    }
    fn code(self) -> i32 {
        -1
    }
    fn from(code: i32) -> Option<Self>
    where
        Self: Sized,
    {
        if code == -1 {
            return Some(Self);
        }
        None
    }
}

fn start_session(
    session: &AgentSession,
    name: String,
    cancellable: gio::Cancellable,
    task: gio::Task<String>,
    cookie: String,
    count: Arc<AtomicU8>,
) {
    let sub_loop = glib::MainLoop::new(None, true);
    let name2 = name.clone();
    let cancellable2 = cancellable.clone();

    let sub_loop_2 = sub_loop.clone();
    session.connect_completed(move |session, success| {
        let name2 = name2.clone();
        let cancellable2 = cancellable2.clone();
        let task = task.clone();
        let cookie = cookie.clone();
        let count = count.clone();
        if !success {
            count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            if count.load(std::sync::atomic::Ordering::Relaxed) >= 3 {
                unsafe {
                    task.return_result(Err(glib::Error::new(
                        SessionError,
                        "You have used all attempts",
                    )));
                }
                session.cancel();

                sub_loop_2.quit();
                return;
            }
            let user: UnixUser = UnixUser::new_for_name(&name2).unwrap();
            let session = AgentSession::new(&user, &cookie);
            start_session(&session, name2, cancellable2, task, cookie, count);
        } else {
            unsafe {
                task.return_result(Ok("success".to_string()));
            }
        }
        session.cancel();

        sub_loop_2.quit();
    });
    session.connect_show_info(|_session, info| {
        println!("info: {info}");
    });
    session.connect_show_error(|_session, error| {
        eprintln!("error: {error}");
    });
    session.connect_request(move |session, request, _echo_on| {
        println!("{}", request);
        if !request.starts_with("Password:") {
            return;
        }

        // sender.try_send(Message::Request_password)
        // reciever.wait(Message) -> Message->Cancled, Message->password(String)
        // iced daemon
        // Create window
        let Ok(password) = prompt_password(format!("{name} password: ")) else {
            session.cancel();
            cancellable.cancel();
            return;
        };

        // On Submit call this
        session.response(&password);
    });
    session.initiate();
    sub_loop.run();
}

#[derive(Debug, Clone)]
pub struct Session {
    identifier: Vec<polkit::Identity>,
    cancellable: gio::Cancellable,
    cookie: String,
    session: Option<AgentSession>,
}

impl Session {
    fn new(id: Vec<polkit::Identity>, cal: gio::Cancellable, cookie: String) -> Self {
        Self {
            identifier: id,
            cancellable: cal,
            cookie,
            session: None,
        }
    }
}
impl ListenerImpl for MyPolkit {
    type Message = String;
    fn initiate_authentication(
        &self,
        _action_id: &str,
        _message: &str,
        _icon_name: &str,
        _details: &polkit::Details,
        cookie: &str,
        identities: Vec<polkit::Identity>,
        cancellable: gio::Cancellable,
        task: gio::Task<Self::Message>,
    ) {
        println!("icon name {:?} {:?}", _message, _icon_name);
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
                    _message.to_string(),
                    _icon_name.to_string(),
                ));
            } else {
                println!("Mypolkit not initialized correctly");
            }
        } else {
            eprintln!("Failed to aquire lock");
        }
    }

    fn initiate_authentication_finish(
        &self,
        gio_result: Result<gio::Task<Self::Message>, glib::Error>,
    ) -> bool {
        match gio_result {
            Ok(_) => true,
            Err(err) => {
                println!("err: {err:?}");
                false
            }
        }
    }
}

impl Default for MyPolkit {
    fn default() -> Self {
        Self {
            sender: Arc::new(Mutex::new(None)),
        }
    }
}

#[glib::object_subclass]
impl ObjectSubclass for MyPolkit {
    const NAME: &'static str = "MyPolkit";
    type Type = super::MyPolkit;
    type ParentType = super::Listener;
}

impl ObjectImpl for MyPolkit {}
