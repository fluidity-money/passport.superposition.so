use borsh::{BorshDeserialize, BorshSerialize};

use stylus_sdk::prelude::calls::errors::Error as StylusErr;

pub use crate::{applicative::ApplicativeLabel, result::Res};

use stylus_sdk::alloy_primitives::{Address, U256, FixedBytes};

#[derive(BorshSerialize, BorshDeserialize, Clone, PartialEq, Debug)]
pub enum MathContext {
    ApplyCommitLeftAmtFilled,
    ApplyCommitRightAmtFilled,
    IncreaseInterim,
    DecreaseInterim,
    IncreaseWithdrawal,
    DecreaseWithdrawal,
    IncreaseOrder,
    DecreaseOrder,
    ApplyCommitBalanceAmount,
    ApplyCommitLeftAmtUnfilled,
    ApplyCommitRightAmtUnfilled,
    AddLiq,
    Onboard,
}

/// ErrorDiscriminant is not shown to users, even if it contains any
/// information. It could have its contents printed while running on the
/// native host.
#[derive(BorshSerialize, BorshDeserialize, Clone, PartialEq, Debug)]
pub enum ErrorDiscriminant {
    /// The error wasn't created properly.
    Unknown,

    /// Generic call error that we translated directly.
    BadCall,

    /// A token was taken that's inconsistent with the user's goal.
    GoalInconsistent,

    /// Goal not met for checking goal amounts.
    GoalNotMet,

    /// Incorrect Applicative transition. To and from.
    BadApplicativeTransition,

    /// The nonce was inconsistent with our local storage of it!
    BadNonce,

    /// The deadline is out of date.
    BadDeadline,

    /// An unpack returned errorneously.
    BadUnpack,

    /// Bad signer of a ecrecover call.
    BadEcrecoverSigner,

    /// Bad verifying key creation.
    BadVerifyingKey,

    /// Bad creation of a signature from what's in the applicative structure.
    BadSignatureCreation(ApplicativeLabel),

    /// Bad verifying of a signature using strict methods.
    BadStrictVerify(ApplicativeLabel),

    /// The signer wasn't found using their id.
    SignerNotFoundId,

    /// The signer wasn't found using a verifying key.
    SignerNotFoundKey,

    /// The convert stage couldn't find the ID given.
    AccountIdNotFound,

    /// The convert stage couldn't find the address.
    AccountKeyNotFound,

    /// A bad conversion from took place from the applicative form to the
    /// state machine form.
    BadConversionFrom,

    /// One side of the commit was asking for a side that doesn't match up
    /// with the other side.
    BadAssetComparison,

    /// Inconsistent owners for a multiple step operation.
    InconsistentOwners,

    /// Not enough from the derivative amount to fill.
    NotEnoughForDeriv,

    /// No left excess is available to the user from this commit!
    NoLeftExcess,

    /// Checked sub overflow in the math!
    CheckedSub(MathContext, u128, u128),

    /// Checked add overflow in the math!
    CheckedAdd(MathContext, u128, u128),

    /// Zero amount in the balance object.
    ZeroBalanceAmount,

    SameAssets,

    BadAssetAsks,

    Erc20TransferFromCall,

    Erc20TransferFromDecode,

    Erc20TransferFromFalse,

    Erc20BalanceOfCall,

    Erc20BalanceOfDecode,

    Erc20TransferCall,

    Erc20TransferDecode,

    Erc20TransferFalse,

    /// This happens if the amount that the user asked to transition from their balance to their
    /// order is incorrect during the Order stage.
    BalanceTransitionToOrderBad,

    /// The hash was already seen onchain!
    HashAlreadyOnchain([u8; 64]),

    /// The token has no code!
    TokenNoCode,

    /// The signature given during onboarding was bad!
    BadOnboardingSig,

    /// It wasn't possible to verify a signature during a sig_two validate.
    BadStrictVerifyTwo(ApplicativeLabel, u8),

    /// During testing, there wasn't enough balance for a transfer!
    TestNotEnoughBalForTransfer,

    /// During testing, there wasn't enough allowance!
    TestNotEnoughAllowance,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ErrorTestContext {
    pub sender: Address,
    pub recipient: Address,
    pub asset: Address,
    pub amt: U256,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ErrorTestInterimDetails {
    pub owner_l: Address,
    pub owner_r: Address,
    pub asset_l: Address,
    pub asset_r: Address,
    pub amt: u128,
    pub hash: FixedBytes<32>,
    pub thread_recorded_owner: Address,
    pub thread_recorded_asset: Address,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ErrorInterimAccessContext {
    pub accessed_hash: FixedBytes<32>,
    pub interim_hashes: Vec<ErrorTestInterimDetails>
}

pub struct Error {
    pub typ: ErrorDiscriminant,
    // Used to hint information about the app when an error happens if this
    // is tagged on.
    pub test_context: Option<ErrorTestContext>,
    pub test_interim: Option<ErrorInterimAccessContext>,
}

impl Error {
    pub fn test_context(mut self, e: ErrorTestContext) -> Self {
        self.test_context = Some(e);
        self
    }

    pub fn test_interim(mut self, v: ErrorInterimAccessContext) -> Self {
        self.test_interim = Some(v);
        self
    }
}

impl Default for Error {
    fn default() -> Self {
        Error {
            typ: ErrorDiscriminant::Unknown,
            test_context: None,
            test_interim: None
        }
    }
}

impl borsh::ser::BorshSerialize for Error {
    fn serialize<W: borsh::io::Write>(&self, writer: &mut W) -> borsh::io::Result<()> {
        self.typ.serialize(writer)
    }
}

impl borsh::de::BorshDeserialize for Error {
    fn deserialize_reader<R: borsh::io::Read>(r: &mut R) -> borsh::io::Result<Self> {
        Ok(Error::from(ErrorDiscriminant::deserialize_reader(r)?))
    }
}

impl core::fmt::Debug for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
        let mut d = f.debug_struct("Error");
        d.field("typ", &self.typ);
        if let Some(ref c) = self.test_context {
            d.field("eip20 context", c);
        }
        if let Some(ref c) = self.test_interim {
            d.field("interim values", c);
        }
        d.finish()
    }
}

impl From<StylusErr> for Error {
    fn from(x: StylusErr) -> Error {
        map_stylus_err(ErrorDiscriminant::BadCall, ErrorDiscriminant::BadUnpack, x)
    }
}

pub type R = Result<Res, Error>;

pub fn err_cd(typ: ErrorDiscriminant) -> R {
    Err(Error::from(typ))
}

pub fn map_stylus_err(
    call_unp: ErrorDiscriminant,
    unpack_unp: ErrorDiscriminant,
    x: StylusErr,
) -> Error {
    match x {
        StylusErr::AbiDecodingFailed(_) => Error::from(unpack_unp),
        StylusErr::Revert(_) => Error::from(call_unp),
    }
}

impl From<ErrorDiscriminant> for Error {
    fn from(x: ErrorDiscriminant) -> Self {
        let mut e = Error::default();
        e.typ = x;
        e
    }
}

impl From<alloy_sol_types::Error> for Error {
    fn from(_: alloy_sol_types::Error) -> Self {
        // It's likely we're using this for a failed decoding, so that's what
        // we're always assuming.
        Error::from(ErrorDiscriminant::BadUnpack)
    }
}

pub fn err(x: ErrorDiscriminant) -> R {
    err_cd(x)
}

pub const NOOP: R = Ok(Res::Noop);

pub const DONE_UNIT: R = Ok(Res::DoneUnit);

pub fn done_u64(x: u64) -> R {
    Ok(Res::DoneU64(x))
}

#[macro_export]
macro_rules! require {
    ($cond:expr, $err:ident) => {
        if !($cond) {
            err(ErrorDiscriminant::$err)?;
        }
    };
}
