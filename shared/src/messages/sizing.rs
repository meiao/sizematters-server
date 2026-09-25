use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "actix", derive(actix::Message))]
#[cfg_attr(feature = "actix", rtype(result = "()"))]
#[serde(tag = "type", content = "data")]
pub enum SizingMessage {
    Vote { size: String },
    NewVote,
    Randomize,
}
