# AME! LIVE — versão desktop

App de salas, mensagens e chamadas para Windows.

## Download

[**Baixar AME! LIVE para Windows — versão v1 fix**](https://github.com/Luymera/AME-LIVE-rust/releases/download/v1-fix/AME-LIVE-app-v1-fix.zip)

Baixe o ZIP e extraia **todos** os arquivos para a mesma pasta. Abra `AME LIVE.exe`; mantenha `WebView2Loader.dll` e `cloudflared.exe` ao lado dele.

Requer o Microsoft Edge WebView2 Runtime. O app pode pedir acesso à câmera e ao microfone. Ao gerar um link de convite externo, o app usa o Cloudflare Tunnel incluído no pacote; o link fica público enquanto o app estiver aberto.

O código-fonte atualizado está em `codigo-fonte/`. Para recompilar, instale Rust e execute `cargo build --release` nessa pasta. O perfil e os dados locais ficam em `%LOCALAPPDATA%\\AME-LIVE`.

O pacote inclui o Cloudflare Tunnel, distribuído sob licença Apache-2.0; consulte `LEIA-ME.txt` para os créditos.
