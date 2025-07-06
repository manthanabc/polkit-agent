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

fn start_session(session: &AgentSession, password: String, task: gio::Task<String>) {
    let sub_loop = glib::MainLoop::new(None, true);

    let sub_loop_2 = sub_loop.clone();

    session.connect_completed(move |session, _success| {
        unsafe {
            task.clone().return_result(Ok("success".to_string()));
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
}

#[derive(Debug, Default)]
struct PolkitApp {
    sessions: BTreeMap<iced::window::Id, AuthSession>,
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

                    futures::future::ready(())
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
                if let Some(session) = self.sessions.get(&id) {
                    let user: UnixUser = UnixUser::new_for_name(&session.selected_user).unwrap();
                    let ass = AgentSession::new(&user, &session.cookie);

                    start_session(&ass, session.password.clone(), session.task.clone());
                } else {
                    return Command::none();
                }
                task::effect(Action::Window(WindowAction::Close(id)))
            }

            Message::AuthenticationSuccess(id) => {
                task::effect(Action::Window(WindowAction::Close(id)))
            }

            Message::AuthenticationFailed(id, error) => {
                if let Some(session) = self.sessions.get_mut(&id) {
                    session.error = Some(error);
                }
                Command::none()
            }

            Message::Cancel(id) | Message::Close(id) => {
                task::effect(Action::Window(WindowAction::Close(id)))
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
        if let Some(_error) = &session.error {
            // content = content
            //     .push(text(error).style(|theme| iced::theme::Text::Color(theme.palette().danger)));
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
