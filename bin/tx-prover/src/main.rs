pub mod api;
pub mod commands;
mod proxy;
mod utils;
use commands::Cli;
use utils::setup_tracing;

fn main() -> Result<(), String> {
    use clap::Parser;

    setup_tracing();

    // read command-line args
    let cli = Cli::parse();

    // execute cli action
    cli.execute()
}

// TESTS
// ================================================================================================

#[cfg(test)]
mod test {
    use std::time::Duration;

    use miden_lib::transaction::TransactionKernel;
    use miden_objects::{
        accounts::account_id::testing::{ACCOUNT_ID_FUNGIBLE_FAUCET_ON_CHAIN, ACCOUNT_ID_SENDER},
        assets::{Asset, FungibleAsset},
        notes::NoteType,
        testing::account_code::DEFAULT_AUTH_SCRIPT,
        transaction::{ProvenTransaction, TransactionScript, TransactionWitness},
    };
    use miden_tx::{
        testing::mock_chain::{Auth, MockChain},
        utils::Serializable,
    };
    use miden_tx_prover::generated::{
        api_client::ApiClient, api_server::ApiServer, ProveTransactionRequest,
    };
    use tokio::net::TcpListener;
    use tonic::Request;

    use crate::api::ProverRpcApi;

    #[tokio::test(flavor = "multi_thread", worker_threads = 3)]
    async fn test_prove_transaction() {
        // Start the server in the background
        let listener = TcpListener::bind("127.0.0.1:50052").await.unwrap();
        let api_service = ApiServer::new(ProverRpcApi::default());

        // Spawn the server as a background task
        tokio::spawn(async move {
            tonic::transport::Server::builder()
                .accept_http1(true)
                .add_service(tonic_web::enable(api_service))
                .serve_with_incoming(tokio_stream::wrappers::TcpListenerStream::new(listener))
                .await
                .unwrap();
        });

        // Give the server some time to start
        tokio::time::sleep(Duration::from_secs(1)).await;

        // Set up a gRPC client to send the request
        let mut client = ApiClient::connect("http://127.0.0.1:50052").await.unwrap();
        let mut client_2 = ApiClient::connect("http://127.0.0.1:50052").await.unwrap();

        // Create a mock transaction to send to the server
        let mut mock_chain = MockChain::new();
        let account = mock_chain.add_existing_wallet(Auth::BasicAuth, vec![]);

        let fungible_asset_1: Asset =
            FungibleAsset::new(ACCOUNT_ID_FUNGIBLE_FAUCET_ON_CHAIN.try_into().unwrap(), 100)
                .unwrap()
                .into();
        let note_1 = mock_chain
            .add_p2id_note(
                ACCOUNT_ID_SENDER.try_into().unwrap(),
                account.id(),
                &[fungible_asset_1],
                NoteType::Private,
            )
            .unwrap();

        let tx_script =
            TransactionScript::compile(DEFAULT_AUTH_SCRIPT, vec![], TransactionKernel::assembler())
                .unwrap();
        let tx_context = mock_chain
            .build_tx_context(account.id())
            .input_notes(vec![note_1])
            .tx_script(tx_script)
            .build();

        let executed_transaction = tx_context.execute().unwrap();

        let transaction_witness = TransactionWitness::from(executed_transaction);

        let request_1 = Request::new(ProveTransactionRequest {
            transaction_witness: transaction_witness.to_bytes(),
        });

        let request_2 = Request::new(ProveTransactionRequest {
            transaction_witness: transaction_witness.to_bytes(),
        });

        // Send both requests concurrently
        let (t1, t2) = (
            tokio::spawn(async move { client.prove_transaction(request_1).await }),
            tokio::spawn(async move { client_2.prove_transaction(request_2).await }),
        );

        let (response_1, response_2) = (t1.await.unwrap(), t2.await.unwrap());

        // Check the success response
        assert!(response_1.is_ok() || response_2.is_ok());

        // Check the failure response
        assert!(response_1.is_err() || response_2.is_err());

        let response_success = response_1.or(response_2).unwrap();

        // Cast into a ProvenTransaction
        let _proven_transaction: ProvenTransaction =
            response_success.into_inner().try_into().expect("Failed to convert response");
    }
}
