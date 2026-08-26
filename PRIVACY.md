# Política de Privacidade — FLS Chat Overlay

Última atualização: 26 de agosto de 2026

## Resumo

Este aplicativo **não coleta, não armazena e não transmite nenhum dado pessoal.**
Não há cadastro, não há login, não há telemetria, não há analytics, não há
servidor nosso em lugar nenhum.

## Que dados o aplicativo trata

O aplicativo guarda, **apenas no seu computador**, as suas preferências:

- o nome do canal da Twitch que você escolheu exibir;
- opacidade, tamanho da fonte, posição e tamanho da janela do overlay;
- número de mensagens mantidas na tela e o tempo do aviso do modo mover.

Esses valores ficam num arquivo `settings.json` dentro da pasta de configuração
do aplicativo no seu perfil de usuário do Windows. O caminho exato é exibido no
rodapé da janela de configurações. Você pode abrir, editar ou apagar esse
arquivo quando quiser. Nada disso sai da sua máquina.

## Conexões de rede

O aplicativo faz **uma única** conexão de rede: ao servidor público de chat da
Twitch, em `wss://irc-ws.chat.twitch.tv:443`.

Essa conexão é feita de forma **anônima**, usando o mecanismo de leitura sem
autenticação da própria Twitch (login no formato `justinfan<número>`). O
aplicativo não envia credenciais, não solicita OAuth, não tem acesso à sua conta
da Twitch e não é capaz de enviar mensagens ao chat. A única informação
transmitida é o nome do canal que você configurou, no comando de entrada na
sala.

Essa restrição é aplicada tecnicamente, não apenas por escolha: a Content
Security Policy do aplicativo permite conexões somente para esse endereço. O
aplicativo é incapaz de contatar qualquer outro servidor.

As mensagens de chat recebidas são exibidas na tela e mantidas somente em
memória. Elas não são gravadas em disco nem enviadas a lugar nenhum.

O tratamento que a Twitch dá aos dados dessa conexão é regido pela política de
privacidade da própria Twitch.

## Dados de terceiros

O aplicativo não integra serviços de anúncios, analytics, crash reporting ou
qualquer SDK de terceiros que colete informação.

## Crianças

O aplicativo não coleta dados de ninguém, incluindo menores de idade.

## Alterações

Mudanças nesta política serão publicadas neste arquivo, no repositório público
do projeto, com a data de atualização acima.

## Contato

Dúvidas ou relatos: abra uma issue no repositório do projeto no GitHub.
