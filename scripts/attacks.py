import sys
import os
import random
import time
import copy

# Add the parent directory and build directory to sys.path
sys.path.append(os.path.abspath(os.path.join(os.path.dirname(__file__), "..")))
sys.path.append(os.path.abspath(os.path.join(os.path.dirname(__file__), "../target/release")))
sys.path.append(os.path.abspath(os.path.join(os.path.dirname(__file__), "../target/debug")))

try:
    import deep_enigma
except ImportError:
    deep_enigma = None

def generate_random_input(k, n):
    return [[random.choice([-1, 1]) for _ in range(n)] for _ in range(k)]

def calculate_weight_difference(w1, w2):
    diff = 0
    for i in range(len(w1)):
        for j in range(len(w1[0])):
            diff += abs(w1[i][j] - w2[i][j])
    return diff

class GeometricAttacker:
    def __init__(self, k, n, l, activation_type):
        self.etpm = deep_enigma.ETPM(k, n, l, activation_type)
        self.k = k
        self.n = n
        self.l = l

    def step(self, x, target_tau, update_rule):
        # Calculate current inner products and outputs
        self.etpm.calculate_output(x)
        current_outputs = self.etpm.get_hidden_outputs()
        
        # Calculate inner products h_i
        w = self.etpm.get_weights()
        h_vals = []
        for i in range(self.k):
            h_i = sum(w[i][j] * x[i][j] for j in range(self.n))
            h_vals.append((abs(h_i), i))
        
        # If output matches target_tau, update normally
        tau_e = 1
        for o in current_outputs:
            tau_e *= o
            
        if tau_e == target_tau:
            self.etpm.update_weights(target_tau, update_rule)
        else:
            # GEOMETRIC ATTACK: Find the hidden unit with the smallest absolute inner product
            # and flip its output to match the target parity.
            h_vals.sort()
            closest_idx = h_vals[0][1]
            
            # Temporarily flip that output
            flipped_outputs = list(current_outputs)
            flipped_outputs[closest_idx] *= -1
            
            # Expose flipped outputs back to the ETPM's internal output state
            # Note: Since ETPM outputs are calculated, we update the weights of the flipped unit
            # manually or by forcing update weights.
            # In our ETPM, update_weights updates unit i if outputs[i] == tau.
            # Since we want to update the unit that *should* have matched, we can simulate the update:
            weights = self.etpm.get_weights()
            for j in range(self.n):
                w_ij = weights[closest_idx][j]
                x_ij = x[closest_idx][j]
                # Apply the correct update rule for the flipped unit
                if update_rule == "hebbian":
                    new_w = w_ij + x_ij * target_tau
                elif update_rule == "antihebbian":
                    new_w = w_ij - x_ij * target_tau
                else:  # randomwalk
                    new_w = w_ij + x_ij
                weights[closest_idx][j] = max(-self.l, min(self.l, new_w))
            self.etpm.set_weights(weights)

class GeneticAttacker:
    def __init__(self, k, n, l, activation_type, population_size=50):
        self.k = k
        self.n = n
        self.l = l
        self.act_type = activation_type
        self.pop_size = population_size
        self.population = [deep_enigma.ETPM(k, n, l, activation_type) for _ in range(population_size)]
        for i, member in enumerate(self.population):
            member.initialize_weights(random.randint(0, 1000000 + i))

    def step(self, x, target_tau, update_rule):
        survivors = []
        for member in self.population:
            tau = member.calculate_output(x)
            if tau == target_tau:
                member.update_weights(target_tau, update_rule)
                survivors.append(member)

        if not survivors:
            # If all died, re-initialize population with random weights
            for member in self.population:
                member.initialize_weights(random.randint(0, 1000000))
            return

        # Crossover & Mutation to restore population size
        new_population = []
        while len(new_population) < self.pop_size:
            parent1 = random.choice(survivors)
            parent2 = random.choice(survivors)
            
            child = deep_enigma.ETPM(self.k, self.n, self.l, self.act_type)
            w1 = parent1.get_weights()
            w2 = parent2.get_weights()
            
            # Crossover: Take hidden units from parents randomly
            child_w = []
            for i in range(self.k):
                if random.random() < 0.5:
                    child_w.append(copy.deepcopy(w1[i]))
                else:
                    child_w.append(copy.deepcopy(w2[i]))
            
            # Mutation: Small probability to mutate some weights
            for i in range(self.k):
                for j in range(self.n):
                    if random.random() < 0.01:
                        child_w[i][j] = max(-self.l, min(self.l, child_w[i][j] + random.choice([-1, 1])))
            
            child.set_weights(child_w)
            new_population.append(child)
            
        self.population = new_population

    def get_best_difference(self, target_weights):
        best_diff = float('inf')
        for member in self.population:
            diff = calculate_weight_difference(member.get_weights(), target_weights)
            if diff < best_diff:
                best_diff = diff
        return best_diff

def benchmark_attacks(k=4, n=128, l=8, activation_type="hybrid", update_rule="hebbian", max_rounds=2000):
    if not deep_enigma:
        print("Maturin extension not loaded.")
        return

    print("=" * 70)
    print(f"Running Cryptanalysis Benchmark against E-TPM")
    print(f"K={k}, N={n}, L={l}, Activation={activation_type}")
    print("=" * 70)

    alice = deep_enigma.ETPM(k, n, l, activation_type)
    bob = deep_enigma.ETPM(k, n, l, activation_type)
    alice.initialize_weights(100)
    bob.initialize_weights(200)

    # Initial Attackers
    passive_eve = deep_enigma.ETPM(k, n, l, activation_type)
    passive_eve.initialize_weights(300)
    
    geom_eve = GeometricAttacker(k, n, l, activation_type)
    genetic_eve = GeneticAttacker(k, n, l, activation_type, population_size=50)

    rounds = 0
    synced = False

    while rounds < max_rounds:
        rounds += 1
        x = generate_random_input(k, n)
        
        tau_a = alice.calculate_output(x)
        tau_b = bob.calculate_output(x)
        
        if tau_a == tau_b:
            alice.update_weights(tau_a, update_rule)
            bob.update_weights(tau_b, update_rule)
            
            # Passive Eve update
            tau_pe = passive_eve.calculate_output(x)
            if tau_pe == tau_a:
                passive_eve.update_weights(tau_pe, update_rule)
                
            # Geometric Eve update
            geom_eve.step(x, tau_a, update_rule)
            
            # Genetic Eve update
            genetic_eve.step(x, tau_a, update_rule)

        w_a = alice.get_weights()
        w_b = bob.get_weights()
        
        diff_ab = calculate_weight_difference(w_a, w_b)
        
        if diff_ab == 0:
            synced = True
            diff_pe = calculate_weight_difference(w_a, passive_eve.get_weights())
            diff_ge = calculate_weight_difference(w_a, geom_eve.etpm.get_weights())
            diff_gene = genetic_eve.get_best_difference(w_a)
            
            print(f"Alice & Bob synced at round {rounds}.")
            print(f"Passive Eve Diff: {diff_pe}")
            print(f"Geometric Eve Diff: {diff_ge}")
            print(f"Genetic Eve (Best) Diff: {diff_gene}")
            
            # Evaluate security
            if diff_ge == 0:
                print("[VULNERABLE] Geometric Attack broke the synchronization.")
            elif diff_gene == 0:
                print("[VULNERABLE] Genetic Attack broke the synchronization.")
            else:
                print("[SECURE] E-TPM withstood all simulated attacks.")
            break

    if not synced:
        print(f"Alice & Bob failed to sync within {max_rounds} rounds.")

if __name__ == "__main__":
    benchmark_attacks(k=3, n=64, l=6, activation_type="hybrid", update_rule="hebbian")
    print("\nComparing with standard non-chaotic TPM:")
    benchmark_attacks(k=3, n=64, l=6, activation_type="standard", update_rule="hebbian")
