A webapp, which allows for both reading and listening of epubs. The audio is generated via a TTS api live, as the user is listening to it.
A webapp, which allows for both reading and listening of epubs. The audio is generated via a TTS api live, as the user is listening to it.

In progress, to be documented better

Tribune_logisitca serves as the backend, which is written in Go. There you will find an API-skeleton.md, which documents the API usage of the backend.
Lector holds the the in progress frontend, which is written in Rust dioxus. Old frontend is placed in lector_old. No specific README yet.
Tribune dictio is a python docker api, which hosts a Chatterbox TTS. Place a 15 sec audio clip to /voiceactors, name actor.wav, and you can use it. You can find an example curl command in run-docker.sh
Tribune archivum is to be used to  add new books to the library. THis is not yet updated for the V 0.3 version.

Tribune porta can be used to turn an epub into an .mp3 file, using the Tribune dictio. Be warned, this might take 24h, and will be really annoying, if the server is in the same room with you


This project will need a dedicated graphics card to run, or external, web TTS api access. Tested and working with 1070 TI
