# 第4章 機械語

## fill.asmのCPU Emulatorによるデバッグについて

- ``Animate``を``No animation``にしないと描画が超遅い

## 機械語

- プロセッサとレジスタを用いてメモリを直接操作する

## プロセッサ

- CPU (Central Proccessing Unit), 中央演算装置
- 命令セットを実行する
    - レジスタやメモリからバイナリデータをフェッチ
    - バイナリデータの演算
    - 演算結果をレジスタやメモリに格納

## レジスタ

- ひとつだけの値を保持する
- メモリと比較して、プロセッサとの距離が極めて近い
    - 高速アクセスが可能

## メモリアクセス

### 直接アドレッシング

- 命令コード中の数値リテラルorシンボルをアドレスとして扱う

### イミディエイトアドレッシング

- 命令コード中の数値リテラルorシンボルを、メモリに格納するデータとして扱う

### 関節アドレッシング

- ポインタ

## 分岐命令

- 機械語は反復、条件分岐、サブルーチン呼び出しといった制御構文を実現するためのジャンプ命令を備える
