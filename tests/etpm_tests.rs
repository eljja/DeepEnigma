use deep_enigma::{ETPM, ActivationType};

#[test]
fn test_etpm_creation() {
    let etpm = ETPM::new(4, 100, 8, "chaotic").unwrap();
    assert_eq!(etpm.k, 4);
    assert_eq!(etpm.n, 100);
    assert_eq!(etpm.l, 8);
    matches!(etpm.activation_type, ActivationType::Chaotic);

    let weights = etpm.get_weights();
    assert_eq!(weights.len(), 4);
    assert_eq!(weights[0].len(), 100);

    for row in weights.iter() {
        for &w in row.iter() {
            assert!(w >= -8 && w <= 8);
        }
    }
}

#[test]
fn test_deterministic_seed() {
    let mut etpm1 = ETPM::new(3, 10, 5, "standard").unwrap();
    let mut etpm2 = ETPM::new(3, 10, 5, "standard").unwrap();

    etpm1.initialize_weights(Some(42)).unwrap();
    etpm2.initialize_weights(Some(42)).unwrap();

    assert_eq!(etpm1.get_weights(), etpm2.get_weights());

    // Different seed should yield different weights
    etpm2.initialize_weights(Some(43)).unwrap();
    assert_ne!(etpm1.get_weights(), etpm2.get_weights());
}

#[test]
fn test_calculate_output_standard() {
    let mut etpm = ETPM::new(2, 5, 3, "standard").unwrap();
    // Set manual weights:
    // W1 = [1, 2, -1, 0, 3]
    // W2 = [-2, 0, 1, 2, -3]
    etpm.set_weights(vec![
        vec![1, 2, -1, 0, 3],
        vec![-2, 0, 1, 2, -3],
    ]).unwrap();

    // Inputs:
    // X1 = [1, 1, -1, 1, -1] -> Inner product h1 = 1(1) + 2(1) + (-1)(-1) + 0(1) + 3(-1) = 1 + 2 + 1 + 0 - 3 = 1 > 0 -> sigma1 = 1
    // X2 = [-1, 1, 1, -1, 1] -> Inner product h2 = -2(-1) + 0(1) + 1(1) + 2(-1) + (-3)(1) = 2 + 0 + 1 - 2 - 3 = -2 < 0 -> sigma2 = -1
    // Parity output tau = sigma1 * sigma2 = 1 * -1 = -1
    let inputs = vec![
        vec![1, 1, -1, 1, -1],
        vec![-1, 1, 1, -1, 1],
    ];

    let tau = etpm.calculate_output(inputs).unwrap();
    assert_eq!(tau, -1);
    assert_eq!(etpm.get_hidden_outputs(), vec![1, -1]);
}

#[test]
fn test_calculate_output_chaotic() {
    let mut etpm = ETPM::new(1, 3, 4, "chaotic").unwrap();
    etpm.set_weights(vec![vec![2, -2, 1]]).unwrap();

    // Inputs:
    // X = [1, -1, 1] -> h = 2(1) + (-2)(-1) + 1(1) = 5
    // Chaotic activation: sin(pi * 5 / 8) -> sin(112.5 degrees) > 0 -> sigma = 1
    let inputs1 = vec![vec![1, -1, 1]];
    let tau1 = etpm.calculate_output(inputs1).unwrap();
    assert_eq!(tau1, 1);

    // Inputs:
    // X = [-1, -1, -1] -> h = 2(-1) + (-2)(-1) + 1(-1) = -1
    // Chaotic activation: sin(pi * -1 / 8) -> sin(-22.5 degrees) < 0 -> sigma = -1
    let inputs2 = vec![vec![-1, -1, -1]];
    let tau2 = etpm.calculate_output(inputs2).unwrap();
    assert_eq!(tau2, -1);
}

#[test]
fn test_update_weights_hebbian() {
    let mut etpm = ETPM::new(1, 3, 5, "standard").unwrap();
    etpm.set_weights(vec![vec![1, -2, 3]]).unwrap();

    // Inputs: X = [1, 1, -1] -> h = 1(1) + -2(1) + 3(-1) = -4 -> sigma = -1
    let inputs = vec![vec![1, 1, -1]];
    let tau = etpm.calculate_output(inputs).unwrap();
    assert_eq!(tau, -1);

    // Update with Hebbian: w = w + x * tau = [1 + 1(-1), -2 + 1(-1), 3 + -1(-1)] = [0, -3, 4]
    etpm.update_weights(tau, "hebbian").unwrap();
    assert_eq!(etpm.get_weights(), vec![vec![0, -3, 4]]);
}

#[test]
fn test_synaptic_depth_scaling() {
    let mut etpm = ETPM::new(1, 3, 2, "standard").unwrap();
    etpm.set_weights(vec![vec![2, -1, 0]]).unwrap();

    // Scale L from 2 to 4 (scale factor = 2.0)
    // Scaled weights: [2 * 2 = 4, -1 * 2 = -2, 0 * 2 = 0]
    etpm.scale_synaptic_depth(4).unwrap();
    assert_eq!(etpm.l, 4);
    assert_eq!(etpm.get_weights(), vec![vec![4, -2, 0]]);
}

#[test]
fn test_input_validation() {
    let mut etpm = ETPM::new(2, 3, 2, "standard").unwrap();

    // Invalid K dimension
    let inputs_bad_k = vec![vec![1, 1, 1]];
    assert!(etpm.calculate_output(inputs_bad_k).is_err());

    // Invalid N dimension
    let inputs_bad_n = vec![vec![1, 1, 1], vec![1, 1]];
    assert!(etpm.calculate_output(inputs_bad_n).is_err());

    // Invalid input values (not -1 or 1)
    let inputs_bad_val = vec![vec![1, 0, 1], vec![1, 1, -1]];
    assert!(etpm.calculate_output(inputs_bad_val).is_err());
}
