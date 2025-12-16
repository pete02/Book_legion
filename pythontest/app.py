from fastapi import FastAPI
from fastapi.responses import StreamingResponse, HTMLResponse, FileResponse
from fastapi.staticfiles import StaticFiles
from fastapi import Request
from pydantic import BaseModel
from playwright.async_api import async_playwright
import asyncio
import io
import hashlib


app = FastAPI()

# -------------------
# Data Models
# -------------------
class ClickData(BaseModel):
    x: int
    y: int

class InputData(BaseModel):
    index: int
    value: str
    
client_viewport = {"width": 1280, "height": 720}
# -------------------
# Global Browser State
# -------------------
browser_instance = None
page_instance = None
update_queue = asyncio.Queue()  # SSE notification queue

async def init_browser():
    global browser_instance, page_instance, client_viewport
    playwright = await async_playwright().start()
    browser_instance = await playwright.chromium.launch(headless=True)
    page_instance = await browser_instance.new_page(viewport=client_viewport)
    await page_instance.goto("https://book.lumilukonservu.duckdns.org/")


async def monitor_page_changes():
    """Background task that watches the page DOM for changes."""
    previous_hash = None
    while True:
        try:
            html = await page_instance.content()  # get full DOM
            current_hash = hashlib.md5(html.encode()).hexdigest()

            if previous_hash is None or current_hash != previous_hash:
                previous_hash = current_hash
                await update_queue.put(True)  # notify clients to refresh snapshot
        except Exception as e:
            print("Error monitoring page:", e)

        await asyncio.sleep(0.2) 


init_lock = asyncio.Lock()
browser_ready_event = asyncio.Event()  # signals that browser + page is ready

async def ensure_browser_ready():
    global browser_instance, page_instance

    if browser_instance is None or page_instance is None:
        async with init_lock:
            # Double-check in case multiple requests hit simultaneously
            if browser_instance is None or page_instance is None:
                await init_browser()
                asyncio.create_task(monitor_page_changes())
                browser_ready_event.set()  # mark as ready
    else:
        # wait if another request is initializing
        await browser_ready_event.wait()

@app.on_event("shutdown")
async def shutdown_event():
    global browser_instance
    if browser_instance:
        await browser_instance.close()

# -------------------
# Static & Index
# -------------------
app.mount("/static", StaticFiles(directory="static"), name="static")

@app.get("/", response_class=HTMLResponse)
async def root():
    await ensure_browser_ready()

    return FileResponse("static/index.html")

# -------------------
# Snapshot
# -------------------
@app.get("/snapshot")
async def get_snapshot():
    await ensure_browser_ready()
    buf = await page_instance.screenshot()
    return StreamingResponse(io.BytesIO(buf), media_type="image/png")

@app.get("/inputs")
async def get_inputs():
    await ensure_browser_ready()
    inputs = await page_instance.query_selector_all('input, textarea')
    result = []

    page_width = await page_instance.evaluate("() => document.body.scrollWidth")
    page_height = await page_instance.evaluate("() => document.body.scrollHeight")

    for idx, inp in enumerate(inputs):
        box = await inp.bounding_box()
        if not box:
            continue

        value = await inp.input_value()
        selector = await inp.evaluate("el => el.getAttribute('id') || el.name || ''")

        result.append({
            "index": idx,
            "selector": selector,
            "x": box['x'],
            "y": box['y'],
            "w": box['width'],
            "h": box['height'],
            "width": page_width,
            "height": page_height,
            "value": value
        })
    return result


# -------------------
# Click/Input endpoints
# -------------------
@app.post("/click")
async def post_click(click: ClickData):
    await ensure_browser_ready()
    await page_instance.mouse.click(click.x, click.y)
    return {"status": "ok"}

@app.post("/input")
async def post_input(data: dict):
    await ensure_browser_ready()
    index = data.get("index")
    value = data.get("value")
    inputs = await page_instance.query_selector_all("input, textarea")
    if index is None or index < 0 or index >= len(inputs):
        return {"status": "invalid"}
    await inputs[index].fill(value)
    await update_queue.put(True)  # notify clients
    return {"status": "ok"}

# -------------------
# SSE for snapshot updates
# -------------------
@app.get("/updates")
async def updates():
    async def event_generator():
        while True:
            await update_queue.get()
            yield "data: update\n\n"
    return StreamingResponse(event_generator(), media_type="text/event-stream")



@app.post("/res")
async def post_resolution(req: Request):
    global client_viewport, page_instance
    data = await req.json()
    width = int(data.get("width", 1280))
    height = int(data.get("height", 720))
    client_viewport = {"width": width, "height": height}

    # Apply viewport size to Playwright page
    if page_instance:
        await page_instance.set_viewport_size(client_viewport)

    print(f"Client viewport set to: {width}x{height}")
    return {"status": "ok"}