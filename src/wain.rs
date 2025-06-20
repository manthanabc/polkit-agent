use iced::widget::{button, Space, column, pick_list, row, text, text_input};
use iced::widget::theme;
use iced::{Bottom, Center, Color, Element, Fill, Right, Task as Command, Theme};
use iced_layershell::reexport::Anchor;
use iced_layershell::settings::{LayerShellSettings, Settings};
use iced_layershell::{Application, to_layer_message};

use iced::alignment::Horizontal;
use iced::widget::Text;



use polkit_agent_rs::RegisterFlags;
use polkit_agent_rs::gio;
use polkit_agent_rs::polkit::UnixSession;
use polkit_agent_rs::traits::ListenerExt;
mod mypolkit;
use mypolkit::MyPolkit;

const OBJECT_PATH: &str = "/org/waycrate/PolicyKit1/AuthenticationAgent";

pub fn main() -> Result<(), iced_layershell::Error> {

    let main_loop = glib::MainLoop::new(None, true);
    // Start a multi window daemon
    // give a sender
    // Message polkit agent -> Iced application
    // Init auth (Show window) //
    // 
    let my_polkit = MyPolkit::default();

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

    
    InputRegionExample::run(Settings {
        layer_settings: LayerShellSettings {
            size: Some((600, 250)),
            anchor: Anchor::Bottom | Anchor::Left | Anchor::Right | Anchor::Top,
            ..Default::default()
        },
        ..Default::default()
    })
}

#[derive(Clone)]
struct InputRegionExample {
    theme: theme::Theme,
}

#[to_layer_message]
#[derive(Debug, Clone)]
#[doc = "Some docs"]
enum Message {
    UserSelected(String),
    PasswordChanged(String),
    Authenticate,
    Cancel,
}

impl Application for InputRegionExample {
    type Message = Message;
    type Flags = ();
    type Theme = theme::Theme;
    type Executor = iced::executor::Default;

    fn new(_flags: ()) -> (Self, Command<Message>) {
        let s :InputRegionExample= InputRegionExample{ theme: Theme::TokyoNight };
        (s, Command::none())
    }

    fn theme(&self) -> Self::Theme {
        self.theme.clone()
    }

    fn namespace(&self) -> String {
        String::from("Wayland policy kit agent")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            _ => unreachable!(),
        }
    }

    fn view(&self) -> Element<Message> {
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
                button(
                    column![
                        text("Cancel")
                    ]   .width(Fill)
                        .align_x(Center)
                ).on_press(Message::Cancel)
                .padding(10),
                    // button("Cancel").on_press(Message::Cancel).padding(10),
                    
                button(
                    column![
                        text("Authenticate")
                    ]   .width(Fill)
                        .align_x(Center)
                ).on_press(Message::Authenticate)
                .padding(10)
                // button("Authenticate")
                //     .on_press(Message::Authenticate)
                //     .padding(10).width(Fill),
            ]
            .spacing(2)
            // .padding(5)
            .width(Fill)
            .align_y(Bottom),
        ].height(Fill)
        .into()
    }
}
