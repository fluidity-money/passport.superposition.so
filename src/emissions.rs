/// Final step of the Applicative --> StateMachine conversion.

use borsh::{BorshDeserialize, BorshSerialize};

use crate::{state_machine::*, encoding::*};

/// Simple view of the Commit structure to send recipients of the trade
/// outcomes.
#[derive(Clone, PartialEq, Debug)]
pub struct Outcome {
    asset: BAddress,
    recipient: BAddress,
    amt: BU256,
}

/// The emissions of the trade that took place here, in the form of a view.
/// Aka the inverted form of the request, or just the left side's fulfilled balance
/// flipped around slash the right side.
#[derive(Clone, PartialEq, Debug)]
pub struct Emissions {
    commit: Commit,
    left_outcome: Outcome,
    right_outcome: Outcome,
}
