use crate::solver::orientation::Orientation;
use crate::solver::token::Token;
use crate::solver::token::TokenType;

use eframe::egui;
use eframe::App;

mod widgets;
use widgets::cell::collections::Bank;
use widgets::cell::collections::Grid;
use widgets::cell::collections::ToBeAdded;

mod resources;

pub struct MyApp {
    cell_size: f32,

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
            cell_size: 100.,
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
        let mut to_be_added_responses = None;
        egui::CentralPanel::default().show(&ctx, |ui| {
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.vertical_centered(|ui| {
                        bank_responses = Some(Bank::new(self.cell_size).show(
                            ui,
                            &self.images,
                            &self.tokens_bank,
                        ));
                    });
                    ui.vertical_centered(|ui| {
                        grid_responses = Some(Grid::new(self.cell_size).show(
                            ui,
                            &self.images,
                            &self.tokens_grid,
                        ));
                    });
                });
                to_be_added_responses = Some(ToBeAdded::new(self.cell_size).show(
                    ui,
                    &self.images,
                    &self.tokens_to_be_added,
                ));
            });
        });

        self.handle_cell_responses(
            ctx,
            grid_responses.unwrap(),
            bank_responses.unwrap(),
            to_be_added_responses.unwrap(),
        );
    }
}

impl MyApp {
    // handles the Response arrays from Bank, Grid, and ToBeAdded cell collections;
    // figures out if we are trying to click and drag to move a Token between cells
    fn handle_cell_responses(
        &mut self,
        ctx: &eframe::egui::Context,
        grid_responses: [eframe::egui::Response; 25],
        bank_responses: [eframe::egui::Response; 11],
        to_be_added_responses: [eframe::egui::Response; 6],
    ) {
        // store the indices of the moved tokens from last frame, before we overwrite them
        let last_frame_token_move_indices = self.token_move_indices;

        // we chan the iterators for the response arrays of the different token repositories, and then enumerate
        // the chained iterator. the order of the chaining will be important to keep in mind later.

        // get the response that was dragged
        // only cells with Some(Token) may have drag Sense
        let dragged_index_response = grid_responses
            .iter()
            .chain(bank_responses.iter())
            .chain(to_be_added_responses.iter())
            .enumerate()
            .find(|(idx, response)| response.dragged());
        // get the response that is hovered
        // cells with None Token may have hover Sense, but not Dragged Sense; this
        // prevents use from short circuiting find() from the Cell we are dragging
        let hovered_index_response = grid_responses
            .iter()
            .chain(bank_responses.iter())
            .chain(to_be_added_responses.iter())
            .enumerate()
            .find(|(idx, response)| response.hovered() && !response.dragged());

        // restructure the tuples returned from find. we only care about the values if we have both Some()
        if let (Some((dragged_index, _)), Some((hovered_index, _))) =
            (dragged_index_response, hovered_index_response)
        {
            self.token_move_indices = Some((dragged_index, hovered_index));
        } else {
            self.token_move_indices = None;
        }

        // if on the last frame we were dragging and hovering two cells, and on this frame we
        // released the primary mouse button, we need to move a token around
        // don't forget that the indices stored in self.token_move_indices are the indices of the
        // chained iterators above, enumerated after chaining
        if let Some((dragged_index, hovered_index)) = last_frame_token_move_indices {
            if ctx.input(|i| i.pointer.primary_released()) {
                // clone the token and set its original position to None
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
                    36..=41 => {
                        let moving_token = self.tokens_to_be_added[dragged_index - 36]
                            .as_ref()
                            .expect("We can only drag cells which have a token")
                            .clone();
                        self.tokens_to_be_added[dragged_index - 36] = None;
                        moving_token
                    }
                    _ => {
                        panic!("impossible case because of fixed array lengths")
                    }
                };
                // move the cloned token into its new place
                match hovered_index {
                    0..=24 => {
                        self.tokens_grid[hovered_index] = Some(moving_token);
                    }
                    25..=35 => {
                        self.tokens_bank[hovered_index - 25] = Some(moving_token);
                    }
                    36..=41 => {
                        self.tokens_to_be_added[hovered_index - 36] = Some(moving_token);
                    }
                    _ => {
                        panic!("impossible case because of fixed array lengths")
                    }
                }
            }
        }
    }
}
