# Web Serial実測capture

このdirectoryには、開発用Web Serial raw viewerで実機から取得したcaptureだけを保存します。
`.bin`が受信byte列の正本で、同名の`.json`がchunk境界、接続状態、環境、SHA-256を記録します。

`device/samples/`はMicroPico vREPLの原文log専用であるため、Web Serial captureを混在させません。

## `diagnostic/`

2026-09-01にRaspberry Pi Pico H（RP2040）、MicroPython v1.29.0、Windows上のChrome 150、
`http://localhost:5173`、115200／8-N-1で取得しました。USB VID／PIDは`2e8a:0005`です。

このcaptureで起動した`button_test.py`の出力は一時的なGPIO診断文です。raw REPLの制御応答も
`.bin`へ含まれます。wire protocol v1のframeやparser fixtureとして扱いません。

### `web-serial-raw-2026-09-01T04-56-45-143Z`

- 6,672 byte、139 chunk
- SHA-256: `49a3e2772b4a02d854eaf6855af9c4aea4f6ea86eae3bcc81d0755b0f01e1557`
- 1接続。利用者操作で正常停止し、`stopConfirmed: true`
- script出力区間はoffset 701以上6,646未満
- 区間内に78状態遷移。すべて7-bit ASCIIかつ妥当なUTF-8で、78個のCRLFを観測
- GP2の保持・短い間隔の3回操作、GP2／GP3／GP5～GP8、GP2とGP5の重ね押しを記録
- このcapture単体にはbutton 3／GP4の遷移がないため、次のcaptureで再確認

### `web-serial-raw-2026-09-01T05-09-37-770Z`

- 2,626 byte、42 chunk
- SHA-256: `218bba776fc061fe751ab0ebb382f127e970066445ebf5b98c5b727bf95e03de`
- 接続1はoffset 1,748でUSBを物理的に抜き、`endReason: read-error`としてcaptureを保持
- USB再接続後の接続2はoffset 1,748から再開し、GP2の押下・解放後に正常停止
- 接続1でbutton 3／GP4（left）とbutton 4／GP5（right）を個別に確認
- 両接続のscript出力区間は7-bit ASCIIかつ妥当なUTF-8で、CRLF以外の改行は未観測

2組の実測を合わせて、GP2～GP8と7 controlの物理対応、browserでのraw受信、正常停止、
USB切断時のcapture保持、利用者操作による再接続を確認しました。

## `protocol-v1/`

Wire protocol v1専用`serial_protocol_poc.py`を実機で確認したcaptureです。metadataの時刻はUTCで、
実施日はJST 2026-09-02です。環境はRaspberry Pi Pico H（RP2040）、MicroPython v1.29.0、Windows上の
Chrome 150、`http://localhost:5173`、115200／8-N-1、USB VID／PID `2e8a:0005`です。

実測時のrepository基準はcommit
`ba9ca7a673955da47a695e2e06f4272b8313a5be`で、次のPoC変更を含むdirty worktreeでした。再現用の
source fingerprintは次のとおりです。

- `device/poc/serial_protocol_poc.py`: `b6f98c0640ef7e37553dd7389dbc6cb471407d380eda877f80c43e077d8c2108`
- `client/src/device-poc/useDeviceSerialPoc.ts`: `7e9e6fddee3c438c319007ca7ed925fa3cf0a4d9cd7950e32c67b6f5c891f168`
- `client/src/device-poc/microPythonRawRepl.ts`: `2bf2c2b2854a0ddc13fb51c147aa855a4cf28a0f6b165dbe5675185f113d02c1`
- `client/src/device-poc/capture.ts`: `a230bcd83856461e89f346a7071ecf607ea90c10d401512567825e5d8ceab662`
- `client/src/device-poc/types.ts`: `1e4ff4ecb5d7d6b6c6422365079421c0088c040aa6a8019b482ab94dc1a3923b`
- `client/src/components/device/SerialRawViewer.vue`: `7186a1bb82c8243fd39219b86ebdf6e4da2ebbde018d4a17f9e253f8cda0792b`

### `web-serial-raw-2026-09-01T15-46-21-325Z`

- 1,133 byte、50 chunk
- SHA-256: `1fc5782324018d2c1d4b7dd094b1e5dc127ace2e2c89edcadcb80a681d000ac2`
- 3接続すべて利用者操作で正常停止し、`stopConfirmed: true`
- 接続1のWire v1区間はoffset 29以上172未満で3 frame、接続2は204以上204未満で0 frame
- 正式な操作matrixを採取した接続3のWire v1区間はoffset 236以上1,130未満で18 frame
- 接続3で全7 controlの`short_press`と`up`の`long_press`を観測
- 接続3の実測順は、`up short`, `up long`, `down short`, `left short`, `down short`,
  `right short`, `red short`, `yellow short`, `green short` 2回、`up short` 3回、
  `right short` 4回、`up long`
- 操作者記録では、接続開始前からGP2をLOWにして保持し、開始後の保持中と最初のreleaseでframeなし、
  次の短押しで最初の`up short`を観測
- 保持中のrepeatなし、短い間隔の3操作、GP2を保持しながらGP5を4回操作した独立入力を確認
- Wire v1区間はすべて7-bit ASCIIかつ妥当なUTF-8で、frame終端はCRLF
- 1 frameが複数の実測read chunkへ分割される境界を含む

接続1と接続2もraw実測の一部として加工せず保存していますが、上記の正式な操作matrixとは分けて扱います。

### `web-serial-raw-2026-09-01T15-55-41-921Z`

- 177 byte、14 chunk
- SHA-256: `f9fe6de3c5ca1064a579c91ed8af3ef253f917188b2bf9ae7d8bfa51deac99b5`
- 接続1のWire v1区間はoffset 29以上77未満で`up short` 1 frame
- 接続1の読取り中にUSBを物理的に抜き、`endReason: read-error`、`stopConfirmed: false`で77 byteを保持
- BOOTSELなしでUSBを再接続し、利用者gestureから再接続
- 接続2のWire v1区間はoffset 126以上174未満で`up short` 1 frame
- 接続2は利用者操作で正常停止し、`stopConfirmed: true`、capture全体の終了offsetは177

実測`.bin`はraw REPL制御byteも含むため、Wire v1 parserにはmetadataのconnectionごとに
`[scriptActiveObservedOffset, stopRequestedOffset)`、切断時は
`[scriptActiveObservedOffset, endedOffset)`だけを元のchunk境界で渡します。

実機が送信しないinvalid／overlong／LF単独／partial frameは、
`client/src/device-poc/__fixtures__/serial-protocol-v1-valid.jsonl`とsynthetic unit testで実機sampleから
区別します。

## 完全性の確認

各`.json`について、次をすべて満たすことを確認済みです。

- `.bin`の実byte数と`totalBytes`が一致する
- `chunks[].length`の合計が`totalBytes`と一致する
- `chunks[].offset`が0から連続する
- `.bin`のSHA-256が`rawSha256`と一致する

raw REPL制御を除くscript出力区間は、各connectionの`scriptActiveObservedOffset`相当の開始offsetから
`stopRequestedOffset`または`endedOffset`までです。diagnostic captureのschema v1では、このfield名は
`buttonTestActiveObservedOffset`です。
