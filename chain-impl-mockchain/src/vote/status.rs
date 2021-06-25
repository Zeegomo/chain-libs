use crate::{
    certificate::{ExternalProposalId, VotePlanId},
    date::BlockDate,
    transaction::UnspecifiedAccountIdentifier,
    vote::{Options, PayloadType, Tally, ValidatedVote},
};
use chain_vote::MemberPublicKey;
use imhamt::Hamt;
use std::collections::hash_map::DefaultHasher;

pub struct VotePlanStatus {
    pub id: VotePlanId,
    pub payload: PayloadType,
    pub vote_start: BlockDate,
    pub vote_end: BlockDate,
    pub committee_end: BlockDate,
    pub committee_public_keys: Vec<MemberPublicKey>,
    pub proposals: Vec<VoteProposalStatus>,
}

pub struct VoteProposalStatus {
    pub index: u8,
    pub proposal_id: ExternalProposalId,
    pub options: Options,
    pub tally: Option<Tally>,
    pub votes: Hamt<DefaultHasher, UnspecifiedAccountIdentifier, ValidatedVote>,
}
