from fastapi import FastAPI, HTTPException
from fastapi.responses import StreamingResponse, HTMLResponse, FileResponse
from fastapi.staticfiles import StaticFiles
from fastapi import Request
from pydantic import BaseModel
from playwright.async_api import async_playwright
import asyncio
import io
import hashlib
import time

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
init_lock = asyncio.Lock()
browser_ready_event = asyncio.Event()  # signals that browser + page is ready
last_heartbeat=None
HEARTBEAT=20
client_token=None

async def init_browser():
    print("create browser")
    global browser_instance, page_instance, client_viewport
    playwright = await async_playwright().start()
    browser_instance = await playwright.chromium.launch(headless=True)
    page_instance = await browser_instance.new_page(viewport=client_viewport)
    await page_instance.goto("https://book.lumilukonservu.duckdns.org/")

async def monitor_page_changes():
    """Background task that watches the page DOM for changes."""
    previous_hash = None

    # Wait until the browser and page are ready
    await browser_ready_event.wait()

    while True:
        try:
            if page_instance is None:
                await browser_ready_event.wait()
                previous_hash = None  # reset hash after restart
                await asyncio.sleep(2)
                continue

            html = await page_instance.content()  # get full DOM
            current_hash = hashlib.md5(html.encode()).hexdigest()

            if previous_hash is None or current_hash != previous_hash:
                previous_hash = current_hash
                await update_queue.put(True)  # notify clients to refresh snapshot

        except Exception as e:
            print("Error monitoring page:", e)

        await asyncio.sleep(0.2)



async def ensure_browser_ready():
    global browser_instance, page_instance

    if browser_instance is None or page_instance is None:
        async with init_lock:
            # Double-check in case multiple requests hit simultaneously
            if browser_instance is None or page_instance is None:
                await init_browser()
                browser_ready_event.set()  # mark as ready
    else:
        # wait if another request is initializing
        await browser_ready_event.wait()

async def monitor_heartbeat():
    global last_heartbeat, browser_instance, page_instance, client_token, browser_ready_event
    while True:
        await asyncio.sleep(5)
        if browser_instance is not None and last_heartbeat is not None and last_heartbeat <= time.time()-HEARTBEAT:
            print("close browser")
            await browser_instance.close()
            last_heartbeat=None
            browser_instance=None
            page_instance=None
            client_token=None
            browser_ready_event=asyncio.Event()



@app.on_event("startup")
async def startup_even():
    asyncio.create_task(monitor_page_changes())
    asyncio.create_task(monitor_heartbeat())

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
    return FileResponse("static/index.html")

# -------------------
# Snapshot
# -------------------
@app.get("/snapshot")
async def get_snapshot():
    await browser_ready_event.wait()
    buf = await page_instance.screenshot()
    return StreamingResponse(io.BytesIO(buf), media_type="image/png")

@app.get("/inputs")
async def get_inputs():
    await browser_ready_event.wait()
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
    await browser_ready_event.wait()
    await page_instance.mouse.click(click.x, click.y)
    return {"status": "ok"}

@app.post("/input")
async def post_input(data: dict):
    await browser_ready_event.wait()
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
    global client_viewport, page_instance, last_heartbeat, client_token
    data = await req.json()

    token = data.get("token")
    if not token or client_token is not None:
        raise HTTPException(status_code=403, detail="Forbidden: token required")
    
    # Set the client token
    client_token = token

    width = int(data.get("width", 1280))
    height = int(data.get("height", 720))
    token=data.get("token")
    
    client_viewport = {"width": width, "height": height}
    await ensure_browser_ready()
    # Apply viewport size to Playwright page
    if page_instance:
        await page_instance.set_viewport_size(client_viewport)
    last_heartbeat=time.time()
    print(f"Client viewport set to: {width}x{height}")
    return {"status": "ok"}

@app.post("/heartbeat")
async def heartbeat():
    global last_heartbeat
    last_heartbeat=time.time()
    return {"status": "ok"}

@app.post("/check")
async def check(req: Request):
    global client_token
    data= await req.json()
    token = data.get("token", "error")
    if not token or token != client_token:
        raise HTTPException(status_code=403, detail="Forbidden: token required")
    else:
        raise HTTPException(status_code=202, detail="Token ok")

    
