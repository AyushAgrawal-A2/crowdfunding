use {
    anchor_lang::{
        prelude::Pubkey,
        solana_program::{clock::Clock, instruction::Instruction, system_program},
        AccountDeserialize, InstructionData, ToAccountMetas,
    },
    anchor_spl::{associated_token, token},
    crowdfunding::{CampaignStatus, CAMPAIGN_SEED, DONOR_SEED},
    litesvm::LiteSVM,
    litesvm_token::{
        get_spl_account, spl_token::state::Account, CreateAssociatedTokenAccount, CreateMint,
        MintTo,
    },
    solana_keypair::Keypair,
    solana_message::{Message, VersionedMessage},
    solana_signer::Signer,
    solana_transaction::versioned::VersionedTransaction,
};

#[test]
fn test_goal_met() {
    let program_id = crowdfunding::id();
    let mut svm = LiteSVM::new();
    let bytes = include_bytes!(concat!(
        env!("CARGO_TARGET_TMPDIR"),
        "/../deploy/crowdfunding.so"
    ));
    svm.add_program(program_id, bytes).unwrap();
    let maker = Keypair::new();
    svm.airdrop(&maker.pubkey(), 1_000_000_000).unwrap();

    let id = 1u64;
    let goal = 1000_000_000_000u64;
    let current_time = svm.get_sysvar::<Clock>().unix_timestamp;
    let deadline = current_time + 60 * 1000;

    let mint = CreateMint::new(&mut svm, &maker)
        .authority(&maker.pubkey())
        .decimals(9)
        .send()
        .unwrap();
    let (campaign, campaign_bump) = Pubkey::find_program_address(
        &[
            CAMPAIGN_SEED,
            maker.pubkey().as_ref(),
            id.to_le_bytes().as_ref(),
        ],
        &program_id,
    );
    let vault = associated_token::get_associated_token_address_with_program_id(
        &campaign,
        &mint,
        &token::ID,
    );

    let instruction = Instruction::new_with_bytes(
        program_id,
        &crowdfunding::instruction::CreateCampaign { id, goal, deadline }.data(),
        crowdfunding::accounts::CreateCampaign {
            maker: maker.pubkey(),
            mint,
            campaign,
            vault,
            associated_token_program: associated_token::ID,
            token_program: token::ID,
            system_program: system_program::ID,
        }
        .to_account_metas(None),
    );
    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[instruction], Some(&maker.pubkey()), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[&maker]).unwrap();
    svm.send_transaction(tx).unwrap();
    let campaign_account = svm.get_account(&campaign).unwrap();
    let mut data: &[u8] = &campaign_account.data;
    let campaign_state = crowdfunding::state::Campaign::try_deserialize(&mut data).unwrap();
    assert_eq!(campaign_state.maker, maker.pubkey());
    assert_eq!(campaign_state.id, id);
    assert_eq!(campaign_state.mint, mint);
    assert_eq!(campaign_state.goal, goal);
    assert_eq!(campaign_state.deadline, deadline);
    assert_eq!(campaign_state.status, CampaignStatus::Ongoing);
    assert_eq!(campaign_state.bump, campaign_bump);

    let donor = Keypair::new();
    svm.airdrop(&donor.pubkey(), 1_000_000_000).unwrap();
    let (donor_pda, _donor_pda_bump) = Pubkey::find_program_address(
        &[DONOR_SEED, campaign.as_ref(), donor.pubkey().as_ref()],
        &program_id,
    );
    let donor_token_ata = CreateAssociatedTokenAccount::new(&mut svm, &donor, &mint)
        .owner(&donor.pubkey())
        .send()
        .unwrap();
    MintTo::new(&mut svm, &maker, &mint, &donor_token_ata, goal)
        .owner(&maker)
        .send()
        .unwrap();

    let instruction = Instruction::new_with_bytes(
        program_id,
        &crowdfunding::instruction::Contribute { id, amount: goal }.data(),
        crowdfunding::accounts::Contribute {
            donor: donor.pubkey(),
            maker: maker.pubkey(),
            mint,
            campaign,
            vault,
            donor_pda,
            donor_token_ata,
            associated_token_program: associated_token::ID,
            token_program: token::ID,
            system_program: system_program::ID,
        }
        .to_account_metas(None),
    );
    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[instruction], Some(&donor.pubkey()), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[&donor]).unwrap();
    svm.send_transaction(tx).unwrap();
    let vault_account: Account = get_spl_account(&svm, &vault).unwrap();
    let vault_balance = vault_account.amount;
    assert_eq!(vault_balance, goal);

    let mut clock = svm.get_sysvar::<Clock>();
    clock.unix_timestamp = deadline + 1;
    svm.set_sysvar(&clock);

    let instruction = Instruction::new_with_bytes(
        program_id,
        &crowdfunding::instruction::Finalize { id }.data(),
        crowdfunding::accounts::Finalize {
            maker: maker.pubkey(),
            mint,
            campaign,
            vault,
            associated_token_program: associated_token::ID,
            token_program: token::ID,
            system_program: system_program::ID,
        }
        .to_account_metas(None),
    );
    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[instruction], Some(&maker.pubkey()), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[&maker]).unwrap();
    svm.send_transaction(tx).unwrap();
    let campaign_account = svm.get_account(&campaign).unwrap();
    let mut data: &[u8] = &campaign_account.data;
    let campaign_state = crowdfunding::state::Campaign::try_deserialize(&mut data).unwrap();
    assert_eq!(campaign_state.status, CampaignStatus::GoalMet);

    let maker_token_ata = associated_token::get_associated_token_address_with_program_id(
        &maker.pubkey(),
        &mint,
        &token::ID,
    );
    let instruction = Instruction::new_with_bytes(
        program_id,
        &crowdfunding::instruction::Withdraw { id }.data(),
        crowdfunding::accounts::Withdraw {
            maker: maker.pubkey(),
            mint,
            campaign,
            vault,
            maker_token_ata,
            associated_token_program: associated_token::ID,
            token_program: token::ID,
            system_program: system_program::ID,
        }
        .to_account_metas(None),
    );
    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[instruction], Some(&maker.pubkey()), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[&maker]).unwrap();
    svm.send_transaction(tx).unwrap();
    let maker_token_ata_account: Account = get_spl_account(&svm, &maker_token_ata).unwrap();
    let maker_token_ata_balance = maker_token_ata_account.amount;
    assert_eq!(maker_token_ata_balance, goal);
}

#[test]
fn test_goal_not_met() {
    let program_id = crowdfunding::id();
    let mut svm = LiteSVM::new();
    let bytes = include_bytes!(concat!(
        env!("CARGO_TARGET_TMPDIR"),
        "/../deploy/crowdfunding.so"
    ));
    svm.add_program(program_id, bytes).unwrap();
    let maker = Keypair::new();
    svm.airdrop(&maker.pubkey(), 1_000_000_000).unwrap();

    let id = 1u64;
    let goal = 1000_000_000_000u64;
    let current_time = svm.get_sysvar::<Clock>().unix_timestamp;
    let deadline = current_time + 60 * 1000;

    let mint = CreateMint::new(&mut svm, &maker)
        .authority(&maker.pubkey())
        .decimals(9)
        .send()
        .unwrap();
    let (campaign, campaign_bump) = Pubkey::find_program_address(
        &[
            CAMPAIGN_SEED,
            maker.pubkey().as_ref(),
            id.to_le_bytes().as_ref(),
        ],
        &program_id,
    );
    let vault = associated_token::get_associated_token_address_with_program_id(
        &campaign,
        &mint,
        &token::ID,
    );

    let instruction = Instruction::new_with_bytes(
        program_id,
        &crowdfunding::instruction::CreateCampaign { id, goal, deadline }.data(),
        crowdfunding::accounts::CreateCampaign {
            maker: maker.pubkey(),
            mint,
            campaign,
            vault,
            associated_token_program: associated_token::ID,
            token_program: token::ID,
            system_program: system_program::ID,
        }
        .to_account_metas(None),
    );
    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[instruction], Some(&maker.pubkey()), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[&maker]).unwrap();
    svm.send_transaction(tx).unwrap();
    let campaign_account = svm.get_account(&campaign).unwrap();
    let mut data: &[u8] = &campaign_account.data;
    let campaign_state = crowdfunding::state::Campaign::try_deserialize(&mut data).unwrap();
    assert_eq!(campaign_state.maker, maker.pubkey());
    assert_eq!(campaign_state.id, id);
    assert_eq!(campaign_state.mint, mint);
    assert_eq!(campaign_state.goal, goal);
    assert_eq!(campaign_state.deadline, deadline);
    assert_eq!(campaign_state.status, CampaignStatus::Ongoing);
    assert_eq!(campaign_state.bump, campaign_bump);

    let donor = Keypair::new();
    svm.airdrop(&donor.pubkey(), 1_000_000_000).unwrap();
    let amount = 1_000_000_000u64;
    let (donor_pda, _donor_pda_bump) = Pubkey::find_program_address(
        &[DONOR_SEED, campaign.as_ref(), donor.pubkey().as_ref()],
        &program_id,
    );
    let donor_token_ata = CreateAssociatedTokenAccount::new(&mut svm, &donor, &mint)
        .owner(&donor.pubkey())
        .send()
        .unwrap();
    MintTo::new(&mut svm, &maker, &mint, &donor_token_ata, amount)
        .owner(&maker)
        .send()
        .unwrap();

    let instruction = Instruction::new_with_bytes(
        program_id,
        &crowdfunding::instruction::Contribute { id, amount }.data(),
        crowdfunding::accounts::Contribute {
            donor: donor.pubkey(),
            maker: maker.pubkey(),
            mint,
            campaign,
            vault,
            donor_pda,
            donor_token_ata,
            associated_token_program: associated_token::ID,
            token_program: token::ID,
            system_program: system_program::ID,
        }
        .to_account_metas(None),
    );
    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[instruction], Some(&donor.pubkey()), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[&donor]).unwrap();
    svm.send_transaction(tx).unwrap();
    let vault_account: Account = get_spl_account(&svm, &vault).unwrap();
    let vault_balance = vault_account.amount;
    assert_eq!(vault_balance, amount);

    let mut clock = svm.get_sysvar::<Clock>();
    clock.unix_timestamp = deadline + 1;
    svm.set_sysvar(&clock);

    let instruction = Instruction::new_with_bytes(
        program_id,
        &crowdfunding::instruction::Finalize { id }.data(),
        crowdfunding::accounts::Finalize {
            maker: maker.pubkey(),
            mint,
            campaign,
            vault,
            associated_token_program: associated_token::ID,
            token_program: token::ID,
            system_program: system_program::ID,
        }
        .to_account_metas(None),
    );
    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[instruction], Some(&maker.pubkey()), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[&maker]).unwrap();
    svm.send_transaction(tx).unwrap();
    let campaign_account = svm.get_account(&campaign).unwrap();
    let mut data: &[u8] = &campaign_account.data;
    let campaign_state = crowdfunding::state::Campaign::try_deserialize(&mut data).unwrap();
    assert_eq!(campaign_state.status, CampaignStatus::GoalNotMet);

    let instruction = Instruction::new_with_bytes(
        program_id,
        &crowdfunding::instruction::Refund { id }.data(),
        crowdfunding::accounts::Refund {
            donor: donor.pubkey(),
            maker: maker.pubkey(),
            mint,
            campaign,
            vault,
            donor_pda,
            donor_token_ata,
            associated_token_program: associated_token::ID,
            token_program: token::ID,
            system_program: system_program::ID,
        }
        .to_account_metas(None),
    );
    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[instruction], Some(&donor.pubkey()), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[&donor]).unwrap();
    svm.send_transaction(tx).unwrap();
    let donor_token_ata_account: Account = get_spl_account(&svm, &donor_token_ata).unwrap();
    let donor_token_ata_balance = donor_token_ata_account.amount;
    assert_eq!(donor_token_ata_balance, amount);
}
