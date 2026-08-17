use {
    crate::{
        common::{parse_pubkey, process_transaction},
        config::Config,
        output::{format_output, println_display},
        CommandResult, Error,
    },
    clap::{ArgMatches, Args},
    serde_derive::{Deserialize, Serialize},
    serde_with::{serde_as, DisplayFromStr},
    solana_clap_v3_utils::{
        input_parsers::signer::{SignerSource, SignerSourceParserBuilder},
        keypair::{signer_from_source_with_config, SignerFromPathConfig},
    },
    solana_cli_output::{display::writeln_name_value, QuietDisplay, VerboseDisplay},
    solana_instruction::Instruction,
    solana_pubkey::Pubkey,
    solana_remote_wallet::remote_wallet::RemoteWalletManager,
    solana_signature::Signature,
    solana_signer::Signer,
    solana_system_interface::{instruction::transfer, program as system_program},
    solana_transaction::Transaction,
    spl_token_2022_interface::{extension::StateWithExtensions, state::Mint},
    spl_token_wrap::{
        get_canonical_pointer_address, id, instruction::set_canonical_pointer,
        state::CanonicalDeploymentPointer,
    },
    std::{
        fmt::{Display, Formatter},
        rc::Rc,
        sync::Arc,
    },
};

#[derive(Clone, Debug, Args)]
pub struct SetCanonicalPointerArgs {
    /// The address of the unwrapped mint
    #[clap(value_parser = parse_pubkey)]
    pub unwrapped_mint: Pubkey,

    /// The Token Wrap program deployment to mark as canonical
    #[clap(value_parser = parse_pubkey)]
    pub canonical_program_id: Pubkey,

    /// Signer source for the unwrapped mint authority if different from the fee
    /// payer
    #[clap(
        long,
        value_parser = SignerSourceParserBuilder::default().allow_all().build()
    )]
    pub mint_authority: Option<SignerSource>,
}

#[serde_as]
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetCanonicalPointerOutput {
    #[serde_as(as = "DisplayFromStr")]
    pub unwrapped_mint_address: Pubkey,

    #[serde_as(as = "DisplayFromStr")]
    pub mint_authority: Pubkey,

    #[serde_as(as = "DisplayFromStr")]
    pub canonical_pointer_address: Pubkey,

    #[serde_as(as = "DisplayFromStr")]
    pub canonical_program_id: Pubkey,

    pub funded_canonical_pointer_lamports: u64,

    #[serde_as(as = "Option<DisplayFromStr>")]
    pub signature: Option<Signature>,
}

impl Display for SetCanonicalPointerOutput {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln_name_value(
            f,
            "Unwrapped mint address:",
            &self.unwrapped_mint_address.to_string(),
        )?;
        writeln_name_value(f, "Mint authority:", &self.mint_authority.to_string())?;
        writeln_name_value(
            f,
            "Canonical pointer address:",
            &self.canonical_pointer_address.to_string(),
        )?;
        writeln_name_value(
            f,
            "Canonical program ID:",
            &self.canonical_program_id.to_string(),
        )?;
        writeln_name_value(
            f,
            "Funded canonical pointer lamports:",
            &self.funded_canonical_pointer_lamports.to_string(),
        )?;

        if let Some(signature) = self.signature {
            writeln_name_value(f, "Signature:", &signature.to_string())?;
        }

        Ok(())
    }
}

impl QuietDisplay for SetCanonicalPointerOutput {
    fn write_str(&self, _: &mut dyn std::fmt::Write) -> std::fmt::Result {
        Ok(())
    }
}
impl VerboseDisplay for SetCanonicalPointerOutput {}

pub async fn command_set_canonical_pointer(
    config: &Config,
    args: SetCanonicalPointerArgs,
    matches: &ArgMatches,
    wallet_manager: &mut Option<Rc<RemoteWalletManager>>,
) -> CommandResult {
    let payer = config.fee_payer()?;
    let mint_authority: Arc<dyn Signer> = if let Some(source) = &args.mint_authority {
        Arc::from(
            signer_from_source_with_config(
                matches,
                source,
                "mint_authority",
                wallet_manager,
                &SignerFromPathConfig {
                    allow_null_signer: false,
                },
            )
            .map_err(|error| error.to_string())?,
        )
    } else {
        payer.clone()
    };

    let canonical_pointer_address = get_canonical_pointer_address(&args.unwrapped_mint);
    let accounts = config
        .rpc_client
        .get_multiple_accounts(&[args.unwrapped_mint, canonical_pointer_address])
        .await?;
    if accounts.len() != 2 {
        return Err(format!(
            "RPC returned {} accounts for a two-account request",
            accounts.len()
        )
        .into());
    }
    let mint_account = accounts
        .first()
        .and_then(Option::as_ref)
        .ok_or_else(|| format!("Unwrapped mint {} does not exist", args.unwrapped_mint))?;
    let canonical_pointer_account = accounts.get(1).and_then(Option::as_ref);

    validate_mint_authority(
        &args.unwrapped_mint,
        &mint_account.owner,
        &mint_account.data,
        &mint_authority.pubkey(),
    )?;

    let pointer_space = std::mem::size_of::<CanonicalDeploymentPointer>();
    let pointer_rent = if canonical_pointer_account.is_some_and(|account| !account.data.is_empty())
    {
        0
    } else {
        config
            .rpc_client
            .get_minimum_balance_for_rent_exemption(pointer_space)
            .await?
    };
    let funded_canonical_pointer_lamports = required_pointer_funding(
        &canonical_pointer_address,
        canonical_pointer_account.map(|account| account.owner),
        canonical_pointer_account.map_or(0, |account| account.data.len()),
        canonical_pointer_account.map_or(0, |account| account.lamports),
        pointer_rent,
    )?;

    println_display(
        config,
        format!(
            "Setting canonical Token Wrap deployment for {} to {}",
            args.unwrapped_mint, args.canonical_program_id
        ),
    );

    let instructions = build_instructions(
        &payer.pubkey(),
        &mint_authority.pubkey(),
        &args.unwrapped_mint,
        &canonical_pointer_address,
        &args.canonical_program_id,
        funded_canonical_pointer_lamports,
    );
    let blockhash = config.rpc_client.get_latest_blockhash().await?;
    let mut transaction = Transaction::new_with_payer(&instructions, Some(&payer.pubkey()));

    let mut signers: Vec<&dyn Signer> = vec![payer.as_ref()];
    if mint_authority.pubkey() != payer.pubkey() {
        signers.push(mint_authority.as_ref());
    }
    transaction.try_sign(&signers, blockhash)?;

    let signature = process_transaction(config, transaction).await?;

    Ok(format_output(
        config,
        SetCanonicalPointerOutput {
            unwrapped_mint_address: args.unwrapped_mint,
            mint_authority: mint_authority.pubkey(),
            canonical_pointer_address,
            canonical_program_id: args.canonical_program_id,
            funded_canonical_pointer_lamports,
            signature,
        },
    ))
}

fn validate_mint_authority(
    mint_address: &Pubkey,
    mint_owner: &Pubkey,
    mint_data: &[u8],
    signer: &Pubkey,
) -> Result<(), Error> {
    if mint_owner != &spl_token::id() && mint_owner != &spl_token_2022_interface::id() {
        return Err(format!(
            "Unwrapped mint {mint_address} is not owned by SPL Token or Token-2022; owner is \
             {mint_owner}"
        )
        .into());
    }

    let mint_state = StateWithExtensions::<Mint>::unpack(mint_data)
        .map_err(|error| format!("Failed to decode unwrapped mint {mint_address}: {error}"))?;
    let onchain_authority =
        Option::<Pubkey>::from(mint_state.base.mint_authority).ok_or_else(|| {
            format!("Unwrapped mint {mint_address} has no mint authority and cannot set a pointer")
        })?;

    if onchain_authority != *signer {
        return Err(format!(
            "Mint authority signer {signer} does not match on-chain mint authority \
             {onchain_authority} for {mint_address}"
        )
        .into());
    }

    Ok(())
}

fn required_pointer_funding(
    pointer_address: &Pubkey,
    owner: Option<Pubkey>,
    data_len: usize,
    lamports: u64,
    rent_exempt_lamports: u64,
) -> Result<u64, Error> {
    match (owner, data_len) {
        (None, 0) if lamports == 0 => Ok(rent_exempt_lamports),
        (Some(owner), 0) if owner == system_program::id() => {
            Ok(rent_exempt_lamports.saturating_sub(lamports))
        }
        (Some(owner), data_len) if owner == id() => {
            let expected_len = std::mem::size_of::<CanonicalDeploymentPointer>();
            if data_len != expected_len {
                return Err(format!(
                    "Canonical pointer {pointer_address} has invalid data length {data_len}; \
                     expected {expected_len}"
                )
                .into());
            }
            Ok(0)
        }
        (Some(owner), _) => {
            Err(format!("Canonical pointer {pointer_address} has unexpected owner {owner}").into())
        }
        (None, data_len) => Err(format!(
            "Canonical pointer {pointer_address} is absent but reported {lamports} lamports and \
             data length {data_len}"
        )
        .into()),
    }
}

fn build_instructions(
    payer: &Pubkey,
    mint_authority: &Pubkey,
    unwrapped_mint: &Pubkey,
    canonical_pointer_address: &Pubkey,
    canonical_program_id: &Pubkey,
    funded_canonical_pointer_lamports: u64,
) -> Vec<Instruction> {
    let mut instructions = Vec::with_capacity(2);
    if funded_canonical_pointer_lamports > 0 {
        instructions.push(transfer(
            payer,
            canonical_pointer_address,
            funded_canonical_pointer_lamports,
        ));
    }
    instructions.push(set_canonical_pointer(
        &id(),
        mint_authority,
        canonical_pointer_address,
        unwrapped_mint,
        canonical_program_id,
    ));
    instructions
}

#[cfg(test)]
mod tests {
    use {super::*, solana_program_pack::Pack, spl_token::solana_program::program_option::COption};

    fn mint_data(mint_authority: COption<Pubkey>) -> Vec<u8> {
        let mint = Mint {
            mint_authority,
            supply: 0,
            decimals: 6,
            is_initialized: true,
            freeze_authority: COption::None,
        };
        let mut data = vec![0; Mint::LEN];
        Mint::pack(mint, &mut data).unwrap();
        data
    }

    #[test]
    fn validates_mint_authority_for_both_token_programs() {
        let mint = Pubkey::new_unique();
        let authority = Pubkey::new_unique();
        let data = mint_data(COption::Some(authority));

        validate_mint_authority(&mint, &spl_token::id(), &data, &authority).unwrap();
        validate_mint_authority(&mint, &spl_token_2022_interface::id(), &data, &authority).unwrap();
    }

    #[test]
    fn rejects_wrong_or_missing_mint_authority() {
        let mint = Pubkey::new_unique();
        let authority = Pubkey::new_unique();
        let wrong_authority = Pubkey::new_unique();

        let error = validate_mint_authority(
            &mint,
            &spl_token::id(),
            &mint_data(COption::Some(authority)),
            &wrong_authority,
        )
        .unwrap_err();
        assert!(error
            .to_string()
            .contains("does not match on-chain mint authority"));

        let error = validate_mint_authority(
            &mint,
            &spl_token::id(),
            &mint_data(COption::None),
            &authority,
        )
        .unwrap_err();
        assert!(error.to_string().contains("has no mint authority"));
    }

    #[test]
    fn calculates_only_the_required_pointer_funding() {
        let mint = Pubkey::new_unique();
        let pointer = get_canonical_pointer_address(&mint);
        let rent = 1_000;

        assert_eq!(
            required_pointer_funding(&pointer, None, 0, 0, rent).unwrap(),
            rent
        );
        assert_eq!(
            required_pointer_funding(&pointer, Some(system_program::id()), 0, 400, rent).unwrap(),
            600
        );
        assert_eq!(
            required_pointer_funding(
                &pointer,
                Some(id()),
                std::mem::size_of::<CanonicalDeploymentPointer>(),
                rent,
                0
            )
            .unwrap(),
            0
        );
        assert!(
            required_pointer_funding(&pointer, Some(Pubkey::new_unique()), 0, rent, rent).is_err()
        );
    }

    #[test]
    fn funds_then_sets_pointer_in_one_instruction_sequence() {
        let payer = Pubkey::new_unique();
        let authority = Pubkey::new_unique();
        let mint = Pubkey::new_unique();
        let pointer = get_canonical_pointer_address(&mint);
        let canonical_program = Pubkey::new_unique();

        let instructions =
            build_instructions(&payer, &authority, &mint, &pointer, &canonical_program, 500);

        assert_eq!(instructions.len(), 2);
        assert_eq!(instructions[0], transfer(&payer, &pointer, 500));
        assert_eq!(
            instructions[1],
            set_canonical_pointer(&id(), &authority, &pointer, &mint, &canonical_program)
        );
        assert_eq!(pointer, get_canonical_pointer_address(&mint));

        let instructions =
            build_instructions(&payer, &authority, &mint, &pointer, &canonical_program, 0);
        assert_eq!(instructions.len(), 1);
        assert_eq!(
            instructions[0],
            set_canonical_pointer(&id(), &authority, &pointer, &mint, &canonical_program)
        );
    }

    #[test]
    fn serializes_dry_run_output_using_cli_field_conventions() {
        let output = SetCanonicalPointerOutput {
            unwrapped_mint_address: Pubkey::new_unique(),
            mint_authority: Pubkey::new_unique(),
            canonical_pointer_address: Pubkey::new_unique(),
            canonical_program_id: Pubkey::new_unique(),
            funded_canonical_pointer_lamports: 500,
            signature: None,
        };

        let value = serde_json::to_value(&output).unwrap();
        assert_eq!(
            value["canonicalPointerAddress"],
            output.canonical_pointer_address.to_string()
        );
        assert_eq!(value["fundedCanonicalPointerLamports"], 500);
        assert!(value["signature"].is_null());
    }
}
