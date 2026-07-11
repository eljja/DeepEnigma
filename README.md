# DeepEnigma 🧠🔒

[English](#english) | [한국어](#한국어)

---

## English

DeepEnigma is a high-performance, quantum-resistant cryptographic key exchange library implementing an **Enhanced Tree Parity Machine (E-TPM)**. It allows two authenticated parties to negotiate a secure 256-bit symmetric key over an insecure public channel using mutual chaotic synchronization.

DeepEnigma is designed specifically for **NPU-enabled IoT/edge devices and mobile systems (iOS/Android)**, utilizing neural networks as a zero-cost hardware-accelerated cryptographic primitive.

🔗 **Try the Interactive Web Simulator**: [https://eljja.github.io/DeepEnigma](https://eljja.github.io/DeepEnigma)

---

### 🌟 Key Innovations

1. **Hybrid Activation Mode**: Chaotic activations (e.g. sine) disrupt the learning gradients required for Alice and Bob to sync. DeepEnigma resolves this by using stable binary sign activations during the synchronization phase, followed by a post-sync chaotic mapping layer to harden the key.
2. **Integer-Only Chaotic Tent Map**: Floating-point chaotic mappings suffer from platform-specific precision differences (ARM vs x86), causing keys to mismatch. We implement an integer-only Tent Map with round-based diffusion, ensuring bit-for-bit key identity on any architecture.
3. **Zero-Knowledge Mutual Authentication**: Standard neural cryptography is vulnerable to Man-in-the-Middle (MitM) attacks. We integrate a lightweight hash-based Zero-Knowledge challenge-response protocol (Fiat-Shamir style) prior to synchronization, verifying identities without leaking secrets.

---

### 📊 Performance & Security Profile

#### Real Benchmark Results (K=4, N=128, L=8)
- **Calculate Output Throughput**: **114,861 ops/sec** (~8.71 µs/op)
- **Update Weights Throughput**: **207,087 ops/sec** (~4.83 µs/op)
- **Average Synchronization Time**: **~700–1200 ms** (4,000 to 7,000 rounds)

#### Cryptanalysis Resistance Profile
Under simulated cryptanalysis attacks, eavesdropper synchronization remains at **0% success rate**:
- **Passive Eavesdropping**: Eve's weights remain randomized (weight difference > 500).
- **Geometric Attack**: Scanning gradient uncertainty fails to trace private weights.
- **Genetic Algorithm Attack**: Spy TPM populations fail to converge due to the high-dimensional weight space ($K \times N$).

---

### 🛠️ Project Structure
```text
├── docs/               # GitHub Pages interactive web simulator
├── scripts/            # Python simulation & cryptanalysis scripts
│   ├── simulation.py   # Key exchange validator
│   └── attacks.py      # Passive, Geometric, and Genetic attack simulators
├── src/                # Core Rust library
│   ├── lib.rs          # PyO3 modules & public bindings
│   ├── etpm.rs         # Enhanced Tree Parity Machine engine
│   ├── protocol.rs     # KeyExchange loop & SHA-256 derivation
│   ├── security.rs     # SecurityAnalyzer & entropy measurement
│   ├── auth.rs         # Zero-Knowledge mutual authentication
│   └── benchmark.rs    # Performance benchmark harness
├── tests/              # Test suites
│   ├── etpm_tests.rs   # Rust unit tests
│   └── test_etpm.py    # Python integration tests
└── Cargo.toml          # Rust package configuration
```

---

### 🚀 Getting Started

#### Prerequisites
- Rust (edition 2021)
- Python (>= 3.9)
- `maturin` (for python bindings)

#### Rust CLI Key Exchange & Benchmark
To run the live key exchange demo:
```bash
cargo run --no-default-features --bin deepenigma
```
To run the operations benchmark:
```bash
cargo run --no-default-features --bin deepenigma -- --benchmark
```

#### Python Setup & Testing
1. Install `maturin`:
   ```bash
   pip install maturin pytest
   ```
2. Build and install the module:
   ```bash
   maturin build --release
   pip install target/wheels/*.whl --force-reinstall
   ```
3. Run test suites and python simulations:
   ```bash
   pytest tests/test_etpm.py
   python scripts/simulation.py
   python scripts/attacks.py
   ```

---

## 한국어

DeepEnigma는 **향상된 트리 패리티 머신 (E-TPM)**을 구현한 고성능 양자 내성 신경망 키 교환 암호 라이브러리입니다. 상호 학습을 통한 무작위 카오스 동기화를 사용하여, 보안 위협이 존재하는 공개 채널 상에서 안전한 256비트 대칭키를 도출할 수 있습니다.

DeepEnigma는 **NPU 탑재 IoT/엣지 디바이스 및 모바일 시스템 (iOS/Android)**을 주 타겟으로 설계되었으며, NPU를 활용한 제로 코스트 하드웨어 가속을 실현합니다.

🔗 **실시간 웹 시뮬레이터 체험**: [https://eljja.github.io/DeepEnigma](https://eljja.github.io/DeepEnigma)

---

### 🌟 핵심 기술적 설계

1. **하이브리드 활성화 함수 모드 (Hybrid Activation)**: Sine 등의 순수 카오스 비선형 활성화 함수는 학습 방향성을 깨뜨려 수렴을 방해합니다. DeepEnigma는 동기화 시점엔 이진 부호 함수(Sign)를 사용하여 빠른 수렴을 유도하고, 동기화 완료 후 키 도출 시점에 비선형 카오스 맵을 거쳐 키를 경화(Hardening)하는 설계를 도입했습니다.
2. **정수 기반 카오스 텐트 맵 (Integer Tent Map)**: ARM(모바일)과 x86(서버) 등 이기종 하드웨어 간 부동 소수점(float) 연산 정밀도 미세 차이는 키 불일치를 만듭니다. 우리는 100% 정수 연산만으로 작동하는 결정성 텐트 맵(Integer Tent Map)을 적용해 플랫폼 무관 비트 단위 일치를 보장합니다.
3. **영지식 상호 인증 (ZKP Mutual Authentication)**: 기존 신경망 키 교환은 중간자 공격(MitM)에 취약했습니다. 동기화 전 단계에 가볍고 안전한 해시 기반 영지식 challenge-response 프로토콜을 탑재하여 신원을 검증함으로써 MitM을 완벽히 차단합니다.

---

### 📊 성능 및 보안성 통계

#### 실측 벤치마크 결과 (K=4, N=128, L=8)
- **출력 계산 속도 (calculate_output)**: **114,861 ops/sec** (~8.71 µs/op)
- **가중치 업데이트 속도 (update_weights)**: **207,087 ops/sec** (~4.83 µs/op)
- **평균 동기화 시간**: **~700–1200 ms** (4,000 ~ 7,000 라운드)

#### 암호 분석 공격 저항성
시뮬레이션된 암호학적 공격 모델에서 도청자 동기화 성공률은 **0.00%**를 유지합니다.
- **수동적 도청 (Passive Eavesdropping)**: Eve의 가중치 상태는 완전히 무작위 난수로 유지됩니다.
- **기하학적 분석 공격 (Geometric Attack)**: 경계 가중치의 모호성을 파고드는 수학적 추적조차 카오스 경화에 차단됩니다.
- **유전 알고리즘 공격 (Genetic Attack)**: 스파이 TPM 군집의 유전적 교배 시도는 가중치 공간의 차원 ($K \times N$) 장벽에 의해 제한 시간 내에 수렴에 실패합니다.

---

### 🚀 시작 가이드

#### 필수 조건
- Rust (edition 2021)
- Python (>= 3.9)
- `maturin` (파이썬 바인딩 빌드용)

#### Rust CLI 실행 및 벤치마크
실시간 대칭키 동기화 데모 실행:
```bash
cargo run --no-default-features --bin deepenigma
```
E-TPM 연산 성능 벤치마크 측정:
```bash
cargo run --no-default-features --bin deepenigma -- --benchmark
```

#### Python 바인딩 빌드 및 테스트
1. `maturin` 설치:
   ```bash
   pip install maturin pytest
   ```
2. 파이썬 모듈 빌드 및 설치:
   ```bash
   maturin build --release
   pip install target/wheels/*.whl --force-reinstall
   ```
3. 테스트 스위트 및 공격 분석 시뮬레이터 실행:
   ```bash
   pytest tests/test_etpm.py
   python scripts/simulation.py
   python scripts/attacks.py
   ```
