use std::path::PathBuf;
use std::sync::Arc;

use clap::Parser;
use gtk::gdk::{Cursor, CursorType};
use gtk::glib::{clone, MainContext, Priority, Sender};
use gtk::{self, glib, prelude::*, Image};
use itertools::Itertools;
use regex::Regex;
use serde::{Deserialize, Serialize};
use warp::http::{Response, StatusCode};
use warp::reply::html;

#[derive(Debug)]
struct ViewModel {
    file_to_display: PathBuf,
}

#[derive(Deserialize, Serialize)]
struct MapPostData {
    map: String,
}

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Which resource folder to use
    #[arg(short, long, default_value = ".")]
    resources_path: PathBuf,
}

fn main() -> glib::ExitCode {
    let args: Args = Parser::parse();

    let (channel_tx, channel_rx) = std::sync::mpsc::channel::<Sender<ViewModel>>();

    let args_ = args.clone();
    std::thread::spawn(move || {
        //  TODO handle things when new app is created ?
        let channel = channel_rx.recv().unwrap();
        let runtime = tokio::runtime::Runtime::new().unwrap();

        use warp::Filter;

        let regex = Regex::new("[0-9]+-").unwrap();

        let resources_path = args_.resources_path.clone();
        let index = warp::get().and(warp::path::end()).map(move || {
            let buttons = std::fs::read_dir(&resources_path)
                .unwrap()
                .map(|it| it.unwrap().file_name().to_str().unwrap().to_string())
                .filter(|it| it.ends_with(".png"))
                .sorted()
                .map(|it| {
                    let name = regex
                        .replace(&it.replace(".png", "").replace("_", " "), "")
                        .to_string();
                    format!(
                        r#"<button style = "width:460px; font-size:2em" onclick="change_to('{it}')">
                            <img style = "width:450px" src="img/{it}" />
                            <br/>
                            {name}
                       </button>"#
                    )
                })
                .fold(String::new(), |mut acc, new| {
                    acc.push_str(&new);
                    acc
                });

            let js = r#"
                      function change_to(map= "") {
                            const req = new XMLHttpRequest();
                            req.open("POST", "map");
                            req.setRequestHeader("Content-Type", "application/json")
                            req.send(JSON.stringify({"map": map}));
                
                      }"#;
            html(format!(
                r#"
                <!doctype html>
                <html>
                  <head>
                  <title>Tatoteco</title>
                  </head>
                  <body>
                    <script>
                      {js}
                    </script>
                    <h1>Tatoteco</h1>
                    {buttons}
                  </body>
                </html>
                "#
            ))
        });

        let resources_path = args_.resources_path.clone();
        let img = warp::get()
            .and(warp::path!("img" / String))
            .and_then(move |img: String| {
                let resources_path = resources_path.clone();
                async move {
                    if img.contains('/') || !img.ends_with(".png") {
                        Err(warp::reject())
                    } else {
                        let path = resources_path.clone().join(img);
                        Ok(Response::builder()
                            .header("content-type", "application/png")
                            .header("cache-control", "age:0, max-age:86400")
                            .body(std::fs::read(path).unwrap()))
                    }
                }
            });

        let args = args_.clone();
        let map = warp::post()
            .and(warp::path("map"))
            .and(warp::body::content_length_limit(1024 * 16))
            .and(warp::body::json())
            .map(move |data: MapPostData| {
                channel
                    .send(ViewModel {
                        file_to_display: args.resources_path.join(data.map),
                    })
                    .unwrap();
                warp::reply::with_status("OK", StatusCode::OK)
            });

        let warp = warp::serve(index.or(img).or(map)).run(([0, 0, 0, 0], 8080));

        runtime.block_on(warp);
    });

    let application = Arc::new(gtk::Application::new(
        Some("fr.fredszaq.tabletop-maps"),
        Default::default(),
    ));

    application.connect_activate(move |app| {
        build_ui(
            app,
            channel_tx.clone(),
            ViewModel {
                file_to_display: args.resources_path.join("00-splash.png"),
            },
        )
    });

    application.run_with_args::<&str>(&[])
}

fn build_ui(
    application: &gtk::Application,
    channel_tx: std::sync::mpsc::Sender<Sender<ViewModel>>,
    view_model: ViewModel,
) {
    let window = gtk::ApplicationWindow::new(application);

    window.set_title("Tatoteco");
    window.set_default_size(1920, 1080);

    let picture = Image::from_file(view_model.file_to_display);

    let (view_model_tx, view_model_rx) = MainContext::channel(Priority::default());

    view_model_rx.attach(
        None,
        clone!(@weak picture => @default-return Continue(false), move |view_model: ViewModel| {
            println!("new view model {view_model:?}");
            picture.set_file(view_model.file_to_display.to_str());
            Continue(true)
        }),
    );

    window.add(&picture);

    window.fullscreen();
    window.show_all();

    window
        .window()
        .unwrap()
        .set_cursor(Cursor::for_display(&window.display(), CursorType::BlankCursor).as_ref());

    channel_tx.send(view_model_tx).unwrap();

    println!("presented");
}
