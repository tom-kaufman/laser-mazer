use egui_extras::RetainedImage;

pub struct ImageBank {
    pub cell_empty: RetainedImage,
    pub cell_empty_hovered: RetainedImage,
    pub token_laser: RetainedImage,
    pub token_laser_unoriented: RetainedImage,
    pub token_target_mirror: RetainedImage,
    pub token_target_mirror_unoriented: RetainedImage,
    pub token_target_mirror_must_light: RetainedImage,
    pub token_target_mirror_must_light_unoriented: RetainedImage,
    pub token_beam_splitter: RetainedImage,
    pub token_beam_splitter_unoriented: RetainedImage,
    pub token_double_mirror: RetainedImage,
    pub token_double_mirror_unoriented: RetainedImage,
    pub token_checkpoint: RetainedImage,
    pub token_checkpoint_unoriented: RetainedImage,
    pub token_cell_blocker: RetainedImage,
}

impl Default for ImageBank {
    fn default() -> Self {
        Self {
            cell_empty: Self::cell_empty(),
            cell_empty_hovered: Self::cell_empty_hovered(),
            token_laser: Self::token_laser(),
            token_laser_unoriented: Self::token_laser_unoriented(),
            token_target_mirror: Self::token_target_mirror(),
            token_target_mirror_unoriented: Self::token_target_mirror_unoriented(),
            token_target_mirror_must_light: Self::token_target_mirror_must_light(),
            token_target_mirror_must_light_unoriented:
                Self::token_target_mirror_must_light_unoriented(),
            token_beam_splitter: Self::token_beam_splitter(),
            token_beam_splitter_unoriented: Self::token_beam_splitter_unoriented(),
            token_double_mirror: Self::token_double_mirror(),
            token_double_mirror_unoriented: Self::token_double_mirror_unoriented(),
            token_checkpoint: Self::token_checkpoint(),
            token_checkpoint_unoriented: Self::token_checkpoint_unoriented(),
            token_cell_blocker: Self::token_cell_blocker(),
        }
    }
}

impl ImageBank {
    fn cell_empty() -> RetainedImage {
        RetainedImage::from_image_bytes(
            "cell_empty.png",
            include_bytes!(r#"..\..\assets\cell_empty.png"#),
        )
        .expect("failed to load cell_empty.png")
    }

    fn cell_empty_hovered() -> RetainedImage {
        RetainedImage::from_image_bytes(
            "cell_empty_hovered.png",
            include_bytes!(r#"..\..\assets\cell_empty_hovered.png"#),
        )
        .expect("failed to load cell_empty_hovered.png")
    }

    fn token_laser() -> RetainedImage {
        RetainedImage::from_image_bytes(
            "token_laser.png",
            include_bytes!(r#"..\..\assets\token_laser.png"#),
        )
        .expect("failed to load token_laser.png")
    }

    fn token_laser_unoriented() -> RetainedImage {
        RetainedImage::from_image_bytes(
            "token_laser_unoriented.png",
            include_bytes!(r#"..\..\assets\token_laser_unoriented.png"#),
        )
        .expect("failed to load token_laser_unoriented.png")
    }

    fn token_target_mirror() -> RetainedImage {
        RetainedImage::from_image_bytes(
            "token_target_mirror.png",
            include_bytes!(r#"..\..\assets\token_target_mirror.png"#),
        )
        .expect("failed to load token_target_mirror.png")
    }
    fn token_target_mirror_unoriented() -> RetainedImage {
        RetainedImage::from_image_bytes(
            "token_target_mirror_unoriented.png",
            include_bytes!(r#"..\..\assets\token_target_mirror_unoriented.png"#),
        )
        .expect("failed to load token_target_mirror_unoriented.png")
    }
    fn token_target_mirror_must_light() -> RetainedImage {
        RetainedImage::from_image_bytes(
            "token_target_mirror_must_light.png",
            include_bytes!(r#"..\..\assets\token_target_mirror_must_light.png"#),
        )
        .expect("failed to load token_target_mirror_must_light.png")
    }
    fn token_target_mirror_must_light_unoriented() -> RetainedImage {
        RetainedImage::from_image_bytes(
            "token_target_mirror_must_light_unoriented.png",
            include_bytes!(r#"..\..\assets\token_target_mirror_must_light_unoriented.png"#),
        )
        .expect("failed to load token_target_mirror_must_light_unoriented.png")
    }
    fn token_beam_splitter() -> RetainedImage {
        RetainedImage::from_image_bytes(
            "token_beam_splitter.png",
            include_bytes!(r#"..\..\assets\token_beam_splitter.png"#),
        )
        .expect("failed to load token_beam_splitter.png")
    }
    fn token_beam_splitter_unoriented() -> RetainedImage {
        RetainedImage::from_image_bytes(
            "token_beam_splitter_unoriented.png",
            include_bytes!(r#"..\..\assets\token_beam_splitter_unoriented.png"#),
        )
        .expect("failed to load token_beam_splitter_unoriented.png")
    }
    fn token_double_mirror() -> RetainedImage {
        RetainedImage::from_image_bytes(
            "token_double_mirror.png",
            include_bytes!(r#"..\..\assets\token_double_mirror.png"#),
        )
        .expect("failed to load token_double_mirror.png")
    }
    fn token_double_mirror_unoriented() -> RetainedImage {
        RetainedImage::from_image_bytes(
            "token_double_mirror_unoriented.png",
            include_bytes!(r#"..\..\assets\token_double_mirror_unoriented.png"#),
        )
        .expect("failed to load token_double_mirror_unoriented.png")
    }
    fn token_checkpoint() -> RetainedImage {
        RetainedImage::from_image_bytes(
            "token_checkpoint.png",
            include_bytes!(r#"..\..\assets\token_checkpoint.png"#),
        )
        .expect("failed to load token_checkpoint.png")
    }
    fn token_checkpoint_unoriented() -> RetainedImage {
        RetainedImage::from_image_bytes(
            "token_checkpoint_unoriented.png",
            include_bytes!(r#"..\..\assets\token_checkpoint_unoriented.png"#),
        )
        .expect("failed to load token_checkpoint_unoriented.png")
    }
    fn token_cell_blocker() -> RetainedImage {
        RetainedImage::from_image_bytes(
            "token_cell_blocker.png",
            include_bytes!(r#"..\..\assets\token_cell_blocker.png"#),
        )
        .expect("failed to load token_cell_blocker.png")
    }
}
