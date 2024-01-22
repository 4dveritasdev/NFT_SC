#![doc = include_str!("../README.md")]
#![allow(unused_variables)]

#[macro_use]
extern crate pbc_contract_codegen;

use create_type_spec_derive::CreateTypeSpec;
use pbc_contract_common::address::Address;
use pbc_contract_common::context::ContractContext;
use pbc_contract_common::sorted_vec_map::{SortedVec, SortedVecMap};
use read_write_state_derive::ReadWriteState;

/// A permission to transfer and approve NFTs given from an NFT owner to a separate address, called an operator.
#[derive(ReadWriteState, CreateTypeSpec, PartialEq, Copy, Clone, Ord, PartialOrd, Eq)]
struct OperatorApproval {
    /// NFT owner.
    owner: Address,
    /// Operator of the owner's tokens.u12
    operator: Address,
}

#[derive(ReadWriteState, CreateTypeSpec, PartialEq, Clone, Ord, PartialOrd, Eq)]
struct UserMetadata {
    id: String,
    wallet: Address
}

#[derive(ReadWriteState, CreateTypeSpec, PartialEq, Clone, Ord, PartialOrd, Eq)]
struct ProductMetadata {
    contract_address: Address,
    id: u128,
}

/// State of the contract.
#[state]
pub struct NFTContractState {
    /// Descriptive name for the collection of NFTs in this contract.
    name: String,
    /// Abbreviated name for NFTs in this contract.
    symbol: String,
    /// Mapping from token_id to the owner of the token.
    owners: SortedVecMap<u128, Address>,
    /// Mapping from token_id to the approved address who can transfer the token.
    token_approvals: SortedVecMap<u128, Address>,
    /// Containing approved operators of owners. Operators can transfer and change approvals on all tokens owned by owner.
    operator_approvals: SortedVec<OperatorApproval>,
    /// Template which the uri's of the NFTs fit into.
    uri_template: String,
    /// Mapping from token_id to the URI of the token.
    user_list: SortedVecMap<u128, UserMetadata>,
    /// Owner of the contract. Is allowed to mint new NFTs.
    contract_owner: Address,
    wallet_owner: SortedVecMap<Address, u128>,
    user_product_list: SortedVecMap<u128, SortedVec<ProductMetadata>>,
    total_count: u128
}

impl NFTContractState {
    /// Find the owner of an NFT.
    /// Throws if no such token exists.
    ///
    /// ### Parameters:
    ///
    /// * `token_id`: [`u128`] The identifier for an NFT.
    ///
    /// ### Returns:
    ///
    /// An [`Address`] for the owner of the NFT.
    pub fn owner_of(&self, token_id: u128) -> Address {
        let owner_opt = self.owners.get(&token_id);
        match owner_opt {
            None => panic!("MPC-721: owner query for nonexistent token"),
            Some(owner) => *owner,
        }
    }

    /// Get the approved address for a single NFT.
    ///
    /// ### Parameters:
    ///
    /// * `token_id`: [`u128`] The NFT to find the approved address for.
    ///
    /// ### Returns:
    ///
    /// An [`Option<Address>`] The approved address for this NFT, or none if there is none.
    pub fn get_approved(&self, token_id: u128) -> Option<Address> {
        self.token_approvals.get(&token_id).copied()
    }

    /// Query if an address is an authorized operator for another address.
    ///
    /// ### Parameters:
    ///
    /// * `owner`: [`Address`] The address that owns the NFTs.
    ///
    /// * `operator`: [`Address`] The address that acts on behalf of the owner.
    ///
    /// ### Returns:
    ///
    /// A [`bool`] true if `operator` is an approved operator for `owner`, false otherwise.
    pub fn is_approved_for_all(&self, owner: Address, operator: Address) -> bool {
        let as_operator_approval: OperatorApproval = OperatorApproval { owner, operator };
        self.operator_approvals.contains(&as_operator_approval)
    }

    /// Helper function to check whether a tokenId exists.
    ///
    /// Tokens start existing when they are minted (`mint`),
    /// and stop existing when they are burned (`burn`).
    ///
    /// ### Parameters:
    ///
    /// * `token_id`: [`u128`] The tokenId that is checked.
    ///
    /// ### Returns:
    ///
    /// A [`bool`] True if `token_id` is in use, false otherwise.
    pub fn exists(&self, token_id: u128) -> bool {
        let owner = self.owners.get(&token_id);
        owner.is_some()
    }

    /// Helper function to check whether a spender is owner or approved for a given token.
    /// Throws if token_id does not exist.
    ///
    /// ### Parameters:
    ///
    /// * `spender`: [`Address`] The address to check ownership for.
    ///
    /// * `token_id`: [`u128`] The tokenId which is checked.
    ///
    /// ### Returns:
    ///
    /// A [`bool`] True if `token_id` is owned or approved for `spender`, false otherwise.
    pub fn is_approved_or_owner(&self, spender: Address, token_id: u128) -> bool {
        let owner = self.owner_of(token_id);
        spender == owner
            || self.get_approved(token_id) == Some(spender)
            || self.is_approved_for_all(owner, spender)
    }

    /// Mutates the state by approving `to` to operate on `token_id`.
    /// None indicates there is no approved address.
    ///
    /// ### Parameters:
    ///
    /// * `approved`: [`Option<Address>`], The new approved NFT controller.
    ///
    /// * `token_id`: [`u128`], The NFT to approve.
    pub fn _approve(&mut self, approved: Option<Address>, token_id: u128) {
        if let Some(appr) = approved {
            self.token_approvals.insert(token_id, appr);
        } else {
            self.token_approvals.remove(&token_id);
        }
    }

    /// Mutates the state by transferring `token_id` from `from` to `to`.
    /// As opposed to {transfer_from}, this imposes no restrictions on `ctx.sender`.
    ///
    /// Throws if `from` is not the owner of `token_id`.
    ///
    /// ### Parameters:
    ///
    /// * `from`: [`Address`], The current owner of the NFT
    ///
    /// * `to`: [`Address`], The new owner
    ///
    /// * `token_id`: [`u128`], The NFT to transfer
    pub fn _transfer(&mut self, from: Address, to: Address, token_id: u128) {
        if self.owner_of(token_id) != from {
            panic!("MPC-721: transfer from incorrect owner")
        } else {
            // clear approvals from the previous owner
            self._approve(None, token_id);
            self.owners.insert(token_id, to);
        }
    }
}

/// Initial function to bootstrap the contracts state.
///
/// ### Parameters:
///
/// * `ctx`: [`ContractContext`], initial context.
///
/// * `name`: [`String`], A descriptive name for a collection of NFTs in this contract.
///
/// * `symbol`: [`String`], An abbreviated name for NFTs in this contract.
///
/// * `uri_template`: [`String`], Template for uri´s associated with NFTs in this contract.
///
/// ### Returns:
///
/// The new state object of type [`NFTContractState`].
#[init]
pub fn initialize(
    ctx: ContractContext,
    name: String,
    symbol: String,
    uri_template: String,
) -> NFTContractState {
    NFTContractState {
        name,
        symbol,
        owners: SortedVecMap::new(),
        token_approvals: SortedVecMap::new(),
        operator_approvals: SortedVec::new(),
        uri_template,
        user_list: SortedVecMap::new(),
        contract_owner: ctx.sender,
        wallet_owner: SortedVecMap::new(),
        user_product_list: SortedVecMap::new(),
        total_count: 0
    }
}

/// Mints `token_id` and transfers it to an owner.
///
/// Requirements:
///
/// - `token_id` must not exist
/// - `ctx.sender` owns the contract
///
/// ### Parameters:
///
/// * `ctx`: [`ContractContext`], the context for the action call.
///
/// * `state`: [`NFTContractState`], the current state of the contract.
///
/// * `to`: [`Address`], the owner of the minted token.
///
/// * `token_id`: [`u128`], The new id for the minted token.
///
/// ### Returns
///
/// The new state object of type [`NFTContractState`] with an updated ledger.
#[action(shortname = 0x01)]
pub fn mint(
    ctx: ContractContext,
    mut state: NFTContractState,
    user_id: String,
    wallet: Address,
) -> NFTContractState {
    if ctx.sender != state.contract_owner {
        panic!("MPC-721: mint only callable by the contract owner")
    } else {
        state.total_count += 1;
        let token_uri = UserMetadata {
            id: user_id,
            wallet: wallet
        };

        state.user_list.insert(state.total_count, token_uri);
        state.wallet_owner.insert(wallet, state.total_count);
        state
    }
}

#[action(shortname = 0x02)]
pub fn transfer_product(
    ctx: ContractContext,
    mut state: NFTContractState,
    from: Address,
    to: Address,
    product_address: Address,
    product_id: u128
) -> NFTContractState {
    if ctx.sender != state.contract_owner {
        panic!("MPC-721: mint only callable by the contract owner")
    } else {

        let to_id = state.wallet_owner.get(&to).unwrap();
        let to_product_list = state.user_product_list.get_mut(&to_id).unwrap();

        let product_uri = ProductMetadata {
            contract_address: product_address,
            id: product_id
        };
        to_product_list.insert(product_uri.clone());

        let from_id = state.wallet_owner.get(&from).unwrap();
        let from_product_list = state.user_product_list.get_mut(&from_id).unwrap();
        from_product_list.remove(&product_uri.clone());

        state
    }
}

#[action(shortname = 0x03)]
pub fn mint_product(
    ctx: ContractContext,
    mut state: NFTContractState,
    to: Address,
    product_address: Address,
    product_id: u128
) -> NFTContractState {
    if ctx.sender != state.contract_owner {
        panic!("MPC-721: mint only callable by the contract owner")
    } else {

        let to_id = state.wallet_owner.get(&to).unwrap();
        let to_product_list = state.user_product_list.get_mut(&to_id).unwrap();

        let product_uri = ProductMetadata {
            contract_address: product_address,
            id: product_id
        };
        to_product_list.insert(product_uri.clone());

        state
    }
}