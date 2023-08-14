use crate::solver::orientation::Orientation;
use crate::solver::token::Token;
use crate::solver::token::TokenType;
use crate::solver::LaserMazeSolver;

use eframe::egui;
use eframe::App;

mod widgets;
use eframe::egui::Key;
use eframe::egui::Slider;
use widgets::cell::collections::Bank;
use widgets::cell::collections::Grid;
use widgets::cell::collections::ToBeAdded;

mod resources;

pub struct MyApp {
    cell_size: f32,

    tokens_grid: [Option<Token>; 25],
    tokens_to_be_added: [Option<Token>; 6],
    tokens_bank: [Option<Token>; 11],
    targets: u8,

    images: resources::ImageBank,

    token_move_indices: Option<(usize, usize)>,

    message_text: String,
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
            targets: 1,
            images: resources::ImageBank::default(),
            token_move_indices: Default::default(),
            message_text: Default::default(),
        }
    }
}

impl App for MyApp {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        // responses don't have a default value, and the closure is in its own scope,
        // so we make an Option<[Response; N]> and unwrap it later
        let mut bank_responses = None;
        let mut grid_responses = None;
        let mut to_be_added_responses = None;
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.heading("Bank");
                    bank_responses =
                        Some(Bank::new(self.cell_size).show(ui, &self.images, &self.tokens_bank));
                });
                ui.vertical(|ui| {
                    ui.heading("To Be Added");
                    to_be_added_responses = Some(ToBeAdded::new(self.cell_size * 0.82).show(
                        ui,
                        &self.images,
                        &self.tokens_to_be_added,
                    ));
                    ui.heading("Grid");
                    grid_responses =
                        Some(Grid::new(self.cell_size).show(ui, &self.images, &self.tokens_grid));
                });
            });
            ui.horizontal(|ui| {
                ui.label("Number of Targets:");
                ui.add(Slider::new(&mut self.targets, 1..=3));
            });
            if ui.button("Check").clicked() {
                if self.check() {
                    self.message_text = "This laser maze is solved!".into()
                } else {
                    self.message_text = "This laser maze is not solved.".into()
                }
            }
            if ui.button("Solve").clicked() {
                if self.solve() {
                    self.message_text = "Here's the solution!".into()
                } else {
                    self.message_text = "This laser maze is not solvable!".into()
                }
            }
            ui.label(format!("Message: {}", self.message_text));
        });

        self.handle_moving_tokens(
            ctx,
            grid_responses.as_ref().unwrap(),
            bank_responses.as_ref().unwrap(),
            to_be_added_responses.as_ref().unwrap(),
        );
        self.handle_orientation_shortcuts(
            ctx,
            grid_responses.as_ref().unwrap(),
            bank_responses.as_ref().unwrap(),
            to_be_added_responses.as_ref().unwrap(),
        );
    }
}

impl MyApp {
    // handles the Response arrays from Bank, Grid, and ToBeAdded cell collections;
    // figures out if we are trying to click and drag to move a Token between cells
    fn handle_moving_tokens(
        &mut self,
        ctx: &eframe::egui::Context,
        grid_responses: &[eframe::egui::Response; 25],
        bank_responses: &[eframe::egui::Response; 11],
        to_be_added_responses: &[eframe::egui::Response; 6],
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
            .find(|(_idx, response)| response.dragged());
        // get the response that is hovered
        // cells with None Token may have hover Sense, but not Dragged Sense; this
        // prevents use from short circuiting find() from the Cell we are dragging
        let hovered_index_response = grid_responses
            .iter()
            .chain(bank_responses.iter())
            .chain(to_be_added_responses.iter())
            .enumerate()
            .find(|(_idx, response)| response.hovered() && !response.dragged());

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

    fn handle_orientation_shortcuts(
        &mut self,
        ctx: &eframe::egui::Context,
        grid_responses: &[eframe::egui::Response; 25],
        bank_responses: &[eframe::egui::Response; 11],
        to_be_added_responses: &[eframe::egui::Response; 6],
    ) {
        // get the response that is hovered
        // cells with None Token may have hover Sense, but not Dragged Sense; this
        // prevents use from short circuiting find() from the Cell we are dragging
        if let Some((hovered_index, _)) = grid_responses
            .iter()
            .chain(bank_responses.iter())
            .chain(to_be_added_responses.iter())
            .enumerate()
            .find(|(_idx, response)| response.hovered())
        {
            if let Some(token) = match hovered_index {
                0..=24 => self.tokens_grid[hovered_index].as_mut(),
                25..=35 => self.tokens_bank[hovered_index - 25].as_mut(),
                36..=41 => self.tokens_to_be_added[hovered_index - 36].as_mut(),
                _ => {
                    panic!("impossible case because of fixed array lengths")
                }
            } {
                if ctx.input(|i| i.key_pressed(Key::W)) {
                    token.orientation = Some(Orientation::North);
                } else if ctx.input(|i| i.key_pressed(Key::D)) {
                    token.orientation = Some(Orientation::East);
                } else if ctx.input(|i| i.key_pressed(Key::S)) {
                    token.orientation = Some(Orientation::South);
                } else if ctx.input(|i| i.key_pressed(Key::A)) {
                    token.orientation = Some(Orientation::West);
                } else if ctx.input(|i| i.key_pressed(Key::R)) {
                    token.orientation = None;
                } else if ctx.input(|i| i.key_pressed(Key::M)) {
                    token.toggle_must_light();
                }
            }
        }
    }

    fn check(&self) -> bool {
        self.generate_solver()
            .stack
            .pop()
            .expect("LaserMazeSolver initializes with a node")
            .check()
            .solved()
    }

    fn run_solver(&self) -> Option<[Option<Token>; 25]> {
        self.generate_solver().solve()
    }

    #[allow(clippy::needless_range_loop)]
    fn solve(&mut self) -> bool {
        if let Some(solved_grid) = self.run_solver() {
            self.tokens_to_be_added = Default::default();
            for i in 0..25 {
                let transformed_index = Self::translate_model_index(i);
                self.tokens_grid[transformed_index] = solved_grid[i].clone()
            }
            true
        } else {
            false
        }
    }

    fn generate_solver(&self) -> LaserMazeSolver {
        let mut grid: [Option<Token>; 25] = Default::default();
        for i in 0..25 {
            let transformed_index = Self::translate_model_index(i);
            grid[transformed_index] = self.tokens_grid[i].clone();
        }

        let mut to_be_added = vec![];
        for token in self.tokens_to_be_added.iter().flatten() {
            to_be_added.push(token.clone());
        }

        LaserMazeSolver::new(grid, to_be_added, self.targets) // TODO add input for # target
    }

    // because of how egui adds items, the gui has cell 0 at top left, while the model
    // was built with cell 0 as bottom left.
    // luckily this operation is symmetric so we don't need a similar match statement
    pub fn translate_model_index(index: usize) -> usize {
        match index {
            0..=4 => index + 20,
            5..=9 => index + 10,
            10..=14 => index,
            15..=19 => index - 10,
            20..=24 => index - 20,
            _ => {
                panic!("index out of grid range")
            }
        }
    }

    pub fn change_grid(&mut self, new_grid: [Option<Token>; 25]) {
        // accepts the coordinates used by the Solver, not visual coords
        for i in 0..25 {
            self.tokens_grid[i] = new_grid[Self::translate_model_index(i)].clone();
        }
    }
}
