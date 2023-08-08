use std::sync::mpsc;

use crate::solver::token::Token;
use crate::solver::token::TokenType;
use crate::solver::orientation::Orientation;

use eframe::App;
use eframe::egui;

mod widgets;
use widgets::cell::Cell;
use widgets::grid::Grid;

mod resources;

pub struct MyApp {
    solver_tx: mpsc::Sender<Option<[Option<Token>; 25]>>,
    solver_rx: mpsc::Receiver<Option<[Option<Token>; 25]>>,

    images: resources::ImageBank,

    tokens: Vec<Token>,
}

impl Default for MyApp {
    fn default() -> Self {
        let (solver_tx, solver_rx) = mpsc::channel();

        let tokens = vec![
            Token::new(TokenType::Laser, None, false),
            Token::new(TokenType::TargetMirror, None, false),
        ];
        
        Self {
            solver_tx,
            solver_rx,
            images: resources::ImageBank::default(),
            tokens,
        }
    }
}

impl App for MyApp {
    fn update(&mut self, ctx: &eframe::egui::Context, frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(&ctx, |ui| {
            ui.label("Hello");

            let laser = Some(Token::new(TokenType::Laser, Some(Orientation::East), false));
            Cell::new(100.).show(ui, &self.images, &laser);
            
            let laser_south = Some(Token::new(TokenType::Laser, Some(Orientation::South), false));
            Cell::new(100.).show(ui, &self.images, &laser_south);
            
            let laser_west = Some(Token::new(TokenType::Laser, None, false));
            Cell::new(100.).show(ui, &self.images, &laser_west);
            
            let laser_north = Some(Token::new(TokenType::Laser, Some(Orientation::North), false));
            Cell::new(100.).show(ui, &self.images, &laser_north);
        });
    }
}
