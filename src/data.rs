use serde::{Deserialize, Serialize};
use std::clone::Clone;
use std::hash::Hash;

#[derive(Serialize, Deserialize, Clone, Hash, Eq, PartialEq)]
pub struct UserData {
    pub user_id: String,
    pub name: String,
    pub gravatar_id: String,
}

impl UserData {
    pub fn new(user_id: String) -> UserData {
        let name = "Shirtless Muppet".to_string();
        let gravatar_id = format!("{:x}", md5::compute(user_id.clone()));
        UserData {
            user_id,
            name,
            gravatar_id,
        }
    }

    pub fn set_avatar(&mut self, avatar: &str) {
        self.gravatar_id = format!("{:x}", md5::compute(avatar));
    }
}
