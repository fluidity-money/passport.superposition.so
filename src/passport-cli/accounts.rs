#[derive(Debug, Clone)]
pub enum RecipeUnsolvedDecodeErr {
    BadKey,
    NoAccountNameField,
    NoAccountKeyField,
    NoOffsetField,
    BadKeyLengthIfHashbangMaybeCutoff,
    BadOffset,
}

#[cfg(not(target_arch = "wasm32"))]
impl std::fmt::Display for RecipeUnsolvedDecodeErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl serde::ser::StdError for RecipeUnsolvedDecodeErr {}

#[derive(Clone, Debug)]
pub struct Key(pub [u8; 32]);

impl std::str::FromStr for Key {
    type Err = RecipeUnsolvedDecodeErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let k: [u8; 32] = const_hex::decode(s)
            .map_err(|_| RecipeUnsolvedDecodeErr::BadKey)?
            .try_into()
            .map_err(|_| RecipeUnsolvedDecodeErr::BadKeyLengthIfHashbangMaybeCutoff)?;
        Ok(Key(k))
    }
}

unsafe impl std::marker::Send for Key {}
unsafe impl std::marker::Sync for Key {}

#[derive(Clone, Debug)]
pub struct Account {
    pub name: String,
    pub key: Key,
    pub offset: u64,
}

unsafe impl std::marker::Send for Account {}
unsafe impl std::marker::Sync for Account {}

#[derive(Clone, Debug)]
pub struct Accounts(pub Vec<Account>);

unsafe impl std::marker::Send for Accounts {}
unsafe impl std::marker::Sync for Accounts {}

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
            key: Key::from_str(i.next().ok_or(RecipeUnsolvedDecodeErr::NoAccountKeyField)?)?,
            offset: u64::from_str(i.next().ok_or(RecipeUnsolvedDecodeErr::NoOffsetField)?)
                .map_err(|_| RecipeUnsolvedDecodeErr::BadOffset)?,
        })
    }
}
