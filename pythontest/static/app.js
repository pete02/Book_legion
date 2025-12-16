const container = document.getElementById('snapshot-container');
const snapshot = document.getElementById('snapshot');

let busy = false;
let inputRects = []; // cached input hitboxes (snapshot-space)

async function fetchInputs() {
    const res = await fetch('/inputs');
    const inputs = await res.json();

    inputRects = inputs;

    document.querySelectorAll('.overlay-input').forEach(el => el.remove());

    inputs.forEach(input => {
        const el = document.createElement('input');
        el.className = 'overlay-input';
        console.log(input)
        // Show the current input value as text
        el.textContent = input.value || '';

        el.style.left = (input.x * snapshot.width / input.width) + 'px';
        el.style.top = (input.y * snapshot.height / input.height) + 'px';
        el.style.width = (input.w * snapshot.width / input.width) + 'px';
        el.style.height = (input.h * snapshot.height / input.height) + 'px';

        el.addEventListener('input', async (e) => {
            const value = e.target.value;
            console.log(input.index)
            await fetch('/input', {
                method: 'POST',
                headers: {'Content-Type': 'application/json'},
                body: JSON.stringify({ index: input.index, value })
            });
        });

        container.appendChild(el);
    });
}

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

async function refresh() {
    return new Promise(resolve => {
        snapshot.onload = async () => {
            await fetchInputs();
            resolve();
        };
        snapshot.src = '/snapshot?ts=' + Date.now();
    });
}

container.addEventListener('click', async (e) => {
    if (busy) return;

    const rect = snapshot.getBoundingClientRect();
    const x = Math.round((e.clientX - rect.left) * (snapshot.naturalWidth / rect.width));
    const y = Math.round((e.clientY - rect.top) * (snapshot.naturalHeight / rect.height));

    if (clickHitsInput(x, y)) {
        return;
    }

    busy = true;

    fetch('/click', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ x, y })
    });

    await refresh();
    busy = false;
});

// Initial load
refresh();
