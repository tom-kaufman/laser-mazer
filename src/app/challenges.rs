use std::fmt;
use lazy_static::lazy_static;
use crate::app::Tokens;

#[derive(Clone, Copy, PartialEq)]
pub enum Challenges {
    Challenge1,
    Challenge2,
    BonusChallenge1,
    BonusChallenge2,
    // ... add more variants
}

lazy_static! {
    static ref CHALLENGE_ORDER: [Challenges; 4] = [
        Challenges::Challenge1,
        Challenges::Challenge2,
        Challenges::BonusChallenge1,
        Challenges::BonusChallenge2,
    ];
}

impl Default for Challenges {
    fn default() -> Self {
        Self::Challenge1
    }
}

impl Challenges {
    pub fn tokens(&self) -> Tokens {
        let text = match self {
            Challenges::Challenge1 => r#"{"grid":[{"type_":"TargetMirror","orientation":null,"lit":false,"target_lit":false,"must_light":false},{"type_":"BeamSplitter","orientation":null,"lit":false,"target_lit":null,"must_light":false},null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null],"to_be_added":[{"type_":"Laser","orientation":null,"lit":true,"target_lit":null,"must_light":false},null,null,null,null,null],"bank":[null,{"type_":"TargetMirror","orientation":null,"lit":false,"target_lit":false,"must_light":false},{"type_":"TargetMirror","orientation":null,"lit":false,"target_lit":false,"must_light":false},{"type_":"TargetMirror","orientation":null,"lit":false,"target_lit":false,"must_light":false},null,{"type_":"TargetMirror","orientation":null,"lit":false,"target_lit":false,"must_light":false},{"type_":"BeamSplitter","orientation":null,"lit":false,"target_lit":null,"must_light":false},null,{"type_":"DoubleMirror","orientation":null,"lit":false,"target_lit":null,"must_light":false},{"type_":"Checkpoint","orientation":null,"lit":false,"target_lit":null,"must_light":false},{"type_":"CellBlocker","orientation":"North","lit":true,"target_lit":null,"must_light":false}]}"#,
            Challenges::Challenge2 => r#"{"grid":[{"type_":"CellBlocker","orientation":"North","lit":true,"target_lit":null,"must_light":false},null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null],"to_be_added":[null,null,null,null,null,null],"bank":[{"type_":"Laser","orientation":null,"lit":true,"target_lit":null,"must_light":false},{"type_":"TargetMirror","orientation":null,"lit":false,"target_lit":false,"must_light":false},{"type_":"TargetMirror","orientation":null,"lit":false,"target_lit":false,"must_light":false},{"type_":"TargetMirror","orientation":null,"lit":false,"target_lit":false,"must_light":false},{"type_":"TargetMirror","orientation":null,"lit":false,"target_lit":false,"must_light":false},{"type_":"TargetMirror","orientation":null,"lit":false,"target_lit":false,"must_light":false},{"type_":"BeamSplitter","orientation":null,"lit":false,"target_lit":null,"must_light":false},{"type_":"BeamSplitter","orientation":null,"lit":false,"target_lit":null,"must_light":false},{"type_":"DoubleMirror","orientation":null,"lit":false,"target_lit":null,"must_light":false},{"type_":"Checkpoint","orientation":null,"lit":false,"target_lit":null,"must_light":false},null]}"#,
            Challenges::BonusChallenge1 => r#"{"grid":[{"type_":"CellBlocker","orientation":"North","lit":true,"target_lit":null,"must_light":false},null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,{"type_":"TargetMirror","orientation":null,"lit":false,"target_lit":false,"must_light":false},null,null,null,null,null,null,null],"to_be_added":[null,null,null,null,null,null],"bank":[{"type_":"Laser","orientation":null,"lit":true,"target_lit":null,"must_light":false},{"type_":"TargetMirror","orientation":null,"lit":false,"target_lit":false,"must_light":false},{"type_":"TargetMirror","orientation":null,"lit":false,"target_lit":false,"must_light":false},{"type_":"TargetMirror","orientation":null,"lit":false,"target_lit":false,"must_light":false},null,{"type_":"TargetMirror","orientation":null,"lit":false,"target_lit":false,"must_light":false},{"type_":"BeamSplitter","orientation":null,"lit":false,"target_lit":null,"must_light":false},{"type_":"BeamSplitter","orientation":null,"lit":false,"target_lit":null,"must_light":false},{"type_":"DoubleMirror","orientation":null,"lit":false,"target_lit":null,"must_light":false},{"type_":"Checkpoint","orientation":null,"lit":false,"target_lit":null,"must_light":false},null]}"#,
            Challenges::BonusChallenge2 => r#"{"grid":[{"type_":"CellBlocker","orientation":"North","lit":true,"target_lit":null,"must_light":false},null,null,null,null,null,null,null,{"type_":"DoubleMirror","orientation":null,"lit":false,"target_lit":null,"must_light":false},null,null,null,null,null,null,null,null,{"type_":"TargetMirror","orientation":null,"lit":false,"target_lit":false,"must_light":false},null,null,null,null,null,null,null],"to_be_added":[null,null,null,null,null,null],"bank":[{"type_":"Laser","orientation":null,"lit":true,"target_lit":null,"must_light":false},{"type_":"TargetMirror","orientation":null,"lit":false,"target_lit":false,"must_light":false},{"type_":"TargetMirror","orientation":null,"lit":false,"target_lit":false,"must_light":false},{"type_":"TargetMirror","orientation":null,"lit":false,"target_lit":false,"must_light":false},null,{"type_":"TargetMirror","orientation":null,"lit":false,"target_lit":false,"must_light":false},{"type_":"BeamSplitter","orientation":null,"lit":false,"target_lit":null,"must_light":false},{"type_":"BeamSplitter","orientation":null,"lit":false,"target_lit":null,"must_light":false},null,{"type_":"Checkpoint","orientation":null,"lit":false,"target_lit":null,"must_light":false},null]}"#,
        };
        serde_json::from_str(text).unwrap()
    }

    pub fn iter() -> std::slice::Iter<'static, Challenges> {
        CHALLENGE_ORDER.iter()
    }
}

macro_rules! impl_display_for_challenges {
    ($($variant:ident),*) => {
        impl fmt::Display for Challenges {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                match self {
                    $(Challenges::$variant => {
                        let variant_name = stringify!($variant);
                        let formatted_name = variant_name
                            .chars()
                            .enumerate()
                            .flat_map(|(i, c)| {
                                if i > 0 && (c.is_uppercase() || c.is_numeric()) {
                                    vec![' ', c]
                                } else {
                                    vec![c]
                                }
                            })
                            .collect::<String>();
                        write!(f, "{}", formatted_name)
                    },)*
                }
            }
        }
    };
}

impl_display_for_challenges!(Challenge1, Challenge2, BonusChallenge1, BonusChallenge2);