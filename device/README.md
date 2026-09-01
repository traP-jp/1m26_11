# Raspberry Pi Pico H device PoC

Raspberry Pi Pico H（RP2040）に接続した機械式スイッチを確認するdevice PoCです。用途を混同しない
ように、次の2 scriptを分けています。

- `button_test.py`: MicroPico vREPLでraw GPIO遷移とbounceを観測する一時診断用
- `serial_protocol_poc.py`: 確定したWire v1どおりにdebounceとshort／long pressを判定する実機確認用

`button_test.py`の人向け表示は最終serial protocolではなく、本番用debounce、repeat、長押し判定を
実装しません。Wire v1の規範は[`SERIAL_PROTOCOL.md`](SERIAL_PROTOCOL.md)を参照してください。

## 現在の確認状況

| 項目 | 状態 |
| --- | --- |
| Pico H が BOOTSEL の `RPI-RP2` として認識される | 2026-08-27 確認済み |
| `RPI_PICO-20260824-v1.29.0.uf2` の書込み後に USB serial device として認識される | 2026-08-27 確認済み |
| GP2／スイッチ1の押下・保持・解放・短い間隔の連続操作 | 2026-08-27 確認済み |
| GP3～GP8／スイッチ2～7 | 2026-08-27 確認済み |
| USB抜き差し後のMicroPico再接続・再実行 | 2026-08-27 確認済み（WSLへ再attach） |
| 診断scriptだけだった時点の`Upload Project`転送内容 | 2026-08-27 確認済み |
| 2 script構成の`Upload Project`転送内容 | 2026-09-02 確認済み |
| スイッチ1～7とgame controlの対応 | 2026-08-29 確定 |
| `button_test.py`のWeb Serial raw受信・切断・再接続 | 2026-09-01 確認済み |
| Wire v1契約と専用firmware／parser PoC | 2026-09-02 実装・実機確認済み |
| Issue #92手順の別担当による再現 | 未実施 |

Wire v1は`serial_protocol_poc.py`を実機へ転送し、Web Serialでshort／long press、全7 control、
切断・再接続まで確認済みです。ただしIssue #92の完了条件にある別担当による再現はまだ実施していないため、
その確認前にIssue全体を完了扱いにしません。

## 使用機材

- Raspberry Pi Pico H（RP2040。Pico 2ではない）
- 光らない機械式スイッチ 7個
- breadboardまたは同等の配線基板
- jumper wire
- data通信対応micro-USB cable
- VS Code
- Python 3.10以上（MicroPicoを実行するhost側）
- MicroPico拡張機能 `paulober.pico-w-go`（初回確認はv4.3.4）
- Raspberry Pi Pico用MicroPython firmware

LED付きスイッチやLED用電源は想定していません。

## ファイル構成

```text
device/
├── .micropico                 # MicroPico project marker
├── .vscode/
│   ├── extensions.json        # MicroPicoだけを推奨
│   └── settings.json          # portableな転送設定だけを共有
├── poc/
│   ├── button_test.py         # 手動実行するraw GPIO確認用script
│   └── serial_protocol_poc.py # Wire v1実機確認用script
├── samples/                   # 実機vREPLで取得したlogだけを保存
├── README.md
└── SERIAL_PROTOCOL.md         # device／frontend間のWire v1規範
```

`main.py`と`boot.py`は置きません。Picoへ転送してもUSB接続時には自動起動せず、検証者が
用途に合うscriptを明示的に実行します。Web Serial raw viewerはraw REPLから
`/serial_protocol_poc.py`を起動します。

`samples/`には実機vREPLから取得した原文だけを保存します。想定出力や手作りのsampleは追加しません。

## ピン割当

Pico HをUSB connectorが上になる向きで見たときの、board上の物理pin番号も併記します。

| スイッチ | GPIO | 物理pin | game control | サイズ | もう一方の端子 |
| --- | --- | --- | --- | --- | --- |
| 1 | GP2 | 4 | `up` | — | 共通GND |
| 2 | GP3 | 5 | `down` | — | 共通GND |
| 3 | GP4 | 6 | `left` | — | 共通GND |
| 4 | GP5 | 7 | `right` | — | 共通GND |
| 5 | GP6 | 9 | `red` | small | 共通GND |
| 6 | GP7 | 10 | `yellow` | middle | 共通GND |
| 7 | GP8 | 11 | `green` | big | 共通GND |
| 共通GND | GND | 8 | — | — | 全スイッチで共有 |

GPIOは `Pin.IN` と内部 `Pin.PULL_UP` で初期化します。

- 未押下: HIGH（`level=1`、`RELEASED`）
- 押下: LOW（`level=0`、`PRESSED`）

この物理スイッチとgame controlの対応は2026-08-29に確定しました。`button_test.py`はraw GPIO診断用の
ため、引き続きbutton番号とGPIOだけを表示し、control名は出力しません。`serial_protocol_poc.py`は
Wire v1のcontrol名をJSON frameとして出力します。

公式pinout: [Raspberry Pi Pico pinout](https://datasheets.raspberrypi.com/pico/Pico-R3-A4-Pinout.pdf)

## 配線

短絡や誤配線を避けるため、配線を変える前にUSB cableを抜いてください。

1. 最初はスイッチ1個だけを使用します。
2. スイッチ1の一方の端子をGP2（物理pin 4）へ接続します。
3. もう一方の端子をGND（物理pin 8）へ接続します。
4. GP2とGND以外へ接続していないことを確認してからUSBを接続します。
5. GP2の検証成功後に限り、スイッチ2～7を表のGP3～GP8へ追加します。
6. 各スイッチのGND側は、物理pin 8へつながる同じGND railを共有します。

内部pull-upを使うため、外付けpull-up抵抗は不要です。スイッチを `VBUS`、`VSYS`、`3V3(OUT)`へ
接続しないでください。

## MicroPythonの導入

このPoCではPico H用として、Pico 2／Pico W用ではなく `RPI_PICO` firmwareを使います。
2026-08-27の初回確認ではstable release
`RPI_PICO-20260824-v1.29.0.uf2` を使用しました。preview buildは使用していません。

1. [MicroPythonのRPI_PICO download page](https://micropython.org/download/RPI_PICO/)から、
   Pico用stable `.uf2` を取得します。
2. PicoからUSB cableを抜きます。
3. `BOOTSEL`を押したままUSB cableを接続します。
4. `RPI-RP2` driveが現れたら`BOOTSEL`を離します。
5. `.uf2`を`RPI-RP2`へcopyします。
6. copy完了後、Picoが自動再起動してUSB serial deviceとして現れるまで待ちます。

通常の接続や再接続では`BOOTSEL`を押しません。`BOOTSEL`が必要なのはfirmwareを書き込むときです。
導入手順の正本は
[Raspberry Pi公式MicroPython documentation](https://www.raspberrypi.com/documentation/microcontrollers/micropython.html)
です。

接続時のvREPL banner、またはvREPLで次を実行して、実際のMicroPython buildとboardを確認します。

```python
import sys
print(sys.implementation)
```

このPoCではvREPL bannerを`device/samples/20260827-firmware-vrepl.log`へ保存しました。UF2 binary自体は
repositoryへ追加しません。

## MicroPicoの初期設定

1. repository rootではなく、`device/`だけをVS Codeのfolderとして開きます。
2. hostへPython 3.10以上を導入し、VS Codeでそのinterpreterを選択します。
3. 推奨表示からMicroPicoを入れるか、拡張機能ID `paulober.pico-w-go` を指定してinstallします。
   このPoCを確認したMicroPicoはv4.3.4です。別versionでは接続・転送手順を再確認します。
4. `.micropico`と`.vscode/settings.json`は共有済みなので、
   `MicroPico: Initialize MicroPico project`の再実行は不要です。
5. PicoをBOOTSELなしでUSB接続します。
6. 自動接続されない場合はcommand paletteから`MicroPico: Connect`を実行します。
7. 複数台のPicoがある場合は`MicroPico: Switch Pico`で現在使うportを選びます。

VS CodeをWSL／Dev Container／SSHなどのremote windowで開く場合、拡張機能を実行する側からUSB serial
deviceが見える必要があります。WindowsのCOM portを使う場合は、USBをWSLへ転送していない限り、
Windows local windowで`device/`を開いてください。

### WSLから接続する場合

WindowsからWSLへUSBを転送する場合は、Windows側へ`usbipd-win` 5.0.0以上を導入します。WSLのshellを
開いた状態で、管理者PowerShellから現在のPicoのBUSIDを確認し、最初の1回だけshareします。

```powershell
usbipd list
usbipd bind --busid <BUSID>
```

続いて通常権限のPowerShellからWSLへattachします。

```powershell
usbipd attach --wsl --busid <BUSID>
```

BUSIDは環境ごとの値なのでrepositoryへ保存しません。Picoを物理的に抜き差しした後はattach状態が失われる
ため、現在のBUSIDを確認して`usbipd attach`を再実行します。WSL側ではserial deviceを確認します。

```sh
ls -l /dev/ttyACM*
```

現在のuserに読み書き権限がない場合は`dialout` groupなど、その環境のserial device権限を設定し、WSLと
VS CodeのWSL windowを開き直します。`/dev/ttyACM*`が見えて読み書き可能になってから`device/`を開きます。

COM番号、`/dev/tty*`、絶対path、`micropico.manualComDevice`は共有設定へ保存しません。port名はUSBの
抜き差しやPCによって変わるため、その都度現在のPicoを選択します。

### 共有するMicroPico生成ファイル

このrepositoryでは、MicroPicoが生成するもののうち次だけを可搬ファイルとして残します。

- `.micropico`: project識別marker
- `.vscode/extensions.json`: MicroPicoの推奨
- `.vscode/settings.json`: `poc/`だけを転送する設定とvREPL用のportable設定

MicroPicoのInitializeを再実行すると、`python.analysis.typeshedPaths`や
`python.analysis.extraPaths`へ `~/.micropico-stubs/...` が追加される場合があります。これらのlocal stub
path、legacyな`.vscode/Pico-W-Stub`、COM設定はcommitしません。補完用stubを使う場合は、各PCの
VS Code User／Profile設定で管理します。

## GP2を最初に確認する

第1段階では`button_test.py`をスイッチ1／GP2だけに限定して、次の手順を実施しました。GP2成功後の
現在のscriptはスイッチ1～7／GP2～GP8を監視します。

1. 「配線」の手順どおり、スイッチ1だけをGP2とGNDの間へ接続します。
2. VS Codeで`device/poc/button_test.py`を開きます。
3. command paletteから`MicroPico: Run current file on Pico`を実行します。
4. MicroPico vREPLに開始messageと、GP2の初期状態が表示されることを確認します。
5. スイッチ1を1回押し、LOW／`PRESSED`への遷移を確認します。
6. 押したまま保持し、GPIO変化がなければ追加のedge行が出ないことを確認します。
7. 解放し、HIGH／`RELEASED`への遷移を確認します。
8. 短い間隔で複数回押下・解放し、観測された遷移を確認します。
9. `MicroPico: Stop execution`で停止します。

各scan後に1 ms sleepし、直前と異なるlevelをすべて表示します。実際のscan間隔は処理時間を含むため
1 ms以上で変動し、このsleepはdebounce時間ではありません。機械的bounceがpollingで観測されれば、
`PRESSED`と`RELEASED`の短い反転列が加工
されずに表示されます。ただし、このscriptはlogic analyzerではないため、pollingより速い電気的edgeを
すべて捕捉する保証はありません。bounceが表示されなくても、bounceが存在しないとは断定しません。

出力field、順序、表記、timestamp単位は診断用の仮仕様です。parser fixtureや最終serial protocolとして
使用しないでください。

## GP3～GP8へ広げる

GP2の初期状態、押下、保持、解放、短い間隔の操作が成功し、実測logを保存してから行います。

1. USB cableを抜きます。
2. スイッチ2～7をピン割当表どおりGP3～GP8と共通GNDへ配線します。
3. GP2成功後にスイッチ1～7へ拡張済みの`button_test.py`を再実行します。
4. 各スイッチを1個ずつ押下・解放し、対応するbutton番号とGPIOだけが変化することを確認します。
5. 複数スイッチを短い間隔で操作し、raw遷移を確認します。

GP2未確認のまま監視対象や配線を広げません。

## Wire v1専用PoC

`poc/serial_protocol_poc.py`は[`SERIAL_PROTOCOL.md`](SERIAL_PROTOCOL.md)の実機確認用です。
GP2～GP8を1 ms sleepを挟むpollingで読み、buttonごとに20 ms連続して安定したlevelだけを状態遷移として
採用します。debounce済みの押下から解放までが700 ms未満なら`short_press`、700 ms以上なら
`long_press`を解放時に1 frameだけ出力し、保持中のrepeatは出力しません。

出力は次のcompact JSONと改行だけです。bannerやraw GPIO診断行は混在させません。MicroPythonの
`print`によるCRLFもfrontendが受理します。

```text
{"v":1,"control":"up","gesture":"short_press"}
```

起動時からLOWのbuttonは、20 ms以上安定してHIGHへ戻るまで入力受付を開始しません。各buttonの状態は
独立しており、重なった操作もreleaseが確定したstream順に1 frameずつ出力します。

MicroPico vREPLで単体確認する場合はlocalの`device/poc/serial_protocol_poc.py`を開き、
`MicroPico: Run current file on Pico`を実行します。Web Serial確認では`Upload Project`だけを実行し、
MicroPico側からscriptを開始せず、browserの`Connect & start`に起動させます。

## Upload Project

`device/.vscode/settings.json`は、`Upload Project`の送信元を`device/poc/`、file typeを`.py`だけに
固定します。現在の転送対象は`button_test.py`と`serial_protocol_poc.py`で、Pico filesystem rootへ
同名で配置されます。`README.md`、`SERIAL_PROTOCOL.md`、`samples/`、`.vscode/`、`.micropico`は
転送対象外です。

1. MicroPicoが対象Picoへ接続済みであることを確認します。
2. `MicroPico: Upload project to Pico`を実行します。
3. `MicroPico: Toggle Virtual File System (reloads UI and closes existing vREPLs)`で、転送された
   `/button_test.py`と`/serial_protocol_poc.py`を確認します。このcommandは既存vREPLを閉じるため、
   実行中のscriptを先に停止します。
4. READMEやsample logが新たに転送されていないことを確認します。
5. Upload済みcopy自体を確認する場合は、Virtual File System上の対象scriptを開き、
   `MicroPico: Run current file on Pico`で実行します。Web Serial確認前には停止します。
6. 通常の開発時はlocalの使用するscriptを開き、`MicroPico: Run current file on Pico`で実行します。

`Upload Project`は送信元を限定しますが、Pico上の既存fileを全削除する操作ではありません。再利用する
Picoでは、既存の`main.py`や`boot.py`が自動起動しないかをfilesystem表示で確認してください。既存fileを
削除する場合は、対象を確認してから個別に行います。

repository rootをMicroPico projectとして開くと、`device/`自体や`device/samples/`まで誤って転送対象に
するlocal設定が生成される場合があります。必ず`device/`だけを別のVS Code windowで開き、repository rootの
`.vscode/settings.json`へMicroPico設定を保存しません。2026-09-02の最終確認ではPico rootが次の2 fileだけ
であることをREPLの`os.listdir()`で確認しました。

```text
['button_test.py', 'serial_protocol_poc.py']
```

## USBを抜き差しした後

1. 実行中なら`MicroPico: Stop execution`で停止します。
2. 自動再接続を試す場合は`MicroPico: Disconnect`を実行せず、USB cableを抜きます。
3. 数秒待ってから、`BOOTSEL`を押さずにUSB cableを接続します。
4. WSL利用時は、Windows側から現在のPicoをWSLへ再attachします。
5. 自動再接続を待ちます。接続しなければ`MicroPico: Connect`を実行します。
6. 意図的に`MicroPico: Disconnect`した場合、自動再接続は待たず`MicroPico: Connect`を実行します。
7. portが変わった、または複数台ある場合は`MicroPico: Switch Pico`で現在のPicoを選びます。
8. raw配線確認では`button_test.py`を再実行し、同じGPIO遷移を確認します。Wire v1確認では
   browserの`Connect & start`から`serial_protocol_poc.py`を再起動します。

## 2台目以降のセットアップ

1. 1台目と同じPico H／RP2040構成であることを確認します。
2. 1台目の実機試験に記録したものと同じ`RPI_PICO` stable UF2を導入します。
3. ピン割当表どおりに配線します。最初は2台目でもGP2／スイッチ1だけを確認します。
4. `device/`をVS Codeで開きます。PC固有設定のcopyは不要です。
5. WSLで使う場合、新しいPicoは別deviceとして管理者PowerShellで現在のBUSIDをbindし、WSLへattachします。
6. `MicroPico: Switch Pico`で2台目の現在のportを選びます。
7. `Upload Project`を実行し、転送対象を確認します。
8. raw配線確認ではlocalの`device/poc/button_test.py`を`Run current file on Pico`で実行します。
   Wire v1確認ではbrowserから`serial_protocol_poc.py`を起動します。
9. GP2成功後にGP3～GP8を確認し、USB再接続試験も行います。

## Web Serialとの排他

MicroPicoとbrowserは同じserial portを同時に開けません。vREPL terminalを閉じるだけでは接続が残る
場合があるため、Web Serial確認前には必ず`MicroPico: Disconnect`を実行し、Disconnected表示を確認
します。browserはUpload済み`/serial_protocol_poc.py`をraw REPLから起動します。browser側で
`Stop & disconnect`してportをcloseした後に、`MicroPico: Connect`で再接続します。

PicoをWSLへattachしている間はWindows側から利用できません。Windows browserでWeb Serialを確認する
場合は、MicroPicoをDisconnectした後、PowerShellで`usbipd detach --busid <BUSID>`を実行してからbrowserで
portを開きます。browser側でportをcloseした後は、`usbipd attach --wsl --busid <BUSID>`、WSL側の
`/dev/ttyACM*`確認、`MicroPico: Connect`の順で戻します。

## 実測logの保存

実機vREPLで取得した原文だけを`device/samples/`へ保存します。

- filename例: `YYYYMMDD-gp2-vrepl.log`
- vREPL transcriptのedgeを削除、重複排除、並べ替え、時刻補完しない
- 操作日時、配線、実施した操作種別、software version、結果、対応log filenameはこのREADMEへ記録する
- bounce未観測時は「今回の操作では観測されなかった」とだけ記録する
- 想定log、生成log、説明用の架空logを保存しない

### 2026-08-27 GP2／スイッチ1

- 環境: Raspberry Pi Pico H（RP2040）、MicroPython v1.29.0 UF2、MicroPico v4.3.4、VS Code WSL window
- firmware確認log: `samples/20260827-firmware-vrepl.log`
- 配線: スイッチ1をGP2（物理pin 4）とGND（物理pin 8）の間へ接続
- 初期状態: 未押下で`level=1 state=RELEASED`
- 押下／解放: 押下で`level=0 state=PRESSED`、解放で`level=1 state=RELEASED`
- 保持: PRESSEDで開始後、解放まで約41.8秒の間に追加の状態遷移なし
- 短い間隔の操作: 6行のraw遷移を取得。約1.8 ms間隔の反転を含むが、原因をbounceとは断定しない
- 実測log: `samples/20260827-gp2-vrepl.log`、`samples/20260827-gp2-hold-vrepl.log`、
  `samples/20260827-gp2-rapid-vrepl.log`
- `Upload Project`の転送内容は後述の全7入力試験後に確認

### 2026-08-27 GP2～GP8／スイッチ1～7

- USBを通常接続し直し、Windows側からPicoをWSLへ再attachした後、MicroPicoの自動再接続を確認
- 再接続後に拡張済みの`button_test.py`をvREPLで再実行
- 全7入力の初期状態が`level=1 state=RELEASED`
- スイッチ1～7の各入力で、押下時の`level=0 state=PRESSED`と解放時の
  `level=1 state=RELEASED`を確認
- 複数入力の押下が重なる場合も、それぞれのbutton番号とGPIOで遷移を取得
- 約1 ms単位の短い反転を含むraw遷移を取得したが、原因をbounceとは断定しない
- 実測log: `samples/20260827-gp2-gp8-vrepl.log`
- `MicroPico: Upload project to Pico`完了後、実機REPLの`os.listdir()`でPico filesystem rootが
  `['button_test.py']`だけであることを確認
- `README.md`、`samples/`、`.vscode/`、`.micropico`が転送されていないことを確認
- Upload確認log: `samples/20260827-upload-project-vrepl.log`
- Pico filesystem上の`button_test.py`を実行し、全7入力が初期化されることを確認
- Upload済みscript実行log: `samples/20260827-uploaded-button-test-vrepl.log`

### 2026-09-01 Web Serial診断capture

- Windows上のChrome 150、`http://localhost:5173/device-poc`、115200／8-N-1で実施
- `button_test.py`をbrowserのraw REPL bootstrapから起動し、全7 GPIOの押下／解放をraw byteで確認
- 正常停止、読取り中のUSB物理切断、capture保持、利用者操作による再接続、再接続後のGP2を確認
- 実測captureとSHA-256、script出力区間、操作結果は
  [`client/samples/web-serial/README.md`](../client/samples/web-serial/README.md)に記録
- このcaptureは人向け診断出力であり、Wire v1のJSON frame実測sampleには流用しない

### 2026-09-02 Wire v1 Web Serial capture

- `Upload Project`後のPico rootが`button_test.py`と`serial_protocol_poc.py`だけであることを確認
- 起動時にスイッチ1を押したまま接続し、保持中と最初の解放ではframeが出ず、その次の短押しだけが
  `up / short_press`になることを確認
- GP2～GP8で`up`, `down`, `left`, `right`, `red`, `yellow`, `green`の全7 controlを確認
- GP2の700 ms以上の保持で、保持中はrepeatせず、解放時に`up / long_press`が1 frameだけ出ることを確認
- GP2の短い間隔の3操作が`up / short_press` 3 frameとなることを確認
- GP2を保持しながらGP5を4回操作し、`right / short_press` 4 frameの後、GP2解放時に
  `up / long_press`が出る独立入力を確認
- 正常停止と、読取り中のUSB物理切断によるcapture保持、BOOTSELなしの再接続、利用者gestureでの
  再起動、再接続後の`up / short_press`、正常停止を確認
- 実測`.bin`／`.json`、SHA-256、connectionごとのoffset、raw frame列は
  [`client/samples/web-serial/README.md`](../client/samples/web-serial/README.md)に記録
- raw REPL制御byteを含むcapture全体と、Wire v1 parserへ渡すhalf-open intervalを分けて保存

## 参照資料

- [MicroPico v4.3.4](https://github.com/paulober/MicroPico/releases/tag/v4.3.4)
- [MicroPico v4.3.4 projectと実行手順](https://github.com/paulober/MicroPico/blob/v4.3.4/README.md)
- [MicroPython v1.29.0 RP2 quick reference](https://docs.micropython.org/en/v1.29.0/rp2/quickref.html)
- [Raspberry Pi公式MicroPython導入手順](https://www.raspberrypi.com/documentation/microcontrollers/micropython.html)
- [Raspberry Pi Pico用MicroPython](https://micropython.org/download/RPI_PICO/)
- [Microsoft公式: WSLへUSB deviceを接続する](https://learn.microsoft.com/windows/wsl/connect-usb)
