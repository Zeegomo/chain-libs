use crate::{
    certificate::{DecryptedPrivateTally, DecryptedPrivateTallyProposal},
    testing::data::CommitteeMembersManager,
    vote::VotePlanStatus,
};
use chain_vote::TallyOptimizationTable;

use rand::thread_rng;

pub fn decrypt_tally(
    vote_plan_status: &VotePlanStatus,
    members: &CommitteeMembersManager,
) -> DecryptedPrivateTally {
    let encrypted_tally = vote_plan_status
        .proposals
        .iter()
        .map(|proposal| {
            let tally_state = proposal.tally.as_ref().unwrap();
            let encrypted_tally = tally_state.private_encrypted().unwrap().0.clone();
            let max_votes = tally_state.private_total_power().unwrap();
            (encrypted_tally, max_votes)
        })
        .collect::<Vec<_>>();

    let absolute_max_votes = encrypted_tally
        .iter()
        .map(|(_encrypted_tally, max_votes)| *max_votes)
        .max()
        .unwrap();
    let table = TallyOptimizationTable::generate_with_balance(absolute_max_votes, 1);

    let members_pks: Vec<chain_vote::MemberPublicKey> = members
        .members()
        .iter()
        .map(|member| member.public_key())
        .collect();
    let proposals = encrypted_tally
        .into_iter()
        .map(|(encrypted_tally, max_votes)| {
            let decrypt_shares = members
                .members()
                .iter()
                .map(|member| member.secret_key())
                .map(|secret_key| encrypted_tally.partial_decrypt(&mut thread_rng(), secret_key))
                .collect::<Vec<_>>();

            let tally = encrypted_tally
                .decrypt_tally(max_votes, &decrypt_shares, &members_pks, &table)
                .unwrap();
            DecryptedPrivateTallyProposal {
                decrypt_shares: decrypt_shares.into_boxed_slice(),
                tally_result: tally.votes.into_boxed_slice(),
            }
        })
        .collect::<Vec<_>>();

    DecryptedPrivateTally::new(proposals)
}
