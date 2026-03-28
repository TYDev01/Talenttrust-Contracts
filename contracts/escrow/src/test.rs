extern crate std;

use soroban_sdk::{testutils::Address as _, vec, Address, Env, Vec};

use crate::{ContractStatus, Escrow, EscrowClient, EscrowContractData, Milestone};

#[path = "create_contract.rs"]
mod create_contract;
#[path = "deposit.rs"]
mod deposit;
#[path = "refund.rs"]
mod refund;
#[path = "release.rs"]
mod release;

fn setup() -> (Env, Address, Address) {
    let env = Env::default();
    env.mock_all_auths();

    let client_addr = Address::generate(&env);
    let freelancer_addr = Address::generate(&env);

    (env, client_addr, freelancer_addr)
}

fn create_client(env: &Env) -> EscrowClient<'_> {
    let contract_id = env.register(Escrow, ());
    EscrowClient::new(env, &contract_id)
}

fn create_default_contract(
    env: &Env,
    client: &EscrowClient<'_>,
    client_addr: &Address,
    freelancer_addr: &Address,
) -> u32 {
    let milestones = vec![env, 200_0000000_i128, 400_0000000_i128, 600_0000000_i128];
    client.create_contract(client_addr, freelancer_addr, &milestones)
}

fn assert_contract_state(
    contract: EscrowContractData,
    expected_status: ContractStatus,
    expected_funded: i128,
    expected_released: i128,
    expected_refunded: i128,
) {
    assert_eq!(contract.status, expected_status);
    assert_eq!(contract.funded_amount, expected_funded);
    assert_eq!(contract.released_amount, expected_released);
    assert_eq!(contract.refunded_amount, expected_refunded);
}

fn assert_milestone_flags(
    milestones: Vec<Milestone>,
    milestone_id: u32,
    expected_released: bool,
    expected_refunded: bool,
) {
    let milestone = milestones.get(milestone_id).unwrap();
    assert_eq!(milestone.released, expected_released);
    assert_eq!(milestone.refunded, expected_refunded);
}
