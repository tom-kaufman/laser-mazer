#![forbid(unsafe_code)]

mod app;
mod solver;

use std::time::Duration;

use tokio::runtime::Runtime;

use crate::{
    app::MyApp,
    solver::{
        orientation::Orientation,
        token::{Token, TokenType},
        LaserMazeSolver,
    },
};
use std::time;

fn main() {
    let mut cells: [Option<Token>; 25] = Default::default();

    cells[10] = Some(Token::new(
        TokenType::Checkpoint,
        Some(Orientation::North),
        false,
    ));
    cells[16] = Some(Token::new(
        TokenType::DoubleMirror,
        Some(Orientation::North),
        false,
    ));
    cells[20] = Some(Token::new(
        TokenType::CellBlocker,
        Some(Orientation::North),
        false,
    ));
    cells[23] = Some(Token::new(TokenType::Laser, None, false));

    let mut tokens_to_be_added = vec![];
    tokens_to_be_added.push(Token::new(TokenType::TargetMirror, None, false));
    tokens_to_be_added.push(Token::new(TokenType::TargetMirror, None, false));
    tokens_to_be_added.push(Token::new(TokenType::TargetMirror, None, false));
    tokens_to_be_added.push(Token::new(TokenType::TargetMirror, None, false));
    tokens_to_be_added.push(Token::new(TokenType::BeamSplitter, None, false));
    tokens_to_be_added.push(Token::new(TokenType::BeamSplitter, None, false));

    let mut solver = LaserMazeSolver::new(cells, tokens_to_be_added, 2);

    let t0 = time::Instant::now();
    let result = solver.solve();
    let t1 = time::Instant::now();
    println!("Solved puzzle 159 in {:?}", t1 - t0);

    let mut my_app = MyApp::default();
    my_app.change_grid(result.unwrap());

    let rt = Runtime::new().expect("failed to make new Tokio Runtime");

    let _enter = rt.enter();

    // Execute the runtime in its own thread.
    // The future doesn't have to do anything. In this example, it just sleeps forever.
    std::thread::spawn(move || {
        rt.block_on(async {
            loop {
                tokio::time::sleep(Duration::from_secs(3600)).await;
            }
        })
    });

    // Run the GUI in the main thread.
    eframe::run_native(
        "Laser Mazer",
        eframe::NativeOptions {
            initial_window_size: Some(eframe::egui::vec2(1600., 900.)),
            ..Default::default()
        },
        Box::new(|_cc| Box::new(my_app)),
        // Box::new(|_cc| Box::<app::MyApp>::default()),
    )
    .expect("Failed to launch app");
}
