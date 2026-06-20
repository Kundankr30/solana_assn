use anchor_lang::prelude::Pubkey;
use {
    anchor_lang::{solana_program::instruction::Instruction, InstructionData, ToAccountMetas},
    litesvm::LiteSVM,
    solana_keypair::Keypair,
    solana_message::{Message, VersionedMessage},
    solana_signer::Signer,
    solana_transaction::versioned::VersionedTransaction,
};
#[test]
fn test_initialize() {
    let program_id = tip_jar::id();
    let payer = Keypair::new();
    let mut svm = LiteSVM::new();
    let bytes = include_bytes!("../../../target/deploy/tip_jar.so");
    svm.add_program(program_id, bytes).unwrap();
    svm.airdrop(&payer.pubkey(), 1_000_000_000).unwrap();

    let (jar_pda, _bump) =
        Pubkey::find_program_address(&[b"tip_jar", payer.pubkey().as_ref()], &program_id);

    let instruction = Instruction::new_with_bytes(
        program_id,
        &tip_jar::instruction::Initialize {}.data(),
        tip_jar::accounts::Initialize {
            owner: payer.pubkey(),
            jar: jar_pda,
            system_program: anchor_lang::solana_program::system_program::ID,
        }
        .to_account_metas(None),
    );
    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[instruction], Some(&payer.pubkey()), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[payer]).unwrap();
    let res = svm.send_transaction(tx);
    assert!(res.is_ok(), "Transaction failed: {:?}", res.err());
}
