use macroquad::prelude::*;

mod domain;
mod ui;

fn app_conf() -> Conf {
    Conf {
        window_title: "Bible Reader".to_string(),
        window_width: 1024,
        window_height: 768,
        ..Default::default()
    }
}

#[macroquad::main(app_conf)]
async fn main() {
    ui::run().await;
}