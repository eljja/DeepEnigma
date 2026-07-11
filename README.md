# DeepEnigma 🧠🔒

DeepEnigma is a neural network-based cryptographic key exchange library implementing an **Enhanced Tree Parity Machine (E-TPM)**. It enables two parties to establish a secure shared symmetric key over an insecure public channel through mutual synchronization, offering a unique, adaptive approach to quantum-resistant cryptography.

## 🌟 Core Concepts

Traditional public-key cryptography (like RSA) or post-quantum cryptography (like ML-KEM) relies on the hardness of static mathematical problems (integer factorization, learning-with-errors). 

DeepEnigma shifts the paradigm by relying on **chaotic synchronization in non-linear dynamical systems**:

1. **Enhanced Tree Parity Machine (E-TPM)**: A feed-forward neural network with $K$ hidden units, $N$ inputs per hidden unit, and a synaptic depth of $L$.
2. **Mutual Learning (양방향 상호 학습)**: Two machines (Alice and Bob) exchange output bits and adjust their weights using shared learning rules. They converge to identical weight vectors in polynomial time.
3. **Unidirectional Learning Resistance**: An eavesdropper (Eve) who only monitors the exchange cannot easily synchronize her weights because she cannot affect the outputs of Alice and Bob (passive learning).
4. **Chaotic Activation Functions**: Replaces the basic binary sign activation with chaotic, non-linear functions (e.g., mixtures of $\tanh$ and $\sin$) to eliminate simple geometric attack vulnerabilities.
5. **Dynamic Synaptic Depth ($L$)**: The boundary limits of the weight values dynamically adapt if suspicious synchronization attempts are detected, exponentially reducing the success probability of geometric and genetic attacks.

---

## 📊 Target Benchmarks

DeepEnigma is designed as an **adaptive, complementary cryptographic layer** targeting NPU-enabled IoT/edge and mobile devices.

| Metric | RSA-4096 (Classical) | ML-KEM-1024 (PQC Standard) | DeepEnigma E-TPM (Target) |
| :--- | :--- | :--- | :--- |
| **Quantum Resistance** | No | Yes (FIPS 203) | **Yes (Theoretical)** |
| **Security Foundation** | Integer Factorization | Module-LWE | **Dynamic Chaos & Non-Abelian Inversion** |
| **Key Size** | 512 B | 1,568 B | **< 100 B** (Seed + Exchange bits) |
| **Encapsulation Speed** | Slow (~5M+ cycles) | Fast (~210K cycles) | **Ultra-Fast (< 100K cycles with NPU)** |
| **Protocol Round Trips** | 1 RTT | 1 RTT | **~200-500 RTT** (Ideal for persistent channels) |

---

## 🛠️ Project Structure

The project is implemented in **Rust** for performance, memory safety, and cross-compilation support, with **Python bindings** via PyO3 for easy prototyping, simulation, and analysis.

```text
├── Cargo.toml          # Rust package configuration with PyO3
├── src/
│   ├── lib.rs          # PyO3 module definition and library entry
│   └── etpm.rs         # Enhanced Tree Parity Machine implementation
├── tests/              # Rust unit/integration tests
├── scripts/            # Python simulation and attack scripts
└── README.md
```

## 🚀 Getting Started

### Prerequisites

- Rust (latest stable edition, 2024)
- Python (>= 3.9)
- `maturin` (for building Python bindings)

### Build Instructions

1. **Clone the repository**:
   ```bash
   git clone https://github.com/eljja/DeepEnigma.git
   cd DeepEnigma
   ```

2. **Build the Rust library**:
   ```bash
   cargo build --release
   ```

3. **Install the Python extension module**:
   Using `maturin` to build and install the module directly into your active python environment:
   ```bash
   pip install maturin
   maturin develop --release
   ```

4. **Verify import in Python**:
   ```python
   import deep_enigma
   print(dir(deep_enigma))
   ```

## 📜 License

This project is licensed under the MIT License - see the LICENSE file for details.
