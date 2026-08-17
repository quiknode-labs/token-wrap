use {
    crate::common::helpers::{TestEnv, TOKEN_WRAP_CLI_BIN},
    serde_json::Value,
    solana_keypair::{write_keypair_file, Keypair},
    solana_program_pack::Pack,
    solana_pubkey::Pubkey,
    solana_signer::Signer,
    solana_transaction::Transaction,
    spl_token::state::Mint,
    spl_token_wrap::{get_canonical_pointer_address, state::CanonicalDeploymentPointer},
    std::{path::Path, process::Command},
    tempfile::NamedTempFile,
};

pub async fn test_set_canonical_pointer_with_distinct_mint_authority(env: &TestEnv) {
    let mint_authority = Keypair::new();
    assert_ne!(mint_authority.pubkey(), env.payer.pubkey());

    let unwrapped_mint_keypair = Keypair::new();
    let unwrapped_mint = unwrapped_mint_keypair.pubkey();
    let rent = env
        .rpc_client
        .get_minimum_balance_for_rent_exemption(Mint::LEN)
        .await
        .unwrap();
    let create_mint_transaction = Transaction::new_signed_with_payer(
        &[
            solana_system_interface::instruction::create_account(
                &env.payer.pubkey(),
                &unwrapped_mint,
                rent,
                Mint::LEN as u64,
                &spl_token::id(),
            ),
            spl_token::instruction::initialize_mint(
                &spl_token::id(),
                &unwrapped_mint,
                &mint_authority.pubkey(),
                None,
                9,
            )
            .unwrap(),
        ],
        Some(&env.payer.pubkey()),
        &[&env.payer, &unwrapped_mint_keypair],
        env.rpc_client.get_latest_blockhash().await.unwrap(),
    );
    env.rpc_client
        .send_and_confirm_transaction(&create_mint_transaction)
        .await
        .unwrap();

    let mint_authority_file = NamedTempFile::new().unwrap();
    write_keypair_file(&mint_authority, &mint_authority_file).unwrap();

    let initial_canonical_program = spl_token_wrap::id();
    let initial_output = execute_set_canonical_pointer(
        env,
        &unwrapped_mint,
        &initial_canonical_program,
        mint_authority_file.path(),
    );

    let canonical_pointer = get_canonical_pointer_address(&unwrapped_mint);
    assert_eq!(
        initial_output["canonicalPointerAddress"],
        canonical_pointer.to_string()
    );
    assert_eq!(
        initial_output["mintAuthority"],
        mint_authority.pubkey().to_string()
    );
    assert_eq!(
        initial_output["canonicalProgramId"],
        initial_canonical_program.to_string()
    );
    assert!(
        initial_output["fundedCanonicalPointerLamports"]
            .as_u64()
            .unwrap()
            > 0
    );
    assert!(initial_output["signature"].as_str().is_some());

    assert_canonical_pointer(env, &canonical_pointer, &initial_canonical_program).await;

    let updated_canonical_program = Pubkey::new_unique();
    let update_output = execute_set_canonical_pointer(
        env,
        &unwrapped_mint,
        &updated_canonical_program,
        mint_authority_file.path(),
    );
    assert_eq!(update_output["fundedCanonicalPointerLamports"], 0);
    assert_canonical_pointer(env, &canonical_pointer, &updated_canonical_program).await;
}

fn execute_set_canonical_pointer(
    env: &TestEnv,
    unwrapped_mint: &Pubkey,
    canonical_program: &Pubkey,
    mint_authority_path: &Path,
) -> Value {
    let output = Command::new(TOKEN_WRAP_CLI_BIN)
        .args([
            "set-canonical-pointer",
            "-C",
            &env.config_file_path,
            &unwrapped_mint.to_string(),
            &canonical_program.to_string(),
            "--mint-authority",
            mint_authority_path.to_str().unwrap(),
            "--output",
            "json",
        ])
        .output()
        .unwrap();

    if !output.status.success() {
        panic!(
            "set-canonical-pointer failed:\nSTDOUT:\n{}\nSTDERR:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    serde_json::from_slice(&output.stdout).unwrap()
}

async fn assert_canonical_pointer(
    env: &TestEnv,
    pointer_address: &Pubkey,
    expected_program: &Pubkey,
) {
    let account = env.rpc_client.get_account(pointer_address).await.unwrap();
    assert_eq!(account.owner, spl_token_wrap::id());
    assert_eq!(
        account.data.len(),
        std::mem::size_of::<CanonicalDeploymentPointer>()
    );

    let pointer = bytemuck::from_bytes::<CanonicalDeploymentPointer>(&account.data);
    assert_eq!(pointer.program_id, *expected_program);
}
