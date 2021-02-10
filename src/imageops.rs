use core::ops::AddAssign;
use num_traits::One;
use std::unreachable;

pub fn accumulate_histogram<T: One + AddAssign>(
    out: &mut [T],
    values: impl IntoIterator<Item = u8>,
) {
    let out = &mut out[0..256];
    for value in values {
        out[value as usize] += T::one();
    }
}

pub fn find_threshold(histogram: &[u32]) -> Option<usize> {
    let histogram = &histogram[0..256];

    // sum = histogram.sum()
    //
    // omega0(t) = (0..t).map(|i| histogram[i]).sum() / sum
    // omega1(t) = (t..256).map(|i| histogram[i]).sum() / sum
    //
    // mu0(t) = (0..t).map(|i| histogram[i] * i).sum() / sum / omega0(t)
    // mu1(t) = (t..256).map(|i| histogram[i] * i).sum() / sum / omega1(t)
    //
    // mu0(t) = mu0'(t) / omega0(t)
    // mu1(t) = mu1'(t) / omega1(t)
    // mu0'(t) = (0..t).map(|i| histogram[i] * i).sum() / sum
    // mu1'(t) = (t..256).map(|i| histogram[i] * i).sum() / sum
    //
    // sigma²(t) = omega0(t) * omega1(t) *
    //             (mu0(t) - mu1(t))²
    //
    // sigma²(t) = omega0(t) * (sum/sum) * omega1(t) * (sum/sum) *
    //             (mu0'(t) / omega0(t) - mu1'(t) / omega1(t))²

    let mut omega0: u64 = 0;
    let mut omega1: u64 = histogram.iter().map(|&x| x as u64).sum();
    let mut mu0_part: u64 = 0;
    let mut mu1_part: u64 = histogram
        .iter()
        .enumerate()
        .map(|(i, &x)| x as u64 * i as u64)
        .sum();

    (1..256)
        .filter_map(|i| {
            let delta_omega_part = histogram[i - 1] as u64;
            omega0 += delta_omega_part;
            omega1 -= delta_omega_part;

            let delta_mu_part = histogram[i - 1] as u64 * (i - 1) as u64;
            mu0_part += delta_mu_part;
            mu1_part -= delta_mu_part;

            if omega0 == 0 || omega1 == 0 {
                return None;
            }

            let mu0 = mu0_part as f64 / omega0 as f64;
            let mu1 = mu1_part as f64 / omega1 as f64;
            let sigma = omega0 as f64 * omega1 as f64 * (mu0 - mu1) * (mu0 - mu1);

            Some((sigma, i))
        })
        .max_by_key(|(sigma, _threshold)| sigma.to_bits())
        .map(|(_sigma, threshold)| threshold)
}

pub fn equalization_map(out: &mut [u8; 256], histogram: &[u32; 256]) {
    let sum: u32 = histogram.iter().sum();
    let mut partial_sum = 0;
    if sum == 0 {
        // avoid zero division
        return;
    }
    for (in_luma, out_luma) in out.iter_mut().enumerate() {
        *out_luma = (partial_sum * 256 / sum as u64).min(255) as u8;
        partial_sum += histogram[in_luma] as u64;
    }
}

pub fn median(histogram: &[u32]) -> usize {
    let sum: u32 = histogram.iter().sum();
    let mut partial_sum = 0;
    for (i, h) in histogram.iter().enumerate() {
        partial_sum += *h as u64;
        if partial_sum * 2 >= sum as u64 {
            return i;
        }
    }
    unreachable!()
}
