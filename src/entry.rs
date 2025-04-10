use alloc::vec::Vec;

use stylus_sdk::{alloy_primitives::*, prelude::*};

pub use crate::storage::*;

use crate::{utils::*, *};

impl StoragePassport {
    pub fn dummy(&self) -> R {
        NOTHING
    }

    pub fn set_addr(&mut self, _y: BAddress) -> R {
        NOTHING
    }

    pub fn simple_match(
        &mut self,
        requests: Vec<(MatchReq, EdSig)>,
        bonding: Vec<BondingReq>,
        permits: Vec<PermitReq>,
    ) -> R {
        // Match sends multiple tokens from users, optionally using the degraded
        // permit form if it's available prior to using transferFrom. If bonding
        // signatures are provided, it will execute those prior to the address.
        // It transfers each given token to the contract, which it then transfers
        // to the following address in the requests array. The last user of the
        // requests array sends their token amount to the first user in the match
        // function. This code will not run unless the requests array is not
        // even. Bumps internal nonces for interactions completed here per
        // signature.
        require!(
            requests.len() > 0 && requests.len() % 2 == 0,
            UnusualRequestsAmount
        );
        for PermitReq {
            token,
            value,
            deadline,
            v,
            r,
            s,
        } in permits
        {
            erc20_call::permit(token.x, value.x, deadline.x, v, r, s)?;
        }
        for BondingReq {
            ed_addr,
            eth_addr,
            deadline,
            r,
            s,
            v,
        } in bonding
        {}
        let mut last_token = None;
        let mut last_amt = U256::ZERO;
        for (req, sig) in requests.iter() {
            let MatchReq {
                sender,
                spend_token,
                spend_amt,
                goal_token,
                goal_amt,
                nonce,
                deadline,
            } = req;
            require!(signatures::validate_req(sender, &req, &sig), InvalidRequest);
            let sender = FixedBytes::<32>::from(sender);
            let owner = self.owners.get(sender);
            require!(self.nonces.get(sender) == nonce.x, BadNonce);
            require!(self.vm().block_timestamp() >= *deadline, BadDeadline);
            if let Some(last_token) = last_token {
                // Check that the goal amount is consistent with the user's request here.
                require!(last_token == goal_token.x, GoalInconsistent);
                require!(last_amt >= goal_amt.x, GoalNotMet);
            }
            erc20_call::transfer_from(owner, spend_token.x, spend_amt.x)?;
            last_token = Some(spend_token.x);
            last_amt = spend_amt.x;
        }
        // Check that the token amount and address for the first request is correct.
        require!(
            last_token.unwrap() == requests[0].0.goal_token.x,
            GoalInconsistent
        );
        require!(last_amt >= requests[0].0.goal_amt.x, GoalNotMet);
        DONE
    }
}
