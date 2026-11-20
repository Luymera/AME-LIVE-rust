# AME LIVE! OVERDOSE

Versão desktop do aplicativo de salas, mensagens e chamadas.

## Baixar e abrir

Baixe [AME-LIVE-app.zip](AME-LIVE-app.zip), extraia todos os arquivos para a mesma pasta e abra `AME LIVE.exe`. Mantenha `WebView2Loader.dll` ao lado do executável. O app usa o Microsoft Edge WebView2 Runtime.

O perfil e os dados ficam no computador em `%LOCALAPPDATA%\\AME-LIVE`. Para recompilar, instale Rust e rode `cargo build --release` dentro de `codigo-fonte`.

Para conectar a um servidor online, crie `servidor.txt` ao lado do executável e coloque a URL do servidor em uma linha.
