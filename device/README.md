# Raspberry Pi Pico H button PoC

Raspberry Pi Pico H（RP2040）に接続した機械式スイッチの raw GPIO 状態遷移を、
VS Code の MicroPico vREPL で確認するための PoC です。

この PoC の表示は人が配線と bounce を確認するための一時的な診断出力です。最終シリアル
protocol、encoding、delimiter、frame、button と game control の対応を定義するものではありません。
また、本番用 debounce、連打の集約、repeat、長押し判定は実装しません。

## 現在の確認状況

| 項目 | 状態 |
| --- | --- |
| Pico H が BOOTSEL の `RPI-RP2` として認識される | 2026-08-27 確認済み |
| `RPI_PICO-20260824-v1.29.0.uf2` の書込み後に USB serial device として認識される | 2026-08-27 確認済み |
| GP2／スイッチ1の押下・保持・解放・短い間隔の連続操作 | 2026-08-27 確認済み |
| GP3～GP8／スイッチ2～7 | 2026-08-27 確認済み |
| USB抜き差し後のMicroPico再接続・再実行 | 2026-08-27 確認済み（WSLへ再attach） |
| `Upload Project` の実機転送内容 | 2026-08-27 確認済み |

GPIOの実機確認が終わるまでは、Issue #92のdevice側PoCも完了扱いにしません。また、Issue #92全体の
完了条件であるWeb Serial確認と入力契約確定は、このPoCの対象外です。

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
│   └── button_test.py         # 手動実行するraw GPIO確認用script
├── samples/                   # 実機vREPLで取得したlogだけを保存
└── README.md
```

`main.py`と`boot.py`は置きません。Picoへ転送してもUSB接続時には自動起動せず、検証者が
`button_test.py`を明示的に実行します。

`samples/`には実機vREPLから取得した原文だけを保存します。想定出力や手作りのsampleは追加しません。

## ピン割当

Pico HをUSB connectorが上になる向きで見たときの、board上の物理pin番号も併記します。

| スイッチ | GPIO | 物理pin | もう一方の端子 |
| --- | --- | --- | --- |
| 1 | GP2 | 4 | 共通GND |
| 2 | GP3 | 5 | 共通GND |
| 3 | GP4 | 6 | 共通GND |
| 4 | GP5 | 7 | 共通GND |
| 5 | GP6 | 9 | 共通GND |
| 6 | GP7 | 10 | 共通GND |
| 7 | GP8 | 11 | 共通GND |
| 共通GND | GND | 8 | 全スイッチで共有 |

GPIOは `Pin.IN` と内部 `Pin.PULL_UP` で初期化します。

- 未押下: HIGH（`level=1`、`RELEASED`）
- 押下: LOW（`level=0`、`PRESSED`）

スイッチ番号は物理配線を識別する仮番号です。`up`、`down`などのgame controlへはまだ割り当てません。

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

## Upload Project

`device/.vscode/settings.json`は、`Upload Project`の送信元を`device/poc/`、file typeを`.py`だけに
固定します。現在の転送対象は`button_test.py`だけで、Pico filesystem rootの`button_test.py`として
配置されます。`README.md`、`samples/`、`.vscode/`、`.micropico`は転送対象外です。

1. MicroPicoが対象Picoへ接続済みであることを確認します。
2. `MicroPico: Upload project to Pico`を実行します。
3. `MicroPico: Toggle Virtual File System (reloads UI and closes existing vREPLs)`で、転送された
   `/button_test.py`を確認します。このcommandは既存vREPLを閉じるため、実行中のscriptを先に停止します。
4. READMEやsample logが新たに転送されていないことを確認します。
5. Upload済みcopy自体を確認する場合は、Virtual File System上の`/button_test.py`を開き、
   `MicroPico: Run current file on Pico`で実行します。
6. 通常の開発時はlocalの`device/poc/button_test.py`を開き、`MicroPico: Run current file on Pico`で実行します。

`Upload Project`は送信元を限定しますが、Pico上の既存fileを全削除する操作ではありません。再利用する
Picoでは、既存の`main.py`や`boot.py`が自動起動しないかをfilesystem表示で確認してください。既存fileを
削除する場合は、対象を確認してから個別に行います。

## USBを抜き差しした後

1. 実行中なら`MicroPico: Stop execution`で停止します。
2. 自動再接続を試す場合は`MicroPico: Disconnect`を実行せず、USB cableを抜きます。
3. 数秒待ってから、`BOOTSEL`を押さずにUSB cableを接続します。
4. WSL利用時は、Windows側から現在のPicoをWSLへ再attachします。
5. 自動再接続を待ちます。接続しなければ`MicroPico: Connect`を実行します。
6. 意図的に`MicroPico: Disconnect`した場合、自動再接続は待たず`MicroPico: Connect`を実行します。
7. portが変わった、または複数台ある場合は`MicroPico: Switch Pico`で現在のPicoを選びます。
8. `button_test.py`を再実行し、同じGPIO遷移を確認します。

## 2台目以降のセットアップ

1. 1台目と同じPico H／RP2040構成であることを確認します。
2. 1台目の実機試験に記録したものと同じ`RPI_PICO` stable UF2を導入します。
3. ピン割当表どおりに配線します。最初は2台目でもGP2／スイッチ1だけを確認します。
4. `device/`をVS Codeで開きます。PC固有設定のcopyは不要です。
5. WSLで使う場合、新しいPicoは別deviceとして管理者PowerShellで現在のBUSIDをbindし、WSLへattachします。
6. `MicroPico: Switch Pico`で2台目の現在のportを選びます。
7. `Upload Project`を実行し、転送対象を確認します。
8. localの`device/poc/button_test.py`を`Run current file on Pico`で実行します。
9. GP2成功後にGP3～GP8を確認し、USB再接続試験も行います。

## Web Serialとの排他

MicroPicoとbrowserは同じserial portを同時に開けません。vREPL terminalを閉じるだけでは接続が残る
場合があるため、Web Serial確認前には必ず`MicroPico: Disconnect`を実行し、Disconnected表示を確認
します。browser側でportをcloseした後に、`MicroPico: Connect`で再接続します。

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

## 参照資料

- [MicroPico v4.3.4](https://github.com/paulober/MicroPico/releases/tag/v4.3.4)
- [MicroPico v4.3.4 projectと実行手順](https://github.com/paulober/MicroPico/blob/v4.3.4/README.md)
- [MicroPython v1.29.0 RP2 quick reference](https://docs.micropython.org/en/v1.29.0/rp2/quickref.html)
- [Raspberry Pi公式MicroPython導入手順](https://www.raspberrypi.com/documentation/microcontrollers/micropython.html)
- [Raspberry Pi Pico用MicroPython](https://micropython.org/download/RPI_PICO/)
- [Microsoft公式: WSLへUSB deviceを接続する](https://learn.microsoft.com/windows/wsl/connect-usb)
