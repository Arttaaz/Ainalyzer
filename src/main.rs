use iced::{widget::canvas::Canvas, widget::canvas::Layer, Command, Application, Subscription, Element, Container, Length, Size};
use std::sync::Arc;

#[derive(Debug, Clone, Copy)]
enum Message {

}


struct App {}

impl Application for App {

    type Executor = iced::executor::Default;
    type Message = Message;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        (
            App {},
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("AInalyzer")
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        Command::none()
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        Subscription::none()
    }

    fn view(&mut self) -> Element<'_, Self::Message> {
        let canvas = Canvas::new()
        .width(400.into())
        .height(400.into());

        Container::new(canvas).width(Length::Fill)
            .height(Length::Fill)
            .padding(20)
            .center_x()
            .center_y()
            .into()
    }

}

fn main() {
    App::run(iced::Settings::default());
}
