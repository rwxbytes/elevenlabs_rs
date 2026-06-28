An unofficial lib crate for [ElevenLabs' Conversational AI](https://elevenlabs.io/docs/conversational-ai/docs/introduction)

#### Examples

- [Microphone](https://github.com/rwxbytes/elevenlabs_rs/tree/master/examples/microphone/src/main.rs)

WebSocket conversations return an `AgentWebSocketSession`. The session is a stream
of server messages and also exposes `close`, `abort`, and `join` so applications
can shut down and inspect background reader/writer/audio tasks explicitly.

