# Mock問題データ仕様

この文書は、Issue #45で使用するmock問題データの形式、判定設定、およびvalidation規則を説明します。
`mock-problem-data/`内の1部屋4問は、ローカルで読込・validation・判定処理を確認するためのdummy dataであり、
正式な問題内容ではありません。
HTTP APIの契約は`openapi/openapi-v1.yaml`、DB schemaは`server/migrations/0001_schema.sql`を正本とし、
この文書ではそれらを変更せずに入力データから型付きの内部modelへ変換する方法を定めます。

## 対象範囲

- 1部屋につき1つのJSONファイルを読み込む
- 小なぞ3問と大なぞ1問を型付きmodelへ変換する
- problem間の依存関係、入力定義、判定設定をvalidationする
- validation済みmodelに`rooms`と`problems`へ保存するための情報を保持する
- 公開可能なproblem dataをallow-list方式で組み立てる

このIssueではSQL、seed、API route、assetの保存・配信・validationを実装しません。

## ディレクトリ構成

問題データはリポジトリルートの`mock-problem-data/rooms/`へ、部屋ごとに分けて格納します。

```text
mock-problem-data/
├── README.md
└── rooms/
    └── <room_id>/
        ├── room.json
```

- `<room_id>`は小文字のハイフン付きUUID文字列とする
- ディレクトリ名のUUIDと`room.json`の`room.room_id`は一致させる
- JSONはUTF-8、トップレベルはobjectとする
- JSON内の未知fieldは入力ミスとして拒否する
- 画像などのbinary fileはmock問題データへ含めない
- このIssueでは各problemの`assets`を空配列`[]`にする

## トップレベル

`room.json`は`room`と`problems`を必須fieldとして持ちます。

```json
{
  "room": {
    "room_id": "11111111-1111-4111-8111-111111111111",
    "number": 1,
    "name": "最初の部屋",
    "genre": "logic",
    "description": "動作確認用の問題セットです"
  },
  "problems": []
}
```

### room

| field | JSON型 | 規則 |
| --- | --- | --- |
| `room_id` | string | UUID。ディレクトリ名と一致する |
| `number` | integer | `1`以上 |
| `name` | string | 前後空白を除いて空でない |
| `genre` | string | 前後空白を除いて空でない |
| `description` | string | 前後空白を除いて空でない |

DBの`rooms.is_published`は入稿時には`false`とし、公開作業で別途変更します。
`created_at`はDBの`CURRENT_TIMESTAMP(3)`を使用するため入稿データへ含めません。

## problem

各要素はDBの`problems`へ保存するため、次のfieldをすべて必須とします。

| field | JSON型 | 規則 |
| --- | --- | --- |
| `problem_id` | string | UUID。全roomを通して重複不可 |
| `room_id` | string | 親の`room.room_id`と一致する |
| `number` | integer | `1`以上。同じroom内で重複不可 |
| `problem_type` | string | `small`または`final` |
| `title` | string | 前後空白を除いて空でない |
| `body_markdown` | string | 前後空白を除いて空でない |
| `submission_type` | string | `operation_sequence`または`string` |
| `assets` | array | 必須。このIssueでは空配列`[]`だけを許可する |
| `input_schema` | object | 公開可能な入力制限 |
| `hints` | array | `Hint`の配列。空配列可 |
| `judge_config` | object | 非公開の判定設定 |
| `depends_on_problem_id` | stringまたはnull | 同じroom内の別problemを参照する |
| `is_required` | boolean | 現在の必須4問では`true` |

内部modelでは`problem_id`の名前を維持します。公開用modelへ変換するときだけOpenAPIの`id`へ写します。

## 入力schema

現在のOpenAPIでは`query`と`answer`がどちらも必須なので、入稿データでも両方を必須とします。
このIssueではOpenAPIを変更しません。

```json
{
  "query": {
    "type": "operation_sequence",
    "allowed_controls": ["down", "right", "up", "left"],
    "max_operations": 100
  },
  "answer": {
    "type": "string",
    "max_length": 50
  }
}
```

### query

- `type`は`operation_sequence`だけを許可する
- `allowed_controls`は空でない文字列の配列とする
- `allowed_controls`は重複を許可しない
- `max_operations`は`1`以上とする
- `max_operations`は`count`の合計へ適用する

### answer

- `type`は`string`だけを許可する
- `max_length`は`1`以上とする
- 長さはUTF-8のbyte数ではなくUnicode scalar value数で数える
- 長さ制限は正規化前のユーザー入力へ適用する

使われない側のschemaも公開APIとの整合のため保持しますが、`submission_type`に対応する側を判定に使用します。

## 操作列

操作列の1要素は次の形式です。

```json
{
  "control": "down",
  "count": 2
}
```

- `control`は対象problemの`input_schema.query.allowed_controls`に含まれていなければならない
- `count`は`1`以上とする
- 全要素の`count`合計は`max_operations`以下とする
- 空の操作列は許可しない

### 正規化

隣接する同じ`control`を1要素へまとめ、`count`を加算します。

```json
[
  { "control": "down", "count": 1 },
  { "control": "down", "count": 2 },
  { "control": "right", "count": 1 }
]
```

は次へ正規化します。

```json
[
  { "control": "down", "count": 3 },
  { "control": "right", "count": 1 }
]
```

`count`が0以下、未知control、合計超過は正規化で修復せずvalidation errorとします。

## JudgeConfig

`JudgeConfig`はJSONの`type`で判別するtag付きenumです。Rustでは次の形に対応させます。

```rust
#[serde(tag = "type", rename_all = "snake_case")]
enum JudgeConfig {
    OperationSequence { /* fields */ },
    String { /* fields */ },
}
```

`submission_type`と`judge_config.type`は必ず一致させます。

### operation_sequence

```json
{
  "type": "operation_sequence",
  "correct_operations": [
    { "control": "down", "count": 1 },
    { "control": "right", "count": 2 }
  ],
  "candidates": [
    {
      "candidate_id": "pattern-a",
      "operations": [
        { "control": "down", "count": 1 },
        { "control": "right", "count": 2 }
      ]
    },
    {
      "candidate_id": "pattern-b",
      "operations": [
        { "control": "down", "count": 1 },
        { "control": "up", "count": 1 }
      ]
    }
  ]
}
```

規則は次のとおりです。

- `correct_operations`と全candidateの`operations`を同じ規則で正規化する
- `candidates`は1件以上とする
- `candidate_id`はproblem内で重複せず、前後空白を除いて空でない
- 正規化後に同じ操作列となるcandidateを重複として拒否する
- `correct_operations`は正規化後のcandidateのいずれか1件と完全一致しなければならない
- 送信操作列が`correct_operations`と完全一致した場合だけ正解とする
- candidate判定では、送信操作列がcandidate操作列の先頭と一致するcandidateを残す
- `remaining_pattern_count`は残ったcandidate数とする

先頭一致は正規化後の操作回数を1操作単位へ展開した列として比較します。例えば`down`を2回送った場合、
先頭が`down`1回だけのcandidateとは一致しません。

### string

```json
{
  "type": "string",
  "accepted_answers": ["かおもじくん", "顔文字くん"],
  "normalization": {
    "unicode": "nfkc",
    "trim_outer_whitespace": true,
    "collapse_internal_whitespace": false,
    "case_sensitive": false
  }
}
```

#### 正規化順序

1. 正規化前の入力が`input_schema.answer.max_length`以下か確認する
2. `unicode = nfkc`ならUnicode NFKCを適用する
3. `trim_outer_whitespace = true`なら前後のUnicode空白を除去する
4. `collapse_internal_whitespace = true`なら連続するUnicode空白を半角空白1個へ置換する
5. `case_sensitive = false`ならUnicode lowercaseへ変換する

判定時はユーザー入力と`accepted_answers`を同じ規則で正規化して完全一致を確認します。

validationでは次を確認します。

- `accepted_answers`は1件以上
- 各answerは正規化後も空でない
- 各answerは正規化前に`max_length`以下
- 正規化後に重複する別解を拒否する
- `unicode`は現在`nfkc`だけを許可する

正規化で長さ制限を回避できないよう、`max_length`は正規化前に適用します。

## Asset

assetの保存・配信・validationは別Issue／別PRで扱います。

このmock dataでは`assets` fieldを必須の空配列`[]`とします。非空のassetはvalidationで拒否します。

後続のasset対応では、問題データに画像binaryや環境別URLを含めず、object keyだけを保存します。
API返却時にobject keyから環境別URLを組み立てる方針です。object storageへのupload、MIME、size、
cache、404の規則はasset用PRで決定します。動画対応は今回の対象外です。

## Hint

hintは配列順を表示順として扱います。

```json
{
  "body_markdown": "最初の文字に注目してください"
}
```

- `body_markdown`は前後空白を除いて空でない
- hint本文は非公開dataとする
- 公開用problem modelには本文ではなく`hints.len()`から求めた`hint_count`だけを含める

## 問題セット全体のvalidation

全roomを読み込んでから、次を検証します。

- room IDが重複していない
- roomの`number`が重複していない
- problem IDが全roomを通して重複していない
- problemの`room_id`が親roomと一致する
- 同じroom内でproblemの`number`が重複していない
- 各roomに`small`が3問、`final`が1問ある
- 現在の必須4問はすべて`is_required = true`である
- 依存先が存在する
- 自分自身を依存先にしない
- 別roomのproblemを依存先にしない
- 依存graphに循環がない
- `small`を`number`の昇順に並べたとき、1問目は依存なし、2問目は1問目、3問目は2問目に依存する
- `final`は`depends_on_problem_id = null`で最初からavailableとなる

この構成では大なぞを先に正解しても、必須小なぞが残っている間はrunをclearにできません。実際のrun clear処理は
後続Issueで実装します。

## 公開dataと非公開data

公開用modelはvalidation済みmodelから次のfieldだけを明示的にcopyします。

- `id`：内部の`problem_id`
- `number`
- `type`：内部の`problem_type`
- `title`
- `body_markdown`
- `submission_type`
- `assets`
- `input_schema`
- `hint_count`：`hints.len()`

`status`はこのIssueでは作らず、後続の問題取得APIが`problem_progress`から付与します。

次のfieldは公開用modelへ含めません。

- `room_id`
- `judge_config`
- 正解操作列、正解文字列
- candidate一覧
- hint本文
- `depends_on_problem_id`
- `is_required`

公開modelは内部modelのserialize結果からfieldを削除して作らず、許可fieldを1つずつcopyして組み立てます。

## validation errorとlog

errorには次の安全な識別情報を含めます。

- I/O・JSON parse errorでは読込対象のfile path
- validation errorでは問題があるJSON field path
- errorの種類と、秘密を含まない説明

次の値はerror message、`Debug`出力、logへ含めません。

- `judge_config`全体
- `correct_operations`
- `accepted_answers`
- candidate内容
- hint本文

例えば文字列正解が空だった場合も、実際の正解値は表示せず、
`problems[1].judge_config.accepted_answers[0]: normalized answer must not be empty`のようにfield位置だけを示します。

## 完全な入力例

次は1部屋、小なぞ3問、大なぞ1問を含む完全な構造例です。実際の問題文とUUIDは正式データ作成時に確定します。

```json
{
  "room": {
    "room_id": "11111111-1111-4111-8111-111111111111",
    "number": 1,
    "name": "最初の部屋",
    "genre": "logic",
    "description": "動作確認用の問題セットです"
  },
  "problems": [
    {
      "problem_id": "22222222-2222-4222-8222-222222222221",
      "room_id": "11111111-1111-4111-8111-111111111111",
      "number": 1,
      "problem_type": "small",
      "title": "生年月日",
      "body_markdown": "問題文です",
      "submission_type": "operation_sequence",
      "assets": [],
      "input_schema": {
        "query": {
          "type": "operation_sequence",
          "allowed_controls": ["down", "right", "up", "left"],
          "max_operations": 100
        },
        "answer": {
          "type": "string",
          "max_length": 50
        }
      },
      "hints": [
        { "body_markdown": "最初の操作に注目してください" }
      ],
      "judge_config": {
        "type": "operation_sequence",
        "correct_operations": [
          { "control": "down", "count": 1 },
          { "control": "right", "count": 2 }
        ],
        "candidates": [
          {
            "candidate_id": "pattern-a",
            "operations": [
              { "control": "down", "count": 1 },
              { "control": "right", "count": 2 }
            ]
          },
          {
            "candidate_id": "pattern-b",
            "operations": [
              { "control": "down", "count": 1 },
              { "control": "up", "count": 1 }
            ]
          }
        ]
      },
      "depends_on_problem_id": null,
      "is_required": true
    },
    {
      "problem_id": "22222222-2222-4222-8222-222222222222",
      "room_id": "11111111-1111-4111-8111-111111111111",
      "number": 2,
      "problem_type": "small",
      "title": "合言葉",
      "body_markdown": "問題文です",
      "submission_type": "string",
      "assets": [],
      "input_schema": {
        "query": {
          "type": "operation_sequence",
          "allowed_controls": ["down", "right", "up", "left"],
          "max_operations": 100
        },
        "answer": {
          "type": "string",
          "max_length": 50
        }
      },
      "hints": [],
      "judge_config": {
        "type": "string",
        "accepted_answers": ["かおもじくん", "顔文字くん"],
        "normalization": {
          "unicode": "nfkc",
          "trim_outer_whitespace": true,
          "collapse_internal_whitespace": false,
          "case_sensitive": false
        }
      },
      "depends_on_problem_id": "22222222-2222-4222-8222-222222222221",
      "is_required": true
    },
    {
      "problem_id": "22222222-2222-4222-8222-222222222223",
      "room_id": "11111111-1111-4111-8111-111111111111",
      "number": 3,
      "problem_type": "small",
      "title": "最後の小なぞ",
      "body_markdown": "問題文です",
      "submission_type": "operation_sequence",
      "assets": [],
      "input_schema": {
        "query": {
          "type": "operation_sequence",
          "allowed_controls": ["down", "right", "up", "left"],
          "max_operations": 100
        },
        "answer": {
          "type": "string",
          "max_length": 50
        }
      },
      "hints": [],
      "judge_config": {
        "type": "operation_sequence",
        "correct_operations": [
          { "control": "left", "count": 1 },
          { "control": "up", "count": 1 }
        ],
        "candidates": [
          {
            "candidate_id": "pattern-c",
            "operations": [
              { "control": "left", "count": 1 },
              { "control": "up", "count": 1 }
            ]
          }
        ]
      },
      "depends_on_problem_id": "22222222-2222-4222-8222-222222222222",
      "is_required": true
    },
    {
      "problem_id": "22222222-2222-4222-8222-222222222224",
      "room_id": "11111111-1111-4111-8111-111111111111",
      "number": 4,
      "problem_type": "final",
      "title": "大なぞ",
      "body_markdown": "問題文です",
      "submission_type": "string",
      "assets": [],
      "input_schema": {
        "query": {
          "type": "operation_sequence",
          "allowed_controls": ["down", "right", "up", "left"],
          "max_operations": 100
        },
        "answer": {
          "type": "string",
          "max_length": 50
        }
      },
      "hints": [
        { "body_markdown": "3つの小なぞを振り返ってください" }
      ],
      "judge_config": {
        "type": "string",
        "accepted_answers": ["ワンマンソン"],
        "normalization": {
          "unicode": "nfkc",
          "trim_outer_whitespace": true,
          "collapse_internal_whitespace": false,
          "case_sensitive": false
        }
      },
      "depends_on_problem_id": null,
      "is_required": true
    }
  ]
}
```

## Rust modelとの境界

実装では次の3段階を分けます。

1. JSONのfieldをそのまま受けるvalidation前の入力型
2. UUID、enum、参照、正規化済み判定設定を持つvalidation済み内部型
3. 許可fieldだけを持つ公開用problem model

JSON parse errorや必須field不足を1段階目で検出し、複数problemにまたがる重複・参照・循環を2段階目への
変換時に検出します。DB row型やOpenAPI生成型を入稿JSONのdeserialize型として直接使用しません。

## 後続処理への変換

validation済みmodelを正本とし、利用側に応じて次のprojectionを作ります。

### MariaDB用

- `room.room_id`から`rooms.room_id`を作る
- roomの`number`、`name`、`genre`、`description`を同名列へ渡す
- `rooms.is_published`は`false`とする
- problemの各fieldを`problems`の同名列へ渡す
- `assets`はJSONの空配列`[]`として扱う
- `input_schema`、`hints`、`judge_config`は型付きmodelからJSONへserializeする
- `created_at`はDB既定値を使用する

validation済みmodelにはこの変換に必要な値をすべて保持しますが、このIssueではSQL文、seed、汎用repository
methodを作りません。

### frontend fixture用

frontendへは入稿JSON全体を渡さず、公開用problem modelだけをJSONへserializeします。動的な`status`が必要な
API fixtureでは、後続処理が`problem_progress`に相当する状態を付与します。正解、candidate、hint本文をfixtureへ
copyしません。
