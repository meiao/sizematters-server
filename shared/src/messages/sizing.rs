use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "actix", derive(actix::Message))]
#[cfg_attr(feature = "actix", rtype(result = "()"))]
#[serde(tag = "type", content = "data")]
pub enum SizingRequestMessage {
    Vote { size: String },
    NewVote,
    Randomize,
    RegisterToVote,
    RevokeVotingRights { target_id: String },
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "actix", derive(actix::Message))]
#[cfg_attr(feature = "actix", rtype(result = "()"))]
#[serde(tag = "type", content = "data")]
pub enum SizingResponseMessage {
    OwnVote {
        size: String,
    },
    VoteStatus {
        votes: HashMap<String, bool>,
        spectators: Vec<String>,
    },
    VoteResults {
        votes: HashMap<String, String>,
    },
    NewVote,
    VotingOver,
}
