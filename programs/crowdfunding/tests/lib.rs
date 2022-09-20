#![cfg(feature = "test-bpf")]
#![feature(assert_matches)]

use solana_program_test::tokio;

mod api;
mod ctx;
mod scenario;
mod test_instructions;
mod utils;

#[tokio::test]
async fn scenario_test() {
    scenario::scenario_test().await;
}

#[tokio::test]
async fn test_instructions() {
    test_instructions::test_instructions().await;
}
