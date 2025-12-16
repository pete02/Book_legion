from fastapi import FastAPI
from fastapi.responses import StreamingResponse, HTMLResponse
from pydantic import BaseModel
from playwright.async_api import async_playwright
import asyncio
import io
from fastapi.responses import FileResponse
from fastapi.staticfiles import StaticFiles


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

app.mount("/static", StaticFiles(directory="/static"), name="static")
@app.get("/", response_class=HTMLResponse)
async def root():
    return FileResponse("static/index.html")

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
    """
    Return positions, sizes, and current values of all visible input fields.
    Each input includes its index for backend identification.
    """
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
            "index": idx,          # Unique index for identifying this input
            "selector": selector,   # Optional: for debugging
            "x": box['x'],
            "y": box['y'],
            "w": box['width'],
            "h": box['height'],
            "width": page_width,
            "height": page_height,
            "value": value
        })

    return result


@app.post("/input")
async def post_input(data: dict):
    index = data.get("index")
    value = data.get("value")

    inputs = await page_instance.query_selector_all("input, textarea")
    if index is None or index < 0 or index >= len(inputs):
        return {"status": "invalid"}

    el = inputs[index]
    await el.fill(value)  # Playwright fills the input
    return {"status": "ok"}