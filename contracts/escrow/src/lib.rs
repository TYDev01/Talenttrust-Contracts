#![no_std]

use soroban_sdk::{contract, contracterror, contractimpl, contracttype, Address, Env, Symbol, Vec};

#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContractStatus {
    Created = 0,
    Funded = 1,
    Completed = 2,
    Disputed = 3,
    Refunded = 4,
}

#[contracterror]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EscrowError {
    InvalidParticipant = 1,
    EmptyMilestones = 2,
    InvalidMilestoneAmount = 3,
    ContractNotFound = 4,
    InvalidDepositAmount = 5,
    DepositExceedsTotal = 6,
    InvalidMilestone = 7,
    MilestoneAlreadyReleased = 8,
    MilestoneAlreadyRefunded = 9,
    InsufficientEscrowBalance = 10,
    InvalidStatus = 11,
    EmptyRefundRequest = 12,
    DuplicateMilestone = 13,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Milestone {
    pub amount: i128,
    pub released: bool,
    pub refunded: bool,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EscrowContractData {
    pub client: Address,
    pub freelancer: Address,
    pub status: ContractStatus,
    pub total_amount: i128,
    pub funded_amount: i128,
    pub released_amount: i128,
    pub refunded_amount: i128,
}

#[contracttype]
#[derive(Clone)]
enum DataKey {
    Contract(u32),
    Milestones(u32),
    ContractCount,
}

#[contract]
pub struct Escrow;

#[contractimpl]
impl Escrow {
    /// Creates a new escrow agreement with milestone amounts that can later be
    /// released to the freelancer or refunded back to the client.
    pub fn create_contract(
        env: Env,
        client: Address,
        freelancer: Address,
        milestone_amounts: Vec<i128>,
    ) -> u32 {
        client.require_auth();

        if client == freelancer {
            env.panic_with_error(EscrowError::InvalidParticipant);
        }
        if milestone_amounts.is_empty() {
            env.panic_with_error(EscrowError::EmptyMilestones);
        }

        let mut total_amount = 0_i128;
        let mut milestones = Vec::new(&env);

        for amount in milestone_amounts.iter() {
            if amount <= 0 {
                env.panic_with_error(EscrowError::InvalidMilestoneAmount);
            }

            total_amount += amount;
            milestones.push_back(Milestone {
                amount,
                released: false,
                refunded: false,
            });
        }

        let contract_id = env
            .storage()
            .persistent()
            .get::<_, u32>(&DataKey::ContractCount)
            .unwrap_or(0)
            + 1;

        let contract = EscrowContractData {
            client,
            freelancer,
            status: ContractStatus::Created,
            total_amount,
            funded_amount: 0,
            released_amount: 0,
            refunded_amount: 0,
        };

        env.storage()
            .persistent()
            .set(&DataKey::Contract(contract_id), &contract);
        env.storage()
            .persistent()
            .set(&DataKey::Milestones(contract_id), &milestones);
        env.storage()
            .persistent()
            .set(&DataKey::ContractCount, &contract_id);

        contract_id
    }

    /// Deposits additional client funds into escrow.
    pub fn deposit_funds(env: Env, contract_id: u32, amount: i128) -> bool {
        if amount <= 0 {
            env.panic_with_error(EscrowError::InvalidDepositAmount);
        }

        let mut contract = Self::get_contract_data(&env, contract_id);
        Self::assert_client_auth(&contract);
        Self::assert_open_status(&env, contract.status);

        let updated_amount = contract.funded_amount + amount;
        if updated_amount > contract.total_amount {
            env.panic_with_error(EscrowError::DepositExceedsTotal);
        }

        contract.funded_amount = updated_amount;
        contract.status =
            Self::derive_status(&contract, &Self::get_milestones_data(&env, contract_id));
        Self::save_contract(&env, contract_id, &contract);

        true
    }

    /// Releases a funded milestone to the freelancer. Only the client may
    /// authorize a release.
    pub fn release_milestone(env: Env, contract_id: u32, milestone_id: u32) -> bool {
        let mut contract = Self::get_contract_data(&env, contract_id);
        Self::assert_client_auth(&contract);
        Self::assert_open_status(&env, contract.status);

        let mut milestones = Self::get_milestones_data(&env, contract_id);
        let index = Self::milestone_index(&env, &milestones, milestone_id);
        let mut milestone = milestones.get(index).unwrap();

        if milestone.released {
            env.panic_with_error(EscrowError::MilestoneAlreadyReleased);
        }
        if milestone.refunded {
            env.panic_with_error(EscrowError::MilestoneAlreadyRefunded);
        }
        if Self::escrow_balance(&contract) < milestone.amount {
            env.panic_with_error(EscrowError::InsufficientEscrowBalance);
        }

        milestone.released = true;
        contract.released_amount += milestone.amount;
        milestones.set(index, milestone);

        contract.status = Self::derive_status(&contract, &milestones);
        Self::save_contract(&env, contract_id, &contract);
        Self::save_milestones(&env, contract_id, &milestones);

        true
    }

    /// Refunds selected unreleased milestone balances back to the client.
    ///
    /// The caller must be the contract client. Each requested milestone must be
    /// unique, unreleased, and not previously refunded.
    pub fn refund_unreleased_milestones(
        env: Env,
        contract_id: u32,
        milestone_ids: Vec<u32>,
    ) -> i128 {
        if milestone_ids.is_empty() {
            env.panic_with_error(EscrowError::EmptyRefundRequest);
        }

        let mut contract = Self::get_contract_data(&env, contract_id);
        Self::assert_client_auth(&contract);
        Self::assert_open_status(&env, contract.status);

        let mut milestones = Self::get_milestones_data(&env, contract_id);
        let mut refund_amount = 0_i128;
        let mut seen_ids = Vec::new(&env);

        for milestone_id in milestone_ids.iter() {
            if seen_ids.contains(milestone_id) {
                env.panic_with_error(EscrowError::DuplicateMilestone);
            }
            seen_ids.push_back(milestone_id);

            let index = Self::milestone_index(&env, &milestones, milestone_id);
            let milestone = milestones.get(index).unwrap();

            if milestone.released {
                env.panic_with_error(EscrowError::MilestoneAlreadyReleased);
            }
            if milestone.refunded {
                env.panic_with_error(EscrowError::MilestoneAlreadyRefunded);
            }

            refund_amount += milestone.amount;
        }

        if Self::escrow_balance(&contract) < refund_amount {
            env.panic_with_error(EscrowError::InsufficientEscrowBalance);
        }

        for milestone_id in milestone_ids.iter() {
            let index = Self::milestone_index(&env, &milestones, milestone_id);
            let mut milestone = milestones.get(index).unwrap();
            milestone.refunded = true;
            milestones.set(index, milestone);
        }

        contract.refunded_amount += refund_amount;
        contract.status = Self::derive_status(&contract, &milestones);

        Self::save_contract(&env, contract_id, &contract);
        Self::save_milestones(&env, contract_id, &milestones);

        refund_amount
    }

    /// Returns the full contract state for external inspection and tests.
    pub fn get_contract(env: Env, contract_id: u32) -> EscrowContractData {
        Self::get_contract_data(&env, contract_id)
    }

    /// Returns the milestone list for the specified escrow.
    pub fn get_milestones(env: Env, contract_id: u32) -> Vec<Milestone> {
        Self::get_milestones_data(&env, contract_id)
    }

    /// Returns the currently refundable balance for unreleased milestones.
    pub fn get_refundable_balance(env: Env, contract_id: u32) -> i128 {
        let contract = Self::get_contract_data(&env, contract_id);
        Self::escrow_balance(&contract)
    }

    /// Issue a reputation credential for the freelancer after contract completion.
    pub fn issue_reputation(_env: Env, _freelancer: Address, _rating: i128) -> bool {
        true
    }

    /// Hello-world style function for testing and CI.
    pub fn hello(_env: Env, to: Symbol) -> Symbol {
        to
    }

    fn get_contract_data(env: &Env, contract_id: u32) -> EscrowContractData {
        env.storage()
            .persistent()
            .get(&DataKey::Contract(contract_id))
            .unwrap_or_else(|| env.panic_with_error(EscrowError::ContractNotFound))
    }

    fn get_milestones_data(env: &Env, contract_id: u32) -> Vec<Milestone> {
        env.storage()
            .persistent()
            .get(&DataKey::Milestones(contract_id))
            .unwrap_or_else(|| env.panic_with_error(EscrowError::ContractNotFound))
    }

    fn save_contract(env: &Env, contract_id: u32, contract: &EscrowContractData) {
        env.storage()
            .persistent()
            .set(&DataKey::Contract(contract_id), contract);
    }

    fn save_milestones(env: &Env, contract_id: u32, milestones: &Vec<Milestone>) {
        env.storage()
            .persistent()
            .set(&DataKey::Milestones(contract_id), milestones);
    }

    fn assert_client_auth(contract: &EscrowContractData) {
        contract.client.require_auth();
    }

    fn assert_open_status(env: &Env, status: ContractStatus) {
        if matches!(
            status,
            ContractStatus::Completed | ContractStatus::Disputed | ContractStatus::Refunded
        ) {
            env.panic_with_error(EscrowError::InvalidStatus);
        }
    }

    fn milestone_index(env: &Env, milestones: &Vec<Milestone>, milestone_id: u32) -> u32 {
        if milestone_id >= milestones.len() {
            env.panic_with_error(EscrowError::InvalidMilestone);
        }

        milestone_id
    }

    fn escrow_balance(contract: &EscrowContractData) -> i128 {
        contract.funded_amount - contract.released_amount - contract.refunded_amount
    }

    fn derive_status(contract: &EscrowContractData, milestones: &Vec<Milestone>) -> ContractStatus {
        let mut all_released = true;
        let mut all_resolved = true;
        let mut any_refunded = false;

        for milestone in milestones.iter() {
            if !milestone.released {
                all_released = false;
            }
            if !milestone.released && !milestone.refunded {
                all_resolved = false;
            }
            if milestone.refunded {
                any_refunded = true;
            }
        }

        if all_released {
            ContractStatus::Completed
        } else if all_resolved && any_refunded {
            ContractStatus::Refunded
        } else if contract.funded_amount == contract.total_amount {
            ContractStatus::Funded
        } else {
            ContractStatus::Created
        }
    }
}

#[cfg(test)]
mod test;
