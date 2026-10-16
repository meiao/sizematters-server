use crate::actors::messages::RoomMessages;
use crate::actors::rooms::jyanken_behavior::JyankenBehavior;
use crate::actors::rooms::room::Room;
use crate::actors::rooms::sizing_behavior::SizingBehavior;
use actix::{Actor, Recipient};
use sizematters_shared::RoomType;
use std::sync::Arc;

pub(super) fn create(
    room_name: Arc<String>,
    password: Arc<String>,
    password_is_hash: bool,
    room_type: RoomType,
    room_manager: Recipient<RoomMessages>,
) -> Recipient<RoomMessages> {
    match room_type {
        RoomType::Sizing => Room::new(
            room_name,
            password,
            password_is_hash,
            room_manager,
            room_type,
            Box::new(SizingBehavior::new()),
        )
        .start()
        .recipient(),
        RoomType::Jyanken => Room::new(
            room_name,
            password,
            password_is_hash,
            room_manager,
            room_type,
            Box::new(JyankenBehavior::new()),
        )
        .start()
        .recipient(),
    }
}
