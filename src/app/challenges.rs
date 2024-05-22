use crate::app::Tokens;
use lazy_static::lazy_static;
use std::fmt;

#[derive(Clone, Copy, PartialEq)]
pub enum Challenges {
    BonusChallenge1,
    BonusChallenge2,
    BonusChallenge3,
    BonusChallenge26,
    // ... add more variants
}

lazy_static! {
    static ref CHALLENGE_ORDER: [Challenges; 4] = [
        // Challenges::Challenge1,  // TODO add included/base challenges
        Challenges::BonusChallenge1,
        Challenges::BonusChallenge2,
        Challenges::BonusChallenge3,
        Challenges::BonusChallenge26,
    ];
}

impl Default for Challenges {
    fn default() -> Self {
        Self::BonusChallenge1
    }
}

impl Challenges {
    pub fn tokens(&self) -> Tokens {
        let text = match self {
            Challenges::BonusChallenge1 => {
                r#"{"targets":3,"grid":[null,{"type_":"TargetMirror","orientation":null,"lit":false,"target_lit":false,"must_light":false},null,null,{"type_":"TargetMirror","orientation":null,"lit":false,"target_lit":false,"must_light":false},null,null,null,null,null,null,null,null,null,null,{"type_":"Laser","orientation":null,"lit":true,"target_lit":null,"must_light":false},{"type_":"CellBlocker","orientation":"North","lit":true,"target_lit":null,"must_light":false},{"type_":"BeamSplitter","orientation":"North","lit":false,"target_lit":null,"must_light":false},{"type_":"Checkpoint","orientation":null,"lit":false,"target_lit":null,"must_light":false},null,null,null,null,null,{"type_":"TargetMirror","orientation":"East","lit":false,"target_lit":false,"must_light":false}],"to_be_added":[{"type_":"TargetMirror","orientation":null,"lit":false,"target_lit":false,"must_light":false},{"type_":"TargetMirror","orientation":null,"lit":false,"target_lit":false,"must_light":false},{"type_":"DoubleMirror","orientation":null,"lit":false,"target_lit":null,"must_light":false},{"type_":"BeamSplitter","orientation":null,"lit":false,"target_lit":null,"must_light":false},null,null],"bank":[null,null,null,null,null,null,null,null,null,null,null]}"#
            }
            Challenges::BonusChallenge2 => {
                r#"{"targets":2,"grid":[null,null,{"type_":"TargetMirror","orientation":null,"lit":false,"target_lit":false,"must_light":false},null,null,null,null,{"type_":"Checkpoint","orientation":"East","lit":false,"target_lit":null,"must_light":false},null,null,null,{"type_":"Laser","orientation":null,"lit":true,"target_lit":null,"must_light":false},null,null,{"type_":"DoubleMirror","orientation":null,"lit":false,"target_lit":null,"must_light":false},null,null,null,null,null,{"type_":"TargetMirror","orientation":null,"lit":false,"target_lit":false,"must_light":false},null,null,null,null],"to_be_added":[{"type_":"TargetMirror","orientation":null,"lit":false,"target_lit":false,"must_light":false},{"type_":"TargetMirror","orientation":null,"lit":false,"target_lit":false,"must_light":false},{"type_":"TargetMirror","orientation":null,"lit":false,"target_lit":false,"must_light":false},{"type_":"BeamSplitter","orientation":null,"lit":false,"target_lit":null,"must_light":false},{"type_":"BeamSplitter","orientation":null,"lit":false,"target_lit":null,"must_light":false},null],"bank":[{"type_":"CellBlocker","orientation":"North","lit":true,"target_lit":null,"must_light":false},null,null,null,null,null,null,null,null,null,null]}"#
            }
            Challenges::BonusChallenge3 => {
                r#"{"targets":3,"grid":[{"type_":"Laser","orientation":null,"lit":true,"target_lit":null,"must_light":false},null,null,null,null,null,{"type_":"TargetMirror","orientation":"South","lit":false,"target_lit":false,"must_light":false},null,null,null,null,null,{"type_":"Checkpoint","orientation":null,"lit":false,"target_lit":null,"must_light":false},{"type_":"DoubleMirror","orientation":null,"lit":false,"target_lit":null,"must_light":false},{"type_":"TargetMirror","orientation":"East","lit":false,"target_lit":false,"must_light":false},null,null,null,{"type_":"CellBlocker","orientation":"North","lit":true,"target_lit":null,"must_light":false},null,null,null,null,null,null],"to_be_added":[{"type_":"TargetMirror","orientation":null,"lit":false,"target_lit":false,"must_light":false},{"type_":"TargetMirror","orientation":null,"lit":false,"target_lit":false,"must_light":false},{"type_":"TargetMirror","orientation":null,"lit":false,"target_lit":false,"must_light":false},{"type_":"BeamSplitter","orientation":null,"lit":false,"target_lit":null,"must_light":false},{"type_":"BeamSplitter","orientation":null,"lit":false,"target_lit":null,"must_light":false},null],"bank":[null,null,null,null,null,null,null,null,null,null,null]}"#
            }
            Challenges::BonusChallenge26 => {
                r#"{"grid":[null,null,null,null,null,null,null,null,null,{"type_":"TargetMirror","orientation":null,"lit":false,"target_lit":false,"must_light":false},null,{"type_":"Checkpoint","orientation":null,"lit":false,"target_lit":null,"must_light":false},null,{"type_":"DoubleMirror","orientation":null,"lit":false,"target_lit":null,"must_light":false},null,null,null,null,null,null,null,{"type_":"CellBlocker","orientation":"North","lit":true,"target_lit":null,"must_light":false},{"type_":"BeamSplitter","orientation":"East","lit":false,"target_lit":null,"must_light":false},null,null],"to_be_added":[{"type_":"Laser","orientation":null,"lit":true,"target_lit":null,"must_light":false},{"type_":"TargetMirror","orientation":null,"lit":false,"target_lit":false,"must_light":false},{"type_":"TargetMirror","orientation":null,"lit":false,"target_lit":false,"must_light":false},{"type_":"TargetMirror","orientation":null,"lit":false,"target_lit":false,"must_light":false},{"type_":"BeamSplitter","orientation":null,"lit":false,"target_lit":null,"must_light":false},null],"bank":[{"type_":"TargetMirror","orientation":"South","lit":false,"target_lit":false,"must_light":false},null,null,null,null,null,null,null,null,null,null],"targets":2}"#
            }
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
                                if i > 0 {
                                    if c.is_uppercase() {
                                        vec![' ', c]
                                    } else if c.is_numeric() {
                                        let c2 = variant_name.chars().nth(i-1).expect("i>0");
                                        if c2.is_numeric() {
                                            vec![c]
                                        } else {
                                            vec![' ', c]
                                        }
                                    } else {
                                        vec![c]
                                    }
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

impl_display_for_challenges!(
    BonusChallenge1,
    BonusChallenge2,
    BonusChallenge3,
    BonusChallenge26
);
