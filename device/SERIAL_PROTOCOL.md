# Device Serial Protocol v1

## 位置付け

この文書は、Issue #92で定めるRaspberry Pi Pico Hとfrontend間のSerial Protocol v1の規範です。
deviceが生成するbutton操作eventと、frontendが受信してdevice event列へ追加するまでの契約を定めます。

この文書で「必須」とする事項がv1の契約です。現在のraw GPIO診断用`button_test.py`の出力仕様では
ありません。診断PoCとの違いは「実測根拠とsampleの区別」を参照してください。

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
schema適合frameも受理します。ただし、deviceは再現可能なsampleを作るため上記canonical形式で送信します。

Serial portのopen設定やUSBの接続手順はpayload／framing契約とは分離します。現在のPoC設定と手順は
[client README](../client/README.md)を参照してください。

### Raw REPLを使うPoCのstream境界

raw REPLのbanner、command受付応答、停止応答はtransport制御であり、Wire v1 streamではありません。
現在の開発用raw viewerのようにraw REPLからscriptを起動する場合、capture自体には全byteを残します。
Wire v1 parserを製品frontendへ統合するときは、parserへ渡す範囲を接続ごとに次のように限定します。
現raw viewerはこの切出しやparser処理をまだ行いません。

- 開始: metadataの`scriptActiveObservedOffset`。raw REPLの`OK`を確認し、専用scriptの起動継続を確認した後
- 終了: 正常停止時は`stopRequestedOffset`、切断またはread error時は`endedOffset`

byte範囲は開始を含み終了を含まないhalf-open intervalです。

frontendはこの開始offsetでframe bufferを空にしてからv1の処理を始め、終了offsetで残ったpartial frameを
破棄します。開始offsetより前と終了offset以後のbyteをv1 parserへ渡しません。offsetの意味とraw captureの
保存方法は[Web Serial sample README](../client/samples/web-serial/README.md)も参照してください。

製品用firmwareがraw REPLを介さずWire v1だけを送る構成では、このPoC固有のoffset境界は不要です。

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

## 責任境界

### Device

- GP2～GP8を内部pull-up付き入力として読み取る
- buttonごとに20 ms debounceを行う
- 700 ms thresholdで`short_press`または`long_press`をrelease時に1回だけ確定する
- button／GPIOをcontrolへ対応付ける
- Frame schemaに一致するUTF-8 JSON lineをstream順に送る
- repeat、ack、再送を行わない

### Frontend

- 利用者gestureからserial接続と再接続を開始し、切断時は自動再接続しない
- read chunkを接続ごとのbyte streamとして結合し、LFでframeへ分割する
- CRLF、strict UTF-8、JSON、Frame schema、256 byte上限を検証する
- invalid frameをline単位で破棄し、次のLFから再同期する
- valid frameをstream順にdevice event列へ1回ずつ追加し、切断や再接続をまたいで保持する
- device側のdebounce、長押し判定、control対応を再判定しない

deviceはfrontendの画面状態、event列、query送信を管理しません。frontendはraw GPIO edgeを受け取らず、
deviceが確定したgestureだけを扱います。Wire v1 eventをOpenAPIの`Operation { control, count }`へ変換する
製品Adapter、とくに`long_press`を`count`や別の画面動作へどう対応させるかはこの契約では定めません。
未確定の変換を推測してWire v1 eventをquery APIへ直接送らないでください。

## 実測根拠とsampleの区別

2026-09-01の[Web Serial diagnostic captures](../client/samples/web-serial/diagnostic/)では、Raspberry Pi
Pico Hの全7入力、CRLFを含む診断出力、読取り中のUSB切断と利用者操作による再接続を確認しました。
これらはraw REPL応答と`button_test.py`の人向け診断行を含む、transportとGPIO挙動の実測記録です。

この診断captureはJSON Frame schema、20 ms debounce、700 ms threshold、short／long gestureを送るWire v1
の実測記録ではなく、protocol parserのcanonical sampleとして流用しません。

2026-09-02の[Wire v1 captures](../client/samples/web-serial/protocol-v1/)では、専用PoCを実機で実行し、
次を確認しました。

- 全7 controlの`short_press`と、`up`の`long_press`
- 起動時LOWの最初のreleaseを無視し、次のgestureから出力する動作
- 保持中repeatなし、短い間隔の複数操作、buttonを重ねた独立入力
- compact ASCII JSON、CRLF、1 frameが複数read chunkへ分割されるtransport境界
- 正常停止、読取り中のUSB切断、capture保持、利用者gestureによる再接続と再開

元の`.bin`とmetadata、SHA-256、取得環境、接続境界を保持し、raw REPL制御を除いたWire v1範囲を
`scriptActiveObservedOffset`から`stopRequestedOffset`または`endedOffset`までとして記録しています。
実機sampleを元のchunk境界でparserへ流すtestもこの範囲を使用します。

実機が送信しないLF単独、invalid UTF-8／JSON／schema、overlong、切断時partial frameは
contract-synthetic fixtureで検証し、実機由来のcaseとは明示的に区別します。

[device vREPL samples](samples/)もraw GPIO診断の実測記録であり、canonical protocol sampleではありません。

## 対象外

- 製品版の画面や操作UI
- 製品版parser／Adapterの実装
- firmware完成版の実装と配布
- backend処理、HTTP API、query送信処理

この文書は上記の実装完了を主張せず、それらが従うdevice／frontend間のWire v1契約だけを定義します。
