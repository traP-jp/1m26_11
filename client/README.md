# Client

Vue 3、Vite、TypeScript、pnpm で構成したフロントエンドです。タスクはリポジトリルートの `mise.toml` から実行します。

## Setup

```sh
mise install
mise run client-install
```

## Development

backend、MariaDB、Adminer と Vite dev server をまとめて起動します。

```sh
mise run dev
```

Vite のみ起動する場合は次を実行します。developmentではMSWが既定で有効になり、backendなしでAPIを利用できます。

```sh
mise run client-dev
```

## Device Web Serial PoC

development serverには、Raspberry Pi Pico Hから受信したraw byteを観測する診断画面があります。
production buildにはこのrouteを含めません。

```text
http://localhost:<Viteが表示したport>/device-poc
```

この画面はraw captureを取得するためのもので、受信frameを操作へ変換する製品画面ではありません。
専用firmware PoCが送るWire v1の規範は
[`device/SERIAL_PROTOCOL.md`](../device/SERIAL_PROTOCOL.md)です。PoC用のport open設定として
`115200 / 8-N-1`を使用しますが、この値はpayload／framing契約とは分離します。

### 実機へ接続する前提

- Web Serial対応のdesktop Chromium browserを使用します。
- HTTPSまたは`localhost`のsecure contextから開きます。
- `device/`をMicroPico projectとして開き、先に`Upload Project`を完了させます。
- Pico上のrootに`button_firmware.py`、`main.py`、`serial_protocol_poc.py`があることを確認します。
  `Run current file on Pico`は実行しません。
- MicroPico vREPLを閉じるだけでなく、`MicroPico: Disconnect`でserial portを解放します。
- 同じserial portをMicroPicoとbrowserから同時に開きません。

Windows browserとWSL上のMicroPicoを切り替える場合は、MicroPicoをDisconnectした後、Windows側で
PicoをWSLからdetachしてからbrowserを開きます。

```powershell
usbipd list
usbipd detach --busid <BUSID>
```

### 接続とcapture

1. `mise run client-dev`を実行します。
2. 対応browserでdevelopment URLの`/device-poc`を開きます。
3. `Connect & start`を押し、Pico Hを選択します。
4. 画面が`running`になってから物理スイッチを操作します。
5. `Stop & disconnect`で`serial_protocol_poc.py`を停止し、portを解放します。
6. `Download capture.bin`と`Download capture.json`を順に押してcaptureを保存します。

画面は接続後に自動実行中の`main.py`を停止してMicroPython raw REPLへ入り、Upload済み
`/serial_protocol_poc.py`をPoC専用commandで起動します。このentrypointはproduction `main.py`と同じ
`button_firmware.py`を使用しますが、電源投入時の自動起動そのものはdevice側の手順で確認します。
`capture.bin`にはscript出力だけでなくraw REPLの受信応答も含まれます。`capture.json`は接続ごとの
script path、bootstrap／script起動要求／起動継続観測時のoffset、read chunk境界、環境、総byte数、
SHA-256を記録します。
raw byteの正本は`.bin`であり、画面のUTF-8欄は参考表示です。

USBを抜いた場合も受信済みcaptureは画面に残します。再接続は自動では行わず、Picoを挿し直してから
再度`Connect & start`を押します。MicroPicoへ戻す場合はbrowser側でStopした後、必要に応じてPicoを
WSLへattachし直してから`MicroPico: Connect`を実行します。詳細なdevice setupは
[`device/README.md`](../device/README.md)を参照してください。

### 実機確認で記録する内容

実機確認では、想定結果ではなく実際に観測した結果だけを記録します。

- 実施日時、OSとbrowserのversion、Pico H、MicroPython version、clientとdeviceのrevision
- Viteを開いたorigin、Web Serial対応判定、PoC open設定、取得できたUSB VID／PID
- MicroPico Disconnectと、WSL利用時のdetachを実施したこと
- GP2～GP8の各short pressと、対応する7 controlのJSON frame
- 700 ms未満／以上の境界、保持中にrepeatが出ないこと、release時だけ1 frame出ること
- 起動時に押下中のbuttonをreleaseまで無視することと、複数スイッチを重ねた操作
- Stop後の再接続、読取り中のUSB切断、USB再接続後の再実行
- `.bin`と`.json`の両方を保存でき、JSONの`totalBytes`とchunk length合計が一致すること
- `.bin`のSHA-256がJSONの`rawSha256`と一致すること

SHA-256は、保存した環境に応じて`sha256sum <capture.bin>`またはPowerShellの
`Get-FileHash <capture.bin> -Algorithm SHA256`で照合できます。

実機Web Serial captureはローカルでの確認にだけ使用し、repositoryへcommitしたりPRへ添付したり
しません。実機で行った操作、環境、観測結果だけを[`device/README.md`](../device/README.md)へ記録します。

1行を複数chunkへ分ける場合、複数行を1 chunkへまとめる場合、invalid UTF-8／JSON／schema、overlong
frameは実機が送ったdataと偽らず、`src/device-poc/__tests__/serialProtocolPoc.spec.ts`の
contract-synthetic testとして区別します。`SerialProtocolPocParser`はWire契約の適合確認用PoCであり、
製品画面の操作列へはまだ接続していません。

mock responseは`../openapi/openapi-v1.yaml`、`../openapi/examples/`、`../openapi/scenarios/p0-cases.yaml`を直接読みます。response payloadをclient内に複製していないため、OpenAPIのexampleとscenarioがmockの正本です。

実backendへ接続する場合は`.env.example`を参考に`client/.env.local`を作成し、`VITE_ENABLE_MSW=false`を設定してください。その場合、Viteは`/api`と`/openapi.yaml`を`API_PROXY_TARGET`（既定は`http://127.0.0.1:8080`）へproxyします。特定の状態からmockを開始する場合は、`VITE_MSW_SCENARIO`へscenarioのcase id（例: `demo_login_and_logout`、`query_correct`）を指定します。production buildではMSWは常に無効です。

## Component preview (Histoire)

Histoireを使うと、アプリ全体を起動せずにVueコンポーネントを状態ごとに表示できます。

リポジトリルートで次を実行し、ターミナルに表示されたURLをブラウザで開きます。

```sh
mise run client-histoire
```

プレビューは対象コンポーネントと同じディレクトリに`*.story.vue`として作成します。`Story`がコンポーネントのまとまり、`Variant`が状態ごとの表示です。

```vue
<!-- src/components/ExampleButton.story.vue -->
<script setup lang="ts">
import ExampleButton from './ExampleButton.vue'
</script>

<template>
  <Story title="Components/ExampleButton">
    <Variant title="Default">
      <ExampleButton>Default</ExampleButton>
    </Variant>

    <Variant title="Disabled">
      <ExampleButton disabled>Disabled</ExampleButton>
    </Variant>
  </Story>
</template>
```

`src/histoire.setup.ts`が`src/assets/main.css`を読み込むため、アプリと同じTailwind CSSとグローバルスタイルが適用されます。PiniaやVue Routerなどのアプリプラグインが必要になった場合も、このsetupファイルでHistoireのVueアプリへ登録します。

静的サイトとしてのビルドと、その成果物の確認には次を使います。

```sh
mise run client-histoire-build
mise run client-histoire-preview
```

`mise run client-check`にもHistoireのproduction buildが含まれ、すべてのstoryをビルドできることを検証します。

現在はVite 8対応を取り込んだ正式版が未公開のため、Histoireを`1.0.0-beta.1`へ固定し、`pnpm-workspace.yaml`では同版に限ってVite 8のpeer依存警告を許容しています。この設定自体が互換性を保証するものではありませんが、現在の環境ではdevelopment server、build、previewの動作を確認済みです。build時にbeta版由来の`IMPORT_IS_UNDEFINED`警告が表示されることがありますが、末尾に`Built`と表示されればbuildは成功しています。Histoire更新時はこの許容設定と注意書きが不要になっていないか確認してください。

## Checks

```sh
mise run client-build
mise run client-typecheck
mise run client-lint
mise run client-format-check
mise run client-test
mise run client-check
```

自動修正には `mise run client-lint-fix` と `mise run client-format` を使用します。

## OpenAPI生成型

`src/generated/api.d.ts`は`../openapi/openapi-v1.yaml`から生成される通信境界の型です。直接編集せず、契約変更時はリポジトリルートで次を実行します。

```sh
mise run openapi-generate-client
```

API呼び出しでは`paths`または`components`からrequest／response型を参照し、mock payloadは`../openapi/examples/`の共通fixtureを使用してください。
