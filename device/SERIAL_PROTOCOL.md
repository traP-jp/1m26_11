# Device Serial Protocol v1

## 位置付け

この文書は、Issue #92で定めたRaspberry Pi Pico Hとfrontend間のSerial Protocol v1の規範です。
deviceが生成するbutton操作eventと、frontendが受信してdevice event列へ追加するまでの契約を定めます。
Issue #81のproduction firmwareは`firmware/button_firmware.py`、自動起動は`firmware/main.py`でこの規範を
実装します。Issue #92の契約変更とIssue #81の実装変更を同一視せず、規範への適合をunit testと実機確認の
記録で追跡します。

この文書で「必須」とする事項がv1の契約です。現在のraw GPIO診断用`button_test.py`の出力仕様では
ありません。`poc/`はIssue #92を確定した時点の調査履歴で、production entrypointではありません。
診断PoCとの違いは「実測根拠とsampleの区別」を参照してください。

## Wire format

- byte列はUTF-8です。
- v1のJSON payloadはASCII subsetだけで構成します。
- 1 lineにJSON objectを1個だけ格納します。複数objectを同じlineへ格納しません。
- frame delimiterはLF（`0x0a`）です。1回のserial readで受信したchunk境界をframe境界とは扱いません。
- deviceは各frameを必ずLFで終端します。runtimeがLFの直前へCRを加えるCRLF出力もv1準拠です。
- frontendはline末尾のLF直前にCR（`0x0d`）が1個あるCRLFも受理し、そのCRをpayloadから除きます。
- LFまたはCRLFのどちらでも、delimiterはpayload長に含めません。
- 1 frameの上限は、delimiterとCRLFのCRを除いたUTF-8 byte列で256 byteです。

deviceが送るcanonical JSONは空白を含めず、keyを`v`、`control`、`gesture`の順に並べます。
たとえば、上方向buttonの短押しは次のbyte列です。末尾の`\n`はLFを表します。

```text
{"v":1,"control":"up","gesture":"short_press"}\n
```

JSON objectのkey順とJSON上許される空白は意味を持たないため、frontendは順序や空白だけが異なる
schema適合frameも受理します。ただし、deviceは再現可能な出力にするため上記canonical形式で送信します。

Serial portのopen設定やUSBの接続手順はpayload／framing契約とは分離します。production firmwareの
書込み・起動・確認手順は[device README](README.md)、過去のPoC viewer設定は
[client README](../client/README.md)を参照してください。

### Productionの直接readとRaw REPL PoCのstream境界

Issue #81のproduction firmwareはPico起動時に`/main.py`から自動起動します。Issue #27の製品frontendは、
利用者のgestureでportをopenした後、このbyte streamを直接readします。raw REPLへの切替え、script起動／停止
command、capture offsetによるWire区間の切出しは行いません。接続中にdeviceが送るapplication出力はWire v1
frameだけでなければなりません。

raw REPLのbanner、command受付応答、停止応答はtransport制御であり、Wire v1 streamではありません。
過去の開発用raw viewerのようにraw REPLからPoC scriptを起動する場合、capture自体には全byteを残します。
PoC sampleをparserへ入力するときだけ、parserへ渡す範囲を接続ごとに次のように限定します。現raw viewerは
この切出しやparser処理をまだ行いません。

- 開始: metadataの`scriptActiveObservedOffset`。raw REPLの`OK`を確認し、専用scriptの起動継続を確認した後
- 終了: 正常停止時は`stopRequestedOffset`、切断またはread error時は`endedOffset`

byte範囲は開始を含み終了を含まないhalf-open intervalです。

frontendはこの開始offsetでframe bufferを空にしてからv1の処理を始め、終了offsetで残ったpartial frameを
破棄します。開始offsetより前と終了offset以後のbyteをv1 parserへ渡しません。offsetの意味とraw captureの
取得方法は[client README](../client/README.md)も参照してください。取得したcaptureはローカルでの確認にだけ
使用し、repositoryやPRには添付しません。

production firmwareでは、このPoC固有のoffset境界は不要です。production確認時のraw streamへraw REPL
制御byteが含まれていた場合は、offsetで除外して合格にせず、確認手順または接続方法の誤りとして扱います。

## Frame schema

frameは次の3 keyだけを、それぞれ1回ずつ持つJSON objectです。keyの欠落、追加、重複は許可しません。

| key | JSON type | 許可値 | 意味 |
| --- | --- | --- | --- |
| `v` | number | integer `1` | protocol version |
| `control` | string | `up`, `down`, `left`, `right`, `red`, `yellow`, `green` | game control |
| `gesture` | string | `short_press`, `long_press` | release時に確定した操作 |

`v`が`1`以外、未知の`control`または`gesture`、型が異なる値はinvalid frameです。v1にはtimestamp、
button番号、GPIO番号、押下／解放level、event IDを追加しません。

deviceはv1 event streamへbanner、debug log、raw GPIO診断行を混在させません。混在したlineはfrontendが
invalid frameとして破棄しますが、その破棄によって非JSON出力をv1準拠とみなすものではありません。

同じ内容のvalid frameが複数届いた場合、frontendは各lineを別々の操作として受理します。v1にはack、
再送、重複排除の仕組みはありません。

## Button、GPIO、control対応

GPIOは入力と内部pull-upを使用し、未押下はHIGH、押下はLOWです。配線の詳細は
[device README](README.md)を参照してください。

| button | GPIO | control |
| --- | --- | --- |
| 1 | GP2 | `up` |
| 2 | GP3 | `down` |
| 3 | GP4 | `left` |
| 4 | GP5 | `right` |
| 5 | GP6 | `red` |
| 6 | GP7 | `yellow` |
| 7 | GP8 | `green` |

## Debounceとgesture判定

debounceとgesture判定はdeviceの責任です。各buttonを独立して、次の規則で処理します。

1. raw GPIO levelが変化した後、同じlevelが20 ms以上連続した時点で状態遷移を確定します。
2. debounce済みのHIGHからLOWへの遷移で押下開始時刻を記録します。この時点ではframeを送りません。
3. 次のdebounce済みLOWからHIGHへの遷移で保持時間を確定します。
4. 保持時間が700 ms未満なら`short_press`、700 ms以上なら`long_press`とします。
5. release時に、判定したgestureのframeをどちらか1個だけ送ります。

保持時間は、debounce済みの押下遷移を確定した時刻から、debounce済みの解放遷移を確定した時刻まで
です。押下保持中のrepeatは送らず、同じ押下に`short_press`と`long_press`の両方を送りません。
frontendは追加のdebounceや保持時間の再計算を行わず、valid frameの`gesture`をそのまま使用します。

device起動時または入力監視開始時にGPIOがLOWだったbuttonは、開始前から押されていたものとして無視
します。そのbuttonが20 ms以上連続してHIGHとなった最初のreleaseではframeを送らず、以後の押下から
受付を開始します。これにより、USB再接続時に押したままのbuttonを1操作として誤送信しません。
起動時にHIGHだったbuttonはその時点で受付可能とし、起動後のLOW遷移に通常の20 ms debounceを
適用します。HIGH入力のみを理由に別の起動待ち時間は設けません。

複数buttonは、debounce状態、押下開始時刻、gestureを相互に影響させず独立して管理します。同時または
重なって操作された場合も、各releaseからframeを1個ずつ生成します。deviceがserial streamへ書き込んだ
line順が操作順であり、frontendは並べ替えたり「同時操作」へまとめたりしません。同じpollで複数frameが
生成された場合も、stream上の順序を正とします。

## Frontendのframe処理

frontendはserial readのchunkを接続ごとのbyte bufferへ順番どおり追加し、byte値`0x0a`でlineを
切り出してからUTF-8 decodeとJSON parseを行います。1 frameが複数chunkへ分割された場合も、複数frameが
1 chunkへまとまった場合も同じ結果にします。

lineごとの処理順は次のとおりです。

1. LFまでのbyte列を1 lineとして切り出します。
2. line末尾がCRなら、そのCRを1個だけ除きます。
3. CRを除いたpayloadが256 byteを超える場合はoverlongとして破棄します。
4. payloadをstrict UTF-8としてdecodeします。置換文字で補完しません。
5. JSON objectをparseし、3 keyだけを持つFrame schemaへ照合します。
6. valid frameだけを、受信line順に1件のdevice eventとして受理済みevent列へ追加します。

LF受信前は、256 byteのpayloadにCRを1個加えた最大257 byteまでを候補として保持できます。257 byte目が
CRでない場合、257 byteのCRの後へLF以外のbyteが続いた場合、または候補が257 byteを超えた場合は、その
lineをoverlongと確定して次のLFまで読み飛ばします。これにより、上限を超える未完lineをbufferへ保持し
続けません。

次はすべてinvalid frameです。

- 空lineまたはUTF-8として不正なbyte列
- JSONとしてparseできないpayload、またはJSON object以外の値
- keyの欠落、追加、重複、型不一致
- 未知のversion、control、gesture
- 256 byteを超えるpayload

invalid frameはline単位で破棄し、受理済みevent列へ追加しません。接続全体を失敗扱いにせず、次のLFから通常の
処理へ復帰します。invalid payloadに含まれる文字列を部分的に解釈したり、最も近いcontrolへ補正したり
しません。raw captureを有効にしている場合は、parserが破棄したbyteも観測証拠としてraw captureには
残します。

## 切断と再接続

byte bufferとframeの組立て状態はserial connectionごとに分離します。LFを受信する前に切断したpartial
frameは破棄し、再接続後のbyteと連結しません。overlong lineを次のLFまで読み飛ばしている状態も接続終了時
に破棄します。切断前にLFまで受信してschema検証を通過した操作は取り消しません。

初回接続と再接続は、どちらも利用者が明示的に接続操作を行ったgestureから開始します。frontendは切断後に
portを自動で再openせず、切断状態を表示して利用者の再接続操作を待ちます。

切断、read error、invalid frame、再接続によって、それまでに受理したdevice event列を自動消去しません。
再接続後のvalid frameは既存event列の末尾へ追加します。event列を消去するのは、利用者が明示的に入力を
clearまたは新しい入力sessionを開始した場合だけです。そのUIや送信後の画面遷移はこのprotocolの対象外です。

ここでfrontendの「再接続」とdeviceの「再起動」を区別します。

- hostがserial portをcloseして同じ給電中のPicoを再openしても、deviceの状態機械はresetされません。
- Issue #81の電源再投入／USB再接続試験は、PicoがUSBだけで給電されている状態でcableを物理的に抜き差し
  するpower cycle／hard resetです。再接続後は`main.py`が状態機械を新しく作って自動起動します。
- 給電中のhost close／openの間に開始または完了した操作がhostへ必ず届くことはv1では保証しません。v1には
  offline queue、ack、再送がありません。
- host close／open後も、押下中のbuttonなどdevice側の状態は継続し得ます。frontendはopenをdevice resetと
  みなさず、connectionごとのparser bufferだけを新しくします。

hard reset時にLOWだったbuttonは「Debounceとgesture判定」の起動時規則に従い、最初の安定したreleaseを
eventにしません。host close／openだけでは、この起動時規則をもう一度適用しません。

## 責任境界

### Device

- power cycle／hard reset後に`main.py`から監視を自動起動し、状態を新しく作る
- GP2～GP8を内部pull-up付き入力として読み取る
- buttonごとに20 ms debounceを行う
- 700 ms thresholdで`short_press`または`long_press`をrelease時に1回だけ確定する
- button／GPIOをcontrolへ対応付ける
- Frame schemaに一致するUTF-8 JSON lineをstream順に送る
- Wire v1 streamへbanner、debug log、raw REPL応答を混在させない
- repeat、offline queue、ack、再送を行わない

### Frontend

- 利用者gestureからserial接続と再接続を開始し、切断時は自動再接続しない
- read chunkを接続ごとのbyte streamとして結合し、LFでframeへ分割する
- CRLF、strict UTF-8、JSON、Frame schema、256 byte上限を検証する
- invalid frameをline単位で破棄し、次のLFから再同期する
- valid frameをstream順にdevice event列へ1回ずつ追加し、切断や再接続をまたいで保持する
- device側のdebounce、長押し判定、control対応を再判定しない

deviceはfrontendの画面状態、event列、query送信を管理しません。frontendはraw GPIO edgeを受け取らず、
deviceが確定したgestureだけを扱います。WebSerialInputAdapterは各valid Wire v1 frameを次の共通入力eventへ
1回ずつ変換します。

```json
{"type":"condition-changed","source":"serial","control":"up","count":1}
```

- `source`は`serial`
- `control`は受理したframeの値をそのまま使用する
- `short_press`と`long_press`はどちらも、確定した1 gestureとして`count: 1`へ変換する
- `gesture`は共通入力eventへ追加しない
- 現在の問題で許可されていない`control`は共通入力eventへ変換しない
- frontendでdebounce、長押しの再判定、controlの再mapping、重複除去を行わない

隣接する同じ`control`の共通入力eventは、後段のOperationBufferが`count`へまとめます。この集約は
frontendでbuttonのbounceや重複frameを除去する処理ではありません。`long_press`にも`count: 1`以外の
特殊な動作を割り当てません。

Issue #27はこのFrontend責任のうちport lifecycle、接続状態、切断復帰、cleanup、代替入力への導線を扱います。
production firmwareの起動commandを送ることや、hostのport close／openでdevice状態をresetすることは
Issue #27の責任に含めません。

## 実測確認とsynthetic testの区別

2026-09-01のWeb Serial診断では、Raspberry Pi Pico Hの全7入力、CRLFを含む診断出力、読取り中の
USB切断と利用者操作による再接続を確認しました。raw REPL応答と`button_test.py`の人向け診断行を含む
transportとGPIO挙動を確認しましたが、取得したcapture自体はrepositoryへ保存しません。

この診断captureはJSON Frame schema、20 ms debounce、700 ms threshold、short／long gestureを送るWire v1
の実測記録ではなく、protocol parserのcanonical sampleとして流用しません。

2026-09-02のWire v1確認では、`poc/serial_protocol_poc.py`をraw REPLから実機で実行し、次を確認しました。

- 全7 controlの`short_press`と、`up`の`long_press`
- 起動時LOWの最初のreleaseを無視し、次のgestureから出力する動作
- 保持中repeatなし、短い間隔の複数操作、buttonを重ねた独立入力
- compact ASCII JSON、CRLF、1 frameが複数read chunkへ分割されるtransport境界
- 正常停止、読取り中のUSB切断、capture保持、利用者gestureによる再接続と再開

実機確認時はraw REPL制御を除いたWire v1範囲を、`scriptActiveObservedOffset`から
`stopRequestedOffset`または`endedOffset`までとして確認しました。取得した`.bin`とmetadataは
ローカル確認後にrepositoryやPRへ添付しません。

実機が送信しないLF単独、invalid UTF-8／JSON／schema、overlong、切断時partial frameは
contract-synthetic fixtureで検証し、実機由来のcaseとは明示的に区別します。

過去のdevice vREPL transcriptもraw GPIO診断の実測記録であり、canonical protocol sampleではありません。

Issue #81の`firmware/`はhardware非依存の状態機械testを`mise run device-test`で検証します。このtestは
19／20 ms、699／700 ms、bounce、連打、長押し、起動時LOW、複数button、tick wrapなどを決定的に確認する
synthetic入力であり、production実機sampleではありません。

2026-09-02にproduction `main.py`を書き込んだPicoを直接readし、canonical frame以外がないこと、物理的な
電源再投入、押下中再投入を確認しました。captureは同日のPoCとは別にローカルで照合し、repositoryやPRへは
添付しません。別担当による再現は未実施です。

## 対象外

- 製品版の画面や操作UI
- Issue #27のport管理、切断復帰、代替入力UI
- custom UF2やfirmware updaterとしての配布
- backend処理、HTTP API、query送信処理

frontendの純粋parserとWebSerialInputAdapterはこの契約に従って実装します。この文書は対象外の製品統合や
backendの実装完了、および未実施の別担当再現を主張しません。
