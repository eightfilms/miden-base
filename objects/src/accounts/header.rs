use super::{hash_account, Account, AccountId, Digest, Felt};

// ACCOUNT HEADER
// ================================================================================================

/// A header of an account which contains information that succinctly describes the state of the
/// components of the account.
///
/// The [AccountHeader] is composed of:
/// - id: the account id ([AccountId]) of the account.
/// - nonce: the nonce of the account.
/// - vault_root: a commitment to the account's vault ([super::AssetVault]).
/// - storage_commitment: a commitment to the account's storage ([super::AccountStorage]).
/// - code_commitment: a commitment to the account's code ([super::AccountCode]).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccountHeader {
    id: AccountId,
    nonce: Felt,
    vault_root: Digest,
    storage_commitment: Digest,
    code_commitment: Digest,
}

impl AccountHeader {
    // CONSTRUCTORS
    // --------------------------------------------------------------------------------------------
    /// Creates a new [AccountHeader].
    pub fn new(
        id: AccountId,
        nonce: Felt,
        vault_root: Digest,
        storage_commitment: Digest,
        code_commitment: Digest,
    ) -> Self {
        Self {
            id,
            nonce,
            vault_root,
            storage_commitment,
            code_commitment,
        }
    }

    // PUBLIC ACCESSORS
    // --------------------------------------------------------------------------------------------
    /// Returns hash of this account.
    ///
    /// Hash of an account is computed as hash(id, nonce, vault_root, storage_commitment,
    /// code_commitment). Computing the account hash requires 2 permutations of the hash
    /// function.
    pub fn hash(&self) -> Digest {
        hash_account(
            self.id,
            self.nonce,
            self.vault_root,
            self.storage_commitment,
            self.code_commitment,
        )
    }

    /// Returns the id of this account.
    pub fn id(&self) -> AccountId {
        self.id
    }

    /// Returns the nonce of this account.
    pub fn nonce(&self) -> Felt {
        self.nonce
    }

    /// Returns the vault root of this account.
    pub fn vault_root(&self) -> Digest {
        self.vault_root
    }

    /// Returns the storage commitment of this account.
    pub fn storage_commitment(&self) -> Digest {
        self.storage_commitment
    }

    /// Returns the code commitment of this account.
    pub fn code_commitment(&self) -> Digest {
        self.code_commitment
    }
}

impl From<Account> for AccountHeader {
    fn from(account: Account) -> Self {
        (&account).into()
    }
}

impl From<&Account> for AccountHeader {
    fn from(account: &Account) -> Self {
        Self {
            id: account.id(),
            nonce: account.nonce(),
            vault_root: account.vault().commitment(),
            storage_commitment: account.storage().commitment(),
            code_commitment: account.code().commitment(),
        }
    }
}
