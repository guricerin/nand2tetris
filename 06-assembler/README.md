# 第6章 アセンブラ

- アセンブリコードを機械語に変換するプログラム

## 仕様

- アセンブリを構文解析し、基本となる領域に分割
- 各領域において、対応する機械語のビットを生成
- シンボルによる参照を数字に依るメモリアドレスに置換
    - シンボルテーブルを使う
- 領域ごとのバイナリコードを組み合わせ、完全な機械語命令を作成