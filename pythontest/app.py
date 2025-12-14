from fastapi import FastAPI
from fastapi.responses import StreamingResponse, HTMLResponse
from pydantic import BaseModel
from playwright.async_api import async_playwright
import asyncio
import io

app = FastAPI()

# -------------------
# Data Models
# -------------------
class ClickData(BaseModel):
    x: int
    y: int

class InputData(BaseModel):
    selector: str
    value: str

# -------------------
# Global Browser State
# -------------------
browser_instance = None
page_instance = None

async def init_browser():
    global browser_instance, page_instance
    playwright = await async_playwright().start()
    browser_instance = await playwright.chromium.launch(headless=True)
    page_instance = await browser_instance.new_page()
    await page_instance.goto("https://book.lumilukonservu.duckdns.org/")

@app.on_event("startup")
async def startup_event():
    await init_browser()

@app.on_event("shutdown")
async def shutdown_event():
    global browser_instance
    if browser_instance:
        await browser_instance.close()

# -------------------
# Routes
# -------------------

@app.get("/", response_class=HTMLResponse)
async def root():
    return """
    <!DOCTYPE html>
    <html lang="en">
    <head>
        <meta charset="UTF-8">
        <title>Remote Browser Viewer</title>
        <style>
            body { font-family: sans-serif; margin: 0; padding: 0; text-align: center; }
            #snapshot-container { position: relative; display: inline-block; }
            #snapshot { border: 1px solid #ccc; max-width: 100%; }
            .overlay-input {
                position: absolute;
                border: 1px solid #333;
                background: rgba(255,255,255,0.8);
                font-size: 14px;
                padding: 2px;
            }
        </style>
    </head>
    <body>
        <h1>Remote Browser Viewer</h1>
        <div id="snapshot-container">
            <img id="snapshot" src="/snapshot" />
        </div>

        <script>
            const container = document.getElementById('snapshot-container');
            const snapshot = document.getElementById('snapshot');

            // Fetch input positions from backend
            async function fetchInputs() {
                const res = await fetch('/inputs');  // New endpoint to return input positions
                const inputs = await res.json();

                // Remove old overlays
                document.querySelectorAll('.overlay-input').forEach(el => el.remove());

                inputs.forEach(input => {
                    const el = document.createElement('input');
                    el.className = 'overlay-input';
                    el.value = input.value || '';
                    el.style.left = (input.x * snapshot.width / input.width) + 'px';
                    el.style.top = (input.y * snapshot.height / input.height) + 'px';
                    el.style.width = (input.w * snapshot.width / input.width) + 'px';
                    el.style.height = (input.h * snapshot.height / input.height) + 'px';

                    el.addEventListener('change', async () => {
                        await fetch('/input', {
                            method: 'POST',
                            headers: {'Content-Type': 'application/json'},
                            body: JSON.stringify({selector: input.selector, value: el.value})
                        });
                        await refreshSnapshot();
                    });

                    container.appendChild(el);
                });
            }

            async function refreshSnapshot() {
                snapshot.src = '/snapshot?ts=' + Date.now();
                await fetchInputs();
            }

            snapshot.onload = refreshSnapshot;
            window.onresize = refreshSnapshot;

            // Click handling (optional)
            container.addEventListener('click', async e => {
                const rect = snapshot.getBoundingClientRect();
                const x = Math.round((e.clientX - rect.left) * (snapshot.naturalWidth / rect.width));
                const y = Math.round((e.clientY - rect.top) * (snapshot.naturalHeight / rect.height));
                await fetch('/click', {
                    method: 'POST',
                    headers: {'Content-Type': 'application/json'},
                    body: JSON.stringify({x, y})
                });
                await refreshSnapshot();
            });

            // Refresh every 5s
            setInterval(refreshSnapshot, 5000);
        </script>
    </body>
    </html>
    """


@app.get("/snapshot")
async def get_snapshot():
    buf = await page_instance.screenshot()
    return StreamingResponse(io.BytesIO(buf), media_type="image/png")

@app.post("/click")
async def post_click(click: ClickData):
    await page_instance.mouse.click(click.x, click.y)
    return {"status": "ok"}

@app.get("/inputs")
async def get_inputs():
    """Return positions and sizes of all visible input fields."""
    inputs = await page_instance.query_selector_all('input, textarea')
    result = []

    for inp in inputs:
        box = await inp.bounding_box()
        if box:
            # Unique selector
            selector = await inp.evaluate("el => el.getAttribute('id') || el.name || ''")
            value = await inp.input_value()
            result.append({
                "selector": selector,
                "x": box['x'],
                "y": box['y'],
                "w": box['width'],
                "h": box['height'],
                "width": (await page_instance.evaluate("() => document.body.scrollWidth")),
                "height": (await page_instance.evaluate("() => document.body.scrollHeight")),
                "value": value
            })
    return JSONResponse(result)