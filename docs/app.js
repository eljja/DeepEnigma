// DeepEnigma Interactive Web Simulator & App Logic
import init, { WasmETPM } from './wasm/deep_enigma.js';

// --- TREE PARITY MACHINE SIMULATOR IN JS (FALLBACK) ---
class JSTPM {
    constructor(k, n, l, activationType) {
        this.k = k;
        this.n = n;
        this.l = l;
        this.activationType = activationType;
        this.weights = [];
        this.outputs = [];
        this.lastInput = [];
        this.initializeWeights();
    }

    initializeWeights() {
        this.weights = [];
        for (let i = 0; i < this.k; i++) {
            let row = [];
            for (let j = 0; j < this.n; j++) {
                // Random weight between -L and L
                row.push(Math.floor(Math.random() * (2 * this.l + 1)) - this.l);
            }
            this.weights.push(row);
        }
        this.outputs = new Array(this.k).fill(0);
    }

    calculateOutput(inputs) {
        this.lastInput = inputs;
        let tau = 1;

        for (let i = 0; i < this.k; i++) {
            let h = 0;
            for (let j = 0; j < this.n; j++) {
                h += this.weights[i][j] * inputs[i][j];
            }

            let sigma = 1;
            if (this.activationType === 'standard' || this.activationType === 'hybrid') {
                sigma = h > 0 ? 1 : (h < 0 ? -1 : 1);
            } else if (this.activationType === 'chaotic') {
                // Integer modulo approximation of sign(sin(πh/2L)) — matches Rust exactly
                let two_l = 2 * this.l;
                // rem_euclid equivalent for JS: ((h % (2*two_l)) + (2*two_l)) % (2*two_l)
                let h_mod = ((h % (2 * two_l)) + (2 * two_l)) % (2 * two_l);
                sigma = h_mod < two_l ? 1 : -1;
            }

            this.outputs[i] = sigma;
            tau *= sigma;
        }
        return tau;
    }

    calculateLocalFields(inputs) {
        let fields = [];
        for (let i = 0; i < this.k; i++) {
            let sum = 0;
            for (let j = 0; j < this.n; j++) {
                sum += this.weights[i][j] * inputs[i][j];
            }
            fields.push(sum);
        }
        return fields;
    }

    updateWeights(tau, rule) {
        for (let i = 0; i < this.k; i++) {
            if (this.outputs[i] === tau) {
                for (let j = 0; j < this.n; j++) {
                    let w_ij = this.weights[i][j];
                    let x_ij = this.lastInput[i][j];
                    let new_w = w_ij;

                    if (rule === 'hebbian') {
                        new_w = w_ij + x_ij * tau;
                    } else if (rule === 'antihebbian') {
                        new_w = w_ij - x_ij * tau;
                    } else if (rule === 'randomwalk') {
                        new_w = w_ij + x_ij;
                    }

                    // Clamp weights to [-L, L]
                    this.weights[i][j] = Math.max(-this.l, Math.min(this.l, new_w));
                }
            }
        }
    }

    chaoticTransform(iterations) {
        let result = JSON.parse(JSON.stringify(this.weights));
        let m = BigInt(2 * this.l);
        const MASK64 = (1n << 64n) - 1n;

        for (let i = 0; i < this.k; i++) {
            for (let j = 0; j < this.n; j++) {
                let w = this.weights[i][j];
                let x = BigInt(w + this.l); // unsigned range [0, 2L]

                for (let round = 0; round < iterations; round++) {
                    let half = m / 2n;
                    let next_tent = x < half ? 2n * x : 2n * (m - x);

                    // SipHash-inspired non-linear mixing (matches Rust exactly)
                    let mix_key = ((BigInt(i) * 0x517cc1b727220a95n) & MASK64)
                        ^ ((BigInt(j) * 0x6c62272e07bb0142n) & MASK64)
                        ^ ((BigInt(round) * 0x9e3779b97f4a7c15n) & MASK64);
                    let mixed = (next_tent + mix_key) & MASK64;
                    mixed = mixed ^ (mixed >> 17n);
                    mixed = (mixed * 0xbf58476d1ce4e5b9n) & MASK64;
                    mixed = mixed ^ (mixed >> 31n);

                    x = mixed % (m + 1n);
                }

                result[i][j] = Number(x) - this.l;
            }
        }
        return result;
    }

    getWeightSum() {
        let sum = 0;
        for (let i = 0; i < this.k; i++) {
            for (let j = 0; j < this.n; j++) {
                sum += this.weights[i][j];
            }
        }
        return sum;
    }
}

// --- TREE PARITY MACHINE RUST WASM WRAPPER ---
class WasmTPMWrapper {
    constructor(k, n, l, activationType) {
        this.k = k;
        this.n = n;
        this.l = l;
        this.activationType = activationType;
        this.inner = new WasmETPM(k, n, l, activationType);
        this.lastInput = null;
    }

    initializeWeights() {
        // Re-initialize a fresh inner WASM instance to randomize weights
        this.inner = new WasmETPM(this.k, this.n, this.l, this.activationType);
    }

    calculateOutput(inputs) {
        this.lastInput = inputs;
        // Flatten 2D inputs for WASM boundary passing
        let flatInputs = [];
        for (let i = 0; i < this.k; i++) {
            for (let j = 0; j < this.n; j++) {
                flatInputs.push(inputs[i][j]);
            }
        }
        return this.inner.calculate_output(flatInputs);
    }

    calculateLocalFields(inputs) {
        let flatInputs = [];
        for (let i = 0; i < this.k; i++) {
            for (let j = 0; j < this.n; j++) {
                flatInputs.push(inputs[i][j]);
            }
        }
        return this.inner.calculate_local_fields(flatInputs);
    }

    updateWeights(tau, rule) {
        this.inner.update_weights(tau, rule);
    }

    get weights() {
        let flat = this.inner.get_weights_flat();
        let w2d = [];
        for (let i = 0; i < this.k; i++) {
            let row = [];
            for (let j = 0; j < this.n; j++) {
                row.push(flat[i * this.n + j]);
            }
            w2d.push(row);
        }
        return w2d;
    }

    chaoticTransform(iterations) {
        let flat = this.inner.chaotic_transform_flat(iterations);
        let w2d = [];
        for (let i = 0; i < this.k; i++) {
            let row = [];
            for (let j = 0; j < this.n; j++) {
                row.push(flat[i * this.n + j]);
            }
            w2d.push(row);
        }
        return w2d;
    }

    getWeightSum() {
        let flat = this.inner.get_weights_flat();
        return flat.reduce((a, b) => a + b, 0);
    }
}

// Active TPM engine class (defaults to JS, upgraded to WASM on load)
let TPMClass = JSTPM;

function updateEngineBadge(isWasm) {
    const badge = document.getElementById('engine-status');
    if (!badge) return;

    if (isWasm) {
        badge.className = 'engine-badge wasm';
        badge.innerHTML = '<span class="ko">⚡ WASM 엔진 활성</span><span class="en">⚡ WASM Engine Active</span>';
    } else {
        badge.className = 'engine-badge js';
        badge.innerHTML = '<span class="ko">⚠️ JS 엔진 (대체)</span><span class="en">⚠️ JS Engine (Fallback)</span>';
    }
}

// Start loading WASM in background
init().then(() => {
    console.log("DeepEnigma Cryptographic WASM Core Engine Loaded successfully.");
    TPMClass = WasmTPMWrapper;
    updateEngineBadge(true);
    // Trigger reset to swap instances to WASM if not running
    if (typeof window.triggerReset === 'function') {
        window.triggerReset();
    }
}).catch(err => {
    console.warn("WASM Engine load failed, using pure JS fallback:", err);
    TPMClass = JSTPM;
    updateEngineBadge(false);
});


document.addEventListener('DOMContentLoaded', () => {
    // --- LANGUAGE SWITCHER ---
    const body = document.body;
    const btnLang = document.getElementById('lang-toggle');
    btnLang.addEventListener('click', () => {
        if (body.classList.contains('lang-ko')) {
            body.classList.remove('lang-ko');
            body.classList.add('lang-en');
        } else {
            body.classList.remove('lang-en');
            body.classList.add('lang-ko');
        }
    });

    // --- SLIDERS ---
    const paramK = document.getElementById('param-k');
    const paramN = document.getElementById('param-n');
    const paramL = document.getElementById('param-l');
    
    const valK = document.getElementById('val-k');
    const valN = document.getElementById('val-n');
    const valL = document.getElementById('val-l');

    paramK.addEventListener('input', () => valK.textContent = paramK.value);
    paramN.addEventListener('input', () => valN.textContent = paramN.value);
    paramL.addEventListener('input', () => valL.textContent = paramL.value);

    // Active Query Controls
    const activeQueryToggle = document.getElementById('param-active-query-toggle');
    const activeQueryThresholdContainer = document.getElementById('active-query-threshold-container');
    const activeQueryThreshold = document.getElementById('param-active-query-threshold');
    const valActiveQueryThreshold = document.getElementById('val-active-query-threshold');

    activeQueryToggle.addEventListener('change', () => {
        activeQueryThresholdContainer.style.display = activeQueryToggle.checked ? 'block' : 'none';
    });
    activeQueryThreshold.addEventListener('input', () => {
        valActiveQueryThreshold.textContent = activeQueryThreshold.value;
    });

    // --- CHART.JS CONFIGURATION ---
    const ctx = document.getElementById('sync-chart').getContext('2d');
    let syncChart = new Chart(ctx, {
        type: 'line',
        data: {
            labels: [],
            datasets: [
                {
                    label: 'Alice ↔ Bob Agreement',
                    data: [],
                    borderColor: '#00f2fe',
                    backgroundColor: 'rgba(0, 242, 254, 0.05)',
                    borderWidth: 2,
                    pointRadius: 0,
                    fill: true,
                    tension: 0.1
                },
                {
                    label: 'Alice ↔ Eve Agreement',
                    data: [],
                    borderColor: '#ff4b5c',
                    backgroundColor: 'rgba(255, 75, 92, 0.05)',
                    borderWidth: 2,
                    pointRadius: 0,
                    fill: true,
                    tension: 0.1
                }
            ]
        },
        options: {
            responsive: true,
            maintainAspectRatio: false,
            scales: {
                x: {
                    grid: { color: 'rgba(255,255,255,0.05)' },
                    title: { display: true, text: 'Rounds', color: '#9ca3af' },
                    ticks: { color: '#9ca3af' }
                },
                y: {
                    min: 0,
                    max: 100,
                    grid: { color: 'rgba(255,255,255,0.05)' },
                    title: { display: true, text: 'Agreement (%)', color: '#9ca3af' },
                    ticks: { color: '#9ca3af' }
                }
            },
            plugins: {
                legend: {
                    labels: { color: '#f3f4f6', font: { family: 'Outfit' } }
                }
            }
        }
    });

    function calculateOverlap(w1, w2) {
        let total = w1.length * w1[0].length;
        let matching = 0;
        for (let i = 0; i < w1.length; i++) {
            for (let j = 0; j < w1[0].length; j++) {
                if (w1[i][j] === w2[i][j]) {
                    matching++;
                }
            }
        }
        return (matching / total) * 100;
    }

    function calculateDifference(w1, w2) {
        let diff = 0;
        for (let i = 0; i < w1.length; i++) {
            for (let j = 0; j < w1[0].length; j++) {
                diff += Math.abs(w1[i][j] - w2[i][j]);
            }
        }
        return diff;
    }

    // --- CRYPTO KEY GENERATOR ---
    // NOTE: This JS demo uses plain SHA-256 for key derivation.
    // The Rust library uses HKDF-SHA256 with salt and info="DeepEnigma-Symmetric-Key",
    // which produces a different (stronger) key from the same weights.
    // WASM mode delegates to Rust's HKDF and is authoritative.
    async function deriveSha256Key(weights) {
        // Flatten weights into byte buffer
        const buffer = new ArrayBuffer(weights.length * weights[0].length * 4);
        const view = new DataView(buffer);
        let offset = 0;
        for (let i = 0; i < weights.length; i++) {
            for (let j = 0; j < weights[0].length; j++) {
                view.setInt32(offset, weights[i][j], true); // Little endian
                offset += 4;
            }
        }
        const hashBuffer = await crypto.subtle.digest('SHA-256', buffer);
        const hashArray = Array.from(new Uint8Array(hashBuffer));
        const hashHex = hashArray.map(b => b.toString(16).padStart(2, '0')).join('');
        return hashHex;
    }

    // --- SIMULATOR EXECUTION CONTROLLER ---
    let alice, bob, eve;
    let simInterval = null;
    let currentRound = 0;
    
    const btnStart = document.getElementById('btn-start');
    const btnReset = document.getElementById('btn-reset');
    const lblRounds = document.getElementById('lbl-rounds');
    const lblOverlap = document.getElementById('lbl-overlap');
    const barOverlap = document.getElementById('bar-overlap');
    const syncStatus = document.getElementById('sync-status');
    const keyPanel = document.getElementById('key-panel');
    const keyOutput = document.getElementById('key-output');

    const outAlice = document.getElementById('out-alice');
    const outBob = document.getElementById('out-bob');
    const outEve = document.getElementById('out-eve');
    const wsumAlice = document.getElementById('wsum-alice');
    const wsumBob = document.getElementById('wsum-bob');
    const diffEve = document.getElementById('diff-eve');

    function resetSimulation() {
        if (simInterval) clearInterval(simInterval);
        simInterval = null;
        currentRound = 0;

        btnStart.disabled = false;
        btnStart.innerHTML = '<i class="fa-solid fa-play"></i> <span class="ko">동기화 시작</span><span class="en">Start Sync</span>';
        
        lblRounds.textContent = '0';
        lblOverlap.textContent = '0.0%';
        barOverlap.style.width = '0%';
        
        syncStatus.className = 'status-indicator';
        syncStatus.querySelector('.ko').textContent = '대기 중';
        syncStatus.querySelector('.en').textContent = 'Ready';

        keyPanel.classList.add('hidden');
        
        outAlice.textContent = '-';
        outBob.textContent = '-';
        outEve.textContent = '-';
        wsumAlice.textContent = '-';
        wsumBob.textContent = '-';
        diffEve.textContent = '-';

        // Reset Chart
        syncChart.data.labels = [];
        syncChart.data.datasets[0].data = [];
        syncChart.data.datasets[1].data = [];
        syncChart.update();

        // Initialize ETPM instances based on current parameters
        const k = parseInt(paramK.value);
        const n = parseInt(paramN.value);
        const l = parseInt(paramL.value);
        const act = document.getElementById('param-act').value;

        alice = new TPMClass(k, n, l, act);
        bob = new TPMClass(k, n, l, act);
        eve = new TPMClass(k, n, l, act);

        // Make sure Alice and Bob are randomized differently
        let safetyCounter = 0;
        while (calculateOverlap(alice.weights, bob.weights) > 30 && safetyCounter < 10) {
            alice.initializeWeights();
            bob.initializeWeights();
            safetyCounter++;
        }
        eve.initializeWeights();
    }

    // Export reset function to window so WASM loader can trigger a reset when ready
    window.triggerReset = () => {
        if (!simInterval && currentRound === 0) {
            resetSimulation();
        }
    };

    function runSimulationStep() {
        currentRound++;
        const k = alice.k;
        const n = alice.n;
        const rule = document.getElementById('param-rule').value;
        const act = document.getElementById('param-act').value;

        // 1. Generate random public input x (optionally filtered by Active Query)
        let x = [];
        const activeQuery = activeQueryToggle.checked;
        const threshold = parseInt(activeQueryThreshold.value);

        if (activeQuery) {
            let attempts = 0;
            while (attempts < 100) {
                let candidate = [];
                for (let i = 0; i < k; i++) {
                    let row = [];
                    for (let j = 0; j < n; j++) {
                        row.push(Math.random() >= 0.5 ? 1 : -1);
                    }
                    candidate.push(row);
                }

                // Calculate Alice's local fields
                let fields = alice.calculateLocalFields(candidate);
                let minField = Math.min(...fields.map(Math.abs));
                if (minField <= threshold) {
                    x = candidate;
                    break;
                }
                attempts++;
            }
            // Fallback if no candidate meets criteria after 100 attempts
            if (x.length === 0) {
                for (let i = 0; i < k; i++) {
                    let row = [];
                    for (let j = 0; j < n; j++) {
                        row.push(Math.random() >= 0.5 ? 1 : -1);
                    }
                    x.push(row);
                }
            }
        } else {
            for (let i = 0; i < k; i++) {
                let row = [];
                for (let j = 0; j < n; j++) {
                    row.push(Math.random() >= 0.5 ? 1 : -1);
                }
                x.push(row);
            }
        }

        // 2. Compute outputs
        let tau_a = alice.calculateOutput(x);
        let tau_b = bob.calculateOutput(x);
        let tau_e = eve.calculateOutput(x);

        // Update UI displays
        outAlice.textContent = tau_a === 1 ? '+1' : '-1';
        outBob.textContent = tau_b === 1 ? '+1' : '-1';
        outEve.textContent = tau_e === 1 ? '+1' : '-1';

        wsumAlice.textContent = alice.getWeightSum();
        wsumBob.textContent = bob.getWeightSum();
        diffEve.textContent = calculateDifference(alice.weights, eve.weights);

        // 3. Update weights when Alice and Bob match
        if (tau_a === tau_b) {
            alice.updateWeights(tau_a, rule);
            bob.updateWeights(tau_b, rule);

            if (tau_e === tau_a) {
                eve.updateWeights(tau_e, rule);
            }
        }

        // Calculate agreements
        let overlap_ab = calculateOverlap(alice.weights, bob.weights);
        let overlap_ae = calculateOverlap(alice.weights, eve.weights);

        lblRounds.textContent = currentRound;
        lblOverlap.textContent = overlap_ab.toFixed(1) + '%';
        barOverlap.style.width = overlap_ab + '%';

        // Update Chart every 10 rounds for performance
        if (currentRound === 1 || currentRound % 10 === 0 || overlap_ab >= 100) {
            syncChart.data.labels.push(currentRound);
            syncChart.data.datasets[0].data.push(overlap_ab);
            syncChart.data.datasets[1].data.push(overlap_ae);
            syncChart.update('none'); // Update without animation for speed
        }

        // Check if Alice and Bob are fully synchronized
        if (overlap_ab >= 100) {
            clearInterval(simInterval);
            simInterval = null;

            syncStatus.className = 'status-indicator synced';
            syncStatus.querySelector('.ko').textContent = '동기화 완료';
            syncStatus.querySelector('.en').textContent = 'Synchronized';
            
            btnStart.disabled = true;

            // Derive key
            const finalWeights = act === 'hybrid' ? alice.chaoticTransform(100) : alice.weights;
            deriveSha256Key(finalWeights).then(key => {
                keyOutput.textContent = key;
                keyPanel.classList.remove('hidden');
            });
        } else if (currentRound >= 10000) {
            // Cap at 10000 rounds
            clearInterval(simInterval);
            simInterval = null;

            syncStatus.className = 'status-indicator';
            syncStatus.querySelector('.ko').textContent = '라운드 초과 (실패)';
            syncStatus.querySelector('.en').textContent = 'Failed (Limit Exceeded)';
            btnStart.disabled = true;
        }
    }

    btnStart.addEventListener('click', () => {
        if (simInterval) {
            // Pause
            clearInterval(simInterval);
            simInterval = null;
            btnStart.innerHTML = '<i class="fa-solid fa-play"></i> <span class="ko">동기화 재개</span><span class="en">Resume Sync</span>';
            syncStatus.className = 'status-indicator';
            syncStatus.querySelector('.ko').textContent = '일시 정지';
            syncStatus.querySelector('.en').textContent = 'Paused';
        } else {
            // Start
            btnStart.innerHTML = '<i class="fa-solid fa-pause"></i> <span class="ko">시뮬레이션 일시정지</span><span class="en">Pause Simulation</span>';
            syncStatus.className = 'status-indicator running';
            syncStatus.querySelector('.ko').textContent = '동기화 중...';
            syncStatus.querySelector('.en').textContent = 'Synchronizing...';
            simInterval = setInterval(runSimulationStep, 20); // Faster simulation (20ms per round)
        }
    });

    btnReset.addEventListener('click', resetSimulation);

    // Initial Reset to setup ETPMs
    resetSimulation();
});
