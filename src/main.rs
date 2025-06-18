use iced::widget::{button, column, pick_list, row, text, text_input};
use iced::{Bottom, Center, Color, Element, Fill, Right, Task as Command, Theme};
use iced_layershell::reexport::Anchor;
use iced_layershell::settings::{LayerShellSettings, Settings};
use iced_layershell::{Application, to_layer_message};

pub fn main() -> Result<(), iced_layershell::Error> {
    InputRegionExample::run(Settings {
        layer_settings: LayerShellSettings {
            size: Some((800, 300)),
            anchor: Anchor::Bottom | Anchor::Left | Anchor::Right | Anchor::Top,
            ..Default::default()
        },
        ..Default::default()
    })
}

#[derive(Copy, Clone)]
struct InputRegionExample;

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
    type Theme = Theme;
    type Executor = iced::executor::Default;

    fn new(_flags: ()) -> (Self, Command<Message>) {
        (Self, Command::none())
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
                row![
                    text("Authenticating as:").size(16),
                    user_picker.text_size(16)
                ]
                .spacing(5)
                .align_y(Center),
                password_input,
            ]
            .spacing(20),
            column![
                row![
                    button("Cancel").on_press(Message::Cancel).padding(10),
                    button("Authenticate")
                        .on_press(Message::Authenticate)
                        .padding(10),
                ]
                .spacing(20)
                .height(Fill)
                .align_y(Bottom),
            ]
            .width(Fill)
            .align_x(Right)
        ]
        .spacing(15)
        .padding(40)
        .max_width(800)
        .into()
    }

    fn style(&self, theme: &Self::Theme) -> iced_layershell::Appearance {
        use iced_layershell::Appearance;
        Appearance {
            background_color: Color::from_rgba(0.00, 0.00, 0.00, 1.00),
            text_color:Color::from_rgba(0.80, 0.80, 0.80, 1.00),
        }
    }
}
