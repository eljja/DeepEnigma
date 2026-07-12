import sys
import os
import random
import time

# Add the parent directory and build directory to sys.path so we can import deep_enigma
sys.path.append(os.path.abspath(os.path.join(os.path.dirname(__file__), "..")))
# Target directory path for maturin develop or local builds
sys.path.append(os.path.abspath(os.path.join(os.path.dirname(__file__), "../target/release")))
sys.path.append(os.path.abspath(os.path.join(os.path.dirname(__file__), "../target/debug")))

try:
    import deep_enigma
except ImportError:
    print("Warning: deep_enigma not found. Run 'maturin develop' or compile the library first.")
    # Fallback mock for script validation
    deep_enigma = None

def generate_random_input(k, n):
    """Generates a K x N matrix with values in {-1, 1}."""
    return [[random.choice([-1, 1]) for _ in range(n)] for _ in range(k)]

def calculate_weight_difference(w1, w2):
    """Calculates the total absolute difference between two weight matrices."""
    diff = 0
    k = len(w1)
    n = len(w1[0])
    for i in range(k):
        for j in range(n):
            diff += abs(w1[i][j] - w2[i][j])
    return diff

def run_key_exchange(k=4, n=256, l=16, activation_type="hybrid", update_rule="hebbian", max_rounds=5000):
    if not deep_enigma:
        print("Cannot run simulation: deep_enigma module is not imported.")
        return

    print("=" * 60)
    print(f"DeepEnigma Key Exchange Simulation")
    print(f"Parameters: K={k}, N={n}, L={l}, Activation={activation_type}, Rule={update_rule}")
    print("=" * 60)

    # Initialize Alice and Bob with DIFFERENT weights
    alice = deep_enigma.ETPM(k, n, l, activation_type)
    bob = deep_enigma.ETPM(k, n, l, activation_type)
    
    # Randomize weights deterministically to show different starting points
    alice.initialize_weights(seed=random.randint(1, 100000))
    bob.initialize_weights(seed=random.randint(100001, 200000))

    # Initialize Eve (Eavesdropper)
    eve = deep_enigma.ETPM(k, n, l, activation_type)
    eve.initialize_weights(seed=random.randint(200001, 300000))

    start_time = time.time()
    rounds = 0
    updates = 0
    synced = False
    
    initial_diff = calculate_weight_difference(alice.get_weights(), bob.get_weights())
    print(f"Initial Alice-Bob weight difference: {initial_diff}")

    while rounds < max_rounds:
        rounds += 1
        
        # 1. Generate shared public input
        x = generate_random_input(k, n)
        
        # 2. Compute outputs
        tau_a = alice.calculate_output(x)
        tau_b = bob.calculate_output(x)
        tau_e = eve.calculate_output(x)
        
        # 3. If outputs match, update weights
        if tau_a == tau_b:
            updates += 1
            # Legitimate parties update
            alice.update_weights(tau_a, update_rule)
            bob.update_weights(tau_b, update_rule)
            
            # Eve updates only if her output matched their output
            if tau_e == tau_a:
                eve.update_weights(tau_e, update_rule)
        
        # 4. Check synchronization
        w_a = alice.get_weights()
        w_b = bob.get_weights()
        w_e = eve.get_weights()
        
        diff_ab = calculate_weight_difference(w_a, w_b)
        diff_ae = calculate_weight_difference(w_a, w_e)

        if diff_ab == 0:
            synced = True
            elapsed = time.time() - start_time
            print(f"\nSUCCESS: Alice and Bob synchronized!")
            print(f"Total Rounds: {rounds}")
            print(f"Parity Matches (Updates): {updates}")
            print(f"Time Taken: {elapsed:.4f} seconds")
            print(f"Eve's final weight difference: {diff_ae}")
            
            if diff_ae == 0:
                print("[WARNING] Eve synchronized successfully with Alice and Bob!")
            else:
                print("[SECURE] Eve failed to synchronize.")
            
            # Key derivation with chaotic mapping if Hybrid mode is active
            if activation_type == "hybrid":
                final_w_a = alice.chaotic_transform(100)
                final_w_b = bob.chaotic_transform(100)
                assert final_w_a == final_w_b, "Chaotic transform mismatch!"
                
                import hashlib
                hasher = hashlib.sha256()
                for row in final_w_a:
                    for w in row:
                        hasher.update(w.to_bytes(4, byteorder='little', signed=True))
                key = hasher.hexdigest()
                print(f"Derived 256-bit Key (hex): {key}")
            break
            
        if rounds % 200 == 0:
            print(f"Round {rounds:4d} | AB Diff: {diff_ab:5d} | AE Diff: {diff_ae:5d} | Update Rate: {updates/rounds:.2%}")

    if not synced:
        print(f"\nFAILURE: Alice and Bob failed to synchronize within {max_rounds} rounds.")
        print(f"Final AB Diff: {calculate_weight_difference(alice.get_weights(), bob.get_weights())}")

if __name__ == "__main__":
    # Run a test key exchange
    run_key_exchange(k=3, n=64, l=6, activation_type="hybrid", update_rule="hebbian", max_rounds=10000)
