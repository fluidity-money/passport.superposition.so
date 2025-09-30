#[derive(Clone, Debug)]
pub struct Account {
    pub name: String,
    pub key: [u8; 32],
    pub place: u64,
}

unsafe impl std::marker::Send for Account {}
unsafe impl std::marker::Sync for Account {}

#[derive(Clone, Debug)]
pub struct Accounts(pub Vec<Account>);

unsafe impl std::marker::Send for Accounts {}
unsafe impl std::marker::Sync for Accounts {}

#[derive(Debug, Clone)]
pub enum RecipeUnsolvedDecodeErr {
    BadNumber,
    NoAccountNameField,
    NoAccountKeyField,
    NoPlacementField,
    BadKeyLength,
    BadPlace,
}

#[cfg(not(target_arch = "wasm32"))]
impl std::fmt::Display for RecipeUnsolvedDecodeErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl serde::ser::StdError for RecipeUnsolvedDecodeErr {}

impl std::str::FromStr for Accounts {
    type Err = RecipeUnsolvedDecodeErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Accounts(
            s.split(',')
                .map(Account::from_str)
                .collect::<Result<Vec<_>, _>>()?,
        ))
    }
}

impl std::str::FromStr for Account {
    type Err = RecipeUnsolvedDecodeErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut i = s.split(':');
        Ok(Account {
            name: i
                .next()
                .ok_or(RecipeUnsolvedDecodeErr::NoAccountNameField)?
                .to_string(),
            key: const_hex::decode(i.next().ok_or(RecipeUnsolvedDecodeErr::NoAccountKeyField)?)
                .map_err(|_| RecipeUnsolvedDecodeErr::BadNumber)?
                .try_into()
                .map_err(|_| RecipeUnsolvedDecodeErr::BadKeyLength)?,
            place: u64::from_str(i.next().ok_or(RecipeUnsolvedDecodeErr::NoPlacementField)?)
                .map_err(|_| RecipeUnsolvedDecodeErr::BadPlace)?,
        })
    }
}
