//! Mocked DEAP VMs for testing

use mpz_common::executor::{test_st_executor, STExecutor};
use mpz_core::Block;
use mpz_ot::ideal::ot::{ideal_ot, IdealOTReceiver, IdealOTSender};
use serio::channel::MemoryDuplex;

use crate::{
    config::Role,
    protocol::deap::vm::{DEAPThread, DEAPVm},
};

type OTSender = IdealOTSender<[Block; 2]>;
type OTReceiver = IdealOTReceiver<Block>;
type Ctx = STExecutor<MemoryDuplex>;

/// Mock DEAP Leader VM.
pub type MockLeader = DEAPVm<Ctx, OTSender, OTReceiver>;
/// Mock DEAP Leader thread.
pub type MockLeaderThread = DEAPThread<Ctx, OTSender, OTReceiver>;
/// Mock DEAP Follower VM.
pub type MockFollower = DEAPVm<Ctx, OTSender, OTReceiver>;
/// Mock DEAP Follower thread.
pub type MockFollowerThread = DEAPThread<Ctx, OTSender, OTReceiver>;

/// Create a pair of mocked DEAP VMs
pub fn create_mock_deap_vm() -> (MockLeader, MockFollower) {
    let (leader_ctx, follower_ctx) = test_st_executor(128);
    let (leader_ot_send, follower_ot_recv) = ideal_ot();
    let (follower_ot_send, leader_ot_recv) = ideal_ot();

    let leader = DEAPVm::new(
        Role::Leader,
        [42u8; 32],
        leader_ctx,
        leader_ot_send,
        leader_ot_recv,
    );

    let follower = DEAPVm::new(
        Role::Follower,
        [69u8; 32],
        follower_ctx,
        follower_ot_send,
        follower_ot_recv,
    );

    (leader, follower)
}
