use eframe::{egui::{Image, Sense, vec2, Context}, epaint::{Rect, Color32, pos2, TextureId, Vec2}};
use egui_extras::RetainedImage;

use crate::{solver::token::{TokenType, Token}, app::resources::ImageBank};

pub struct Cell {
    size: f32,
}

impl Cell {
    pub fn new(size: f32) -> Self {
        Self {
            size,
        }
    }

    pub fn show(self, ui: &mut eframe::egui::Ui, images: &ImageBank, token: &Option<Token>) -> eframe::egui::Response {
        let rect_size = vec2(self.size, self.size);
        let (rect, response) = ui.allocate_at_least(rect_size, Sense::click_and_drag());

        if ui.is_rect_visible(rect) {
            let image = if response.hovered() {
                &images.cell_empty_hovered
            } else {
                &images.cell_empty
            };
            let painter = ui.painter();
            painter.image(image.texture_id(ui.ctx()), rect, Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)), Color32::WHITE);

            if let Some(token_image) = Self::get_token_image(ui.ctx(), token, images, rect.size()) {
                token_image.paint_at(ui, rect)
            }
        }

        response
    }

    fn get_token_image<'a>(ctx: &Context, token: &Option<Token>, images: &ImageBank, rect_size: Vec2) -> Option<Image> {
        let mut rotation_radians = 0.;
        let unrotated_image = match token {
            Some(token) => {
                match &token.orientation {
                    Some(orientation) => {
                        rotation_radians = (90.0 * (orientation.to_index() as f32)).to_radians();
                         match token.type_() {
                            TokenType::Laser => { &images.token_laser }
                            _ => { todo!() }
                        }
                        
                    }
                    None => {
                        match token.type_() {
                            TokenType::Laser => { &images.token_laser_unoriented }
                            _ => { todo!() }
                        }                        
                    }
                }
            }
            None => { return None }
        };
        Some(Image::new(unrotated_image.texture_id(ctx), rect_size).rotate(rotation_radians, vec2(0.5, 0.5)))
    }
}