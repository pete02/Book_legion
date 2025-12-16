const container = document.getElementById('snapshot-container');
const snapshot = document.getElementById('snapshot');

let busy = false;
let inputRects = []; // cached input hitboxes
let focusedInputIndex = null;
let blocked=true;

function generateRandomToken(length = 32) {
    const array = new Uint8Array(length);
    window.crypto.getRandomValues(array);
    // Convert each byte to a hexadecimal string and join
    return Array.from(array, byte => byte.toString(16).padStart(2, '0')).join('');
}

// Example usage:
const client_token = generateRandomToken()

// Fetch input positions and create overlay inputs
async function fetchInputs() {
    const res = await fetch('/inputs');
    const inputs = await res.json();

    inputRects = inputs;

    const existingEls = Array.from(container.querySelectorAll('.overlay-input'));
    const existingMap = new Map(existingEls.map(el => [parseInt(el.dataset.index), el]));

    const newIndices = new Set();

    inputs.forEach(input => {
        newIndices.add(input.index);
        let el = existingMap.get(input.index);

        if (!el) {
            // Create new element if it doesn't exist
            el = document.createElement('input');
            el.className = 'overlay-input';
            el.dataset.index = input.index;

            el.addEventListener('focus', () => {
                focusedInputIndex = input.index;
            });
            el.addEventListener('blur', () => {
                focusedInputIndex = null;
            });

            el.addEventListener('input', async (e) => {
                const value = e.target.value;
                if(!blocked){
                    await fetch('/input', {
                        method: 'POST',
                        headers: { 'Content-Type': 'application/json' },
                        body: JSON.stringify({ index: input.index, value })
                    });
                }
            });

            container.appendChild(el);
        }

        // Update position, size, and value
        el.style.left = (input.x * snapshot.width / input.width) + 'px';
        el.style.top = (input.y * snapshot.height / input.height) + 'px';
        el.style.width = (input.w * snapshot.width / input.width) + 'px';
        el.style.height = (input.h * snapshot.height / input.height) + 'px';
        el.value = input.value || '';
    });

    // Remove old inputs that are no longer present
    existingEls.forEach(el => {
        const idx = parseInt(el.dataset.index);
        if (!newIndices.has(idx)) {
            el.remove();
        }
    });

    // Restore focus if needed
    if (focusedInputIndex !== null) {
        const el = container.querySelector(`.overlay-input[data-index="${focusedInputIndex}"]`);
        if (el) el.focus();
    }
}

// Detect if click hits any input overlay
function clickHitsInput(x, y) {
    return inputRects.some(input => {
        return (
            x >= input.x &&
            x <= input.x + input.w &&
            y >= input.y &&
            y <= input.y + input.h
        );
    });
}

async function check() {
    await fetch('/check', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ token: client_token })
    }).then(a=>{
        if (a.status== 202){
            console.log("token ok")
            blocked=false
            return true
        }else{
            console.log("not good")
            blocked=true
            return false
        }
    });
}

// Refresh snapshot
async function refresh() {
    return new Promise(resolve => {
        snapshot.onload = async () => {
            const ok = await check();
            if (!ok){
                resolve()
            }
            await fetchInputs();
            resolve();
        };
        snapshot.src = '/snapshot?ts=' + Date.now();
    });
}

// Click handler
container.addEventListener('click', async (e) => {
    if (busy) return;

    const rect = snapshot.getBoundingClientRect();
    const x = Math.round((e.clientX - rect.left) * (snapshot.naturalWidth / rect.width));
    const y = Math.round((e.clientY - rect.top) * (snapshot.naturalHeight / rect.height));

    if (clickHitsInput(x, y)) {
        return; // allow native input interactions
    }

    busy = true;
    if (!blocked){
        await fetch('/click', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ x, y })
        });
    }

    // No immediate refresh; server will push update via SSE
    busy = false;
});

// --- SSE listener for server-driven snapshot updates ---
const evtSource = new EventSource("/updates");
evtSource.onmessage = async () => {
    if (!busy && !blocked) {
        busy = true;
        await refresh();
        busy = false;
    }
};

async function sendViewportSize() {


    // send token along with screen resolution
    fetch("/res", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ width: window.innerWidth, height: window.innerHeight, token: client_token })
    });

    // heartbeat every 10s
    setInterval(() => {
        if (!blocked){
            fetch("/heartbeat", {
                method: "POST",
                headers: { "Content-Type": "application/json" },
                body: JSON.stringify({ token: "bla" })
            });
        }
    }, 10000);
}

// Call it on first load
sendViewportSize().then(a=>{
    refresh();
})




