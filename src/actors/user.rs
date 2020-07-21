use crate::actors::messages::{ClientResponseMessage, UserInfoUpdate};
use crate::data::{UserData, UserDataDb};
use actix::prelude::*;
use actix::Actor;

pub struct UserManagerActor {
    user_db: UserDataDb,
}

impl Default for UserManagerActor {
    fn default() -> UserManagerActor {
        UserManagerActor {
            user_db: UserDataDb::default(),
        }
    }
}

impl Actor for UserManagerActor {
    type Context = Context<Self>;
}

impl Handler<UserInfoUpdate> for UserManagerActor {
    type Result = ();

    fn handle(&mut self, msg: UserInfoUpdate, ctx: &mut Context<Self>) -> Self::Result {
        match msg {
            UserInfoUpdate::SetName {
                user_id,
                name,
                recipient,
            } => self.set_name(user_id, name, recipient),
            UserInfoUpdate::SetAvatar {
                user_id,
                gravatar_id,
                recipient,
            } => self.set_gravatar(user_id, gravatar_id, recipient),
            UserInfoUpdate::Register { user_id, recipient } => self.register(user_id, recipient),
        }
    }
}

impl UserManagerActor {
    fn set_name(
        &mut self,
        user_id: String,
        name: String,
        recipient: Recipient<ClientResponseMessage>,
    ) {
        self.user_db.update_name(&user_id, name);

        match self.user_db.get_user(&user_id) {
            None => println!("Unable to retrieve updated user."),
            Some(user_data) => self.send_data_back(user_data, recipient),
        };
    }

    fn set_gravatar(
        &mut self,
        user_id: String,
        gravatar_id: String,
        recipient: Recipient<ClientResponseMessage>,
    ) {
        self.user_db.update_gravatar(&user_id, &gravatar_id);

        match self.user_db.get_user(&user_id) {
            None => println!("Unable to retrieve updated user."),
            Some(user_data) => self.send_data_back(user_data, recipient),
        };
    }

    fn register(&mut self, user_id: String, recipient: Recipient<ClientResponseMessage>) {
        let user_data = self.user_db.add(user_id);
        self.send_data_back(user_data, recipient);
    }

    fn send_data_back(&self, user_data: UserData, recipient: Recipient<ClientResponseMessage>) {
        recipient.do_send(ClientResponseMessage::YourData { user: user_data });
    }
}
