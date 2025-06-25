use iced::window::Id;
use iced::{Element, Event, Task as Command, event};
use iced_runtime::window::Action as WindowAction;
use iced_runtime::{Action, task};

use iced_layershell::build_pattern::{MainSettings, daemon};
use iced_layershell::reexport::{Anchor, KeyboardInteractivity, Layer, NewLayerShellSettings};
use iced_layershell::settings::{LayerShellSettings, StartMode};
use iced_layershell::to_layer_message;
use polkit_agent_rs::polkit;
use polkit_agent_rs::polkit::UnixUser;
use std::collections::BTreeMap;

use iced::widget::{Space, button, column, pick_list, row, text, text_input};
use iced::{Bottom, Center, Fill, Theme};
use polkit_agent_rs::traits::ListenerExt;
use std::sync::Arc;
use std::sync::Mutex;
mod mypolkit;
use mypolkit::MyPolkit;

use polkit_agent_rs::Session as AgentSession;

use futures::channel::mpsc;

pub fn main() -> Result<(), iced_layershell::Error> {
    daemon(
        Counter::namespace,
        Counter::update,
        Counter::view,
        Counter::remove_id,
    )
    .subscription(Counter::subscription)
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
    .run_with(|| Counter::new("Hello"))
}

struct AuthSession {
    user: Vec<String>,
    pass: String,
    sess: AgentSession,
}

#[derive(Debug, Default)]
struct Counter {
    value: i32,
    text: String,
    session: BTreeMap<iced::window::Id, AgentSession>,
}

// use mypolkit::imp::Session;
use std::cell::RefCell;
#[to_layer_message(multi)]
#[derive(Debug, Clone)]
enum Message {
    WindowClosed(iced::window::Id),
    UserSelected(String),
    PasswordSubmit(String),
    Authenticate(iced::window::Id),
    Cancel,
    NewWindow,
    NewSession(String, Vec<String>),
    Close(Id),
    IcedEvent(Event),
}

impl Counter {
    fn remove_id(&mut self, _id: iced::window::Id) {}
}

impl Counter {
    fn new(text: &str) -> (Self, Command<Message>) {
        (
            Self {
                value: 0,
                text: text.to_string(),
                session: BTreeMap::new(),
            },
            Command::none(),
        )
    }

    fn namespace(&self) -> String {
        String::from("Counter - Iced")
    }

    fn subscription(&self) -> iced::Subscription<Message> {
        iced::Subscription::batch(vec![
            iced::Subscription::run(|| {
                iced::stream::channel(100, |sender| {
                    let sender = Arc::new(Mutex::new(sender));

                    std::thread::spawn(move || {
                        let main_loop = glib::MainLoop::new(None, true);

                        // let (sender, mut receiver) = mpsc::channel::<Message>(10);
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
        use iced::Event;
        use iced::keyboard;
        use iced::keyboard::key::Named;
        match message {
            Message::IcedEvent(event) => {
                match event {
                    Event::Keyboard(keyboard::Event::KeyPressed {
                        key: keyboard::Key::Named(Named::Escape),
                        ..
                    }) => {}
                    _ => {}
                }
                Command::none()
            }

            Message::NewSession(cookie, user) => {
                // if self.window_shown {
                //     return Command::none();
                // }

                // self.window_shown = true;
                //
                //
                //

                // let users: Vec<UnixUser> = identities
                //     .into_iter()
                //     .flat_map(|idenifier| idenifier.dynamic_cast())
                //     .collect();
                // let Some((name, index)) = choose_user(&users) else {
                //     cancellable.cancel();
                //     return;
                // };
                // let session = AgentSession::new(&users[index], cookie);

                // let count = Arc::new(AtomicU8::new(0));
                // start_session(&session, name, cancellable, task, cookie.to_string(), count);

                // let users: Vec<UnixUser> = identities
                //     .into_iter()
                //     .flat_map(|idenifier| idenifier.dynamic_cast())
                //     .collect();
                // let Some((name, index)) = choose_user(&users) else {
                //     cancellable.cancel();
                //     return;
                // };
                let user: UnixUser = UnixUser::new_for_name(&user[0]).unwrap();
                let session = AgentSession::new(&user, &cookie);

                // let count = Arc::new(AtomicU8::new(0));
                let id = iced::window::Id::unique();
                self.session.insert(id, session);
                // start_session(&session, name, cancellable, task, cookie.to_string(), count);
                Command::done(Message::NewLayerShell {
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
            Message::Close(id) => task::effect(Action::Window(WindowAction::Close(id))),
            _ => unreachable!(),
        }
    }

    fn view(&self, id: iced::window::Id) -> Element<Message> {
        let user_picker = pick_list(
            ["root".to_string(), "admin".to_string(), "user".to_string()],
            Some("root".to_string()),
            Message::UserSelected,
        );

        let password_input = text_input("Password", "uwu")
            .style(|theme, status| {
                let mut style = iced::widget::text_input::default(theme, status);
                style.border.radius = iced::border::radius(8.0);
                style
            })
            .on_input(Message::PasswordSubmit)
            .on_submit(Message::Authenticate(id))
            .padding(10);

        column![
            column![
                text("Authentication Required to set locale")
                    .size(25)
                    .width(Fill),
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
            .padding(30),
            Space::with_height(Fill),
            row![
                button(column![text("Cancel")].width(Fill).align_x(Center))
                    .on_press(Message::NewWindow)
                    .padding(13),
                button(column![text("Authenticate")].width(Fill).align_x(Center))
                    .on_press(Message::Authenticate(id))
                    .padding(13)
            ]
            .spacing(2)
            .width(Fill)
            .align_y(Bottom),
        ]
        .padding(1)
        .height(Fill)
        .into()
    }

    //         button("newwindowLeft").on_press(Message::NewWindowLeft),
    //         button("newwindowRight").on_press(Message::NewWindowRight),
}

use polkit_agent_rs::RegisterFlags;
use polkit_agent_rs::gio;
use polkit_agent_rs::polkit::UnixSession;

const OBJECT_PATH: &str = "/org/waycrate/PolicyKit1/AuthenticationAgent";
