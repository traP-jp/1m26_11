# 本番controller設置・復旧runbook

本番用のRaspberry Pi Pico H controllerを設置し、接続確認、故障時の切替、終了後の回収を行うための
当日手順です。firmwareの実装詳細や厳密な試験は扱いません。

- 配線、MicroPython導入、firmware転送の詳細: [`README.md`](README.md)
- Serial frameの契約: [`SERIAL_PROTOCOL.md`](SERIAL_PROTOCOL.md)
- Room画面の接続・代替入力: [`../client/README.md`](../client/README.md)

この文書は、Issue #82で本番前／当日の接続確認と復旧手順を整備するrunbookです。このPRの完了は、
前提IssueでmergeされたSerialの状態表示、再試行、keyboard／画面button切替と本書の文言・操作が一致し、
確認済み事項と本番前の未確認事項が区別されていることを基準とします。

PRのmergeまたはIssue #82のcloseは、本番候補環境でのsmoke PASSを意味しません。次の使用前確認と
「本番前smoke」は、実際に使用するPC、browser、origin、Roomが決まった後に一度だけ行います。

## 本番使用前の確認

- [ ] Serialの接続、切断、状態通知と、keyboard／画面buttonへの切替が本番buildへmergeされている
- [ ] Roomの共通入力eventが既存の操作列へ接続され、操作結果を画面で確認できる
- [ ] 試験Roomで既存の送信操作と判定結果を確認できる
- [ ] 操作してよい試験Roomと問題、初期操作列、許可controlが確定している
- [ ] 本番候補PC、Chromium browser、origin、USB portでWeb Serialを利用できる

Roomの共通入力eventを操作列へ接続していないbuildでは接続状態までしか確認できず、
「代替入力でゲームを継続」を合格にできません。
このrunbookへfrontend処理を追加せず、下流統合を担当するIssueで解消してから受入確認を行います。

## 最優先の方針

- 未接続または切断表示なら再接続を1回だけ試し、直らなければkeyboardまたは画面buttonへ切り替えます。
- 接続済み表示のまま物理buttonが反応しない場合は、再接続を挟まず代替入力へ切り替えます。
- 機材の調査や物理交換は代替入力へ切り替えた後、ゲーム進行と並行して行えます。ただしbrowser上の
  再接続は代替入力を止めるため、round間など操作を止められるタイミングだけで行います。
- 当日は配線やfirmwareを部分修理せず、配線済みの予備controller一式を交換します。
- Serial portの選択は必ず利用者がRoom画面のbuttonを押して開始します。自動再接続は行いません。

```text
未接続／切断 → 再接続を1回試す ─ 成功 → Serialで継続
                         └─ 失敗 → keyboard／画面buttonで継続
接続済みだが無反応 ────────────────┘

代替入力へ切替後 → 物理交換は並行可能
                 └─ browserでの接続確認はround間などの安全な停止点だけ
```

## 機材とlabel

部品単位ではなく、次の交換単位で管理します。表の個数は推奨最小数で、実数と本番構成は機材担当が
確定します。PicoはPCのUSBから給電し、外部電源は使用しません。

| label         | 内容                                                    | 推奨最小数 | 実数 | 保管場所 | 確認 |
| ------------- | ------------------------------------------------------- | ---------: | ---: | -------- | ---- |
| `CTRL-MAIN`   | 配線済みPico H、非発光button 7個、配線基板、jumper wire |          1 |      |          | [ ]  |
| `CTRL-SPARE`  | `CTRL-MAIN`と同じ構成の配線済み予備一式                 |          1 |      |          | [ ]  |
| `CABLE-MAIN`  | data通信対応micro-USB cable                             |          1 |      |          | [ ]  |
| `CABLE-SPARE` | 確認済み予備data cable                                  |          1 |      |          | [ ]  |
| `PC-MAIN`     | 本番PC、確認済みChromium browser、確認済みUSB port      |          1 |      |          | [ ]  |
| `USB-OPTION`  | adapterまたはhub。本番構成で必要な場合だけ使用          |   0または1 |      |          | [ ]  |

各controller本体、cable両端、使用するPCのUSB portへ対応するlabelを貼ります。buttonにも
`up`、`down`、`left`、`right`、`red`、`yellow`、`green`が利用者から読めるlabelを付けます。

実物を準備したら、配線全体とPicoのpin周辺が判別できる写真をIssue #82へ添付し、そのURLまたは
repository内の保存先を次へ記録します。

- 全景写真: ____________________
- 配線拡大写真: ____________________
- 撮影日／確認者: ____________________

### 配線の確認

Pico HをUSB connectorが上になる向きで確認します。各buttonはGPIOと共通GNDの間へ接続します。

| button label | GPIO | 物理pin | もう一方の端子   |
| ------------ | ---- | ------: | ---------------- |
| `up`         | GP2  |       4 | GND（物理pin 8） |
| `down`       | GP3  |       5 | GND（物理pin 8） |
| `left`       | GP4  |       6 | GND（物理pin 8） |
| `right`      | GP5  |       7 | GND（物理pin 8） |
| `red`        | GP6  |       9 | GND（物理pin 8） |
| `yellow`     | GP7  |      10 | GND（物理pin 8） |
| `green`      | GP8  |      11 | GND（物理pin 8） |

配線を変える前にUSB cableを抜きます。`VBUS`、`VSYS`、`3V3(OUT)`へbuttonを接続しません。
外付けpull-up抵抗は不要です。

## 確定構成の記録

本番機と予備機には、同じcommitから次の3 fileをまとめて転送します。一部だけを更新しません。

```text
button_firmware.py
main.py
serial_protocol_poc.py
```

| 項目                               | 確定値                                                     |
| ---------------------------------- | ---------------------------------------------------------- |
| repository commit SHA              | `5534dc427d50ab47756d5513179a69f54a90a8cc`（実機再確認時） |
| MicroPython build／UF2             | v1.29.0／`RPI_PICO`                                        |
| `CTRL-MAIN`のfirmware元commit      | `c87632274aee3f158f097f40460d3c1eef278005`                 |
| `CTRL-SPARE`のfirmware元commit     | 未確認（本番前に確認）                                     |
| browser名／version                 | 未確認（本番前に確認）                                     |
| OS                                 | Windows＋WSL2 Ubuntu 24.04.2（直接read確認時）             |
| 本番origin（HTTPSまたはlocalhost） | 未確認（本番前に確認）                                     |
| 使用USB port label                 | COM3／USB serial `e665385447753326`（browser未確認）       |
| 試験Room／問題                     | 未確認（本番前に確認）                                     |
| 試験開始時の操作列／許可control    | 未確認（本番前に確認）                                     |
| 送信buttonの表示名                 | 未確認（本番前に確認）                                     |
| 送信後に期待する判定表示           | 未確認（本番前に確認）                                     |
| `CTRL-MAIN`書込み・試験担当／日時  | 現物担当／2026-09-03                                       |
| `CTRL-SPARE`書込み・試験担当／日時 | 未確認（本番前に確認）                                     |

転送は[`README.md`のUpload Project手順](README.md#upload-project)に従います。転送後はMicroPicoを
Disconnectし、BOOTSELを押さずにUSBを物理的に抜き差しして`main.py`を自動起動します。

### 2026-09-03の実機再確認

- Raspberry Pi Pico H、MicroPython v1.29.0、USB serial `e665385447753326`を確認しました。
- Pico上のproduction 3 fileとrepositoryのSHA-256が一致しました。
  - `button_firmware.py`: `f226a9753715044eee491edab648804cc4ecdd18ba982972a0b1b64b3d4b7282`
  - `main.py`: `b3f6f4b505eb80df84b9e47da145f09083860a0d746eebd98090f1c1a66e3001`
  - `serial_protocol_poc.py`: `e4d521a296994859acec6e03c740623327e4a3b69fe906f5acfcc40b3db5e049`
- production `main.py`をhard resetで起動し、直接readで`up`／`short_press`が1 frameだけ出力されることを
  確認しました。
- 全7 controlなどの厳密な再試験は行わず、同一firmwareで確認済みの2026-09-02 Production手動確認matrixを
  継承します。
- 生captureとraw logは保存していません。

## 開場前の接続手順

1. 台帳とlabelを照合し、`CTRL-MAIN`、`CABLE-MAIN`、`PC-MAIN`を設置します。
2. 7 buttonをすべて解放し、配線抜け、Picoの破損、cableの緩みがないことを目視確認します。
3. controllerを固定し、button操作や人の移動でcableが引っ張られないようにします。
4. MicroPico、serial monitor、別のbrowser tabがPicoのportを開いていないことを確認します。
5. 予備機を接続せず、`CABLE-MAIN`でPicoとlabel済みUSB portを接続します。通常接続ではBOOTSELを
   押しません。
6. 対応Chromium browserから記録済みの本番originと試験Roomを開きます。
7. 「接続する」を押し、接続中のPicoをdevice pickerで選びます。
8. 「Serialに接続しました」を確認します。
9. 試験Roomで代表の許可controlを1回押し、対応する操作が1回だけ反映されることを確認します。

device pickerが自動で開いた場合、異なるdeviceしか表示されない場合、操作が重複する場合は開始せず、
代替入力へ切り替えて担当者へ連絡します。

## UI表示と当日操作

表示文言はRoom画面を正とします。

| 表示                               | 当日操作                                                                                |
| ---------------------------------- | --------------------------------------------------------------------------------------- |
| この環境ではSerialを利用できません | browser、HTTPS／localhost、使用PCを確認する。ゲームはkeyboardまたは画面buttonで開始する |
| Serial接続が許可されませんでした   | 初回のdevice pickerを閉じた場合を含む。「再試行する」を1回押すか、代替入力へ切り替える  |
| Serialへ接続中です                 | picker操作または接続完了を待つ。buttonを連打しない                                      |
| Serialに接続しました               | 物理controllerを使用できる。必要なら接続済み状態から代替入力へ切り替えてよい            |
| Serialは接続されていません         | 「接続する」を押すか、代替入力へ切り替える                                              |
| Serialの再接続に失敗しました       | 再接続時のpickerを閉じた場合も含む。代替入力へ切り替えてから機材を交換する              |

portの解放に失敗して「Serialを解放する」が表示された場合も、先に代替入力で進行を続けます。安全な
タイミングで「Serialを解放する」を押し、解放後に改めて「接続する」からportを選択します。

代替入力を選んだ直後は、portの解放が終わるまで接続済み表示が一時的に残る場合があります。この間は
「キーボード入力」または「画面ボタン入力」の表示を現在の入力方法として扱います。

## 故障・切断時

### ゲームを継続する

1. 同じ操作を重ねて入力せず、Roomの現在の操作列を確認します。
2. 未接続、拒否、切断の表示なら、「接続する」または「再試行する」を1回だけ試します。
3. 再接続失敗の表示、接続済み表示のままbuttonが反応しない場合、または手順2で接続できない場合は、
   「キーボードを使う」または「画面ボタンを使う」を押します。
4. 許可されたcontrolを1回操作し、同じRoomで操作が1回だけ増えることを確認します。
5. そのroundまたはゲームは代替入力のまま継続します。以降の物理的な機材調査は進行と並行できます。

keyboardではRoomに表示されたキーだけを使用します。画面buttonには、その問題で許可されたcontrolだけが
表示されます。

### cable、controllerの順に交換する

1. 代替入力で進行できていることを確認します。
2. Picoから`CABLE-MAIN`を抜き、`CABLE-SPARE`へ交換します。
3. round間、ゲーム終了後、または試験Roomなど、入力を止められるタイミングまで待ちます。
4. 「接続する」を押し、Picoを選んで試験操作を1回行います。この操作中は代替入力を使用できません。
5. 接続または試験操作に失敗したら、同じ代替入力を改めて選びます。
6. USB cableを抜き、controller一式を`CTRL-SPARE`へ交換します。現場で配線を直しません。
7. 次の安全な停止点で「接続する」から予備機を選び、試験操作を1回行います。
8. 失敗したら代替入力を改めて選び、そのゲームではSerialへ戻しません。
9. 故障品へ`使用停止`の表示を付け、症状と時刻をIssue #82へ記録します。

USBを抜き差しするとPicoは電源再投入され、`main.py`が自動起動します。browserがportをcloseして再び
openしただけではPicoは再起動されません。

## 本番前smoke

firmwareのdebounce境界、全7 button、power cycle、parserのinvalid frameは既存のunit testと
[`README.md`のProduction手動確認matrix](README.md#production手動確認matrix)を正とし、このrunbookでは
繰り返しません。controllerのUSB serial、MicroPython、production 3 fileのSHA-256が既受入記録と一致する
場合は、代表の許可control 1件だけをsmoke確認します。画面の操作列の前後値と実施環境を記録し、結果欄は
実施後だけPASSまたはFAILへ変更します。

| case           | 操作                                                                      | 合格条件                                                              | 結果   |
| -------------- | ------------------------------------------------------------------------- | --------------------------------------------------------------------- | ------ |
| Serial経路     | `CTRL-MAIN`へ接続し、試験Roomの許可controlを1回操作する                   | 接続済み表示となり、対応する操作が1回だけ増える                       | 未実施 |
| 切断・代替経路 | cableを抜き、同じRoomでkeyboardと画面buttonから許可controlを各1回操作する | 切断表示となり、自動pickerなし。既存操作が消えず各入力で1回ずつ増える | 未実施 |
| ゲーム経路     | 記録した送信buttonを1回押す                                               | 表示中の操作列が送信され、記録した判定表示となる                      | 未実施 |

### 当日smoke

1. 記録済みのcontroller、USB serial、browser、origin、Roomを照合します。
2. Serialへ接続し、代表の許可controlを1回だけ操作して、操作が1回だけ増えることを確認します。
3. keyboard／画面buttonへの切替操作が表示されていることを確認します。Serialが使えない場合だけ一方へ
   切り替え、許可controlを1回操作して継続できることを確認します。
4. 実施者、日時、対象commit、`Serial PASS`／`代替入力で運用`／`FAIL`を記録します。

| 実施記録                | 内容                        |
| ----------------------- | --------------------------- |
| 実施者／日時            |                             |
| 対象commit              |                             |
| 使用機材label           |                             |
| PC／OS／browser／origin |                             |
| 操作列（確認前→確認後） |                             |
| 総合結果                | Serial PASS／代替入力／FAIL |

FAILの場合はその場でfrontendやfirmwareを修正せず、該当領域のIssueを作成します。firmware、MicroPython、
配線を変更した場合は`mise run device-test`を通してから、`README.md`のProduction手動確認matrixを先頭から
実施します。

## 終了・保管

- [ ] Roomを閉じた
- [ ] PCからUSB cableを抜いた
- [ ] `CTRL-MAIN`、`CTRL-SPARE`、2本のcableをlabelと照合した
- [ ] 破損、配線抜け、接続不良の有無を記録した
- [ ] 故障品を正常品と分け、`使用停止`表示を付けた
- [ ] 台帳に返却先と確認者を記録した

保管先: ____________________ 確認者／日時: ____________________
