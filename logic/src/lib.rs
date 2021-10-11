#![allow(dead_code)]
#![allow(unused_imports)]

use std::collections::BTreeMap;

use casper_types::{ApiError, AsymmetricType, ContractPackageHash, Key, URef, U256};

#[cfg(test)]
#[macro_use]
extern crate maplit;

pub mod events;
#[cfg(test)]
pub mod tests;

use events::CEP47Event;

pub type TokenId = String;
pub type Meta = BTreeMap<String, String>;

#[derive(Debug)]
#[repr(u16)]
pub enum Error {
    PermissionDenied = 1,
    ArgumentsError = 2,
    TokenIdAlreadyExists = 3,
    TokenIdDoesntExist = 4,
    NotAnOwner = 5,
}

impl From<Error> for ApiError {
    fn from(error: Error) -> ApiError {
        ApiError::User(error as u16)
    }
}

pub trait WithStorage<Storage: CEP47Storage> {
    fn storage(&self) -> &Storage;
    fn storage_mut(&mut self) -> &mut Storage;
}

pub trait CEP47Contract<Storage: CEP47Storage>: WithStorage<Storage> {
    // Metadata
    fn name(&self) -> String {
        self.storage().name()
    }

    fn symbol(&self) -> String {
        self.storage().symbol()
    }

    fn meta(&self) -> Meta {
        self.storage().meta()
    }

    // Getters
    fn balance_of(&self, owner: &Key, token_id: TokenId) -> U256 {
        self.storage().balance_of(owner, token_id)
    }

    // fn owner_of(&self, token_id: &TokenId) -> Option<Key> {
    //     self.storage().owner_of(token_id)
    // }

    fn total_supply(&self, token_id: TokenId) -> U256 {
        self.storage().total_supply(token_id)
    }

    fn token_meta(&self, token_id: &TokenId) -> Option<Meta> {
        self.storage().token_meta(token_id)
    }

    fn is_paused(&self) -> bool {
        self.storage().is_paused()
    }

    fn pause(&mut self) {
        self.storage_mut().pause();
    }

    fn unpause(&mut self) {
        self.storage_mut().unpause();
    }

    // Minter function.
    // Guarded by the entrypoint group.
    fn mint_one(
        &mut self,
        recipient: &Key,
        token_id: Option<TokenId>,
        token_meta: Meta,
        value: U256,
    ) -> Result<(), Error> {
        self.mint_many(recipient, token_id.map(|id| vec![id]), vec![token_meta], vec![value])
    }

    fn mint_many(
        &mut self,
        recipient: &Key,
        token_ids: Option<Vec<TokenId>>,
        token_metas: Vec<Meta>,
        values: Vec<U256>, 
    ) -> Result<(), Error> {
        let unique_token_ids = match token_ids {
            // Validate token_ids and metas.
            Some(token_ids) => {
                if token_ids.len() != token_metas.len() {
                    return Err(Error::ArgumentsError);
                };
                let valid = self.storage().validate_token_ids(&token_ids);
                if !valid {
                    return Err(Error::TokenIdAlreadyExists);
                };
                Ok(token_ids)
            }
            None => Ok(self.storage_mut().gen_token_ids(token_metas.len() as u32)),
        };

        unique_token_ids.map(|token_ids| {
            // Mint tokens.
            self.storage_mut()
                .mint_many(recipient, &token_ids, &token_metas, &values);

            // Emit event.
            self.storage_mut().emit(CEP47Event::Mint {
                recipient: *recipient,
                token_ids,
                values,
            });
        })
    }

    // fn mint_copies(
    //     &mut self,
    //     recipient: &Key,
    //     token_ids: Option<Vec<TokenId>>,
    //     token_meta: Meta,
    //     count: u32,
    // ) -> Result<(), Error> {
    //     if let Some(token_ids) = &token_ids {
    //         if token_ids.len() != count as usize {
    //             return Err(Error::ArgumentsError);
    //         };
    //     };
    //     let token_metas = vec![token_meta; count as usize];
    //     self.mint_many(recipient, token_ids, token_metas)
    // }

    fn burn_one(&mut self, owner: &Key, token_id: TokenId, value: U256) -> Result<(), Error> {
        self.burn_many(owner, vec![token_id], vec![value])
    }

    fn burn_many(&mut self, owner: &Key, token_ids: Vec<TokenId>, values: Vec<U256>) -> Result<(), Error> {
        if !self.storage().are_all_owner_tokens(owner, &token_ids) {
            return Err(Error::NotAnOwner);
        }

        self.storage_mut().burn_many(owner, &token_ids);

        // Emit burn event.
        self.storage_mut().emit(CEP47Event::Burn {
            owner: *owner,
            token_ids,
            values, 
        });

        Ok(())
    }

    // Transfer functions.
    fn transfer_token(
        &mut self,
        sender: &Key,
        recipient: &Key,
        token_id: &TokenId,
        value: U256
    ) -> Result<(), Error> {
    self.transfer_many_tokens(sender, recipient, &vec![token_id.clone(), vec![value])
    }

    fn transfer_many_tokens(
        &mut self,
        sender: &Key,
        recipient: &Key,
        token_ids: &Vec<TokenId>,
        values: &Vec<U256>,
    ) -> Result<(), Error> {
        if self.is_paused() {
            return Err(Error::PermissionDenied);
        }

        if !self.storage().are_all_owner_tokens(sender, token_ids) {
            return Err(Error::NotAnOwner);
        }

        self.storage_mut()
            .transfer_many(sender, recipient, token_ids, values);

        // Emit transfer event.
        self.storage_mut().emit(CEP47Event::Transfer {
            sender: *sender,
            recipient: *recipient,
            token_ids: token_ids.clone(),
            values: values,
        });
        Ok(())
    }

    fn update_token_metadata(&mut self, token_id: TokenId, meta: Meta) -> Result<(), Error> {
        // Assert token exists.
        if self.owner_of(&token_id).is_none() {
            return Err(Error::TokenIdDoesntExist);
        };

        // Update the storage.
        self.storage_mut().update_token_metadata(&token_id, meta);

        // Emit token update event.
        self.storage_mut()
            .emit(CEP47Event::MetadataUpdate { token_id });

        Ok(())
    }
}

pub trait CEP47Storage {
    // Metadata.
    fn name(&self) -> String;
    fn symbol(&self) -> String;
    fn meta(&self) -> Meta;

    // Getters
    fn balance_of(&self, owner: &Key, token_id: U256) -> U256;
   // fn owner_of(&self, token_id: &TokenId) -> Option<Key>;
    fn total_supply(&self, token_id: U256) -> U256;
    fn token_meta(&self, token_id: &TokenId) -> Option<Meta>;

    // Pause and unpause transfers.
    fn is_paused(&self) -> bool;
    fn pause(&mut self);
    fn unpause(&mut self);

    // Setters
    fn mint_many(&mut self, recipient: &Key, token_ids: &Vec<TokenId>, token_metas: &Vec<Meta>, values: &Vec<U256>);
    fn transfer_many(&mut self, sender: &Key, recipient: &Key, token_ids: &Vec<TokenId>,  values: &Vec<U256>);
    fn burn_many(&mut self, owner: &Key, token_ids: &Vec<TokenId>,  values: &Vec<U256>);
    fn update_token_metadata(&mut self, token_id: &TokenId, meta: Meta);

    fn gen_token_ids(&mut self, n: u32) -> Vec<TokenId>;
    fn validate_token_ids(&self, token_ids: &Vec<TokenId>) -> bool;
    fn are_all_owner_tokens(&self, owner: &Key, token_ids: &Vec<TokenId>) -> bool;

    fn emit(&mut self, event: CEP47Event);
    fn contact_package_hash(&self) -> ContractPackageHash;
}
