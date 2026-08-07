# OpenAPI contract

`openapi-v1.yaml` は、クライアントとサーバーが共有する API 契約です。

このファイルは Go や Rust のサーバーコードを生成するための入力ではありません。サーバーのビルド時に `server/build.rs` が YAML、OpenAPI version、必須 path/method の `operationId` を検証し、サーバーバイナリから参照できるように埋め込みます。API を変更した場合は `mise run build` で契約の基本検証を行い、対応する server test も更新してください。

エンドポイント、`operationId`、リクエスト・レスポンスの Schema はクライアントとサーバー双方に影響する共有インターフェースとして扱ってください。
