use crate::app::{challenges::Challenges, Tokens};
use eframe::egui::{ComboBox, Context, Window};

#[derive(Default)]
pub struct LoadIncludedChallengesMenu {
    pub open: bool,
    selected_challenge: Challenges,
}

impl LoadIncludedChallengesMenu {
    pub fn show(&mut self, ctx: &Context, app_tokens: &mut Tokens) {
        Window::new("Load Included Challenges")
            .collapsible(true)
            .open(&mut self.open)
            .show(ctx, |ui| {
                ComboBox::from_id_source("challenge_selector")
                    .selected_text(format!("{}", &self.selected_challenge))
                    .show_ui(ui, |ui| {
                        for challenge in Challenges::iter() {
                            let value = ui.selectable_value(
                                &mut self.selected_challenge,
                                *challenge,
                                format!("{}", challenge),
                            );
                            if value.clicked() {
                                self.selected_challenge = *challenge;
                            }
                        }
                    });
                if ui.button("Load").clicked() {
                    *app_tokens = self.selected_challenge.tokens()
                }
            });
    }
}
