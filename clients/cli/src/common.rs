use {
    crate::{config::Config, output::println_display, Error},
    clap::ArgMatches,
    solana_clap_v3_utils::keypair::pubkey_from_path,
    solana_client::{
        nonblocking::rpc_client::RpcClient, rpc_response::RpcSimulateTransactionResult,
    },
    solana_presigner::Presigner,
    solana_pubkey::Pubkey,
    solana_signature::Signature,
    solana_transaction::Transaction,
    spl_token_2022_interface::{
        extension::{PodStateWithExtensions, StateWithExtensions},
        pod::PodAccount,
        state::Mint,
    },
    std::str::FromStr,
};

pub fn parse_pubkey(value: &str) -> Result<Pubkey, String> {
    parse_address(value, "pubkey")
}

fn parse_address(path: &str, name: &str) -> Result<Pubkey, String> {
    let mut wallet_manager = None;
    pubkey_from_path(&ArgMatches::default(), path, name, &mut wallet_manager)
        .map_err(|_| format!("Failed to load pubkey {} at {}", name, path))
}

pub fn parse_token_program(value: &str) -> Result<Pubkey, String> {
    let pubkey = parse_pubkey(value)?;
    if pubkey == spl_token::id() || pubkey == spl_token_2022_interface::id() {
        Ok(pubkey)
    } else {
        Err("Invalid token program. Must be spl-token or spl-token-2022".to_string())
    }
}

pub fn parse_presigner(value: &str) -> Result<Presigner, String> {
    let (pubkey_string, sig_string) = value
        .split_once('=')
        .ok_or("failed to split `pubkey=signature` pair")?;
    let pubkey = Pubkey::from_str(pubkey_string)
        .map_err(|_| "Failed to parse pubkey from string".to_string())?;
    let sig = Signature::from_str(sig_string)
        .map_err(|_| "Failed to parse signature from string".to_string())?;
    Ok(Presigner::new(&pubkey, &sig))
}

pub async fn process_transaction(
    config: &Config,
    transaction: Transaction,
) -> Result<Option<Signature>, Error> {
    if config.dry_run {
        let simulation_data = config.rpc_client.simulate_transaction(&transaction).await?;
        ensure_simulation_succeeded(&simulation_data.value)?;

        if config.verbose() {
            if let Some(logs) = simulation_data.value.logs {
                for log in logs {
                    println!("    {}", log);
                }
            }

            if let Some(units_consumed) = simulation_data.value.units_consumed {
                println!(
                    "\nSimulation succeeded, consumed {} compute units",
                    units_consumed
                );
            } else {
                println!("\nSimulation succeeded");
            }
        } else {
            println_display(config, "Simulation succeeded".to_string());
        }

        Ok(None)
    } else {
        Ok(Some(
            config
                .rpc_client
                .send_and_confirm_transaction_with_spinner(&transaction)
                .await?,
        ))
    }
}

fn ensure_simulation_succeeded(simulation: &RpcSimulateTransactionResult) -> Result<(), Error> {
    if let Some(error) = &simulation.err {
        let mut message = format!("Transaction simulation failed: {error}");
        if let Some(logs) = &simulation.logs {
            if !logs.is_empty() {
                message.push_str("\nSimulation logs:\n    ");
                message.push_str(&logs.join("\n    "));
            }
        }
        return Err(message.into());
    }

    Ok(())
}

pub async fn get_mint_for_token_account(
    rpc_client: &RpcClient,
    token_account_address: &Pubkey,
) -> Result<Pubkey, Error> {
    let token_account_info = rpc_client.get_account(token_account_address).await?;
    let unpacked_account = PodStateWithExtensions::<PodAccount>::unpack(&token_account_info.data)?;
    Ok(unpacked_account.base.mint)
}

pub async fn get_account_owner(rpc_client: &RpcClient, account: &Pubkey) -> Result<Pubkey, Error> {
    let owner = rpc_client.get_account(account).await?.owner;
    Ok(owner)
}

pub async fn assert_mint_account(
    rpc_client: &RpcClient,
    account_key: &Pubkey,
) -> Result<(), String> {
    let account_info = rpc_client
        .get_account(account_key)
        .await
        .map_err(|e| format!("Failed to fetch account {}: {}", account_key, e))?;

    let owner = account_info.owner;
    if owner != spl_token::id() && owner != spl_token_2022_interface::id() {
        return Err(format!(
            "Account {} is not owned by a token program. Owner: {}",
            account_key, owner
        ));
    }

    // Attempt to deserialize the data as a mint account
    let _ = StateWithExtensions::<Mint>::unpack(&account_info.data)
        .map_err(|e| format!("Failed to unpack as spl token mint: {:?}", e))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use {
        super::ensure_simulation_succeeded,
        solana_client::rpc_response::RpcSimulateTransactionResult,
    };

    #[test]
    fn rejects_failed_simulation_with_rpc_logs() {
        let simulation: RpcSimulateTransactionResult = serde_json::from_value(serde_json::json!({
            "err": {"InstructionError": [0, {"Custom": 1}]},
            "logs": ["Program log: Instruction: Wrap", "Program log: Error: insufficient funds"]
        }))
        .unwrap();

        let error = ensure_simulation_succeeded(&simulation)
            .unwrap_err()
            .to_string();
        assert!(error.contains("Transaction simulation failed"));
        assert!(error.contains("custom program error: 0x1"));
        assert!(error.contains("Program log: Error: insufficient funds"));
    }

    #[test]
    fn accepts_successful_simulation_without_optional_metrics() {
        let simulation: RpcSimulateTransactionResult = serde_json::from_value(serde_json::json!({
            "err": null
        }))
        .unwrap();

        ensure_simulation_succeeded(&simulation).unwrap();
    }
}
