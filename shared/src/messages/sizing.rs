use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use strum::EnumIter;

#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(feature = "actix", derive(actix::Message))]
#[cfg_attr(feature = "actix", rtype(result = "()"))]
#[serde(tag = "type", content = "data")]
pub enum SizingRequestMessage {
    Vote { size: String },
    ChangeVotingSystem { new_system: VotingSystem },
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
        voting_system: VotingSystem,
        spectators: Vec<String>,
    },
    VoteResults {
        votes: HashMap<String, String>,
    },
    VotingSystemChanged {
        new_system: VotingSystem,
    },
    NewVote,
    VotingOver,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default, Eq, PartialEq, EnumIter, Copy)]
pub enum VotingSystem {
    #[default]
    Fibonacci,
    FistToFive,
    TShirt,
}

impl VotingSystem {
    /// A human-readable label, for UIs that let the user pick a voting system.
    pub fn display_name(&self) -> &'static str {
        match self {
            VotingSystem::Fibonacci => "Fibonacci",
            VotingSystem::FistToFive => "Fist to Five",
            VotingSystem::TShirt => "T-Shirt",
        }
    }

    /// The card values offered to voters, in display order. Also the set the
    /// server accepts as a valid vote.
    pub fn values(&self) -> &'static [&'static str] {
        match self {
            VotingSystem::Fibonacci => &["0", "1", "2", "3", "5", "8", "13", "21"],
            VotingSystem::FistToFive => &["0", "1", "2", "3", "4", "5"],
            VotingSystem::TShirt => &["XS", "S", "M", "L", "XL"],
        }
    }
}
