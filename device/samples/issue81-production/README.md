# Issue #81 production firmware 実機capture

このdirectoryは、2026-09-02にIssue #81のproduction firmwareをRaspberry Pi Pico H（RP2040）で
一次確認したraw serial byteと期待event列を保存します。Pico起動時に`/main.py`から自動起動した
firmwareを直接readしており、raw REPL、手動のscript起動command、banner、promptは含みません。

採用した全caseで、raw byteは[Serial Protocol v1](../../SERIAL_PROTOCOL.md)のcanonical JSONとCRLFだけで
構成され、同じraw byteをfrontendの`SerialProtocolPocParser`へ入力した結果が`.expected.jsonl`と
一致しました。これは下記の1台・1環境での一次確認結果です。実装・capture・byte／parser検証はCodex、
物理buttonとUSBの操作はユーザーが分担して行ったため、**別担当による再現は未実施**です。

## 実測環境

| 項目 | 実測値 |
| --- | --- |
| 実施日 | 2026-09-02（JST。caseごとの時刻は未記録） |
| source | `c87632274aee3f158f097f40460d3c1eef278005`（実施時の`origin/feat/issue-81`） |
| board | Raspberry Pi Pico H（RP2040） |
| MicroPython | v1.29.0 |
| USB serial | `e665385447753326` |
| host | Windows host、WSL2 Ubuntu 24.04.2 LTS |
| WSL kernel | `6.18.33.2-microsoft-standard-WSL2` |
| capture | GNU coreutils 9.4の`stty`と`dd`、serial by-id pathからbinaryで直接保存 |
| serial設定 | 115200 baud、8-N-1、raw、flow controlなし |
| WSL USB転送 | `usbipd-win`（version未記録）、実施時のBUSIDは`2-2` |

物理的なUSB抜き差しまたはhard resetの後は、WSLからserial deviceが見えなくなったため、その都度
Windows側でBUSID `2-2`をWSLへ再attachしてからcaptureを再開しました。BUSIDはこの実施環境固有であり、
別環境で同じ値になるとは限りません。

## Production firmwareの同一性

転送前のlocal fileとPico filesystem上のremote fileで、次のSHA-256が一致することを確認しました。

| Pico root | local source | SHA-256（local／remote） |
| --- | --- | --- |
| `/button_firmware.py` | `device/firmware/button_firmware.py` | `f226a9753715044eee491edab648804cc4ecdd18ba982972a0b1b64b3d4b7282` |
| `/serial_protocol_poc.py` | `device/firmware/serial_protocol_poc.py` | `e4d521a296994859acec6e03c740623327e4a3b69fe906f5acfcc40b3db5e049` |
| `/main.py` | `device/firmware/main.py` | `b3f6f4b505eb80df84b9e47da145f09083860a0d746eebd98090f1c1a66e3001` |

`serial_protocol_poc.py`もproduction 3 fileの一つとして転送されていますが、このcaptureでは
`main.py`の自動起動を使用し、互換entrypointを手動実行していません。

### 今回の書込み記録

今回の実測では、公式[`mpremote`](https://docs.micropython.org/en/latest/reference/mpremote.html) v1.29.0を
repository外の一時venvへ導入し、auto detectではなく確認済みの`/dev/ttyACM0`を明示して操作しました。
書込み前のPico rootは旧`button_test.py`とPoC版`serial_protocol_poc.py`の2 fileでした。両fileを
`/tmp`へbackupし、remote／backupのSHA-256一致を確認してからproduction 3 fileを転送しました。

転送後は上表のlocal／remote SHA-256一致を確認し、backup済みの旧`button_test.py`だけを個別に除去して、
Pico rootがproduction 3 fileだけであることを再確認しました。未知fileの一括削除やfilesystem全体の消去は
行っていません。その後は`mpremote`を閉じ、物理電源再投入後の`main.py`をraw REPL commandなしで直接read
しました。別担当向けの標準的な書込み手順は[device README](../../README.md#upload-project)を参照してください。

## Fileの対応

各prefixについて、次の3 fileを一組として扱います。

- `.bin`: `dd`で直接保存した無加工の受信byte列。改行変換、frameの追加・削除、並べ替えをしていません。
- `.expected.jsonl`: 期待event列。1 eventをcanonical JSON 1行で記し、raw captureそのものではありません。
- `.sha256`: 対応する`.bin`のSHA-256。

eventを期待しない`hard-reset-idle`と`power-cycle-idle`では、`.bin`と`.expected.jsonl`はともに0 byteです。
0-byte `.bin`のSHA-256は
`e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`です。

## 採用した実機matrix

| case | 実操作と無出力区間 | 実event列 | raw byte／SHA-256 | 結果 |
| --- | --- | --- | --- | --- |
| `hard-reset-idle` | 全buttonを解放して`mpremote reset`後に再attachし、無操作で3秒待機 | 0 frame | 0 byte／`e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` | PASS |
| `all-controls` | `up`, `down`, `left`, `right`, `red`, `yellow`, `green`を各1回短押し | 記載順に`short_press` 7 frame | 351 byte／`1a6f7b5a7becf72e777ff5e323a2168e1b7608255501469a23571c49df700bd0` | PASS |
| `rapid` | `up`を、解放を挟んで短い間隔で3回短押し | `up / short_press` 3 frame | 144 byte／`eefa5e8e35118d678b819ad76776abb38ba23b56fc7bddc4adb8a947781c9d1b` | PASS |
| `long` | `up`を700 msより十分長く保持して解放 | 保持中0 frame、解放時に`up / long_press` 1 frame | 47 byte／`06feed2b688293a2deba9411947441e0015fea6e650cd3f546af13a66eb1badd` | PASS |
| `bounce-prone` | switch 1を浅く、ぐらつかせるように1回操作。操作は700 ms以上となった | `up / long_press` 1 frame、重複0 frame | 47 byte／`06feed2b688293a2deba9411947441e0015fea6e650cd3f546af13a66eb1badd` | PASS（下記制約あり） |
| `mixed-colors` | `green`を押したまま`red`を2回短押しし、最後に`green`を解放 | `red / short_press` 2 frame、`green / long_press` 1 frame | 148 byte／`ba6c5a03c9105c2ef8913ea30a61543a92a7661a00359f7a5099fe29c3071cff` | PASS |
| `power-cycle-idle` | USBを物理的に抜き差しし、WSLへ再attachした後、無操作で待機 | 0 frame | 0 byte／`e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` | PASS |
| `power-cycle` | 上記の再接続後に`up`を1回短押し | `up / short_press` 1 frame | 48 byte／`a5ef93510d4f4a5b2bdf672aca3e846cdb5402d469e5b46c712b0d6b5e694828` | PASS |
| `held-power-cycle` | switch 1を押したまま起動し、最初のrelease後に通常の短押しを1回実施 | 起動時保持中と最初のreleaseは0 frame、その後`up / short_press` 1 frame | 48 byte／`a5ef93510d4f4a5b2bdf672aca3e846cdb5402d469e5b46c712b0d6b5e694828` | PASS |

`long`と`bounce-prone`はどちらもcanonicalな`up / long_press` 1 frameだけなので、raw byteとhashが
同一です。同様に、`power-cycle`と`held-power-cycle`も`up / short_press` 1 frameだけなので同一です。
起動条件や手操作の違いはraw frame内に含まれないため、このREADMEの実施記録と組み合わせて扱います。

`hard-reset-idle`から`mixed-colors`までは同じ`main.py`の連続run中に取得し、case間でserial captureだけを
close／openしました。途中には期待した手操作にならず不採用とした操作もありましたが、firmware resetや
raw REPLによる状態初期化は行っていません。その後も採用caseが期待列へ戻ったため、連続操作後に各buttonの
gesture状態が次の完全な操作を受け付けることも確認しました。`power-cycle-*`と`held-power-cycle`は、それぞれ
指定どおり物理電源再投入した別runです。

## Byte／parser確認

採用した各`.bin`について、次を確認しました。

- `.sha256`の値と実際のraw byteのSHA-256が一致する。
- 非空captureは、空白なし、key順が`v`, `control`, `gesture`のcanonical JSONだけを含む。
- 全frameの終端はCRLF（`0x0d 0x0a`）で、LF単独、banner、debug行、空lineは含まれない。
- raw byteを加工せずfrontendの`client/src/device-poc/serialProtocolPoc.ts`のparserへ入力すると、
  `.expected.jsonl`と同じevent列になり、invalid frameは0件となる。
- 操作数とevent数がmatrixの期待値に一致し、余分なrepeatや重複frameはない。

## 証拠範囲と未実施事項

- `.bin`はevent出力だけを保持し、GPIOのraw edge、押下開始時刻、保持時間、無操作の待機時間を記録しません。
  long判定や無出力区間の操作条件は実施時の観察記録であり、raw byte単独から時間を再構成できません。
- `bounce-prone`は、浅くぐらつかせた1操作に対して重複が出なかったことだけを示します。raw GPIO edge
  traceを取得していないため、その操作で実際に接点bounceが発生したことや、発生した全edgeを抑制したことは
  証明しません。
- case条件から外れた押し方や保持時間になった手操作captureは採用していません。採用rawを編集して
  合わせることはせず、このdirectoryの結果にも数えていません。
- `usbipd-win`のversionとcaseごとの実施時刻は未記録です。
- 別の担当者、別のPico、別のhost環境による再現は未実施です。この記録だけを根拠に
  「別担当再現済み」または複数環境でのproduction実機確認済みとは扱いません。
