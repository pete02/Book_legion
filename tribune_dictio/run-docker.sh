docker run --device nvidia.com/gpu=all -p 8000:8000 -v ./voiceactors:/voiceactors lumilukko/tribune_dictio

curl -v -X POST http://192.168.88.244:8888/tts      -H "Content-Type: application/json"      -d '{
    "text": "Hello, this is a test.",
    "voice": "sofia",
    "cfg_weight": 0.4,
    "temperature": 0.9
    }'      --output output.wav