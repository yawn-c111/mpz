use mpz_core::LpnEstimator;

fn main() {
    let sec1 = LpnEstimator::security_for_binary(1 << 10, 652, 57);
    let sec2 = LpnEstimator::security_for_binary(1 << 12, 1589, 98);
    let sec3 = LpnEstimator::security_for_binary(1 << 14, 3482, 198);
    let sec4 = LpnEstimator::security_for_binary(1 << 16, 7391, 389);
    let sec5 = LpnEstimator::security_for_binary(1 << 18, 15336, 760);
    let sec6 = LpnEstimator::security_for_binary(1 << 20, 32771, 1419);
    let sec7 = LpnEstimator::security_for_binary(1 << 22, 67440, 2735);
    println!("lpn_1<<10_652_57 security:\t {} bits", sec1);
    println!("lpn_1<<12_1589_98 security:\t {} bits", sec2);
    println!("lpn_1<<14_3482_198 security:\t {} bits", sec3);
    println!("lpn_1<<16_7391_389 security:\t {} bits", sec4);
    println!("lpn_1<<18_15336_760 security:\t {} bits", sec5);
    println!("lpn_1<<20_32771_1419 security:\t {} bits", sec6);
    println!("lpn_1<<22_67440_2735 security:\t {} bits", sec7);

    let sec1 = LpnEstimator::security_for_binary_regular(1 << 10, 652, 57);
    let sec2 = LpnEstimator::security_for_binary_regular(1 << 12, 1589, 98);
    let sec3 = LpnEstimator::security_for_binary_regular(1 << 14, 3482, 198);
    let sec4 = LpnEstimator::security_for_binary_regular(1 << 16, 7391, 389);
    let sec5 = LpnEstimator::security_for_binary_regular(1 << 18, 15336, 760);
    let sec6 = LpnEstimator::security_for_binary_regular(1 << 20, 32771, 1419);
    let sec7 = LpnEstimator::security_for_binary_regular(1 << 22, 67440, 2735);
    println!("lpn_regular_1<<10_652_57 security:\t {} bits", sec1);
    println!("lpn_regular_1<<12_1589_98 security:\t {} bits", sec2);
    println!("lpn_regular_1<<14_3482_198 security:\t {} bits", sec3);
    println!("lpn_regular_1<<16_7391_389 security:\t {} bits", sec4);
    println!("lpn_regular_1<<18_15336_760 security:\t {} bits", sec5);
    println!("lpn_regular_1<<20_32771_1419 security:\t {} bits", sec6);
    println!("lpn_regular_1<<22_67440_2735 security:\t {} bits", sec7);
}
