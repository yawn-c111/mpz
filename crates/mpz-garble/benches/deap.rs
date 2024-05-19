use criterion::{black_box, criterion_group, criterion_main, Criterion};

use mpz_circuits::circuits::AES128;
use mpz_garble::{protocol::deap::mock::create_mock_deap_vm, Decode, Execute, Memory};

async fn bench_deap() {
    let (mut leader_vm, mut follower_vm) = create_mock_deap_vm();

    let key = [0u8; 16];
    let msg = [0u8; 16];

    let leader_fut = {
        let key_ref = leader_vm.new_private_input::<[u8; 16]>("key").unwrap();
        let msg_ref = leader_vm.new_blind_input::<[u8; 16]>("msg").unwrap();
        let ciphertext_ref = leader_vm.new_output::<[u8; 16]>("ciphertext").unwrap();

        leader_vm.assign(&key_ref, key).unwrap();

        async {
            leader_vm
                .execute(
                    AES128.clone(),
                    &[key_ref, msg_ref],
                    &[ciphertext_ref.clone()],
                )
                .await
                .unwrap();

            leader_vm.decode(&[ciphertext_ref]).await.unwrap();

            leader_vm.finalize().await.unwrap();
        }
    };

    let follower_fut = {
        let key_ref = follower_vm.new_blind_input::<[u8; 16]>("key").unwrap();
        let msg_ref = follower_vm.new_private_input::<[u8; 16]>("msg").unwrap();
        let ciphertext_ref = follower_vm.new_output::<[u8; 16]>("ciphertext").unwrap();

        follower_vm.assign(&msg_ref, msg).unwrap();

        async {
            follower_vm
                .execute(
                    AES128.clone(),
                    &[key_ref, msg_ref],
                    &[ciphertext_ref.clone()],
                )
                .await
                .unwrap();

            follower_vm.decode(&[ciphertext_ref]).await.unwrap();

            follower_vm.finalize().await.unwrap();
        }
    };

    _ = futures::join!(leader_fut, follower_fut)
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("deap");

    let rt = tokio::runtime::Runtime::new().unwrap();
    group.bench_function("aes", |b| {
        b.to_async(&rt).iter(|| async {
            bench_deap().await;
            black_box(())
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
