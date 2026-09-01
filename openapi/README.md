# OpenAPI contract

`openapi-v1.yaml` は、フロントエンドとバックエンドが共有するOpenAPI 3.1.0のAPI契約です。認証・ゲーム・leaderboard APIを定義し、`GET /openapi.yaml`は契約を配信する既存のtooling endpointとして残しています。この契約から通信境界のRust crateとTypeScript型を生成しますが、DB処理やゲームロジックなどのAPI実装本体は生成しません。

## ファイル構成

```text
openapi/
├── openapi-v1.yaml
├── openapitools.json
├── templates/
│   └── rust-axum/
├── examples/
│   ├── auth/
│   ├── runs/
│   ├── leaderboard/
│   ├── progress/
│   ├── problems/
│   ├── queries/
│   └── answers/
├── scenarios/
│   └── p0-cases.yaml
└── README.md
```

request／response payloadの正本は`examples/`のJSONです。`openapi-v1.yaml`は各JSONをExample Objectの`externalValue`で参照し、payloadを重複定義しません。`p0-cases.yaml`にもpayloadは置かず、`operation_id`とOpenAPI上のexample keyだけを記録します。

既存の`GET /openapi.yaml`はmain YAMLだけを配信し、`examples/`をHTTP配信しません。`externalValue`はrepository上の`openapi-v1.yaml`を基準に解決してください。フロントのmockとAxum testはいずれもrepository内の同じJSONを直接読みます。

例に使う`11111111-...`の`room_id`と`22222222-...`の`problem_id`は契約確認用です。実際の最初の部屋・問題を示すIDではありません。

## コード生成

リポジトリルートから次を実行すると、RustとTypeScriptの両方を再生成して各formatterを適用します。

```sh
mise run openapi-generate
```

個別に生成する場合は`mise run openapi-generate-server`または`mise run openapi-generate-client`を使用します。生成後にコミット済み出力との差分が残るか確認する場合は、作業treeがcleanな状態で次を実行します。

```sh
mise run openapi-generate-check
```

生成versionはRust側がOpenAPI Generator CLI wrapper 2.40.1／Generator 7.24.0、TypeScript側がopenapi-typescript 7.13.0です。初回生成にはpackageとGenerator JARの取得、およびJava 11以上が必要です。通常のbuild・testでは再生成しないため不要です。

Rustの`rust-axum` Generator 7.24.0はOpenAPI 3.1の`type: "null"`を標準ではRust型へ変換できません。`openapitools.json`の小文字`null` type mappingと`templates/rust-axum/lib.mustache`により、必須かつnullだけを許す`NullValue`を生成します。`serde_json::Value`や`Option<T>`へ緩めると契約が変わるため使用しません。

生成先は次の2か所です。生成物へ直接変更を加えず、必要な修正はOpenAPI、生成設定、またはtemplateへ反映してください。

- `server/generated/openapi/`: Axum用のAPI trait、response enum、request／response model
- `client/src/generated/api.d.ts`: frontend用のpath、operation、schema型

## operationとexample

| operationId | request example | response status / example |
|---|---|---|
| `getMe` | なし | `200`: `neoshowcase_authenticated`, `neoshowcase_unauthenticated`, `demo_authenticated`, `demo_unauthenticated` |
| `getMeProgress` | なし | `200`: `summary`, `empty`; `401`: `unauthorized` |
| `loginGuest` | `guest_login`, `guest_login_empty`, `guest_login_too_long` | `200`: `guest_authenticated`; `422`: `display_name_required`, `display_name_too_long` |
| `logoutDemo` | bodyなし | `204`: bodyなし |
| `startOrResumeRun` | bodyなし | `200`: `new_run`, `resumed_run`; `401`: `unauthorized` |
| `getCurrentRun` | なし | `200`: `current_run`; `401`: `unauthorized`; `404`: `run_not_found` |
| `getRoomLeaderboard` | なし | `200`: `ranked`, `unauthenticated`, `empty` |
| `getProblem` | なし | `200`: `available_problem`; `401`: `unauthorized`; `409`: `problem_locked` |
| `submitQuery` | `serial_operations`, `invalid_source` | `200`: `incorrect_query`, `correct_query`; `401`: `unauthorized`; `409`: `problem_locked`, `problem_already_cleared`; `422`: `validation_error` |
| `submitAnswer` | `submitted_answer`, `too_long_for_example_problem` | `200`: `incorrect_answer`, `correct_answer_unlocks_problem`, `correct_answer_clears_run`; `401`: `unauthorized`; `409`: `problem_locked` |

`400`、部屋・問題の`404`、`submitAnswer`の`422`、`500`はstatusと共通Error schemaまで定義していますが、未確定の`error.code`を発明しないためresponse exampleは置いていません。`submitQuery`の意味的な入力不備は`422 VALIDATION_ERROR`として確定しており、`invalid_source` requestと`validation_error` responseで表現しています。例示問題の`max_length: 50`に対する51文字のrequestは、transport共通schemaでは文字列として妥当ですが、問題ごとの制限によって不正となる`submitAnswer`の例です。

## フロントエンドのmock

1. `operationId`、status、example keyを`scenarios/p0-cases.yaml`から選びます。
2. `openapi-v1.yaml`の該当Media Type Objectからexample keyを引き、`externalValue`のJSONを読みます。
3. JSONをそのままresponse bodyとして返します。別のTypeScript objectへpayloadを書き写しません。

たとえば`client/src/mocks/`からJSON moduleとして利用する場合は、概ね次の形です。

```ts
import currentRun from '../../../openapi/examples/runs/active-response.json'

return new Response(JSON.stringify(currentRun), {
  status: 200,
  headers: { 'content-type': 'application/json' },
})
```

`openapi/`は`client/`の外側にあるため、browser用mockを実装するIssueではViteがrepository rootを読めるようaliasまたは`server.fs.allow`を設定してください。fixtureを`client/`へコピーして二重管理してはいけません。mockの状態はscenarioの`preconditions`と`state_after`に従って切り替えます。

## Axumテスト

既存の`serde_json`を使い、request fixtureを`include_str!`で読み込んで送信し、statusとJSON bodyを比較できます。

```rust
const REQUEST: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../openapi/examples/queries/request-serial.json"
));
const EXPECTED: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../openapi/examples/queries/response-incorrect.json"
));

let request: serde_json::Value = serde_json::from_str(REQUEST).unwrap();
let expected: serde_json::Value = serde_json::from_str(EXPECTED).unwrap();
let response = post_json(&app, uri, request).await;

assert_eq!(response.status(), axum::http::StatusCode::OK);
assert_eq!(body_json::<serde_json::Value>(response).await, expected);
```

`204`はstatusに加えてbodyが空であることを確認します。demo loginでは`Set-Cookie`がHttpOnlyであること、logoutではsession Cookieを削除する`Set-Cookie`があることも確認します。Cookie名と未確定の属性はまだ固定しません。`401`、`404 RUN_NOT_FOUND`、`409 PROBLEM_LOCKED`は対応するerror fixtureと比較します。response exampleがない未確定errorは、statusとError schemaの形までを確認し、`error.code`確定後にA・B確認のうえfixtureを追加します。

## exampleのschema検証

`mise run build`は、`server/build.rs`を通じてYAML構文、OpenAPI version、必須path/method、`operationId`を検査します。JSON構文は標準libraryだけでも確認できます。

```sh
while IFS= read -r file; do
  python3 -m json.tool "$file" >/dev/null || exit 1
done < <(rg --files openapi/examples -g '*.json')
```

schema適合を確認する際は、OpenAPI 3.1とJSON Schema 2020-12に対応し、ローカルの`externalValue`を読み込めるvalidatorで、各Media Type Objectの`schema`に対して参照先JSONを検証します。negative scenarioのrequestも含め、JSONの構造自体は対応するrequest schemaへ適合させます。現在、repositoryには承認済みのOpenAPI validator packageがないため、新しいpackageを追加する場合は先にA・Bでtoolとversionを決めてください。

検証ではさらに、`p0-cases.yaml`の全`operation_id`が存在すること、statusがそのoperationに定義されること、指定したrequest／response example keyが該当Media Type Objectに存在することを確認します。

## scenarioと状態遷移

`scenarios/p0-cases.yaml`は次だけを保持します。

- `preconditions`: 認証mode、認証有無、active runの有無、問題状態、判定結果など、payload外の事前条件
- `operation_id`: OpenAPI operationへの参照
- `request_example`: requestBodyにあるexample key
- `response.status`と`response.example`: response statusとexample key
- `state_after`: operation後に変化する認証・run・問題状態

たとえば`demo_login_and_logout`は未ログインからguest login、`GET /api/me`、`204` logout、未ログイン状態への復帰を順に表します。`answer_correct_and_clear_run`は最後の必須問題への正解で`problem_status`と`run_status`がともに`cleared`になる遷移を表します。

## UUID・時刻などの動的値

fixtureのUUIDと時刻は再現可能な例示値です。実APIのテストでは次のいずれかを使います。

- clockやUUID generatorを固定できるtestでは、fixtureとbody全体を比較する
- 固定しないtestでは、`query_id`とuser IDはUUID v4としてparseできることを確認してからfixture値へ正規化し、残りのbodyを比較する
- `started_at`はRFC 3339としてparseし、新規開始と再開で同じ値が維持されることを確認する
- `elapsed_ms`はserver clockを固定して完全一致させるか、開始時刻との関係を許容幅付きで確認する
- `cleared_problem_ids`と`unlocked_problem_ids`はtest dataへ投入したUUIDとの一致を確認する

動的値だからという理由でfield自体を比較対象から外さず、形式と意味を個別に検証してください。

## 変更時の扱い

APIのpath、method、schema、status、example、scenarioのいずれかを変更すると、フロントのmockとバックのtestの両方に影響します。変更時は差分、関連fixture、scenario、利用側testを同時に更新し、フロントリーダーA・バックエンドリーダーB双方の確認を受けてください。

## 未確定事項

- 開始導線へ設定する最初の`room_id`と`problem_id`
- 部屋・問題不存在、JSON／UUID不正、入力内容不備、server内部エラーの具体的な`error.code`
- demo表示名の大文字小文字をそのまま保持するか、正規化するか
- queryの空配列、`count`範囲、未知control、`source`の厳密な許容範囲
- demo未認証でlogout APIを呼んだ場合の具体的なstatus

これらはOpenAPIへ推測で追加せず、確定後に更新します。
