use std::str::FromStr;
use std::time::Duration;

use jito_protos::searcher::searcher_service_client::SearcherServiceClient;
use jito_protos::searcher::{
    NextScheduledLeaderRequest, SubscribeBundleResultsRequest,
};
use jito_searcher_client::send_bundle_with_confirmation;
use jito_searcher_client::token_authenticator::ClientInterceptor;
use log::info;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Keypair;
use solana_sdk::signer::Signer;
use solana_sdk::system_instruction::transfer;
use solana_sdk::transaction::Transaction;
use solana_sdk::{instruction::Instruction, transaction::VersionedTransaction};
use tonic::{codegen::InterceptedService, transport::Channel};

use crate::constants;

pub async fn send_swap_tx(
    ixs: Vec<Instruction>,
    tip: u64,
    payer: &Keypair,
    searcher_client: &mut SearcherServiceClient<
        InterceptedService<Channel, ClientInterceptor>,
    >,
    rpc_client: &RpcClient,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut bundle_results_subscription = searcher_client
        .subscribe_bundle_results(SubscribeBundleResultsRequest {})
        .await
        .expect("subscribe to bundle results")
        .into_inner();
    let mut is_leader_slot = false;
    while !is_leader_slot {
        let next_leader = searcher_client
            .get_next_scheduled_leader(NextScheduledLeaderRequest {
                regions: vec![],
            })
            .await
            .expect("gets next scheduled leader")
            .into_inner();
        let num_slots = next_leader.next_leader_slot - next_leader.current_slot;
        is_leader_slot = num_slots <= 2;
        info!(
            "next jito leader slot in {num_slots} slots in {}",
            next_leader.next_leader_region
        );
        tokio::time::sleep(Duration::from_millis(200)).await;
    }
    // build + sign the transactions
    let blockhash = rpc_client
        .get_latest_blockhash()
        .await
        .expect("get blockhash");

    let tip_tx =
        VersionedTransaction::from(Transaction::new_signed_with_payer(
            &[transfer(
                &payer.pubkey(),
                &Pubkey::from_str(constants::JITO_TIP_PUBKEY)?,
                tip,
            )],
            Some(&payer.pubkey()),
            &[payer],
            blockhash,
        ));

    let swap_tx =
        VersionedTransaction::from(Transaction::new_signed_with_payer(
            ixs.as_slice(),
            Some(&payer.pubkey()),
            &[payer],
            blockhash,
        ));

    send_bundle_with_confirmation(
        &[swap_tx, tip_tx],
        rpc_client,
        searcher_client,
        &mut bundle_results_subscription,
    )
    .await
}
