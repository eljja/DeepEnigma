// DeepEnigma Interactive Web Simulator & App Logic
import init, { 
    WasmETPM, 
    WasmIntegerNeuralNet, 
    WasmHyperchaoticSystem,
    wasm_quantize, 
    wasm_dequantize, 
    wasm_hamming_encode, 
    wasm_hamming_decode,
    estimate_wasm_lwe_security
} from './wasm/deep_enigma.js';

let wasmEngine = false;

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

    hyperchaoticTransform(iterations) {
        let result = JSON.parse(JSON.stringify(this.weights));
        let m = 2 * this.l;
        
        let sum_x = 0.0;
        let sum_y = 0.0;
        let sum_z = 0.0;
        let sum_w = 0.0;
        for (let i = 0; i < this.k; i++) {
            for (let j = 0; j < this.n; j++) {
                let val = this.weights[i][j];
                let idx = i * this.n + j;
                if (idx % 4 === 0) sum_x += Math.abs(Math.cos(val));
                else if (idx % 4 === 1) sum_y += Math.abs(Math.sin(val));
                else if (idx % 4 === 2) sum_z += Math.abs(Math.tan(val));
                else sum_w += Math.abs(Math.cos(val) * Math.sin(val));
            }
        }

        let clamp = (s) => {
            let val = Math.abs(s) % 1.0;
            return val === 0.0 ? 0.5 : val;
        };

        let hx = clamp(sum_x);
        let hy = clamp(sum_y);
        let hz = clamp(sum_z);
        let hw = clamp(sum_w);
        let r = 3.99;
        let e = 0.1;

        let next_hc = () => {
            let fx = r * hx * (1 - hx);
            let fy = r * hy * (1 - hy);
            let fz = r * hz * (1 - hz);
            let fw = r * hw * (1 - hw);
            hx = (1 - e) * fx + e * fy;
            hy = (1 - e) * fy + e * fz;
            hz = (1 - e) * fz + e * fw;
            hw = (1 - e) * fw + e * fx;
        };

        for (let round = 0; round < 50; round++) {
            next_hc();
        }

        for (let i = 0; i < this.k; i++) {
            for (let j = 0; j < this.n; j++) {
                let w = this.weights[i][j];
                let x = w + this.l; // unsigned
                for (let round = 0; round < iterations; round++) {
                    next_hc();
                    let mix = Math.floor(Math.abs(hx * 1e9));
                    x = (x + mix) % (m + 1);
                }
                result[i][j] = x - this.l;
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

    hyperchaoticTransform(iterations) {
        let flat = this.inner.hyperchaotic_transform_flat(iterations);
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
    wasmEngine = true;
    TPMClass = WasmTPMWrapper;
    updateEngineBadge(true);
    // Trigger reset to swap instances to WASM if not running
    if (typeof window.triggerReset === 'function') {
        window.triggerReset();
    }
}).catch(err => {
    console.warn("WASM Engine load failed, using pure JS fallback:", err);
    wasmEngine = false;
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

        // 1. Generate public input (optionally filtered by Active Query or simulated physical channel)
        let x_a = [];
        let x_b = [];
        let x_e = [];
        const activeQuery = activeQueryToggle.checked;
        const threshold = parseInt(activeQueryThreshold.value);
        const physicalChannel = physicalChannelToggle.checked;
        const correlation = parseFloat(paramPhysicalChannel.value);

        if (physicalChannel) {
            let alpha = Math.max(0.0, Math.min(1.0, correlation));
            let attempts = 0;
            while (attempts < 100) {
                let cand_a = [];
                let cand_b = [];
                let cand_e = [];
                for (let i = 0; i < k; i++) {
                    let row_a = [];
                    let row_b = [];
                    let row_e = [];
                    for (let j = 0; j < n; j++) {
                        let s = Math.random() * 2 - 1;
                        let n_a = Math.random() * 2 - 1;
                        let n_b = Math.random() * 2 - 1;
                        let n_e = Math.random() * 2 - 1;
                        let val_a = alpha * s + (1.0 - alpha) * n_a;
                        let val_b = alpha * s + (1.0 - alpha) * n_b;
                        let val_e = n_e;
                        
                        row_a.push(val_a >= 0 ? 1 : -1);
                        row_b.push(val_b >= 0 ? 1 : -1);
                        row_e.push(val_e >= 0 ? 1 : -1);
                    }
                    cand_a.push(row_a);
                    cand_b.push(row_b);
                    cand_e.push(row_e);
                }

                if (activeQuery) {
                    let fields = alice.calculateLocalFields(cand_a);
                    let minField = Math.min(...fields.map(Math.abs));
                    if (minField <= threshold) {
                        x_a = cand_a;
                        x_b = cand_b;
                        x_e = cand_e;
                        break;
                    }
                } else {
                    x_a = cand_a;
                    x_b = cand_b;
                    x_e = cand_e;
                    break;
                }
                attempts++;
            }
            if (x_a.length === 0) {
                for (let i = 0; i < k; i++) {
                    let row_a = [], row_b = [], row_e = [];
                    for (let j = 0; j < n; j++) {
                        let s = Math.random() * 2 - 1;
                        let n_a = Math.random() * 2 - 1;
                        let n_b = Math.random() * 2 - 1;
                        let n_e = Math.random() * 2 - 1;
                        row_a.push((alpha * s + (1.0 - alpha) * n_a) >= 0 ? 1 : -1);
                        row_b.push((alpha * s + (1.0 - alpha) * n_b) >= 0 ? 1 : -1);
                        row_e.push(n_e >= 0 ? 1 : -1);
                    }
                    x_a.push(row_a);
                    x_b.push(row_b);
                    x_e.push(row_e);
                }
            }
        } else {
            let x = [];
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

                    let fields = alice.calculateLocalFields(candidate);
                    let minField = Math.min(...fields.map(Math.abs));
                    if (minField <= threshold) {
                        x = candidate;
                        break;
                    }
                    attempts++;
                }
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
            x_a = x;
            x_b = x;
            x_e = x;
        }

        // 2. Compute outputs
        let tau_a = alice.calculateOutput(x_a);
        let tau_b = bob.calculateOutput(x_b);
        let tau_e = eve.calculateOutput(x_e);

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
            const finalWeights = act === 'hyperchaotic'
                ? alice.hyperchaoticTransform(100)
                : (act === 'hybrid' ? alice.chaoticTransform(100) : alice.weights);
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

    // ── PART 2: NEURAL ENIGMA SIMULATOR CONTROLLER ───────────────────────────

    // Tab switcher logic
    const tabButtons = document.querySelectorAll('.tab-button');
    const tabContents = document.querySelectorAll('.tab-content');

    tabButtons.forEach(btn => {
        btn.addEventListener('click', () => {
            const targetTab = btn.getAttribute('data-tab');
            tabButtons.forEach(b => b.classList.remove('active'));
            tabContents.forEach(c => c.classList.remove('active'));
            btn.classList.add('active');
            document.getElementById(targetTab).classList.add('active');
        });
    });

    // 16-bit Key selector grid generation
    const keyGrid = document.getElementById('neural-key-grid');
    const keyBits = new Array(16).fill(0);

    // Initial key seed setting (e.g. 1010101010101010)
    for (let i = 0; i < 16; i++) {
        keyBits[i] = i % 2;
    }

    function renderKeySelector() {
        keyGrid.innerHTML = '';
        for (let i = 0; i < 16; i++) {
            const btn = document.createElement('button');
            btn.className = `key-bit-btn ${keyBits[i] ? 'active' : ''}`;
            btn.textContent = `K${i}: ${keyBits[i]}`;
            btn.addEventListener('click', () => {
                keyBits[i] = keyBits[i] ? 0 : 1;
                btn.className = `key-bit-btn ${keyBits[i] ? 'active' : ''}`;
                btn.textContent = `K${i}: ${keyBits[i]}`;
            });
            keyGrid.appendChild(btn);
        }
    }
    renderKeySelector();

    // --- PART 1-1 PHYSICAL LAYER CORRELATION BINDINGS ---
    const physicalChannelToggle = document.getElementById('param-physical-channel-toggle');
    const physicalChannelContainer = document.getElementById('physical-channel-container');
    const paramPhysicalChannel = document.getElementById('param-physical-channel');
    const valPhysicalChannel = document.getElementById('val-physical-channel');

    physicalChannelToggle.addEventListener('change', () => {
        physicalChannelContainer.style.display = physicalChannelToggle.checked ? 'block' : 'none';
    });
    paramPhysicalChannel.addEventListener('input', () => {
        valPhysicalChannel.textContent = paramPhysicalChannel.value;
    });

    // --- PART 2 LWE SCOREBOARD BINDINGS & UPDATER ---
    function updateLweScoreboard() {
        if (!loadedWeights) return;
        let dim = 0;
        loadedWeights.alice.layers.forEach(l => {
            if (l.weights_int8) {
                dim += l.weights_int8.length * l.weights_int8[0].length;
            } else {
                dim += l.weights.length * l.weights[0].length;
            }
        });
        
        let modulus = 256;
        let error = 3.20;
        let classical = 0.0;
        let quantum = 0.0;

        if (wasmEngine) {
            try {
                let metrics = estimate_wasm_lwe_security(dim, modulus, error);
                classical = metrics.classical_security_bits;
                quantum = metrics.quantum_security_bits;
            } catch (e) {
                let ratio = modulus / error;
                let log_ratio = Math.log2(ratio);
                let beta = Math.max(10.0, log_ratio * 1.85);
                classical = 0.265 * beta * Math.sqrt(dim / 500.0);
                quantum = 0.229 * beta * Math.sqrt(dim / 500.0);
            }
        } else {
            let ratio = modulus / error;
            let log_ratio = Math.log2(ratio);
            let beta = Math.max(10.0, log_ratio * 1.85);
            classical = 0.265 * beta * Math.sqrt(dim / 500.0);
            quantum = 0.229 * beta * Math.sqrt(dim / 500.0);
        }
        
        document.getElementById('lwe-dim').textContent = dim;
        document.getElementById('lwe-error').textContent = error.toFixed(2);
        document.getElementById('lwe-classical').textContent = classical.toFixed(1) + " bits";
        document.getElementById('lwe-quantum').textContent = quantum.toFixed(1) + " bits";
    }

    // Neural Network Weight Loading
    let loadedWeights = null;
    fetch('neural_weights.json')
        .then(res => res.json())
        .then(data => {
            loadedWeights = data;
            console.log('✓ Successfully loaded neural weights.');
            updateLweScoreboard();
        })
        .catch(err => {
            console.warn('Failed to load weights from file, generating pre-trained fallbacks.', err);
            // Dynamic fallback generator matching python's LCG
            function makeWeights(outDim, inDim, seed) {
                let w = [];
                let b = [];
                let s = seed;
                function rand() {
                    s = (s * 9301 + 49297) % 233280;
                    return s / 233280.0;
                }
                for (let i = 0; i < outDim; i++) {
                    let row = [];
                    for (let j = 0; j < inDim; j++) {
                        row.push(rand() - 0.5);
                    }
                    w.push(row);
                    b.push(rand() - 0.1);
                }
                return { weights: w, biases: b };
            }
            const a1 = makeWeights(64, 44, 101);
            const a2 = makeWeights(44, 64, 102);
            const b1 = makeWeights(64, 60, 201);
            const b2 = makeWeights(28, 64, 202);
            loadedWeights = {
                alice: {
                    layers: [
                        { weights: a1.weights, biases: a1.biases, activation: 'relu' },
                        { weights: a2.weights, biases: a2.biases, activation: 'sigmoid' }
                    ]
                },
                bob: {
                    layers: [
                        { weights: b1.weights, biases: b1.biases, activation: 'relu' },
                        { weights: b2.weights, biases: b2.biases, activation: 'sigmoid' }
                    ]
                }
            };
            updateLweScoreboard();
        });

    // Hamming(7,4) ECC Helper functions
    function jsHammingEncode(data) {
        let out = [];
        for (let offset = 0; offset < data.length; offset += 4) {
            let d = data.slice(offset, offset + 4);
            let d1 = d[0] >= 0.5 ? 1 : 0;
            let d2 = d[1] >= 0.5 ? 1 : 0;
            let d3 = d[2] >= 0.5 ? 1 : 0;
            let d4 = d[3] >= 0.5 ? 1 : 0;
            let p1 = d1 ^ d2 ^ d4;
            let p2 = d1 ^ d3 ^ d4;
            let p3 = d2 ^ d3 ^ d4;
            out.push(p1, p2, d1, p3, d2, d3, d4);
        }
        return out;
    }

    function jsHammingDecode(codeword) {
        let out = [];
        for (let offset = 0; offset < codeword.length; offset += 7) {
            let bits = codeword.slice(offset, offset + 7).map(x => x >= 0.5 ? 1 : 0);
            let s1 = bits[0] ^ bits[2] ^ bits[4] ^ bits[6];
            let s2 = bits[1] ^ bits[2] ^ bits[5] ^ bits[6];
            let s3 = bits[3] ^ bits[4] ^ bits[5] ^ bits[6];
            let error_pos = s1 + (s2 << 1) + (s3 << 2);
            if (error_pos > 0 && error_pos <= 7) {
                bits[error_pos - 1] ^= 1;
            }
            out.push(bits[2], bits[4], bits[5], bits[6]);
        }
        return out;
    }

    // Forward pass layer execution in JS
    function jsDenseForward(input, layer) {
        let out = [];
        for (let i = 0; i < layer.biases.length; i++) {
            let sum = layer.biases[i];
            for (let j = 0; j < input.length; j++) {
                sum += layer.weights[i][j] * input[j];
            }
            if (layer.activation === 'relu') {
                out.push(sum > 0 ? sum : 0);
            } else if (layer.activation === 'sigmoid') {
                out.push(1.0 / (1.0 + Math.exp(-sum)));
            } else if (layer.activation === 'step') {
                out.push(sum >= 0.5 ? 1 : 0);
            } else {
                out.push(sum);
            }
        }
        return out;
    }

    function jsNetForward(input, layers) {
        let current = input;
        for (let i = 0; i < layers.length; i++) {
            current = jsDenseForward(current, layers[i]);
        }
        return current;
    }

    // Text to 16 bits binary representation
    function textTo16Bits(text) {
        let bin = [];
        const cleanText = text.slice(0, 2).padEnd(2, ' ');
        for (let i = 0; i < 2; i++) {
            const charCode = cleanText.charCodeAt(i);
            for (let b = 7; b >= 0; b--) {
                bin.push((charCode >> b) & 1);
            }
        }
        return bin;
    }

    // 16 bits binary representation to Text
    function bitsToText(bits) {
        let text = '';
        for (let offset = 0; offset < bits.length; offset += 8) {
            let charCode = 0;
            const slice = bits.slice(offset, offset + 8);
            for (let b = 0; b < 8; b++) {
                if (slice[b] >= 0.5) {
                    charCode |= (1 << (7 - b));
                }
            }
            text += String.fromCharCode(charCode);
        }
        return text;
    }

    // Visualize nodes in container helper with quantized integer tooltips
    function visualizeNodes(containerId, bits, tooltipValues) {
        const container = document.getElementById(containerId);
        container.innerHTML = '';
        bits.forEach((bit, idx) => {
            const circle = document.createElement('div');
            circle.className = `node-circle ${bit >= 0.5 ? 'active-one' : 'active-zero'}`;
            circle.textContent = bit >= 0.5 ? '1' : '0';
            
            let titleText = `Node ${idx}: ${bit}`;
            if (tooltipValues && tooltipValues[idx] !== undefined) {
                titleText += ` (Quantized INT8: ${tooltipValues[idx]})`;
            }
            circle.setAttribute('title', titleText);
            container.appendChild(circle);
        });
    }

    // INT8 Quantization Helpers in JS
    function jsQuantize(x, scale) {
        if (scale === 0.0) return 0;
        let q = Math.round(x / scale);
        return Math.max(-128, Math.min(127, q));
    }

    function jsDequantize(q, scale) {
        return q * scale;
    }

    // Integer Quantized Dense Layer forward pass in JS
    function jsIntegerDenseForward(input, layer) {
        let out = [];
        const scaleAccum = layer.scale_in * layer.scale_w;
        for (let i = 0; i < layer.biases_int32.length; i++) {
            let acc = layer.biases_int32[i];
            for (let j = 0; j < input.length; j++) {
                acc += layer.weights_int8[i][j] * input[j];
            }
            let valFloat = acc * scaleAccum;
            let actFloat;
            if (layer.activation === 'relu') {
                actFloat = valFloat > 0 ? valFloat : 0;
            } else if (layer.activation === 'sigmoid') {
                actFloat = 1.0 / (1.0 + Math.exp(-valFloat));
            } else if (layer.activation === 'step') {
                actFloat = valFloat >= 0.5 ? 1.0 : 0.0;
            } else {
                actFloat = valFloat;
            }
            out.push(jsQuantize(actFloat, layer.scale_out));
        }
        return out;
    }

    function jsIntegerNetForward(input, layers) {
        let current = input;
        for (let i = 0; i < layers.length; i++) {
            current = jsIntegerDenseForward(current, layers[i]);
        }
        return current;
    }

    // Bridge Part 1 key output to Part 2
    const btnApplyKeyNeural = document.getElementById('btn-apply-key-neural');
    btnApplyKeyNeural.addEventListener('click', () => {
        const keyOutput = document.getElementById('key-output').textContent.trim();
        if (keyOutput.includes('Generating') || keyOutput.length < 4) {
            alert('키 동기화가 완료되지 않았습니다. 먼저 동기화를 수행해 주세요.');
            return;
        }
        // Extract first 4 hex characters to get 16 bits
        const hex = keyOutput.slice(0, 4);
        const val = parseInt(hex, 16);
        for (let b = 0; b < 16; b++) {
            keyBits[b] = (val >> (15 - b)) & 1;
        }
        renderKeySelector();
        // Toggle tab active classes
        tabButtons.forEach(btn => {
            if (btn.getAttribute('data-tab') === 'tab-neural') {
                btn.click();
            }
        });
        alert('E-TPM 합의 대칭키가 신경망 암호화 키로 성공적으로 로드되었습니다!');
    });

    // Trigger run encryption
    const btnNeuralRun = document.getElementById('btn-neural-run');
    btnNeuralRun.addEventListener('click', () => {
        if (!loadedWeights) {
            alert('Loading neural weights, please wait.');
            return;
        }

        const plaintextInput = document.getElementById('neural-plaintext').value;
        const msgBits = textTo16Bits(plaintextInput);

        // 1. Encode with Hamming(7,4) ECC
        let encodedBits;
        if (wasmEngine) {
            try {
                encodedBits = wasm_hamming_encode(msgBits);
            } catch (e) {
                console.warn('WASM ECC failed, falling back to JS.', e);
                encodedBits = jsHammingEncode(msgBits);
            }
        } else {
            encodedBits = jsHammingEncode(msgBits);
        }

        // 2. Feed into AliceNet (Quantized INT8 Inference)
        const scaleInAlice = loadedWeights.alice.layers[0].scale_in;
        const scaleOutAlice = loadedWeights.alice.layers[loadedWeights.alice.layers.length - 1].scale_out;
        
        // Quantize Alice Inputs to INT8
        const aliceInputFloat = [...encodedBits, ...keyBits];
        const aliceInputInt8 = aliceInputFloat.map(v => {
            return wasmEngine 
                ? wasm_quantize(v, scaleInAlice)
                : jsQuantize(v, scaleInAlice);
        });

        let cipherInt8;
        if (wasmEngine) {
            try {
                const aliceNet = new WasmIntegerNeuralNet();
                loadedWeights.alice.layers.forEach(layer => {
                    const flatWeights = layer.weights_int8.flat();
                    const outCh = layer.biases_int32.length;
                    const inCh = flatWeights.length / outCh;
                    aliceNet.add_layer(flatWeights, layer.biases_int32, outCh, inCh, layer.scale_in, layer.scale_w, layer.scale_out, layer.activation);
                });
                cipherInt8 = aliceNet.forward(aliceInputInt8);
            } catch (e) {
                console.warn('WASM Alice INT8 forward failed, falling back to JS.', e);
                cipherInt8 = jsIntegerNetForward(aliceInputInt8, loadedWeights.alice.layers);
            }
        } else {
            cipherInt8 = jsIntegerNetForward(aliceInputInt8, loadedWeights.alice.layers);
        }

        // --- PART 3-2: HYPERCHAOTIC SCRAMBLING ---
        const scrambleToggle = document.getElementById('param-hyperchaotic-scrambling-toggle');
        const doScramble = scrambleToggle && scrambleToggle.checked;
        let h_noise = [];

        if (doScramble) {
            let sx = 0.1, sy = 0.2, sz = 0.3, sw = 0.4;
            keyBits.forEach((bit, idx) => {
                if (idx % 4 === 0) sx += bit * 0.05;
                else if (idx % 4 === 1) sy += bit * 0.05;
                else if (idx % 4 === 2) sz += bit * 0.05;
                else sw += bit * 0.05;
            });

            if (wasmEngine) {
                try {
                    let hc = new WasmHyperchaoticSystem(sx, sy, sz, sw);
                    h_noise = hc.generate_sequence(cipherInt8.length);
                } catch (e) {
                    console.warn('WASM Hyperchaotic failed, falling back to JS.', e);
                }
            }

            if (h_noise.length === 0) {
                let hx = sx, hy = sy, hz = sz, hw = sw;
                let r = 3.99, e = 0.1;
                for (let round = 0; round < 50; round++) {
                    let fx = r * hx * (1 - hx);
                    let fy = r * hy * (1 - hy);
                    let fz = r * hz * (1 - hz);
                    let fw = r * hw * (1 - hw);
                    hx = (1 - e) * fx + e * fy;
                    hy = (1 - e) * fy + e * fz;
                    hz = (1 - e) * fz + e * fw;
                    hw = (1 - e) * fw + e * fx;
                }
                for (let c = 0; c < cipherInt8.length; c++) {
                    let fx = r * hx * (1 - hx);
                    let fy = r * hy * (1 - hy);
                    let fz = r * hz * (1 - hz);
                    let fw = r * hw * (1 - hw);
                    hx = (1 - e) * fx + e * fy;
                    hy = (1 - e) * fy + e * fz;
                    hz = (1 - e) * fz + e * fw;
                    hw = (1 - e) * fw + e * fx;
                    h_noise.push(hx * 2.0 - 1.0);
                }
            }

            cipherInt8 = cipherInt8.map((c, idx) => {
                let h_q = wasmEngine 
                    ? wasm_quantize(h_noise[idx], scaleOutAlice)
                    : jsQuantize(h_noise[idx], scaleOutAlice);
                let sum = c + h_q;
                if (sum > 127) sum -= 256;
                if (sum < -128) sum += 256;
                return sum;
            });
        }

        // Dequantize cipher floats for compatibility/display
        const cipherFloats = cipherInt8.map(v => {
            return wasmEngine
                ? wasm_dequantize(v, scaleOutAlice)
                : jsDequantize(v, scaleOutAlice);
        });

        // Render Alice Nodes (inputs shown as binary with INT8 values in tooltip)
        visualizeNodes('neural-alice-nodes', aliceInputFloat, aliceInputInt8);

        // Render Ciphertext INT8 representations
        const cipherContainer = document.getElementById('neural-ciphertext-floats');
        cipherContainer.innerHTML = '';
        cipherInt8.forEach((val, idx) => {
            const box = document.createElement('div');
            box.className = 'cipher-node';
            if (doScramble) {
                box.classList.add('scrambled');
                box.style.boxShadow = '0 0 8px rgba(255, 75, 92, 0.6)';
                box.style.border = '1px solid rgba(255, 75, 92, 0.8)';
            }
            box.textContent = val; // Display the raw INT8 quantized values!
            box.setAttribute('title', `Float value: ${cipherFloats[idx].toFixed(4)}${doScramble ? ' [Scrambled with Hyperchaos]' : ''}`);
            cipherContainer.appendChild(box);
        });

        // 3. Bob Decryption (Correct Key - Quantized INT8 Inference)
        const scaleInBob = loadedWeights.bob.layers[0].scale_in;
        const scaleOutBob = loadedWeights.bob.layers[loadedWeights.bob.layers.length - 1].scale_out;

        // Bob input: Quantized Ciphertext + Quantized Key
        const bobKeyBitsInt8 = keyBits.map(v => {
            return wasmEngine 
                ? wasm_quantize(v, scaleInBob)
                : jsQuantize(v, scaleInBob);
        });

        let bobCipherInputInt8 = [...cipherInt8];
        if (doScramble) {
            // Bob decrypts by first subtracting the identical hyperchaotic sequence
            bobCipherInputInt8 = bobCipherInputInt8.map((c, idx) => {
                let h_q = wasmEngine 
                    ? wasm_quantize(h_noise[idx], scaleOutAlice)
                    : jsQuantize(h_noise[idx], scaleOutAlice);
                let diff = c - h_q;
                if (diff > 127) diff -= 256;
                if (diff < -128) diff += 256;
                return diff;
            });
        }
        const bobInputInt8 = [...bobCipherInputInt8, ...bobKeyBitsInt8];

        let bobDecodedInt8;
        if (wasmEngine) {
            try {
                const bobNet = new WasmIntegerNeuralNet();
                loadedWeights.bob.layers.forEach(layer => {
                    const flatWeights = layer.weights_int8.flat();
                    const outCh = layer.biases_int32.length;
                    const inCh = flatWeights.length / outCh;
                    bobNet.add_layer(flatWeights, layer.biases_int32, outCh, inCh, layer.scale_in, layer.scale_w, layer.scale_out, layer.activation);
                });
                bobDecodedInt8 = bobNet.forward(bobInputInt8);
            } catch (e) {
                console.warn('WASM Bob INT8 forward failed, falling back to JS.', e);
                bobDecodedInt8 = jsIntegerNetForward(bobInputInt8, loadedWeights.bob.layers);
            }
        } else {
            bobDecodedInt8 = jsIntegerNetForward(bobInputInt8, loadedWeights.bob.layers);
        }

        // Dequantize Bob output to bits
        const bobDecodedFloat = bobDecodedInt8.map(v => {
            return wasmEngine
                ? wasm_dequantize(v, scaleOutBob)
                : jsDequantize(v, scaleOutBob);
        });
        const bobCodedBits = bobDecodedFloat.map(v => v >= 0.5 ? 1 : 0);
        visualizeNodes('neural-bob-nodes', bobCodedBits, bobDecodedInt8);

        // Bob Decode Hamming(7,4) ECC
        let bobFinalBits;
        if (wasmEngine) {
            try {
                bobFinalBits = wasm_hamming_decode(bobCodedBits);
            } catch (e) {
                bobFinalBits = jsHammingDecode(bobCodedBits);
            }
        } else {
            bobFinalBits = jsHammingDecode(bobCodedBits);
        }

        const bobText = bitsToText(bobFinalBits);
        document.getElementById('neural-bob-decrypted-text').textContent = `"${bobText}"`;

        // 4. Eve Decryption (No Key - Zero Key)
        const zeroKey = new Array(16).fill(0);
        const eveKeyBitsInt8 = zeroKey.map(v => {
            return wasmEngine 
                ? wasm_quantize(v, scaleInBob)
                : jsQuantize(v, scaleInBob);
        });
        const eveInputInt8 = [...cipherInt8, ...eveKeyBitsInt8];
        let eveDecodedInt8;

        if (wasmEngine) {
            try {
                const eveNet = new WasmIntegerNeuralNet();
                loadedWeights.bob.layers.forEach(layer => {
                    const flatWeights = layer.weights_int8.flat();
                    const outCh = layer.biases_int32.length;
                    const inCh = flatWeights.length / outCh;
                    eveNet.add_layer(flatWeights, layer.biases_int32, outCh, inCh, layer.scale_in, layer.scale_w, layer.scale_out, layer.activation);
                });
                eveDecodedInt8 = eveNet.forward(eveInputInt8);
            } catch (e) {
                eveDecodedInt8 = jsIntegerNetForward(eveInputInt8, loadedWeights.bob.layers);
            }
        } else {
            eveDecodedInt8 = jsIntegerNetForward(eveInputInt8, loadedWeights.bob.layers);
        }

        // Dequantize Eve output to bits
        const eveDecodedFloat = eveDecodedInt8.map(v => {
            return wasmEngine
                ? wasm_dequantize(v, scaleOutBob)
                : jsDequantize(v, scaleOutBob);
        });
        const eveCodedBits = eveDecodedFloat.map(v => v >= 0.5 ? 1 : 0);
        visualizeNodes('neural-eve-nodes', eveCodedBits, eveDecodedInt8);

        // Eve Decode Hamming(7,4) ECC
        let eveFinalBits;
        if (wasmEngine) {
            try {
                eveFinalBits = wasm_hamming_decode(eveCodedBits);
            } catch (e) {
                eveFinalBits = jsHammingDecode(eveCodedBits);
            }
        } else {
            eveFinalBits = jsHammingDecode(eveCodedBits);
        }

        const eveText = bitsToText(eveFinalBits);
        document.getElementById('neural-eve-decrypted-text').textContent = `"${eveText}"`;
    });

    // Initial Reset to setup ETPMs
    resetSimulation();
});
