use serde::{Deserialize, Serialize};
use strum::EnumIter;

#[derive(Serialize, Deserialize, Hash, Eq, PartialEq, Clone, Copy, Debug, Default, EnumIter)]
pub enum AvatarStyle {
    Blobs,
    Glass,
    #[default]
    Identicon,
    Pixelbot,
    Shapes,
    Squircles,
    Waves,
}

#[cfg(test)]
mod avatar_test;
