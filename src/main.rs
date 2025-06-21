use iced::window::Id;
use iced::{Element, Event, Task as Command, event};
use iced_runtime::window::Action as WindowAction;
use iced_runtime::{Action, task};

use iced_layershell::build_pattern::{MainSettings, daemon};
use iced_layershell::reexport::{Anchor, KeyboardInteractivity, Layer, NewLayerShellSettings};
use iced_layershell::settings::{LayerShellSettings, StartMode};
use iced_layershell::to_layer_message;

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
            size: Some((800, 250)),
            exclusive_zone: 400,
            anchor: Anchor::Bottom | Anchor::Left | Anchor::Right,
            ..Default::default()
        },
        ..Default::default()
    })
    .run_with(|| Counter::new("Hello"))
}

#[derive(Debug, Default)]
struct Counter {
    value: i32,
    text: String,
    session: Option<AgentSession>,
}

#[to_layer_message(multi)]
#[derive(Debug, Clone)]
enum Message {
    WindowClosed(iced::window::Id),
    UserSelected(String),
    PasswordChanged(String),
    Authenticate,
    Cancel,
    NewWindow,
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
                session: None,
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
                        println!("STARTED THREAD");
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
                        println!("STARTED mainlopo");

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

            Message::NewWindow => {
                // if self.window_shown {
                //     return Command::none();
                // }

                // self.window_shown = true;
                Command::done(Message::NewLayerShell {
                    settings: NewLayerShellSettings {
                        size: None,
                        exclusive_zone: None,
                        anchor: Anchor::Right | Anchor::Top | Anchor::Left | Anchor::Bottom,
                        layer: Layer::Top,
                        margin: Some((100, 100, 100, 100)),
                        keyboard_interactivity: KeyboardInteractivity::OnDemand,
                        use_last_output: false,
                        ..Default::default()
                    },
                    id: iced::window::Id::unique(),
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
            .on_input(Message::PasswordChanged)
            .on_submit(Message::Authenticate)
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
                    .padding(10),
                button(column![text("Authenticate")].width(Fill).align_x(Center))
                    .on_press(Message::Authenticate)
                    .padding(10)
            ]
            .spacing(2)
            .width(Fill)
            .align_y(Bottom),
        ]
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

pub fn pain() -> Result<(), iced_layershell::Error> {
    let main_loop = glib::MainLoop::new(None, true);

    let (sender, mut receiver) = mpsc::channel::<Message>(10);
    let my_polkit = MyPolkit::new(Arc::new(Mutex::new(sender)));

    let Ok(subject) =
        UnixSession::new_for_process_sync(nix::unistd::getpid().as_raw(), gio::Cancellable::NONE)
    else {
        return Ok(());
    };
    let Ok(_handle) = my_polkit.register(
        RegisterFlags::NONE,
        &subject,
        OBJECT_PATH,
        gio::Cancellable::NONE,
    ) else {
        return Ok(());
    };

    // iced::subscription()
    main_loop.run();
    Ok(())
}
