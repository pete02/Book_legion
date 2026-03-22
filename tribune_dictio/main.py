import perth
from perth.perth_net.perth_net_implicit.perth_watermarker import PerthImplicitWatermarker

perth.PerthImplicitWatermarker = PerthImplicitWatermarker

import io
import torch
import torchaudio as ta
from chatterbox.tts import ChatterboxTTS
from bottle import Bottle, request, HTTPResponse, run
import os
import sys
import re
import time
import contextlib

app = Bottle()

device = "cuda"
print("using: " + device)
model = ChatterboxTTS.from_pretrained(device=device)
torch.set_num_threads(1)
torch.set_num_interop_threads(1)
torch.set_grad_enabled(False)

print(f"PyTorch using {torch.get_num_threads()} threads")
VOICEACTOR_DIR = "/voiceactors"
CUMULATIVE_AUDIO_GEN_TIME = 0.0
MODEL_READY = False


def split_text_into_sentences(text: str) -> list[str]:
    sentence_endings = re.compile(r'(?<=[.!?])\s+')
    sentences = sentence_endings.split(text.strip())
    return [s.strip() for s in sentences if s.strip()]


@app.get("/health")
def health():
    if not MODEL_READY:
        return HTTPResponse("Model loading", status=503)
    return HTTPResponse("OK", status=200)


@app.post("/tts")
def tts():
    global CUMULATIVE_AUDIO_GEN_TIME

    if not MODEL_READY:
        return HTTPResponse("Model not ready", status=503)

    print("[tts] Audio requested")

    req = request.json
    if not req:
        print("[tts] Error: Missing JSON body")
        return HTTPResponse("Missing JSON body", status=400)

    text = req.get("text")
    audio = req.get("voice")
    cfg_weight = float(req.get("cfg_weight", 0.5))
    exaggeration = float(req.get("exaggeration", 0.5))
    temperature = float(req.get("temperature", 0.8))

    if os.getenv("DEBUG", "").lower() in ("1", "true", "yes"):
        print(f"[tts] Text: {text}")

    audio_prompt_path = os.path.join(VOICEACTOR_DIR, f"{audio}.wav")
    if not os.path.exists(audio_prompt_path):
        print(f"[tts] Error: Voice '{audio}' not found at {audio_prompt_path}")
        return HTTPResponse(f"Voice '{audio}' not found", status=400)

    try:
        torch.cuda.empty_cache()
        sentences = split_text_into_sentences(text)

        print(f"[tts] Generating ({len(sentences)} sentence(s))...")
        start = time.perf_counter()

        with open(os.devnull, "w") as devnull, contextlib.redirect_stderr(devnull):
            wav_segments = [
                model.generate(
                    s,
                    audio_prompt_path=audio_prompt_path,
                    cfg_weight=cfg_weight,
                    temperature=temperature,
                    exaggeration=exaggeration,
                )
                for s in sentences
            ]

        wav = torch.cat(wav_segments, dim=-1)

        buffer = io.BytesIO()
        wav16 = (wav * 32767.0).clamp(-32768, 32767).short()
        ta.save(buffer, wav16.cpu(), model.sr, format="mp3", encoding="MP3")
        buffer.seek(0)

        end = time.perf_counter()
        CUMULATIVE_AUDIO_GEN_TIME += end - start
        print(f"[tts] Responding (took {end - start:.2f}s, cumulative={CUMULATIVE_AUDIO_GEN_TIME:.2f}s)")

        return HTTPResponse(body=buffer.read(), content_type="audio/mp3")

    except Exception as e:
        print(f"[tts] Error during generation: {e}")
        return HTTPResponse(f"Generation failed: {e}", status=500)


def check_voiceactors_dir():
    if not os.path.isdir(VOICEACTOR_DIR):
        print(f"ERROR: Voice actor directory '{VOICEACTOR_DIR}' does not exist.", file=sys.stderr)
        sys.exit(1)

    wav_files = [f for f in os.listdir(VOICEACTOR_DIR) if f.lower().endswith(".wav")]
    if not wav_files:
        print(f"ERROR: No .wav files found in '{VOICEACTOR_DIR}'. At least one voice actor is required.", file=sys.stderr)
        sys.exit(1)


if __name__ == "__main__":
    print("starting")
    check_voiceactors_dir()
    MODEL_READY = True
    print("[tts] Model ready, accepting requests")
    run(app, host="0.0.0.0", port=8000, server="paste")