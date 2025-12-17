import perth
from perth.perth_net.perth_net_implicit.perth_watermarker import PerthImplicitWatermarker

perth.PerthImplicitWatermarker =PerthImplicitWatermarker

from chatterbox.tts import ChatterboxTTS

ChatterboxTTS.from_pretrained(device="cpu")