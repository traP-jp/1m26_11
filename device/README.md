# Raspberry Pi Pico H firmwareとdevice確認

Raspberry Pi Pico H（RP2040）に接続した機械式スイッチを読み取り、Serial Protocol v1のeventを送る
firmwareと、その根拠になったdevice PoCを管理します。用途を混同しないように、productionと過去の
PoCをdirectoryで分けています。

- `firmware/button_firmware.py`: Issue #81のproduction状態機械、GPIO adapter、serial frame送信
- `firmware/main.py`: Picoの起動時にproduction firmwareを自動起動するentrypoint
- `firmware/serial_protocol_poc.py`: 既存のscript名を使う手順向けのproduction互換entrypoint
- `poc/button_test.py`: MicroPico vREPLでraw GPIO遷移とbounceを観測する一時診断用
- `poc/serial_protocol_poc.py`: Issue #92でWire v1を確定した時点の手動起動PoC

Wire v1の規範はIssue #92と[`SERIAL_PROTOCOL.md`](SERIAL_PROTOCOL.md)、その規範に従う継続動作用
firmwareはIssue #81として追跡します。`poc/button_test.py`の人向け表示は最終serial protocolではなく、
productionのdebounce、repeat抑止、長押し判定を実装しません。

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
| Issue #92 Wire v1契約と専用firmware | 2026-09-02 実装・実機確認済み |
| Issue #93 Wire v1純粋parser／WebSerialInputAdapter | contract-synthetic unit test済み |
| Issue #81 production firmwareのhost unit test | 2026-09-02 `mise run device-test` 23 test確認済み |
| Issue #81 production firmwareの書込み・実機操作matrix | 2026-09-02 確認済み |
| Issue #81 production firmwareの物理電源再投入・押下中再投入 | 2026-09-02 確認済み（WSLへ再attach） |
| Issue #81手順の別担当による再現 | 未実施 |
| Issue #92手順の別担当による再現 | 未実施 |

2026-09-02にはPoCの確認とは別に、`firmware/main.py`から自動起動するproduction版を実機へ書き込み、
直接readで手動matrixと物理電源再投入を確認しました。captureはローカル検証にだけ使用し、repositoryや
PRには添付しません。実施環境と観測結果はこのREADMEへ記録します。別担当がREADMEだけを使って行う
再現確認は引き続き未実施です。

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
├── firmware/
│   ├── button_firmware.py      # Wire v1状態機械、GPIO adapter、継続run loop
│   ├── main.py                 # 電源投入／hard reset時の自動起動entrypoint
│   └── serial_protocol_poc.py  # production coreを呼ぶ互換entrypoint
├── poc/
│   ├── button_test.py         # 手動実行するraw GPIO確認用script
│   └── serial_protocol_poc.py # Issue #92時点のWire v1実機確認用script
├── samples/                   # 過去のPoCで取得したvREPL transcript
├── tests/                     # hardware非依存のproduction状態機械test
├── README.md
└── SERIAL_PROTOCOL.md         # device／frontend間のWire v1規範
```

`firmware/`をPico filesystem rootへ転送すると、MicroPythonが`/main.py`を起動時に実行し、button監視を
自動で開始します。production確認ではbrowserやvREPLからscriptを起動しません。`boot.py`は置きません。

`poc/`は調査履歴として残し、必要なscriptを検証者が明示的に手動実行します。過去のWeb Serial raw
viewerはraw REPLからPoC版`/serial_protocol_poc.py`を起動していたため、同名のproduction互換entrypoint
や自動起動する`main.py`と取り違えないでください。

既存の`samples/20260827-*`はPoC時点の実測履歴です。production版のraw captureはローカル検証にだけ
使用し、repositoryへcommitしたりPRへ添付したりしません。syntheticな境界caseは`tests/`で管理し、
実機sampleを装いません。

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
- `.vscode/settings.json`: production `firmware/`の3 `.py`だけを転送する設定とvREPL用のportable設定

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

## Wire v1専用PoC（Issue #92の履歴）

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
`MicroPico: Run current file on Pico`を実行します。2026-09-02のWeb Serial PoCでは、当時の設定で
`poc/`をUploadし、browserの`Connect & start`からraw REPLでscriptを起動しました。現在の`Upload Project`
はproduction `firmware/`専用であり、このPoCを転送しません。

この節のWeb Serial手順とraw REPL起動は2026-09-02に行ったPoCの再現用です。production firmwareの
書込み・起動・確認には使用しません。

## Production firmware（Issue #81）

`firmware/button_firmware.py`はIssue #92で確定したWire v1を実装します。7 buttonを独立した状態として
扱い、20 ms debounce、700 msのshort／long threshold、押下保持中のrepeat抑止、起動時LOWの無視を
行います。hardware非依存の状態遷移を同file内で分離し、repository rootから標準taskで境界値と連続操作を
確認します。

```sh
mise run device-test
```

task runnerを介さず同じtestだけを調査するときは、次を実行します。通常の検証記録には標準taskを使用します。

```sh
cd device
python3 -m unittest discover -s tests -p 'test_*.py' -v
```

`firmware/main.py`はPico filesystem rootの`/main.py`としてMicroPythonから自動実行され、production
run loopを開始します。`firmware/serial_protocol_poc.py`はproduction coreを呼び出す互換entrypointで、
PoC版の複製ではありません。productionの通常運用では`main.py`だけが起動責任を持ち、browserからraw
REPL commandを送信しません。run loopが例外終了または想定外にreturnした場合は、serialへdebug文字列を
出さず短いdelay後にmachine resetします。保守用の`KeyboardInterrupt`はresetせずREPLへ返します。

production実機確認のport設定は現PoCと同じ`115200 / 8-N-1`、parityなし、flow controlなしを
使用します。この値はpayload／framing契約には含めず、確認時の接続条件として実施記録へ残します。

起動時、hard reset直後、または監視状態を作り直した直後にLOWだったbuttonは、20 ms以上連続してHIGHに
戻るまで入力対象にしません。最初のreleaseでもframeを送らず、その後の押下から受付を開始します。frame
起動時にHIGHだったbuttonは即時に受付可能とし、次のLOW遷移から20 ms debounceを行います。frame以外の
bannerやdebug文字列をstdoutへ出してはいけません。

### Issue #27へ渡す境界

製品frontendのIssue #27は、利用者のgestureでserial portをopenした後、自動起動済みfirmwareのbyte streamを
直接readします。production接続ではraw REPLへの切替え、script起動command、PoC capture用offsetの切出しは
不要です。frontendへ渡すのはWire v1 frameだけです。

hostがserial portをcloseして再びopenしても、Picoへの給電が続いている限りdeviceの状態機械はresetされません。
そのため、hostのclose／openを「firmware再起動」や「起動時LOW処理」の確認に使わないでください。Issue #81で
いうUSB再接続は、PicoをUSBだけで給電した状態でcableを物理的に抜き、再び挿す操作です。この操作はpower
cycle／hard resetになり、`main.py`から状態機械を初期化して送信を再開します。

製品用のport管理、切断表示、retry、reader／port cleanup、代替入力UIはIssue #27の責任で、このfirmwareの
対象外です。反対にIssue #27側はdebounce、長押し、control対応を再判定しません。

## Upload Project

`device/.vscode/settings.json`は、`Upload Project`の送信元を`device/firmware/`、file typeを`.py`だけに
固定します。productionの転送対象は`button_firmware.py`、`main.py`、`serial_protocol_poc.py`で、Pico
filesystem rootへ同名で配置されます。`poc/`、`tests/`、`README.md`、`SERIAL_PROTOCOL.md`、`samples/`、
`.vscode/`、`.micropico`は転送対象外です。

1. MicroPicoが対象Picoへ接続済みであることを確認します。
2. 実行中の旧scriptがある場合は`MicroPico: Stop execution`で停止します。
3. `device/.vscode/settings.json`の`micropico.syncFolder`が`firmware`であることを確認します。
4. localの`firmware/`に上記3 fileだけがあることを確認します。
5. command paletteから`MicroPico: Upload project to Pico`を実行します。
6. `MicroPico: Toggle Virtual File System (reloads UI and closes existing vREPLs)`で、Pico rootの
   `/button_firmware.py`、`/main.py`、`/serial_protocol_poc.py`を確認します。このcommandは既存vREPLを
   閉じるため、実行中のscriptを先に停止します。
7. `poc/`、README、test、sample logが新たに転送されていないことを確認します。
8. production確認を始める前にMicroPicoをDisconnectし、Picoを物理的に抜き差しして`main.py`をcold
   startします。buttonは、押下中再投入caseを行うとき以外はすべて解放しておきます。

旧版から更新するときも同じ3 fileをまとめて転送し、一部だけを新旧混在させません。転送後はPico rootの
file名を再確認し、hard reset後に手動確認matrixを先頭から実施します。

`Upload Project`は送信元を限定しますが、Pico上の既存fileを全削除する操作ではありません。PoC期に転送した
`button_test.py`、以前の`main.py`、`boot.py`などが残っている場合は、対象Picoを確認してからVirtual File
SystemまたはREPLで個別に除去します。別のPicoやhost filesystemを誤って変更しないでください。

`main.py`の起動失敗や無限resetで通常接続できなくなった場合は、まず全buttonを解放してUSBを抜き、再接続後
すぐにMicroPico／REPLから実行をinterruptして、Pico rootの3 fileと例外を確認します。interruptできない、
または壊れたfilesystemが残る場合は、MicroPython公式の
[RP2 factory reset手順](https://docs.micropython.org/en/latest/rp2/tutorial/reset.html)に従い、BOOTSEL modeで
external flashを専用UF2により完全消去します。この操作はPico内のfilesystemを失うため、対象boardが
Pico H（RP2040）であることを再確認してから行います。消去後は正しい`RPI_PICO` MicroPython UF2、
production 3 fileの順に再書込みし、unit testと手動確認matrixを最初から実施します。

PoCを再調査するときは`Upload Project`の転送元を切り替えず、localの`device/poc/button_test.py`または
`device/poc/serial_protocol_poc.py`を`MicroPico: Run current file on Pico`で一時実行します。終了後に停止し、
productionの確認へ戻る際はhard resetして`main.py`を起動します。

repository rootをMicroPico projectとして開くと、`device/`自体や`device/samples/`まで誤って転送対象に
するlocal設定が生成される場合があります。必ず`device/`だけを別のVS Code windowで開き、repository rootの
`.vscode/settings.json`へMicroPico設定を保存しません。

### PoC期のUpload記録

2026-09-02のPoC確認では、当時の`Upload Project`が`poc/`を転送し、Pico rootが次の2 fileだけであることを
REPLの`os.listdir()`で確認しました。これは履歴であり、production書込み後の期待値ではありません。

```text
['button_test.py', 'serial_protocol_poc.py']
```

production書込み後の期待値は次の3 fileです。2026-09-02にPico rootがこの3 fileだけであることと、
local source／実機fileのSHA-256が一致することを確認しました。

```text
['button_firmware.py', 'main.py', 'serial_protocol_poc.py']
```

## USBを抜き差しした後

Issue #81の再接続試験は、PicoがUSB cableだけから給電され、外部電源がない状態で実施します。

1. 通常caseでは全buttonを解放します。押下中再投入caseだけは、指定したbuttonを押したままにします。
2. serial monitor、browser、MicroPicoがportをopenしている場合はclose／Disconnectします。
3. BOOTSELを押さずにUSB cableを物理的に抜き、Picoの給電を完全に切ります。
4. 数秒待ってから、BOOTSELを押さずにUSB cableを接続します。これをpower cycle／hard resetとします。
5. WSL利用時は、Windows側から現在のPicoをWSLへ再attachします。
6. raw byteをbinary保存できるserial monitorか、Issue #27完了後の製品frontendから利用者操作で
   portをopenし、自動起動済み`main.py`の出力を直接readします。script起動commandは送りません。
7. portが変わった、または複数台ある場合は現在のPicoを選び直します。
8. 通常caseでは次の操作からframeが届くこと、押下中再投入caseでは保持中と最初のreleaseが無出力で、
   その次の押下／解放だけが1 frameになることを確認します。

host側でportをclose／openしただけではPicoはhard resetされず、状態機械も初期化されません。hostだけの
再接続はIssue #27のport lifecycle確認には使えますが、Issue #81の電源再投入／起動状態確認の代用には
なりません。

## 2台目以降のセットアップ

1. 1台目と同じPico H／RP2040構成であることを確認します。
2. 1台目の実機試験に記録したものと同じ`RPI_PICO` stable UF2を導入します。
3. ピン割当表どおりに配線します。最初は2台目でもGP2／スイッチ1だけを確認します。
4. `device/`をVS Codeで開きます。PC固有設定のcopyは不要です。
5. WSLで使う場合、新しいPicoは別deviceとして管理者PowerShellで現在のBUSIDをbindし、WSLへattachします。
6. `MicroPico: Switch Pico`で2台目の現在のportを選びます。
7. `Upload Project`を実行し、production 3 fileだけが転送対象であることを確認します。
8. raw配線確認ではlocalの`device/poc/button_test.py`を`Run current file on Pico`で実行します。
   production確認へ移る前に停止し、USBを物理的に抜き差しして`main.py`を自動起動します。
9. GP2成功後にGP3～GP8と手動確認matrixを確認し、別担当再現記録を残します。

## Web Serialとの排他

MicroPicoとbrowserは同じserial portを同時に開けません。vREPL terminalを閉じるだけでは接続が残る
場合があるため、Web Serial確認前には必ず`MicroPico: Disconnect`を実行し、Disconnected表示を確認
します。Issue #27完了後の製品browserは、起動済み`main.py`のWire v1をportから直接readします。
browser側でportをcloseした後に、`MicroPico: Connect`で再接続します。close／openだけではdevice状態が
resetされない点に注意してください。

現在の開発用`/device-poc`はraw REPLから`/serial_protocol_poc.py`を起動するPoC viewerであり、
productionの直接read toolではありません。その手順で得たraw REPLのbannerやoffset metadataを、
productionの直接read sampleへ混在させません。

PicoをWSLへattachしている間はWindows側から利用できません。Windows browserでWeb Serialを確認する
場合は、MicroPicoをDisconnectした後、PowerShellで`usbipd detach --busid <BUSID>`を実行してからbrowserで
portを開きます。browser側でportをcloseした後は、`usbipd attach --wsl --busid <BUSID>`、WSL側の
`/dev/ttyACM*`確認、`MicroPico: Connect`の順で戻します。

## 実機確認記録の扱い

productionの直接readで取得したraw byteはローカル検証用とし、repositoryやPRへ添付しません。PoCの
vREPL transcriptとproductionの直接read captureを同じ検証結果として扱わず、実施環境、操作、期待値、
観測結果、制約をREADMEまたはIssueへ記録します。

production sampleはcaseごとに、少なくとも次を対応付けます。

- raw byte: `YYYYMMDD-issue81-<case>.bin`。取得後に内容を編集、改行変換、重複排除、並べ替えしない
- 期待event列: `YYYYMMDD-issue81-<case>.expected.jsonl`。期待するcanonical frameを受信順に1 lineずつ記す
- hash: raw `.bin`のSHA-256。metadataまたは`YYYYMMDD-issue81-<case>.sha256`へ記す
- 実施記録: commit SHA、Pico／MicroPython、配線、capture tool、日時、操作手順、操作回数、期待event数、
  実event数、判定を記す

Linux／WSLでは、現在のPicoだけが対象portにつながっていることを確認し、repository rootから
次の手順で直接readしたbyteをrepository外またはignore対象のlocal pathへ無変換で保存できます。
`serial_port`と`capture_path`は実際の値へ変更します。

```sh
serial_port=/dev/ttyACM0
capture_path=/tmp/issue81-single.bin
stty -F "$serial_port" 115200 cs8 -cstopb -parenb -ixon -ixoff -crtscts raw -echo
dd if="$serial_port" of="$capture_path" bs=4096 status=none oflag=excl
```

`dd`を起動してから対象caseを操作し、完了後にCtrl-Cで終了します。`oflag=excl`により既存sampleの
上書きを防ぎます。USBを抜いてreadが終了するcaseと、再接続後のcaseは別のraw fileとし、対応関係を
実施記録へ残します。このcommandはproductionの`main.py`を停止したりraw REPL commandを送ったりしません。

Linux／WSLでは次のようにraw fileのSHA-256をローカルで照合できます。

```sh
sha256sum /tmp/issue81-CASE.bin
```

Windows PowerShellでは次を使用します。

```powershell
Get-FileHash -Algorithm SHA256 .\issue81-CASE.bin
```

serial monitorがtextとして保存する際にLF／CRLFを変換する場合、そのfileはraw正本にしません。binary保存
できるcaptureを正本にし、表示用textは派生物と明記します。production sampleではraw REPLのbanner、prompt、
command、停止応答が含まれていないことも確認します。payloadはcanonical key順・空白なしで、改行はLFまたは
CRLFだけであることをbyte単位で照合します。

既存のPoC vREPL logには従来どおり次を適用します。

- filename例: `YYYYMMDD-gp2-vrepl.log`
- vREPL transcriptのedgeを削除、重複排除、並べ替え、時刻補完しない
- bounce未観測時は「今回の操作では観測されなかった」とだけ記録する
- 想定log、生成log、説明用の架空logを実測sampleとして保存しない

### Production手動確認matrix

最初に`mise run device-test`を通し、production 3 fileを書き込んでhard resetした実機で次を順に確認します。
操作回数とevent数を記録し、余分なframeが1件でもあれば不合格です。

| case | 操作 | 期待event列／確認点 |
| --- | --- | --- |
| cold boot | 全buttonを解放して電源投入し、操作せず待つ | 0 frame。banner、debug行、空lineも出ない |
| single | 各controlを一定回数、短押しする | 各操作につき対応する`short_press`が1 frame。操作数とevent数が一致 |
| rapid | 同じbuttonを解放を挟んで短間隔で連打する | 確定した各操作につき1 frame。欠落、結合、余分なrepeatなし |
| long | 700 ms以上保持して解放する | 保持中は0 frame、解放時に`long_press`が1 frame。`short_press`は出ない |
| bounce-prone | 接点が揺れやすい押し方で押下／解放する | 1操作につき1 frame。bounce由来の規則外重複なし |
| mixed／continuous | 複数buttonの重なりを含む連続操作 | release確定順のevent列と一致し、button間で状態が干渉しない |
| power cycle | 通常操作後、USB単独給電のcableを物理的に抜き差しする | `main.py`が自動再開し、次の操作を1 frameとして送る |
| held power cycle | buttonを押したまま物理的に電源再投入し、その後解放する | 保持中と最初のreleaseは0 frame。次の完全な操作から1 frame |
| canonical only | 上記全caseのraw byteを走査する | Wire v1 canonical frameとLF／CRLF以外の出力がない |

19／20 msと699／700 msの厳密な境界は人手操作で保証せず、host unit testの決定的な時刻入力で確認します。
実機matrixではthresholdの前後に十分な余裕を持たせ、実測した保持条件を記録します。host portのclose／open
だけはpower cycle caseに数えません。

### 別担当による再現check

実装担当者とは別の担当者が、READMEだけを見て次を再現します。

1. 対象commit、Pico H、MicroPython version、host OS、capture toolを記録する。
2. pin割当どおりに配線し、production 3 fileを`Upload Project`で転送する。
3. Pico rootの3 fileを確認し、USBを物理的に抜き差しして`main.py`を自動起動する。
4. 手動確認matrixを実行し、ローカルrawのSHA-256、期待event列、操作数／event数を照合する。
5. frontend parserへ同じraw byteを入力する確認は別工程として行い、firmwareのrawを加工しない。
6. 不一致や手順不足をIssue #81またはPRへ記録し、修正後はmatrixを最初から再実行する。

PRまたはIssueには次のtemplateをcaseごとに埋めます。未実施欄を空欄のまま合格扱いにしません。

```text
実施者:
実施日時／timezone:
commit SHA:
Pico model／MicroPython:
host OS／capture tool:
配線:
case／具体的な操作:
操作数:
期待event列／期待event数:
実event列／実event数:
local raw SHA-256:
canonical frame以外のbyte: なし／あり（内容）
結果: PASS／FAIL
備考:
```

この再現記録がまだない状態で「別担当再現済み」またはproduction実機確認済みとは記載しません。

### 2026-08-27 GP2／スイッチ1

- 環境: Raspberry Pi Pico H（RP2040）、MicroPython v1.29.0 UF2、MicroPico v4.3.4、VS Code WSL window
- firmware bannerでMicroPython v1.29.0とRaspberry Pi Pico（RP2040）を確認
- 配線: スイッチ1をGP2（物理pin 4）とGND（物理pin 8）の間へ接続
- 初期状態: 未押下で`level=1 state=RELEASED`
- 押下／解放: 押下で`level=0 state=PRESSED`、解放で`level=1 state=RELEASED`
- 保持: PRESSEDで開始後、解放まで約41.8秒の間に追加の状態遷移なし
- 短い間隔の操作: 6行のraw遷移を取得。約1.8 ms間隔の反転を含むが、原因をbounceとは断定しない
- `Upload Project`の転送内容は後述の全7入力試験後に確認

### 2026-08-27 GP2～GP8／スイッチ1～7

- USBを通常接続し直し、Windows側からPicoをWSLへ再attachした後、MicroPicoの自動再接続を確認
- 再接続後に拡張済みの`button_test.py`をvREPLで再実行
- 全7入力の初期状態が`level=1 state=RELEASED`
- スイッチ1～7の各入力で、押下時の`level=0 state=PRESSED`と解放時の
  `level=1 state=RELEASED`を確認
- 複数入力の押下が重なる場合も、それぞれのbutton番号とGPIOで遷移を取得
- 約1 ms単位の短い反転を含むraw遷移を取得したが、原因をbounceとは断定しない
- `MicroPico: Upload project to Pico`完了後、実機REPLの`os.listdir()`でPico filesystem rootが
  `['button_test.py']`だけであることを確認
- `README.md`、`samples/`、`.vscode/`、`.micropico`が転送されていないことを確認
- Pico filesystem上の`button_test.py`を実行し、全7入力が初期化されることを確認

### 2026-09-01 Web Serial診断capture

- Windows上のChrome 150、`http://localhost:5173/device-poc`、115200／8-N-1で実施
- `button_test.py`をbrowserのraw REPL bootstrapから起動し、全7 GPIOの押下／解放をraw byteで確認
- 正常停止、読取り中のUSB物理切断、capture保持、利用者操作による再接続、再接続後のGP2を確認
- 実測captureはローカル確認にだけ使用し、repositoryやPRには添付しない
- 実施環境と操作結果はこの節へ記録
- このcaptureは人向け診断出力であり、Wire v1のJSON frame実測sampleには流用しない

### 2026-09-02 Wire v1 PoC Web Serial capture

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
- 実測`.bin`／`.json`はローカル確認にだけ使用し、repositoryやPRには添付しない
- raw REPL制御byteを含むcapture全体と、Wire v1 parserへ渡すhalf-open intervalを分けて確認

この記録は`poc/serial_protocol_poc.py`とraw REPL bootstrapを使ったIssue #92の実測です。production
`main.py`の自動起動、直接read、power cycle後の状態初期化を確認した記録ではありません。

### Issue #81 production実機記録

2026-09-02にproduction 3 fileを書き込み、raw REPLを使わない直接readで全7 control、連打、長押し、
重なったbutton操作、無操作時の余分な出力、物理電源再投入、押下中再投入を確認しました。非空の
captureはcanonical JSONとCRLFだけで構成され、期待event列と一致しました。capture自体はローカル確認に
だけ使用し、repositoryやPRには添付しません。

実装・capture担当とは別の担当者がREADMEだけを使って行う再現確認は未実施です。再現完了までは
「別担当再現済み」とは扱いません。

## 参照資料

- [MicroPico v4.3.4](https://github.com/paulober/MicroPico/releases/tag/v4.3.4)
- [MicroPico v4.3.4 projectと実行手順](https://github.com/paulober/MicroPico/blob/v4.3.4/README.md)
- [MicroPython v1.29.0 RP2 quick reference](https://docs.micropython.org/en/v1.29.0/rp2/quickref.html)
- [Raspberry Pi公式MicroPython導入手順](https://www.raspberrypi.com/documentation/microcontrollers/micropython.html)
- [Raspberry Pi Pico用MicroPython](https://micropython.org/download/RPI_PICO/)
- [Microsoft公式: WSLへUSB deviceを接続する](https://learn.microsoft.com/windows/wsl/connect-usb)
