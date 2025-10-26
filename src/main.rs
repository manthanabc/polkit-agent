use iced::window::Id;
use iced::{Element, Event, Task as Command, event};
use iced_runtime::window::Action as WindowAction;
use iced_runtime::{Action, task};

use iced_layershell::build_pattern::{MainSettings, daemon};
use iced_layershell::reexport::{Anchor, Layer, NewLayerShellSettings};
use iced_layershell::settings::{LayerShellSettings, StartMode};
use iced_layershell::to_layer_message;
use polkit_agent_rs::polkit::UnixUser;
use std::collections::BTreeMap;

use futures::channel::mpsc::Sender;
use iced::widget::{Space, button, column, pick_list, row, text, text_input};
use iced::{Bottom, Center, Fill};
use polkit_agent_rs::RegisterFlags;
use polkit_agent_rs::Session as AgentSession;
use polkit_agent_rs::gio;
use polkit_agent_rs::polkit::UnixSession;
use polkit_agent_rs::traits::ListenerExt;
use std::sync::Arc;
use std::sync::Mutex;
mod mypolkit;
use mypolkit::MyPolkit;

const OBJECT_PATH: &str = "/org/waycrate/PolicyKit1/AuthenticationAgent";
const MAX_RETRIES: u32 = 3;

fn start_session(
    username: String,
    cookie: String,
    password: String,
    task: gio::Task<String>,
    window_id: Id,
    sender: Arc<Mutex<Sender<Message>>>,
) {
    let user: UnixUser = UnixUser::new_for_name(&username).unwrap();
    let session = AgentSession::new(&user, &cookie);
    let sub_loop = glib::MainLoop::new(None, true);

    let sub_loop_2 = sub_loop.clone();
    let sender_clone = sender.clone();

    session.connect_completed(move |session, success| {
        unsafe {
            if success {
                task.clone().return_result(Ok("success".to_string()));
                let _ = sender_clone
                    .lock()
                    .unwrap()
                    .try_send(Message::AuthenticationSuccess(window_id));
            } else {
                task.clone().return_result(Err(glib::Error::new(
                    glib::FileError::Failed,
                    "Authentication failed",
                )));
                let _ = sender_clone
                    .lock()
                    .unwrap()
                    .try_send(Message::AuthenticationFailed(
                        window_id,
                        "Authentication failed".to_string(),
                    ));
            }
        }
        session.cancel();
        sub_loop_2.quit();
    });

    session.connect_show_info(move |_session, info| {
        println!("info: {info}");
    });

    session.connect_show_error(move |_session, error| {
        eprintln!("error: {error}");
    });

    session.connect_request(move |session, request, _echo_on| {
        println!("{}", request);
        if !request.starts_with("Password:") {
            return;
        }
        session.response(&password);
    });
    session.initiate();
    sub_loop.run();
}

pub fn main() -> Result<(), iced_layershell::Error> {
    daemon(
        PolkitApp::namespace,
        PolkitApp::update,
        PolkitApp::view,
        PolkitApp::remove_id,
    )
    .subscription(PolkitApp::subscription)
    .settings(MainSettings {
        layer_settings: LayerShellSettings {
            start_mode: StartMode::Background,
            size: Some((600, 250)),
            exclusive_zone: 400,
            anchor: Anchor::Bottom | Anchor::Left | Anchor::Right,
            ..Default::default()
        },
        ..Default::default()
    })
    .run_with(PolkitApp::new)
}

#[derive(Debug, Clone)]
struct AuthSession {
    users: Vec<String>,
    selected_user: String,
    cookie: String,
    password: String,
    error: Option<String>,
    task: gio::Task<String>,
    message: String,
    retry_count: u32,
    max_retries: u32,
}

#[derive(Debug)]
struct PolkitApp {
    sessions: BTreeMap<iced::window::Id, AuthSession>,
    sender: Option<Arc<Mutex<Sender<Message>>>>,
}

#[to_layer_message(multi)]
#[derive(Debug, Clone)]
pub enum Message {
    WindowClosed(iced::window::Id),
    UserSelected(Id, String),
    PasswordSubmit(Id, String),
    Authenticate(Id),
    Cancel(Id),
    NewSession(String, Vec<String>, gio::Task<String>, String, String),
    Close(Id),
    AuthenticationSuccess(Id),
    AuthenticationFailed(Id, String),
    IcedEvent(Event),
    SetSender(Arc<Mutex<Sender<Message>>>),
}

impl PolkitApp {
    fn remove_id(&mut self, id: iced::window::Id) {
        self.sessions.remove(&id);
    }
}

impl PolkitApp {
    fn new() -> (Self, Command<Message>) {
        (
            Self {
                sessions: BTreeMap::new(),
                sender: None,
            },
            Command::none(),
        )
    }

    fn namespace(&self) -> String {
        String::from("PolkitApp - Iced")
    }

    fn subscription(&self) -> iced::Subscription<Message> {
        iced::Subscription::batch(vec![
            iced::Subscription::run(|| {
                iced::stream::channel(100, |sender| {
                    let sender = Arc::new(Mutex::new(sender));
                    let sender_clone = sender.clone();

                    std::thread::spawn(move || {
                        let main_loop = glib::MainLoop::new(None, true);

                        let my_polkit = MyPolkit::new(sender);

                        let Ok(subject) = UnixSession::new_for_process_sync(
                            nix::unistd::getpid().as_raw(),
                            gio::Cancellable::NONE,
                        ) else {
                            unreachable!();
                        };
                        let Ok(_handle) = my_polkit.register(
                            RegisterFlags::NONE,
                            &subject,
                            OBJECT_PATH,
                            gio::Cancellable::NONE,
                        ) else {
                            unreachable!();
                        };

                        main_loop.run();
                    });

                    async move {
                        let _ = sender_clone
                            .lock()
                            .unwrap()
                            .try_send(Message::SetSender(sender_clone.clone()));
                        futures::future::pending::<()>().await;
                    }
                })
            }),
            iced::window::close_events().map(Message::WindowClosed),
            event::listen().map(Message::IcedEvent),
        ])
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::IcedEvent(event) => {
                if let Event::Keyboard(iced::keyboard::Event::KeyPressed {
                    key: iced::keyboard::Key::Named(iced::keyboard::key::Named::Escape),
                    ..
                }) = event
                {
                    if let Some(id) = self.sessions.keys().next().cloned() {
                        return Command::perform(async move { id }, Message::Close);
                    }
                }
                Command::none()
            }

            Message::NewSession(cookie, users, task, messg, _icon) => {
                let id = iced::window::Id::unique();
                let selected_user = users.first().cloned().unwrap_or_else(|| "root".to_string());
                self.sessions.insert(
                    id,
                    AuthSession {
                        users: users.clone(),
                        selected_user,
                        cookie: cookie.clone(),
                        password: String::new(),
                        error: None,
                        task,
                        message: messg,
                        retry_count: 0,
                        max_retries: MAX_RETRIES,
                    },
                );

                Command::perform(async {}, move |_| Message::NewLayerShell {
                    settings: NewLayerShellSettings {
                        size: Some((600, 250)),
                        anchor: Anchor::Right | Anchor::Top | Anchor::Left | Anchor::Bottom,
                        layer: Layer::Top,
                        use_last_output: false,
                        ..Default::default()
                    },
                    id,
                })
            }

            Message::UserSelected(id, user) => {
                if let Some(session) = self.sessions.get_mut(&id) {
                    session.selected_user = user;
                }
                Command::none()
            }

            Message::PasswordSubmit(id, password) => {
                if let Some(session) = self.sessions.get_mut(&id) {
                    session.password = password;
                }
                Command::none()
            }

            Message::Authenticate(id) => {
                if let Some(session) = self.sessions.get_mut(&id) {
                    let username = session.selected_user.clone();
                    let cookie = session.cookie.clone();
                    let password = session.password.clone();
                    let task = session.task.clone();

                    session.error = None;

                    if let Some(sender) = self.sender.clone() {
                        std::thread::spawn(move || {
                            start_session(username, cookie, password, task, id, sender);
                        });
                    }
                }
                Command::none()
            }

            Message::AuthenticationSuccess(id) => {
                task::effect(Action::Window(WindowAction::Close(id)))
            }

            Message::AuthenticationFailed(id, error) => {
                if let Some(session) = self.sessions.get_mut(&id) {
                    session.retry_count += 1;

                    if session.retry_count >= session.max_retries {
                        return task::effect(Action::Window(WindowAction::Close(id)));
                    }

                    let remaining = session.max_retries - session.retry_count;
                    session.error = Some(format!(
                        "{}. {} attempt{} remaining.",
                        error,
                        remaining,
                        if remaining == 1 { "" } else { "s" }
                    ));
                    session.password.clear();
                }
                Command::none()
            }

            Message::Cancel(id) | Message::Close(id) => {
                task::effect(Action::Window(WindowAction::Close(id)))
            }

            Message::SetSender(sender) => {
                self.sender = Some(sender);
                Command::none()
            }

            _ => Command::none(),
        }
    }

    fn view(&self, id: iced::window::Id) -> Element<Message> {
        let Some(session) = self.sessions.get(&id) else {
            return Space::with_height(0).into();
        };

        let user_picker = pick_list(
            session.users.clone(),
            Some(session.selected_user.clone()),
            move |s| Message::UserSelected(id, s),
        );

        let password_input = text_input("Password", &session.password)
            .style(|theme, status| {
                let mut style = iced::widget::text_input::default(theme, status);
                style.border.radius = iced::border::radius(8.0);
                style
            })
            .on_input(move |s| Message::PasswordSubmit(id, s))
            .on_submit(Message::Authenticate(id))
            .padding(10);

        let mut content = column![
            column![
                text(&session.message).size(20),
                column![
                    row![
                        text("Authenticating as:").size(16),
                        user_picker.text_size(16)
                    ]
                    .spacing(5)
                    .align_y(Center),
                    password_input,
                ]
                .spacing(10)
                .padding(0),
            ]
            .spacing(20)
            .padding(25)
        ];
        if let Some(error) = &session.error {
            content = content.push(
                text(error)
                    .size(14)
                    .color(iced::Color::from_rgb(0.8, 0.0, 0.0)),
            );
        }

        content = content.push(Space::with_height(Fill)).push(
            row![
                button(column![text("Cancel")].width(Fill).align_x(Center))
                    .on_press(Message::Cancel(id))
                    .padding(13),
                button(column![text("Authenticate")].width(Fill).align_x(Center))
                    .on_press(Message::Authenticate(id))
                    .padding(13)
            ]
            .spacing(2)
            .width(Fill)
            .align_y(Bottom),
        );

        content.padding(1).height(Fill).into()
    }
}
