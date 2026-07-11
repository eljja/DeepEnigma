import pytest
import random
import deep_enigma

def test_etpm_creation():
    etpm = deep_enigma.ETPM(3, 64, 6, "standard")
    assert etpm.k == 3
    assert etpm.n == 64
    assert etpm.l == 6
    assert etpm.activation_type == deep_enigma.ActivationType.Standard

def test_weight_initialization():
    etpm1 = deep_enigma.ETPM(3, 10, 5, "standard")
    etpm2 = deep_enigma.ETPM(3, 10, 5, "standard")
    
    etpm1.initialize_weights(42)
    etpm2.initialize_weights(42)
    
    assert etpm1.get_weights() == etpm2.get_weights()
    
    etpm2.initialize_weights(43)
    assert etpm1.get_weights() != etpm2.get_weights()

def test_calculate_output():
    etpm = deep_enigma.ETPM(2, 5, 3, "standard")
    etpm.set_weights([
        [1, 2, -1, 0, 3],
        [-2, 0, 1, 2, -3]
    ])
    
    inputs = [
        [1, 1, -1, 1, -1],
        [-1, 1, 1, -1, 1]
    ]
    
    tau = etpm.calculate_output(inputs)
    assert tau == -1
    assert etpm.get_hidden_outputs() == [1, -1]

def test_update_weights():
    etpm = deep_enigma.ETPM(1, 3, 5, "standard")
    etpm.set_weights([[1, -2, 3]])
    
    inputs = [[1, 1, -1]]
    tau = etpm.calculate_output(inputs)
    assert tau == -1
    
    etpm.update_weights(tau, "hebbian")
    assert etpm.get_weights() == [[0, -3, 4]]

def test_synaptic_depth_scaling():
    etpm = deep_enigma.ETPM(1, 3, 2, "standard")
    etpm.set_weights([[2, -1, 0]])
    
    etpm.scale_synaptic_depth(4)
    assert etpm.l == 4
    assert etpm.get_weights() == [[4, -2, 0]]

def test_invalid_parameters():
    etpm = deep_enigma.ETPM(2, 3, 2, "standard")
    
    # Invalid shape
    with pytest.raises(ValueError):
        etpm.calculate_output([[1, 1, 1]])
        
    # Invalid values
    with pytest.raises(ValueError):
        etpm.calculate_output([[1, 0, 1], [1, 1, 1]])

def test_key_exchange_synchronization():
    k, n, l = 3, 32, 4
    alice = deep_enigma.ETPM(k, n, l, "standard")
    bob = deep_enigma.ETPM(k, n, l, "standard")
    
    alice.initialize_weights(100)
    bob.initialize_weights(200)
    
    rounds = 0
    max_rounds = 5000
    synced = False
    
    while rounds < max_rounds:
        rounds += 1
        x = [[random.choice([-1, 1]) for _ in range(n)] for _ in range(k)]
        
        tau_a = alice.calculate_output(x)
        tau_b = bob.calculate_output(x)
        
        if tau_a == tau_b:
            alice.update_weights(tau_a, "hebbian")
            bob.update_weights(tau_b, "hebbian")
            
        if alice.get_weights() == bob.get_weights():
            synced = True
            break
            
    assert synced, f"Failed to sync in {max_rounds} rounds"

def test_zkp_authentication_py():
    psk = b"anothersecretpassword"
    prover = deep_enigma.ZKPProver(psk)
    verifier = deep_enigma.ZKPVerifier(psk)

    commitment = prover.create_commitment()
    assert len(commitment) == 32

    verifier.receive_commitment(commitment)
    challenge = verifier.create_challenge()
    assert len(challenge) == 32

    response = prover.respond(challenge)
    assert len(response) == 32

    nonce = prover.get_nonce()
    success = verifier.verify(nonce, response)
    assert success

    bad_verifier = deep_enigma.ZKPVerifier(b"wrongpassword")
    bad_verifier.receive_commitment(prover.create_commitment())
    challenge = bad_verifier.create_challenge()
    response = prover.respond(challenge)
    success = bad_verifier.verify(prover.get_nonce(), response)
    assert not success
