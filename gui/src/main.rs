use gtk::prelude::WidgetExt;
use relm::{connect, Relm, Update, Widget};

struct Model;

#[derive(relm_derive::Msg)]
enum Message {
    Quit,
}

struct Window {
    _model: Model,
    root: gtk::Window,
}

impl Update for Window {
    type Model = Model;
    type ModelParam = ();
    type Msg = Message;

    fn model(_relm: &relm::Relm<Self>, _param: Self::ModelParam) -> Self::Model {
        Model
    }

    fn update(&mut self, event: Self::Msg) {
        match event {
            Message::Quit => gtk::main_quit(),
        }
    }
}

impl Widget for Window {
    type Root = gtk::Window;

    fn root(&self) -> Self::Root {
        self.root.clone()
    }

    fn view(relm: &Relm<Self>, model: Self::Model) -> Self {
        let root = gtk::WindowBuilder::new()
            .type_(gtk::WindowType::Toplevel)
            .title("Chess!")
            .build();

        connect!(
            relm,
            root,
            connect_delete_event(_, _),
            return (Some(Message::Quit), gtk::Inhibit(false))
        );

        root.show_all();

        Self {
            _model: model,
            root,
        }
    }
}

fn main() {
    Window::run(()).unwrap();
}
