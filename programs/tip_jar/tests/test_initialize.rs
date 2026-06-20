use anchor_lang::prelude::Pubkey;
use anchor_lang::{AccountDeserialize, InstructionData, ToAccountMetas};
use litesvm::LiteSVM;
use solana_keypair::Keypair;
use solana_message::{Message, VersionedMessage};
use solana_signer::Signer;
use solana_transaction::versioned::VersionedTransaction;
use {anchor_lang::solana_program::instruction::Instruction, tip_jar::state::TipJar};

const STARTING_BALANCE: u64 = 1_000_000_000;

fn new_svm() -> LiteSVM {
    let mut svm = LiteSVM::new();
    let program = include_bytes!("../../../target/deploy/tip_jar.so");
    svm.add_program(tip_jar::id(), program).unwrap();
    svm
}

fn jar_address(owner: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[b"tip_jar", owner.as_ref()], &tip_jar::id())
}

fn send_instruction(
    svm: &mut LiteSVM,
    instruction: Instruction,
    payer: &Keypair,
    additional_signers: &[&Keypair],
) -> bool {
    let message = Message::new_with_blockhash(
        &[instruction],
        Some(&payer.pubkey()),
        &svm.latest_blockhash(),
    );
    let mut signers = vec![payer];
    signers.extend_from_slice(additional_signers);
    let transaction =
        VersionedTransaction::try_new(VersionedMessage::Legacy(message), &signers).unwrap();
    svm.send_transaction(transaction).is_ok()
}

fn initialize(svm: &mut LiteSVM, owner: &Keypair) -> Pubkey {
    let (jar, _) = jar_address(&owner.pubkey());
    let instruction = Instruction::new_with_bytes(
        tip_jar::id(),
        &tip_jar::instruction::Initialize {}.data(),
        tip_jar::accounts::Initialize {
            owner: owner.pubkey(),
            jar,
            system_program: anchor_lang::solana_program::system_program::ID,
        }
        .to_account_metas(None),
    );
    assert!(send_instruction(svm, instruction, owner, &[]));
    jar
}

fn read_jar(svm: &LiteSVM, jar: &Pubkey) -> TipJar {
    let account = svm.get_account(jar).expect("tip jar account should exist");
    TipJar::try_deserialize(&mut account.data.as_slice()).expect("tip jar data should deserialize")
}

fn set_total_tips(svm: &mut LiteSVM, jar: Pubkey, total_tips: u64) {
    let mut account = svm.get_account(&jar).expect("tip jar account should exist");
    // Account layout: 8-byte discriminator, 32-byte owner, then total_tips.
    account.data[40..48].copy_from_slice(&total_tips.to_le_bytes());
    svm.set_account(jar, account).unwrap();
}

fn tip(svm: &mut LiteSVM, tipper: &Keypair, jar: Pubkey, amount: u64) -> bool {
    let instruction = Instruction::new_with_bytes(
        tip_jar::id(),
        &tip_jar::instruction::Tip { tip_amount: amount }.data(),
        tip_jar::accounts::Tip {
            tipper: tipper.pubkey(),
            jar,
            system_program: anchor_lang::solana_program::system_program::ID,
        }
        .to_account_metas(None),
    );
    send_instruction(svm, instruction, tipper, &[])
}

fn withdraw(svm: &mut LiteSVM, signer: &Keypair, jar: Pubkey, amount: u64) -> bool {
    let instruction = Instruction::new_with_bytes(
        tip_jar::id(),
        &tip_jar::instruction::Withdraw {
            withdraw_amount: amount,
        }
        .data(),
        tip_jar::accounts::Withdraw {
            owner: signer.pubkey(),
            jar,
        }
        .to_account_metas(None),
    );
    send_instruction(svm, instruction, signer, &[])
}

#[test]
fn initialize_stores_owner_zero_total_and_canonical_bump() {
    let mut svm = new_svm();
    let owner = Keypair::new();
    svm.airdrop(&owner.pubkey(), STARTING_BALANCE).unwrap();

    let jar = initialize(&mut svm, &owner);
    let (_, expected_bump) = jar_address(&owner.pubkey());
    let state = read_jar(&svm, &jar);

    assert_eq!(state.owner, owner.pubkey());
    assert_eq!(state.total_tips, 0);
    assert_eq!(state.bump, expected_bump);
    assert_eq!(svm.get_account(&jar).unwrap().owner, tip_jar::id());
}

#[test]
fn initializing_the_same_jar_twice_fails() {
    let mut svm = new_svm();
    let owner = Keypair::new();
    svm.airdrop(&owner.pubkey(), STARTING_BALANCE).unwrap();
    let jar = initialize(&mut svm, &owner);

    let instruction = Instruction::new_with_bytes(
        tip_jar::id(),
        &tip_jar::instruction::Initialize {}.data(),
        tip_jar::accounts::Initialize {
            owner: owner.pubkey(),
            jar,
            system_program: anchor_lang::solana_program::system_program::ID,
        }
        .to_account_metas(None),
    );

    assert!(!send_instruction(&mut svm, instruction, &owner, &[]));
}

#[test]
fn anyone_can_tip_and_each_tip_updates_balance_and_total() {
    let mut svm = new_svm();
    let owner = Keypair::new();
    let first_tipper = Keypair::new();
    let second_tipper = Keypair::new();
    for signer in [&owner, &first_tipper, &second_tipper] {
        svm.airdrop(&signer.pubkey(), STARTING_BALANCE).unwrap();
    }
    let jar = initialize(&mut svm, &owner);
    let initial_jar_balance = svm.get_balance(&jar).unwrap();

    assert!(tip(&mut svm, &first_tipper, jar, 25_000));
    assert!(tip(&mut svm, &second_tipper, jar, 75_000));

    assert_eq!(svm.get_balance(&jar), Some(initial_jar_balance + 100_000));
    assert_eq!(read_jar(&svm, &jar).total_tips, 100_000);
}

#[test]
fn owner_can_withdraw_without_changing_lifetime_tip_total() {
    let mut svm = new_svm();
    let owner = Keypair::new();
    let tipper = Keypair::new();
    for signer in [&owner, &tipper] {
        svm.airdrop(&signer.pubkey(), STARTING_BALANCE).unwrap();
    }
    let jar = initialize(&mut svm, &owner);
    assert!(tip(&mut svm, &tipper, jar, 100_000));
    let balance_before = svm.get_balance(&jar).unwrap();

    assert!(withdraw(&mut svm, &owner, jar, 40_000));

    assert_eq!(svm.get_balance(&jar), Some(balance_before - 40_000));
    assert_eq!(read_jar(&svm, &jar).total_tips, 100_000);
}

#[test]
fn non_owner_cannot_withdraw() {
    let mut svm = new_svm();
    let owner = Keypair::new();
    let attacker = Keypair::new();
    let tipper = Keypair::new();
    for signer in [&owner, &attacker, &tipper] {
        svm.airdrop(&signer.pubkey(), STARTING_BALANCE).unwrap();
    }
    let jar = initialize(&mut svm, &owner);
    assert!(tip(&mut svm, &tipper, jar, 100_000));
    let balance_before = svm.get_balance(&jar).unwrap();

    assert!(!withdraw(&mut svm, &attacker, jar, 40_000));

    assert_eq!(svm.get_balance(&jar), Some(balance_before));
    assert_eq!(read_jar(&svm, &jar).total_tips, 100_000);
}

#[test]
fn zero_and_excessive_withdrawals_fail_without_changing_the_jar() {
    let mut svm = new_svm();
    let owner = Keypair::new();
    svm.airdrop(&owner.pubkey(), STARTING_BALANCE).unwrap();
    let jar = initialize(&mut svm, &owner);
    let balance_before = svm.get_balance(&jar).unwrap();

    assert!(!withdraw(&mut svm, &owner, jar, 0));
    assert!(!withdraw(
        &mut svm,
        &owner,
        jar,
        balance_before.saturating_add(1)
    ));

    assert_eq!(svm.get_balance(&jar), Some(balance_before));
    assert_eq!(read_jar(&svm, &jar).total_tips, 0);
}
