use gtk::prelude::*;

fn main() {
    let application = gtk::Application::builder()
        .application_id("space.magnusson.chess")
        .build();

    let game = chess_engine::game::Game::new();

    application.connect_activate(|app| {
        let css_provider = gtk::CssProvider::new();
        let css_style = include_bytes!("style.css");
        css_provider.load_from_data(css_style).unwrap();
        gtk::StyleContext::add_provider_for_screen(
            &gtk::gdk::Screen::default().expect("Error initializing gtk css provider."),
            &css_provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );

        let window = gtk::ApplicationWindow::builder()
            .application(app)
            .title("Chess")
            .default_width(1280)
            .default_height(720)
            .build();

        window.add(&{
            let app_box = gtk::Box::new(gtk::Orientation::Horizontal, 40);
            app_box.set_widget_name("app");

            app_box.set_valign(gtk::Align::Center);

            app_box.pack_start(
                &{
                    let game_grid = gtk::Grid::new();
                    game_grid.set_size_request(800, 800);
                    game_grid.set_widget_name("chess-board");

                    for r in 0..8 {
                        for c in 0..8 {
                            game_grid.attach(
                                &{
                                    let b = gtk::Box::new(gtk::Orientation::Horizontal, 0);
                                    b.style_context().add_class(if (r + c) % 2 == 1 {
                                        "black"
                                    } else {
                                        "white"
                                    });
                                    b.set_size_request(100, 100);
                                    b.add(&{
                                        let img = gtk::Image::from_file("jjjj");

                                        img
                                    });
                                    b
                                },
                                c,
                                r,
                                1,
                                1,
                            );
                        }
                    }

                    game_grid
                },
                true,
                false,
                0,
            );

            app_box.pack_end(
                &{
                    let sidebar_container = gtk::Box::new(gtk::Orientation::Vertical, 0);
                    sidebar_container.set_valign(gtk::Align::Center);
                    sidebar_container.add(&{
                        let sidebar = gtk::Frame::new(None);

                        sidebar.style_context().add_class("aside");
                        sidebar.set_size_request(400, 400);

                        sidebar
                    });
                    sidebar_container
                },
                false,
                false,
                0,
            );

            app_box
        });

        window.show_all();
    });

    application.run();
}
