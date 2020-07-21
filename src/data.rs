use serde::{Deserialize, Serialize};
use std::clone::Clone;
use std::collections::{HashMap, HashSet};
use std::hash::Hash;

#[derive(Serialize, Deserialize, Clone, Hash, Eq, PartialEq)]
pub struct UserData {
    pub user_id: String,
    pub name: String,
    pub gravatar_id: String,
}

impl UserData {
    fn new(user_id: String) -> UserData {
        let name = "Shirtless Muppet".to_string();
        let gravatar_id = format!("{:x}", md5::compute(user_id.clone()));
        UserData {
            user_id,
            name,
            gravatar_id,
        }
    }
}

pub struct UserDataDb {
    user_map: HashMap<String, UserData>,
}

impl Default for UserDataDb {
    fn default() -> Self {
        UserDataDb {
            user_map: Default::default(),
        }
    }
}

impl UserDataDb {
    pub fn add(&mut self, user_id: String) -> UserData {
        let user_data = UserData::new(user_id);
        let user_data_clone = user_data.clone();
        self.user_map.insert(user_data.user_id.clone(), user_data);
        return user_data_clone;
    }

    pub fn update_name(&mut self, user_id: &str, name: String) {
        if let Some(user_data) = self.user_map.get_mut(user_id) {
            user_data.name = name;
        }
    }

    pub fn update_gravatar(&mut self, user_id: &str, gravatar_id: &str) {
        if let Some(user_data) = self.user_map.get_mut(user_id) {
            user_data.gravatar_id = format!("{:x}", md5::compute(gravatar_id));
        }
    }
    pub fn remove(&mut self, user_id: &str) {
        self.user_map.remove(user_id);
    }

    pub fn get_user(&self, user_id: &str) -> Option<UserData> {
        let user = self.user_map.get(user_id).clone();
        match user {
            None => None,
            Some(user) => Some(user.clone()),
        }
    }

    pub fn get_users(&self, user_ids: HashSet<String>) -> HashSet<UserData> {
        user_ids
            .iter()
            .map(|user_id| self.user_map.get::<str>(user_id))
            .flat_map(|f| f)
            .cloned()
            .collect()
    }
}
