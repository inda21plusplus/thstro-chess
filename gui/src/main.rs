use gtk::prelude::*;
use relm::Widget;

pub struct Model;

#[derive(relm_derive::Msg)]
pub enum Message {
    Quit,
}

#[relm_derive::widget]
impl Widget for MainWindow {
    type Root = gtk::Window;

    fn model() -> Model {
        Model
    }

    fn update(&mut self, event: Message) {
        match event {
            Message::Quit => gtk::main_quit(),
        }
    }

    view! {
        gtk::Window(gtk::WindowType::Toplevel) {
            title: "Chess",

            delete_event(_, _) => (Message::Quit, gtk::Inhibit(false))
        }
    }
}

fn main() {
    MainWindow::run(()).unwrap();
}
