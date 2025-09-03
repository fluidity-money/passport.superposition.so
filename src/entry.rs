use crate::{
    accounts::{AccountsExpanded, AccountsList},
    applicative::Applicative,
    call_erc20,
    encoding::*,
    error::*,
    ops::*,
    storage::*,
};

use stylus_sdk::{alloy_primitives::U256, prelude::HostAccess};

impl StoragePassport {
    pub fn dummy(&self) -> R {
        DONE_UNIT
    }

    pub fn query_unused_liq(&self, addr: BAddress, asset: BAddress) -> R {
        DONE_U128(u128::from_le_bytes(
            self.withdrawable.getter(addr.x).get(asset.x).to_le_bytes(),
        ))
    }

    pub fn deposit_unused_liq(
        &mut self,
        DepositUnusedLiquidity {
            asset,
            amount,
            permit,
            association,
        }: DepositUnusedLiquidity,
    ) -> R {
        if let Some(_) = permit {
            todo!();
        }
        if let Some((_ed_addr, _ed_sig)) = association {
            todo!()
        }
        let sender = self.vm().msg_sender();
        call_erc20::transfer_from(
            self.vm(),
            asset.x,
            sender,
            self.vm().contract_address(),
            u128_to_u256(amount),
        )?;
        self.increase_withdrawal(sender, asset.x, amount)?;
        DONE_UNIT
    }

    pub fn solve(&mut self, accounts: AccountsList, applicative: Applicative) -> R {
        let accounts: AccountsExpanded = accounts.into();
        self.apply(self.validate(&accounts, applicative)?)?;
        DONE_UNIT
    }
}

fn u128_to_u256(x: u128) -> U256 {
    let mut b = [0u8; 32];
    b[16..].copy_from_slice(&x.to_be_bytes());
    U256::from_be_bytes(b)
}
