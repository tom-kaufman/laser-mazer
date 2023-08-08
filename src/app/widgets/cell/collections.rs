use crate::{
    app::{resources::ImageBank, widgets::cell::Cell},
    solver::token::Token,
};

pub struct Bank {
    cell_size: f32,
}

impl Bank {
    pub fn new(cell_size: f32) -> Self {
        Self { cell_size }
    }

    pub fn show(
        self,
        ui: &mut eframe::egui::Ui,
        images: &ImageBank,
        tokens: &[Option<Token>; 11],
    ) -> [eframe::egui::Response; 11] {
        let mut responses: Vec<eframe::egui::Response> = Vec::with_capacity(11);
        ui.horizontal(|ui| {
            for range in [0..3, 3..6, 6..9, 9..11] {
                ui.vertical(|ui| {
                    for i in range {
                        responses.push(Cell::new(self.cell_size).show(ui, images, &tokens[i]))
                    }
                });
            }
        });

        responses
            .try_into()
            .expect("We should have made exactly 11 responses")
    }
}

pub struct Grid {
    cell_size: f32,
}

impl Grid {
    pub fn new(cell_size: f32) -> Self {
        Self { cell_size }
    }

    pub fn show(
        self,
        ui: &mut eframe::egui::Ui,
        images: &ImageBank,
        tokens: &[Option<Token>; 25],
    ) -> [eframe::egui::Response; 25] {
        let mut responses: Vec<eframe::egui::Response> = Vec::with_capacity(11);
        ui.vertical(|ui| {
            for range in [0..5, 5..10, 10..15, 15..20, 20..25] {
                ui.horizontal(|ui| {
                    for i in range {
                        responses.push(Cell::new(self.cell_size).show(ui, images, &tokens[i]))
                    }
                });
            }
        });

        responses
            .try_into()
            .expect("We should have made exactly 25 responses")
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
}

pub struct ToBeAdded {
    cell_size: f32,
}

impl ToBeAdded {
    pub fn new(cell_size: f32) -> Self {
        Self { cell_size }
    }

    pub fn show(
        self,
        ui: &mut eframe::egui::Ui,
        images: &ImageBank,
        tokens: &[Option<Token>; 6],
    ) -> [eframe::egui::Response; 6] {
        let mut responses: Vec<eframe::egui::Response> = Vec::with_capacity(6);
        ui.horizontal(|ui| {
            for i in 0..6 {
                responses.push(Cell::new(self.cell_size).show(ui, images, &tokens[i]))
            }
        });

        responses
            .try_into()
            .expect("We should have made exactly 6 responses")
    }
}
