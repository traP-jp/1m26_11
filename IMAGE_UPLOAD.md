# 作問用画像アップロード

この機能は、一般プレイヤーではなく、開発環境で問題を作成する作問者・運営者向けの機能です。

指定した問題へ画像をアップロードすると、画像をS3互換ストレージへ保存し、対象problemの`assets`へ自動的に追加します。

```http
POST /api/rooms/{room_id}/problems/{problem_id}/assets
```

このAPIは次の両方を満たす場合だけ登録されます。

- `AUTH_MODE=demo`
- `IMAGE_UPLOAD_ENABLED=true`

条件を満たさない場合は、endpoint自体が登録されず`404 NOT_FOUND`になります。

現在はdev専用APIであり、Cookieや`x-forwarded-user`による利用者認証は要求しません。代わりに、serverをlocalhostだけで待ち受けさせ、外部ネットワークへ公開しないでください。

## 対応する画像

アップロードできる画像は次の3形式です。

- PNG
- JPEG
- WebP

ファイル名やmultipartで申告された`Content-Type`ではなく、実際のファイル内容から形式を判定します。

次の制限があります。

| 項目 | 制限 |
| --- | --- |
| ファイル数 | 1リクエストにつき1件 |
| ファイルサイズ | 5,242,880 bytes以下 |
| 画像の幅 | 4,096 pixels以下 |
| 画像の高さ | 4,096 pixels以下 |
| 総画素数 | 16,777,216 pixels以下 |
| alt | trim後に1文字以上、200 Unicode文字以下 |
| SVG | 非対応 |

clientから渡されたファイル名はobject keyへ使用しません。object keyはserverが次の形式で生成します。

```text
v1/problems/{room_id}/{problem_id}/{asset_id}.{extension}
```

## 必要な環境変数

画像アップロードを有効にするには、次の環境変数が必要です。

| 変数 | 内容 |
| --- | --- |
| `APP_ADDR` | serverのlisten address。直接起動では`127.0.0.1:8080`を使用する |
| `AUTH_MODE` | `demo`にする |
| `DEMO_COOKIE_SECURE` | localhostのHTTP開発では`false`にする |
| `IMAGE_UPLOAD_ENABLED` | `true`にする |
| `DATABASE_URL` | serverから接続できるMariaDBのURL |
| `S3_ENDPOINT` | S3互換APIのendpoint |
| `S3_BUCKET` | upload先bucket |
| `AWS_ACCESS_KEY_ID` | storage access key |
| `AWS_SECRET_ACCESS_KEY` | storage secret key |
| `AWS_REGION` | S3署名に使用するregion |
| `S3_FORCE_PATH_STYLE` | providerに応じて`true`または`false` |
| `ASSET_PUBLIC_BASE_URL` | 内部object keyからAssetの`url`を生成するためのbase URL |

`ASSET_PUBLIC_BASE_URL`はresponseの`url`を生成するために使用します。この設定だけでstorage objectが匿名公開されるわけではありません。

storageへのuploadはserverがcredentialを使用して行います。credentialを持たないclientがstorageへ直接uploadすることはありません。生成された`url`から画像を取得する方法と、その取得時の認証は本機能の対象外です。

`S3_FORCE_PATH_STYLE`、`IMAGE_UPLOAD_ENABLED`などのboolean値は、小文字の`true`または`false`だけを使用してください。

## ローカル設定ファイル

実際のcredentialは、Git管理外の次のファイルへ保存します。

```text
server/.env.storage.local
```

`server/.gitignore`によってこのファイルはGit管理から除外されます。

内容は次の形式です。`replace-with-...`の部分を担当者から安全な経路で受け取った実値へ置き換えてください。

```sh
APP_ADDR='127.0.0.1:8080'
AUTH_MODE='demo'
DEMO_COOKIE_SECURE='false'
IMAGE_UPLOAD_ENABLED='true'

DATABASE_URL='mysql://replace-with-user:replace-with-password@127.0.0.1:3306/replace-with-database'

S3_ENDPOINT='https://replace-with-storage-endpoint'
S3_BUCKET='replace-with-bucket'
AWS_ACCESS_KEY_ID='replace-with-access-key'
AWS_SECRET_ACCESS_KEY='replace-with-secret-key'
AWS_REGION='replace-with-region'
S3_FORCE_PATH_STYLE='false'
ASSET_PUBLIC_BASE_URL='https://replace-with-public-base-url'
```

次の場所へcredentialを入れてはいけません。

- Git管理対象ファイル
- `README.md`やこの文書
- `VITE_*`環境変数
- frontendのsource
- OpenAPI fixture
- HTTP request
- Issue、PR、チャット、ログ

## serverの起動

現在の`compose.yaml`では画像アップロードが明示的に無効化されています。画像アップロードを試す場合は、環境変数を読み込んでserverを直接起動します。

```sh
set -a
. server/.env.storage.local
set +a

mise exec -- cargo run --locked --manifest-path server/Cargo.toml --bin server
```

serverが次のaddressで起動したことを確認します。

```text
127.0.0.1:8080
```

serverは起動時にmigrationを適用するため、`DATABASE_URL`は必ず開発専用または破棄可能なDBへ向けてください。

## 画像をアップロードする

pathには、対象roomと、そのroomに属するproblemのUUIDを指定します。

multipart fieldは次の2つです。

| field | 内容 |
| --- | --- |
| `file` | 画像ファイル1件 |
| `alt` | 画像内容を説明する代替テキスト |

さらに、HTTP headerへUUID v4形式の`Idempotency-Key`を1個指定します。

```sh
curl \
  --request POST \
  'http://127.0.0.1:8080/api/rooms/11111111-1111-4111-8111-111111111111/problems/22222222-2222-4222-8222-222222222221/assets' \
  --header 'Idempotency-Key: 44444444-4444-4444-8444-444444444444' \
  --form 'file=@./birthday.png' \
  --form 'alt=ろうそくが立った誕生日ケーキ'
```

`curl`がmultipartのboundaryを自動生成するため、`Content-Type` headerを手動で指定しないでください。

例に記載したroom ID、problem ID、Idempotency-Keyは説明用です。実際のuploadでは対象のIDと、新しく生成したUUID v4を使用してください。

## 成功response

成功時は`201 Created`と、追加されたAssetが返ります。

```json
{
  "type": "image",
  "url": "https://assets.example.invalid/v1/problems/11111111-1111-4111-8111-111111111111/22222222-2222-4222-8222-222222222221/33333333-3333-4333-8333-333333333333.png",
  "alt": "ろうそくが立った誕生日ケーキ"
}
```

responseには次の情報を独立したfieldとして含めません。

- 内部`object_key`
- bucket名
- credential
- storage provider固有のraw response
- 署名情報

`url`には、設定したbase URLに応じてstorage providerのhost、bucket名、および`object_key`に由来するpathが含まれる場合があります。これらを`url`の一部として返すことは許容します。

upload成功時には、対象problemの内部`assets`へ次の情報が自動的に追加されます。

```json
{
  "type": "image",
  "object_key": "v1/problems/11111111-1111-4111-8111-111111111111/22222222-2222-4222-8222-222222222221/33333333-3333-4333-8333-333333333333.png",
  "alt": "ろうそくが立った誕生日ケーキ"
}
```

内部`object_key`は`AssetUrlResolver`によってAssetの`url`へ変換されます。この`url`から画像を取得する方法と、取得時に必要な認証は別途定めます。

## アップロード結果の確認

upload APIが`201 Created`を返し、responseに`type`、`url`、`alt`が含まれることを確認します。

同じ画像、同じtrim後のalt、同じ`Idempotency-Key`で再送し、最初と同じAssetが`201 Created`で返ることも確認します。これにより、storageへの重複uploadと`problems.assets`への重複追加を防止できていることを確認できます。

storageへのuploadにはserver-side credentialを使用します。credentialをHTTP requestやfrontendへ含めないでください。

生成された`url`から画像を取得する方法は本機能の対象外です。storageが認証を要求する場合、未認証のGET requestが`403`になることがありますが、これはuploadの失敗を意味しません。確認のためにbucketを匿名公開しないでください。

## Idempotency-Keyと再送

`Idempotency-Key`はUUID v4を使用します。

同じmethod、path、Idempotency-Key、画像内容、trim後のaltで再送した場合は、最初に作成したAssetを`201`で返します。storageへの再uploadや`problems.assets`への重複追加は行いません。

同じIdempotency-Keyを使って画像内容またはaltを変更すると、次を返します。

```text
409 IDEMPOTENCY_KEY_REUSED
```

同じrequestがまだ処理中の場合は、次を返します。

```text
409 IDEMPOTENCY_REQUEST_IN_PROGRESS
```

処理中claimの有効期限は24時間です。

storageへのuploadに失敗した場合はclaimを解放するため、同じrequestを再送できます。

storageへのupload成功後にDB更新が失敗した場合は、upload済みobjectを追跡できるようclaimを解放しません。この場合は、同じIdempotency-Keyを無条件に変更せず、server logとstorageの状態を確認してください。

## Error response

errorはすべて次の形式です。

```json
{
  "error": {
    "code": "INVALID_ALT",
    "message": "image alt text is invalid",
    "details": {}
  }
}
```

主なstatusとcodeは次のとおりです。

| Status | Code | 原因 |
| --- | --- | --- |
| `400` | `INVALID_PATH_PARAMETER` | room IDまたはproblem IDがUUIDではない |
| `400` | `INVALID_MULTIPART` | file／alt不足、重複field、未知field、multipart形式不正 |
| `400` | `IDEMPOTENCY_KEY_REQUIRED` | `Idempotency-Key`がない |
| `400` | `INVALID_IDEMPOTENCY_KEY` | keyがUUID v4でない、または複数指定された |
| `404` | `NOT_FOUND` | upload APIが無効でrouteが登録されていない |
| `404` | `ROOM_OR_PROBLEM_NOT_FOUND` | room／problemが存在しない、または組合せが不正 |
| `409` | `PUBLISHED_ROOM_IMMUTABLE` | 公開済みroomへ追加しようとした |
| `409` | `IDEMPOTENCY_KEY_REUSED` | 同じkeyを異なる画像またはaltで再利用した |
| `409` | `IDEMPOTENCY_REQUEST_IN_PROGRESS` | 同じrequestが処理中 |
| `413` | `IMAGE_TOO_LARGE` | fileが5,242,880 bytesを超えている |
| `415` | `UNSUPPORTED_IMAGE_TYPE` | PNG、JPEG、WebP以外 |
| `422` | `EMPTY_FILE` | fileが空 |
| `422` | `INVALID_IMAGE` | 画像データが壊れている |
| `422` | `IMAGE_DIMENSIONS_EXCEEDED` | 幅、高さ、総画素数の上限超過 |
| `422` | `INVALID_ALT` | trim後のaltが空、200文字超過、またはalt読込み上限超過 |
| `500` | `INTERNAL_SERVER_ERROR` | DB更新やserver内部処理の失敗 |
| `502` | `STORAGE_PROVIDER_ERROR` | storage providerが4xxを返した |
| `503` | `STORAGE_UNAVAILABLE` | storage接続失敗、10秒timeout、またはproviderの5xx |

このdev専用endpointはapplication-levelの利用者認証を行わないため、通常は`401`または`403`を返しません。`AUTH_MODE=neoshowcase`やupload無効時は、routeを登録せず`404`を返します。

## Troubleshooting

### `404 NOT_FOUND`

次を確認してください。

- `AUTH_MODE=demo`
- `IMAGE_UPLOAD_ENABLED=true`
- 必須storage環境変数がすべて設定されている
- 設定を変更した後にserverを再起動した
- request先が画像upload endpointと一致している

### `404 ROOM_OR_PROBLEM_NOT_FOUND`

次を確認してください。

- room IDが正しい
- problem IDが正しい
- problemが指定したroomに属している

### `409 PUBLISHED_ROOM_IMMUTABLE`

公開済みroomには画像を追加できません。対象roomが未公開であることを確認してください。

### `413 IMAGE_TOO_LARGE`

画像を5,242,880 bytes以下へ縮小してください。画像dimensionにも別の上限があります。

### `415 UNSUPPORTED_IMAGE_TYPE`

拡張子ではなく実際のファイル内容を確認してください。PNG、JPEG、WebPだけを使用できます。

### `422 INVALID_ALT`

altを空白以外の1文字以上、200 Unicode文字以下にしてください。

### `502 STORAGE_PROVIDER_ERROR`

次を確認してください。

- bucket名
- access keyの権限
- providerが要求するregion
- `S3_FORCE_PATH_STYLE`
- provider側のrequest拒否理由

providerのraw responseはAPI利用者へ返しません。

### `503 STORAGE_UNAVAILABLE`

次を確認してください。

- `S3_ENDPOINT`
- network接続
- providerの稼働状態
- providerの5xx
- 10秒以内に応答しているか

### `500 INTERNAL_SERVER_ERROR`

server logとDB接続を確認してください。credential、画像binary、providerのraw responseをログへ追加してはいけません。
