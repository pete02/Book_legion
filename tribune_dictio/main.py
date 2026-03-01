import perth
from perth.perth_net.perth_net_implicit.perth_watermarker import PerthImplicitWatermarker

perth.PerthImplicitWatermarker =PerthImplicitWatermarker

import io
import torch
import torchaudio as ta
from chatterbox.tts import ChatterboxTTS
from bottle import Bottle, request, HTTPResponse, run
import os
import sys
import re
import time


app = Bottle()

# Load model once

device = "cuda"
print("using: "+device)
model = ChatterboxTTS.from_pretrained(device=device)
torch.set_num_threads(1)
torch.set_num_interop_threads(1)
torch.set_grad_enabled(False)


print(f"PyTorch using {torch.get_num_threads()} threads")
VOICEACTOR_DIR = "/voiceactors"
VOICEACTOR_DIR = "/voiceactors"
CUMULATIVE_AUDIO_GEN_TIME=0.0

def split_text_into_sentences(text: str) -> list[str]:
    """
    Split text into sentences using ., !, or ? as delimiters.
    Keeps the punctuation at the end of each sentence.
    """
    sentence_endings = re.compile(r'(?<=[.!?])\s+')
    sentences = sentence_endings.split(text.strip())

    sentences = [s.strip() for s in sentences if s.strip()]
    
    return sentences

@app.post("/tts")
def tts():
    global CUMULATIVE_AUDIO_GEN_TIME 
    req = request.json
    if not req:
        return HTTPResponse("Missing JSON body", status=400)
    
    torch.cuda.empty_cache()
    
    text= req.get("text")

    audio = req.get("voice")
    cfg_weight = float(req.get("cfg_weight", 0.5))
    exaggeration = float(req.get("exaggeration", 0.5))
    temperature = float(req.get("temperature", 0.8))

    audio_prompt_path = os.path.join(VOICEACTOR_DIR, f"{audio}.wav")
    print(audio_prompt_path)
    if not os.path.exists(audio_prompt_path):
        raise HTTPResponse(status_code=400, detail=f"Voice '{audio}' not found")

    start = time.perf_counter()

    debug = os.getenv("DEBUG", "").lower() in ("1", "true", "yes")
    if debug:
        print(text)
    sentences = split_text_into_sentences(text)

    wav_segments = [model.generate(s,
        audio_prompt_path=audio_prompt_path,
        cfg_weight=cfg_weight,
        temperature=temperature,
        exaggeration=exaggeration
        ) for s in sentences]

    wav = torch.cat(wav_segments, dim=-1)

    # Stream audio back
    buffer = io.BytesIO()
    wav16 = (wav * 32767.0).clamp(-32768, 32767).short()
    ta.save(buffer, wav16.cpu(), model.sr, format="mp3", encoding="MP3")
    buffer.seek(0)
    end = time.perf_counter()
    CUMULATIVE_AUDIO_GEN_TIME += (end - start)

    print(
        f"[tribune_dictio] cumulative_audio_gen_time="
        f"{CUMULATIVE_AUDIO_GEN_TIME:.4f}s"
    )
    
    return HTTPResponse(body=buffer.read(), content_type="audio/mp3")


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
    run(app, host="0.0.0.0", port=8000, server="paste")

