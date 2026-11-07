# winmessages

native imessage on windows. no mac, no iphone, no relay server. the pc itself pretends to BE a mac and logs into a real apple id.

yes aiyan it's a tauri app on the outside (react front in `app/src`, rust back in `app/src-tauri`). the whole point is underneath that.

## what actually does the work
- `app/src-tauri/src/engine.rs` — the real file. holds a fake mac hardware identity (`baked()`: a mac serial + the encrypted "gestalt" stamp apple checks), logs into the apple id, registers the pc as a mac, sends/receives real imessages.
- `vendor/rustpush` — the apple push/IDS/imessage protocol (openbubbles fork). engine.rs drives this.
- the trick: apple verifies a hardware stamp server-side. we generate a valid one in pure software (the base64 `_enc` values in `baked()`), so it works with zero apple hardware.

## the current blocker (this is "what's wrong")
sign-in reaches apple fine but apple returns **"iMessage Activation Failed" (symptom 23367)**. that is NOT a code bug — it's apple rate-limiting new imessage activations from this pc/account because we tried too many times today. it's a cooldown. every sign-in press resets the timer. fix = leave it ~24h untouched, then ONE clean try.

proof the code is right: it already delivered a real imessage to a real iphone earlier today on a fresh fake-mac identity.

## run it
```
cd app
npm install
npm run tauri dev
```
needs a vite dev server (npm run dev auto-starts via tauri) + a rust build toolchain: msvc, protoc, perl, libclang, cmake. `app/src-tauri/Cargo.toml` path-depends on `../../vendor/rustpush`, so keep the folder layout as-is.
