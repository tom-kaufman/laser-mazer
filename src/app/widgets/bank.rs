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
