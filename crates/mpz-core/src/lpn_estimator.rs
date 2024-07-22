//! An estimator to analyse the security of different LPN parameters.
//! The implementation is according to https://eprint.iacr.org/2022/712.pdf.

#[cfg(feature = "rayon")]
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use rug::Float;

// The precision.
const PRECISION: u32 = 200;

// The highest security level.
const HIGHEST_SECURITY: usize = 256;

/// Define the struct of LPN estimator.
pub struct LpnEstimator;

impl LpnEstimator {
    // Compute the logrithm of combination number of n choose m.
    fn cal_comb_log2(n: u64, m: u64) -> f64 {
        assert!(n >= m);
        let mut res = 0.0;
        let range = std::cmp::min(m, n - m);
        for i in 0..range {
            res += ((n - i) as f64).log2() - ((range - i) as f64).log2();
        }
        res
    }

    /// Compute the bit security under the Pooled Gauss attack, see page 37 in this [paper](https://eprint.iacr.org/2022/712.pdf). Note that it is the same for binary and larger fields.
    ///
    /// # Arguments.
    ///
    /// * `n` - The number of samples.
    /// * `k` - The length of the secret.
    /// * `t` - The Hamming weight of the error.
    ///
    /// NOTE: Run it in the release mode.
    pub fn security_under_pooled_gauss(n: u64, k: u64, t: u64) -> f64 {
        let log_guess_prob = Self::cal_comb_log2(n - k, t) - Self::cal_comb_log2(n, t);
        let matrix_inversion_cost = (std::cmp::min(n - k, k) as f64).powf(2.8);
        matrix_inversion_cost.log2() - log_guess_prob
    }

    // Compute the fomulas inside the min function of Theorem 14 in this [paper](https://eprint.iacr.org/2022/712.pdf).
    fn sub_sd_isd_binary(n: u64, k: u64, t: u64, l: u64, p: u64) -> f64 {
        let log_l_zero = Self::cal_comb_log2((k + l) / 2 + 1, p / 2);
        let l_zero = 2.0_f64.powf(log_l_zero);
        let log_s = 2.0 * log_l_zero - (l as f64);

        // Quick break, the cost should be larger than log_l_zero and log_s.
        if log_l_zero > HIGHEST_SECURITY as f64 || log_s > HIGHEST_SECURITY as f64 {
            return HIGHEST_SECURITY as f64;
        }

        // Computes s = 1 << log_s.
        let s = 2.0_f64.powf(log_s);

        if n - k - l < 1 {
            return u32::MAX as f64;
        }

        // Compute T_Gauss + 2 * L0 * N + 2 * |S| * N
        let mut cost = ((n - k - l) * (n - k)) as f64 / ((n - k - l) as f64).log2();
        cost += 2.0 * l_zero + 2.0 * s;
        cost *= n as f64;

        let mut log_p = Self::cal_comb_log2(n - k - l, t - p) - Self::cal_comb_log2(n, t);

        log_p += l as f64 + log_s;

        // Quick break, the probability should be smaller than 1.
        if log_p >= 0.0 {
            return HIGHEST_SECURITY as f64;
        }

        cost.log2() - log_p
    }

    // Minimize sub_sd_isd_binary when fix p.
    fn min_sub_sd_isd_binary_with_fixed_p(n: u64, k: u64, t: u64, p: u64) -> f64 {
        let mut start = 1;
        let mut end = n - k - 8;

        let t_start = Self::sub_sd_isd_binary(n, k, t, start, p);
        let mut res = t_start;

        while end - start > 30 {
            let mid_left = (end - start) / 3 + start;
            let mid_right = end - (end - start) / 3;

            let left = Self::sub_sd_isd_binary(n, k, t, mid_left, p);
            let right = Self::sub_sd_isd_binary(n, k, t, mid_right, p);

            if left > right {
                start = mid_left;
                res = right;
            } else {
                end = mid_right;
                res = left;
            }
        }

        if start < 10 {
            start = 0;
        } else {
            start = start - 10;
        }

        for l in start..=end + 10 {
            let t = Self::sub_sd_isd_binary(n, k, t, l, p);
            if t <= res {
                res = t;
            }
        }

        res
    }

    /// The security of the lpn parameters under SD_ISD attack for binary field. See Therem 14 in this [paper](https://eprint.iacr.org/2022/712.pdf).
    ///
    /// # Arguments.
    ///
    /// * `n` - The number of samples.
    /// * `k` - The length of the secret.
    /// * `t` - The Hamming weight of the error.
    ///
    /// NOTE: Run it in the release mode.
    pub fn security_under_sd_isd_binary(n: u64, k: u64, t: u64) -> f64 {
        let mut res = Self::sub_sd_isd_binary(n, k, t, 0, 0);

        for p in 0..t / 2 {
            let min = Self::min_sub_sd_isd_binary_with_fixed_p(n, k, t, p);
            if min <= res {
                res = min;
            }
            if min > res + 30.0 {
                break;
            }
        }
        res
    }

    // Compute the fomulas inside the min function of Theorem 16 in this [paper](https://eprint.iacr.org/2022/712.pdf).
    #[allow(clippy::too_many_arguments)]
    fn sub_bjmm_isd_binary(
        n: u64,
        k: u64,
        t: u64,
        p2: u64,
        l: u64,
        e1: u64,
        e2: u64,
        r1: u64,
        r2: u64,
        local_min: f64,
    ) -> f64 {
        assert!(p2 >= e2);
        let p1 = 2 * (p2 - e2);
        assert!(p1 >= e1);
        let p = 2 * (p1 - e1);

        let log_s3 = Self::cal_comb_log2((k + 1) / 2 + 1, p2 / 2);
        let s3 = 2.0_f64.powf(log_s3);

        let log_c3 = log_s3 * 2.0 - r2 as f64;
        let c3 = 2.0_f64.powf(log_c3);

        let log_c2 = log_c3 * 2.0 - r1 as f64;
        let c2 = 2.0_f64.powf(log_c2);

        assert!(k + l >= p2);
        assert!(p2 >= e2);

        let log_mu2 = Self::cal_comb_log2(p2, e2) + Self::cal_comb_log2(k + l - p2, p2 - e2)
            - Self::cal_comb_log2(k + l, p2);

        let log_s1_left = log_mu2 + log_c2;
        let log_s1_right = Self::cal_comb_log2(k + l, p1) - (r1 + r2) as f64;

        let log_s1 = log_s1_left.min(log_s1_right);

        let log_c1 = log_s1 * 2.0 - (l - r1 - r2) as f64;
        let c1 = 2.0_f64.powf(log_c1);

        assert!(k + l >= p1);
        assert!(p1 >= e1);

        let log_mu1 = Self::cal_comb_log2(p1, e1) + Self::cal_comb_log2(k + l - p1, p1 - e1)
            - Self::cal_comb_log2(k + l, p1);

        let log_s_left = log_mu1 + log_c1;
        let log_s_right = Self::cal_comb_log2(k + l, p) - l as f64;

        let log_s = log_s_left.min(log_s_right);

        // Quick break
        if (log_s3 > local_min) || (log_c3 > local_min) || (log_c2 > local_min) {
            return HIGHEST_SECURITY as f64;
        }

        // Compute T_Gauss + (8*S3 + 4*C3 + 2*C2 + 2*C1) * N
        let mut cost = ((n - k - l) * (n - k)) as f64 / ((n - k - l) as f64).log2();
        cost += 8.0 * s3 + 4.0 * c3 + 2.0 * c2 + 2.0 * c1;
        cost *= n as f64;

        // Compute log P(p,l).
        assert!(n >= k + l);
        assert!(t >= p);

        let mut log_p = Self::cal_comb_log2(n - k - l, t - p) - Self::cal_comb_log2(n, t);

        log_p += l as f64 + log_s;

        // Quick break, the probability should be smaller than 1.
        if log_p >= 0.0 {
            return HIGHEST_SECURITY as f64;
        }

        cost.log2() - log_p
    }

    // Minimize sub_bjmm_isd_binary with fixed p2 and l.
    fn min_sub_bjmm_isd_binary_with_fixed_p2_and_l(n: u64, k: u64, t: u64, p2: u64, l: u64) -> f64 {
        let mut local_min = HIGHEST_SECURITY as f64;
        // Try all possible values of e2.
        for e2 in 0..p2 {
            let p1 = 2 * (p2 - e2);

            let e1_start = if p1 <= (t / 2) { 0 } else { p1 - t / 2 };

            // Try all possible values of e1.
            for e1 in e1_start..p1 {
                let p = 2 * (p1 - e1);

                // The following part is according to Equation (6), Page 11 in this [paper](https://eprint.iacr.org/2013/162.pdf).
                assert!(k + l >= p2);
                assert!(k + l >= p1);
                assert!(p2 >= e2);
                assert!(p1 >= e1);
                // Proposition 1 in this [paper](https://eprint.iacr.org/2013/162.pdf).
                let log_mu2 = Self::cal_comb_log2(p2, e2)
                    + Self::cal_comb_log2(k + l - p2, p2 - e2)
                    - Self::cal_comb_log2(k + l, p2);

                let r2 = log_mu2 + 4.0 * (Self::cal_comb_log2((k + l) / 2, p2 / 2))
                    - Self::cal_comb_log2(k + l, p1);

                let r2 = if r2 <= 0.0 {
                    0
                } else if r2 as u64 >= l {
                    l - 1
                } else {
                    r2 as u64
                };

                let r = Self::cal_comb_log2(p1, e1)
                    + Self::cal_comb_log2(k + l - p1, p1 - e1)
                    + Self::cal_comb_log2(k + l, p1)
                    - Self::cal_comb_log2(k + l, p);

                let r1 = r - r2 as f64;
                let r1 = if r1 <= 0.0 {
                    0
                } else if (r1 as u64 + r2) >= l {
                    l - r2 - 1
                } else {
                    r1 as u64
                };

                let middle = Self::sub_bjmm_isd_binary(n, k, t, p2, l, e1, e2, r1, r2, local_min);

                if middle < local_min {
                    local_min = middle;
                }
            }
        }
        local_min
    }

    // Minimize sub_bjmm_isd_binary with fixed p2.
    fn min_sub_bjmm_isd_binary_with_fixed_p2(n: u64, k: u64, t: u64, p2: u64) -> f64 {
        let mut start = 1;
        let mut end = (n - k - 2) / 8;

        let mut min_cost = Self::min_sub_bjmm_isd_binary_with_fixed_p2_and_l(n, k, t, p2, start);

        while end - start > 10 {
            let left = (end - start) / 3 + start;
            let right = end - (end - start) / 3;

            let min_left = Self::min_sub_bjmm_isd_binary_with_fixed_p2_and_l(n, k, t, p2, left);
            let min_right = Self::min_sub_bjmm_isd_binary_with_fixed_p2_and_l(n, k, t, p2, right);

            if min_left > min_right {
                start = left;
                min_cost = min_right;
            } else {
                end = right;
                min_cost = min_left;
            }
        }

        if start < 5 {
            start = 0;
        } else {
            start = start - 5;
        }
        for l in start..end + 5 {
            let min_middle = Self::min_sub_bjmm_isd_binary_with_fixed_p2_and_l(n, k, t, p2, l);
            if min_middle < min_cost {
                min_cost = min_middle;
            }
        }
        min_cost
    }

    /// The security of the lpn parameters under BJMM_ISD attack for binary field. See Therem 16 in this [paper](https://eprint.iacr.org/2022/712.pdf).
    ///
    /// # Arguments.
    ///
    /// * `n` - The number of samples.
    /// * `k` - The length of the secret.
    /// * `t` - The Hamming weight of the error.
    ///
    /// NOTE: Run it in the release mode.
    pub fn security_under_bjmm_isd_binary(n: u64, k: u64, t: u64) -> f64 {
        let mut res = HIGHEST_SECURITY as f64;
        for p2 in (0..t).step_by(2) {
            let min = Self::min_sub_bjmm_isd_binary_with_fixed_p2(n, k, t, p2);
            if min < res {
                res = min;
            }
            if min > res + 8.0 {
                break;
            }
        }
        res
    }

    /// The security of the lpn parameters under SD attack for binary field. See The equation with s = 0 in page 39 in this [paper](https://eprint.iacr.org/2022/712.pdf).
    ///
    /// # Arguments.
    ///
    /// * `n` - The number of samples.
    /// * `k` - The length of the secret.
    /// * `t` - The Hamming weight of the error.
    ///
    /// NOTE: Run it in the release mode.
    pub fn security_under_sd_binary(n: u64, k: u64, t: u64) -> f64 {
        let cost = (n - t + 1) as f64 / (n - k - t) as f64;

        let cost = cost.log2() * 2.0 * t as f64 + 2 as f64;

        ((k + 1) as f64).log2() + cost
    }

    /// The security of the lpn parameters under SD 2.0 attack for binary field. See The equation with in page 39 in this [paper](https://eprint.iacr.org/2022/712.pdf).
    ///
    /// # Arguments.
    ///
    /// * `n` - The number of samples.
    /// * `k` - The length of the secret.
    /// * `t` - The Hamming weight of the error.
    ///
    /// NOTE: Run it in the release mode.
    pub fn security_under_sd2_binary(n: u64, k: u64, t: u64) -> f64 {
        let s = Self::security_under_pooled_gauss(n, k, t) as u64;
        Self::security_under_sd_binary(n, k - s, t)
    }

    /// The security of the exact lpn parameters for binary field.
    /// # Arguments.
    ///
    /// * `n` - The number of samples.
    /// * `k` - The length of the secret.
    /// * `t` - The Hamming weight of the error.
    ///
    /// NOTE: Run it in the release mode.
    pub fn security_for_binary(n: u64, k: u64, t: u64) -> f64 {
        let funcs: Vec<fn(u64, u64, u64) -> f64> = vec![
            Self::security_under_pooled_gauss,
            Self::security_under_sd_binary,
            Self::security_under_sd2_binary,
            Self::security_under_sd_isd_binary,
            Self::security_under_bjmm_isd_binary,
        ];

        cfg_if::cfg_if! {
            if #[cfg(feature = "rayon")]{
                let iter = funcs.par_iter();
            }else{
                let iter = funcs.iter();
            }
        };

        let res: Vec<f64> = iter.map(|&func| func(n, k, t)).collect();

        res.into_iter()
            .min_by(|a, b| a.partial_cmp(b).unwrap())
            .expect("Some error in finding min")
    }

    // Algebraic attack in the [paper](https://eprint.iacr.org/2023/176.pdf)
    fn cost_agb_binary(n: u64, k: u64, t: u64, f: u64, mu: u64) -> f64 {
        let n = n as u128;
        let k = k as u128;
        let t = t as u128;
        let f = f as u128;
        let mu = mu as u128;
        let beta = (n / t) as u128;

        let beta_minus_mu_minus_one = beta - mu - 1;

        let mut a1 = 0;
        let mut a2 = 0;
        let mut a3 = 0;
        let mut a4 = 0;

        if f >= 3 {
            a1 = beta_minus_mu_minus_one * f;
            a2 = beta_minus_mu_minus_one.pow(2) * f * (f - 1) / 2;
            a3 = beta_minus_mu_minus_one.pow(3) * f * (f - 1) * (f - 2) / 6;
            a4 = beta_minus_mu_minus_one.pow(4) * f * (f - 1) * (f - 2) * (f - 3) / 24;
        }

        let beta_minus_one = beta - 1;
        let t_minus_f = t as u128 - f;

        let mut b1 = 0;
        let mut b2 = 0;
        let mut b3 = 0;
        let mut b4 = 0;

        if t_minus_f >= 3 {
            b1 = beta_minus_one * t_minus_f;
            b2 = beta_minus_one.pow(2) * t_minus_f * (t_minus_f - 1) / 2;
            b3 = beta_minus_one.pow(3) * t_minus_f * (t_minus_f - 1) * (t_minus_f - 2) / 6;
            b4 = beta_minus_one.pow(4)
                * t_minus_f
                * (t_minus_f - 1)
                * (t_minus_f - 2)
                * (t_minus_f - 3)
                / 24;
        }

        let minus_c1 = n - k - 1;
        let c2 = (n - k) * (n - k - 1) / 2;
        let minus_c3 = (n - k + 1) * (n - k) * (n - k - 1) / 6;
        let c4 = (n - k + 2) * (n - k + 1) * (n - k) * (n - k - 1) / 24;

        let d2_left = a1 + b1 + a1 * b1 + a2 + b2 + c2;
        let d2_right = (a1 + b1) * minus_c1 + minus_c1;
        // let d2 = a1 + b1 + a1 * b1 + a2 + b2 + c2 - (a1 + b1) * minus_c1 - minus_c1;

        let d3_left = b1 * c2 + b3 + a1 * (b2 + c2) + a2 * b1 + a3;
        let d3_right = minus_c3 + b2 * minus_c1 + a1 * b1 * minus_c1 + a2 * minus_c1;

        // let d3 = b1 * c2 + b3 + a1 * (b2 + c2) + a2 * b1 + a3
        //     - minus_c3
        //     - b2 * minus_c1
        //     - a1 * b1 * minus_c1
        //     - a2 * minus_c1;

        let d4_left =
            c4 + b2 * c2 + b4 + a1 * (b1 * c2 + b2 + b3) + a2 * (b2 + c2 + b1) + a3 * b1 + a4;
        let d4_right = b1 * minus_c3
            + b3 * minus_c1
            + a3 * minus_c1
            + a1 * b2 * minus_c1
            + a2 * b1 * minus_c1
            + a1 * minus_c3;

        // let d4 = c4 + b2 * c2 + b4 + a1 * (b1 * c2 + b2 + b3) + a2 * (b2 + c2 + b1) + a3 * b1 + a4
        //     - b1 * minus_c3
        //     - b3 * minus_c1
        //     - a3 * minus_c1
        //     - a1 * b2 * minus_c1
        //     - a2 * b1 * minus_c1
        //     - a1 * minus_c3;

        // if d2 < 1 {
        if d2_left <= d2_right {
            return 2.0;
        }
        // if d3 < 1 {
        if d3_left <= d3_right {
            return 3.0;
        }
        // if d4 < 1 {
        if d4_left <= d4_right {
            return 4.0;
        }
        0.0
    }

    // Attack in the [paper](https://eprint.iacr.org/2023/176.pdf)
    fn sub_agb_binary(n: u64, k: u64, t: u64, f: u64, mu: u64) -> f64 {
        let f = f as u128;
        let mu = mu as u128;
        let beta = (n / t) as u128;

        let beta_minus_mu_minus_one = beta - mu - 1;

        let mut a1 = 0;
        let mut a2 = 0;
        let mut a3 = 0;
        let mut a4 = 0;

        if f >= 3 {
            a1 = beta_minus_mu_minus_one * f;
            a2 = beta_minus_mu_minus_one.pow(2) * f * (f - 1) / 2;
            a3 = beta_minus_mu_minus_one.pow(3) * f * (f - 1) * (f - 2) / 6;
            a4 = beta_minus_mu_minus_one.pow(4) * f * (f - 1) * (f - 2) * (f - 3) / 24;
        }

        let beta_minus_one = beta - 1;
        let t_minus_f = t as u128 - f;

        let mut b1 = 0;
        let mut b2 = 0;
        let mut b3 = 0;
        let mut b4 = 0;

        if t_minus_f >= 3 {
            b1 = beta_minus_one * t_minus_f;
            b2 = beta_minus_one.pow(2) * t_minus_f * (t_minus_f - 1) / 2;
            b3 = beta_minus_one.pow(3) * t_minus_f * (t_minus_f - 1) * (t_minus_f - 2) / 6;
            b4 = beta_minus_one.pow(4)
                * t_minus_f
                * (t_minus_f - 1)
                * (t_minus_f - 2)
                * (t_minus_f - 3)
                / 24;
        }

        let d = Self::cost_agb_binary(n, k, t, f as u64, mu as u64);

        let mut cost = a1.clone() + b1.clone() + a1.clone() * b1.clone() + a2.clone() + b2.clone();
        if d == 2.0 {
            cost += 0;
        } else if d == 3.0 {
            let d3 = b3 + a1 * b2 + a2 * b1 + a3;
            cost += d3;
        } else if d == 4.0 {
            let d3 = b3.clone() + a1.clone() * b2.clone() + a2.clone() * b1.clone() + a3.clone();
            let d4 = b4 + a1 * b3 + a2 * b2 + a3 * b1 + a4;
            cost += d3 + d4;
        } else {
            return HIGHEST_SECURITY as f64;
        }

        let res: Float = 2 * Float::with_val(PRECISION, cost).log2()
            + Float::with_val(PRECISION, 3 * (k + 1 - f as u64 * mu as u64)).log2()
            - f * Float::with_val(PRECISION, 1.0 - (mu as f64) / (beta as f64)).log2();

        res.to_f64()
    }

    // Attack in the [paper](https://eprint.iacr.org/2023/176.pdf)
    fn security_under_agb_binary(n: u64, k: u64, t: u64) -> f64 {
        let mut res = HIGHEST_SECURITY as f64;
        for f in 0..t {
            for mu in 0..n / t {
                if f * mu < k + 1 {
                    let cost = Self::sub_agb_binary(n, k, t, f, mu);
                    if res > cost {
                        res = cost;
                    }
                }
            }
        }
        res
    }

    /// The security of the regular lpn parameters for binary field.
    /// See Sec 5.1 in this [paper](https://eprint.iacr.org/2022/712.pdf)
    /// # Arguments.
    ///
    /// * `n` - The number of samples.
    /// * `k` - The length of the secret.
    /// * `t` - The Hamming weight of the error.
    ///
    /// NOTE: Run it in the release mode.
    pub fn security_for_binary_regular(n: u64, k: u64, t: u64) -> f64 {
        cfg_if::cfg_if! {
            if #[cfg(feature = "rayon")]{
                let (cost_agb, cost_others) = rayon::join(
                    || Self::security_under_agb_binary(n, k, t),
                    || Self::security_for_binary(n - t, k - t, t),
                );
            }else{
                let cost_agb = Self::security_under_agb_binary(n, k, t);
                let cost_others = Self::security_for_binary(n-t, k-t, t);
            }
        }
        cost_agb.min(cost_others)
    }
}

mod tests {
    #[test]
    fn security_test() {
        let sec = crate::LpnEstimator::security_for_binary(1 << 10, 652, 57);
        let sec_reg = crate::LpnEstimator::security_for_binary_regular(1 << 10, 652, 57);
        assert!(sec < 95.0);
        assert!(sec_reg < 90.0);
    }
}
