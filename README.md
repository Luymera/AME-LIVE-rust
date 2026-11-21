# AME! LIVE — salas, chamadas e transmissões

O AME! LIVE é um aplicativo desktop para Windows criado para reunir amigos em um espaço de comunicação com visual rosa, azul e cyber-kawaii. Crie salas, organize conversas em canais, troque mensagens e participe de chamadas enquanto acompanha transmissões de tela ou câmera.

## O que você pode fazer

- Criar uma sala e convidar amigos por link ou código, ou entrar em uma sala existente.
- Conversar por texto e ver quem está na sala, online ou disponível.
- Participar de chamadas e ativar o microfone e a câmera quando quiser.
- Compartilhar a tela, a câmera e o áudio do PC durante transmissões.
- Personalizar nome, foto, status e disponibilidade do perfil.
- Ajustar dispositivos e volumes nas configurações do app.

## Download

[**Baixar AME! LIVE para Windows — versão v1 fix**](https://github.com/Luymera/AME-LIVE-rust/releases/download/v1-fix/AME-LIVE-app-v1-fix.zip)

Baixe o ZIP e extraia **todos** os arquivos para a mesma pasta. Abra `AME LIVE.exe`; mantenha `WebView2Loader.dll` e `cloudflared.exe` ao lado dele.

É necessário o Microsoft Edge WebView2 Runtime, que já vem no Windows 11 e na maioria das instalações do Windows 10. [Instale o WebView2](https://developer.microsoft.com/microsoft-edge/webview2/) se o app não abrir. Na primeira execução, permita o acesso à câmera e ao microfone quando o Windows solicitar.

## Salas e convites

O servidor para convites vem integrado ao app, então não é preciso instalar ou configurar um servidor separado. Para chamar alguém, abra **Convidar** e selecione **Gerar link para amigos**. A pessoa pode abrir o link no navegador sem instalar o app ou inserir o código na opção **Entrar**.

O link fica ativo enquanto o app estiver aberto e muda a cada geração. Qualquer pessoa com o link pode tentar acessar a sala durante esse período; compartilhe-o apenas com quem deseja convidar. A transmissão de vídeo é enviada diretamente do seu computador para cada participante. O app funciona melhor com grupos pequenos, em torno de 4 ou 5 pessoas. Alguns antivírus e redes podem bloquear o cloudflared.

## Chamadas e transmissão

Ao entrar em uma sala, você participa da chamada. Seu microfone e sua câmera ficam desligados até você ativá-los. Use os controles de volume para ajustar seu microfone, as vozes dos amigos e o som da transmissão. Na lista de usuários, selecione o nome ou a foto de alguém para ver a transmissão dessa pessoa; selecione novamente para voltar à escolha automática.

Para compartilhar o áudio do PC pelo navegador, permita a captura de áudio do sistema. No Chrome ou Edge, escolha **Tela inteira** e marque **Compartilhar áudio do sistema**. O WebView2 do aplicativo pode não capturar o áudio do PC; para transmitir com som, use o navegador ou a versão Electron. Se o navegador bloquear o som de alguém que entrou pelo link, clique em qualquer lugar da página para liberá-lo.

## Código-fonte e dados

O código-fonte está na pasta [`codigo-fonte/`](codigo-fonte/). Para recompilar, instale Rust e execute `cargo build --release` nessa pasta.

O perfil e os dados locais ficam em `%LOCALAPPDATA%\\AME-LIVE`.

O pacote inclui o [cloudflared](https://github.com/cloudflare/cloudflared), da Cloudflare, distribuído sob licença Apache-2.0.
