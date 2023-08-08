use std::sync::mpsc;

use crate::solver::orientation::Orientation;
use crate::solver::token::Token;
use crate::solver::token::TokenType;

use eframe::egui;
use eframe::App;

mod widgets;
use widgets::cell::Cell;
use widgets::grid::Grid;

use self::widgets::bank::Bank;

mod resources;

pub struct MyApp {
    tokens_grid: [Option<Token>; 25],
    tokens_to_be_added: [Option<Token>; 6],
    tokens_bank: [Option<Token>; 11],

    images: resources::ImageBank,

    token_move_indices: Option<(usize, usize)>,
}

impl Default for MyApp {
    fn default() -> Self {
        let tokens_bank = [
            Some(Token::new(TokenType::Laser, None, false)),
            Some(Token::new(TokenType::TargetMirror, None, false)),
            Some(Token::new(TokenType::TargetMirror, None, false)),
            Some(Token::new(TokenType::TargetMirror, None, false)),
            Some(Token::new(TokenType::TargetMirror, None, false)),
            Some(Token::new(TokenType::TargetMirror, None, false)),
            Some(Token::new(TokenType::BeamSplitter, None, false)),
            Some(Token::new(TokenType::BeamSplitter, None, false)),
            Some(Token::new(TokenType::DoubleMirror, None, false)),
            Some(Token::new(TokenType::Checkpoint, None, false)),
            Some(Token::new(TokenType::CellBlocker, None, false)),
        ];

        Self {
            tokens_grid: Default::default(),
            tokens_bank,
            tokens_to_be_added: Default::default(),
            images: resources::ImageBank::default(),
            token_move_indices: Default::default(),
        }
    }
}

impl App for MyApp {
    fn update(&mut self, ctx: &eframe::egui::Context, frame: &mut eframe::Frame) {
        // responses don't have a default value, and the closure is in its own scope,
        // so we make an Option<[Response; N]> and unwrap it later
        let mut bank_responses = None;
        let mut grid_responses = None;
        egui::CentralPanel::default().show(&ctx, |ui| {
            ui.horizontal(|ui| {
                ui.vertical_centered(|ui| {
                    bank_responses =
                        Some(Bank::new(100.).show(ui, &self.images, &self.tokens_bank));
                });
                ui.vertical_centered(|ui| {
                    grid_responses =
                        Some(Grid::new(100.0).show(ui, &self.images, &self.tokens_grid));
                });
            });
        });

        self.handle_cell_responses(ctx, grid_responses.unwrap(), bank_responses.unwrap());
    }
}

impl MyApp {
    fn handle_cell_responses(
        &mut self,
        ctx: &eframe::egui::Context,
        grid_responses: [eframe::egui::Response; 25],
        bank_responses: [eframe::egui::Response; 11],
    ) {
        let last_frame_token_move_indices = self.token_move_indices;

        let dragged_index_response = grid_responses
            .iter()
            .chain(bank_responses.iter())
            .enumerate()
            .find(|(idx, response)| response.dragged());
        let hovered_index_response = grid_responses
            .iter()
            .chain(bank_responses.iter())
            .enumerate()
            .find(|(idx, response)| response.hovered() && !response.dragged());

        if let (Some((dragged_index, _)), Some((hovered_index, _))) =
            (dragged_index_response, hovered_index_response)
        {
            self.token_move_indices = Some((dragged_index, hovered_index));
        } else {
            self.token_move_indices = None;
        }

        if let Some((dragged_index, hovered_index)) = last_frame_token_move_indices {
            if ctx.input(|i| i.pointer.primary_released()) {
                let moving_token = match dragged_index {
                    0..=24 => {
                        let moving_token = self.tokens_grid[dragged_index]
                            .as_ref()
                            .expect("We can only drag cells which have a token")
                            .clone();
                        self.tokens_grid[dragged_index] = None;
                        moving_token
                    }
                    25..=35 => {
                        let moving_token = self.tokens_bank[dragged_index - 25]
                            .as_ref()
                            .expect("We can only drag cells which have a token")
                            .clone();
                        self.tokens_bank[dragged_index - 25] = None;
                        moving_token
                    }
                    _ => {
                        panic!("impossible case because of fixed array lengths")
                    }
                };
                match hovered_index {
                    0..=24 => {
                        self.tokens_grid[hovered_index] = Some(moving_token);
                    }
                    25..=35 => {
                        self.tokens_bank[hovered_index - 25] = Some(moving_token);
                    }
                    _ => {
                        panic!("impossible case because of fixed array lengths")
                    }
                }
            }
        }
    }
}
