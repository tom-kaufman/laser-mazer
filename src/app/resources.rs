use egui_extras::RetainedImage;

pub struct ImageBank {
    pub cell_empty: RetainedImage,
    pub cell_empty_hovered: RetainedImage,
    pub token_laser: RetainedImage,
    pub token_laser_unoriented: RetainedImage,
}

impl Default for ImageBank {
    fn default() -> Self {
        Self {
            cell_empty: Self::cell_empty(),
            cell_empty_hovered: Self::cell_empty_hovered(),
            token_laser: Self::token_laser(),
            token_laser_unoriented: Self::token_laser_unoriented(),
        }
    }
}

impl ImageBank {
    fn cell_empty() -> RetainedImage {
        RetainedImage::from_image_bytes("cell_empty.png", include_bytes!(r#"..\..\assets\cell_empty.png"#)).expect("failed to load cell_empty.png")
    }
    
    fn cell_empty_hovered() -> RetainedImage {
        RetainedImage::from_image_bytes("cell_empty_hovered.png", include_bytes!(r#"..\..\assets\cell_empty_hovered.png"#)).expect("failed to load cell_empty_hovered.png")
    }
    
    fn token_laser() -> RetainedImage {
        RetainedImage::from_image_bytes("token_laser.png", include_bytes!(r#"..\..\assets\token_laser.png"#)).expect("failed to load token_laser.png")
    }
    
    fn token_laser_unoriented() -> RetainedImage {
        RetainedImage::from_image_bytes("token_laser_unoriented.png", include_bytes!(r#"..\..\assets\token_laser_unoriented.png"#)).expect("failed to load token_laser_unoriented.png")
    }
}