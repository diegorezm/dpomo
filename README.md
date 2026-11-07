# 🍅 dpomo

Um timer Pomodoro **bonito**, **leve** e **do seu jeito**, com um axolote que trabalha e descansa junto com você. 🦎💗
Feito em Rust com [egui](https://github.com/emilk/egui).

![dpomo](assets/screenshot.png)

## O que é Pomodoro?

Você trabalha focado por **25 minutos**, descansa **5**. A cada **4 sessões**, faz uma pausa longa de **15**. Só isso. (Todos esses números você pode mudar.)

## Como usar

1. Abra o app.
2. Aperte o botão grande (ou **Espaço**) para começar.
3. Quando o tempo acabar, você recebe uma **notificação**. 🔔

### Fica rodando em segundo plano

O dpomo mora na **bandeja do sistema** (os ícones pequenos perto do relógio):

- Fechar a janela **não fecha o app**: ele continua contando lá na bandeja.
- O ícone mostra o **anel de progresso** ao vivo; passe o mouse para ver o tempo.
- **Clique** no ícone para mostrar/esconder a janela; **botão direito** abre o menu
  (iniciar/pausar, pular, reiniciar, **sair**).
- Abriu o dpomo de novo sem querer? Tudo bem, ele só mostra a janela que já estava aberta.
- Quer que ele abra sozinho quando ligar o computador? Ative **Iniciar com o sistema** nos ajustes.

### Atalhos

| Tecla    | O que faz         |
| -------- | ----------------- |
| `Espaço` | Iniciar / pausar  |
| `R`      | Reiniciar a fase  |
| `S`      | Pular para a próxima fase |
| `1` `2` `3` | Ir para foco / pausa curta / pausa longa |
| `,`      | Abrir ajustes     |
| `Esc`    | Voltar dos ajustes |

Dá para trocar qualquer atalho no `config.toml` (seção `[keys]`, veja [Configuração](#configuração)).

## Personalizar

Clique no ícone de ajustes (canto superior direito). Você pode mudar:

- 🦎 **Estilo**: *Aconchegante* (com o axolote) ou *Clássico* (um anel de progresso minimalista)
- ⏱️ **Tempos** de foco e pausas, e quantas sessões até a pausa longa
- 🎨 **Tema**: 8 prontos (Rosé Pine, Catppuccin, Tokyo Night, Nord, Gruvbox, Mono, Latte, Paper) ou **o seu** (veja abaixo)
- 💍 **Anel** (estilo clássico): espessura e brilho
- 🔍 **Tamanho** da interface
- 📌 **Sempre no topo**, iniciar fases automaticamente e mais
- 🌙 **Segundo plano**: bandeja, notificações, iniciar com o sistema

## Configuração

**Toda** a configuração fica num arquivo de texto, o `config.toml`. Os ajustes do app só editam esse
arquivo por você. Ele é criado na primeira vez que o dpomo abre:

| Sistema | Onde fica |
| ------- | --------- |
| Windows | `%APPDATA%\dpomo\config.toml` |
| Linux   | `~/.config/dpomo/config.toml` |
| macOS   | `~/Library/Application Support/dpomo/config.toml` |

(Nos ajustes há os botões **Abrir config.toml** e **Abrir pasta**.)

- **Mudou e salvou? O app aplica na hora**, não precisa reiniciar.
- Pode apagar o que não quiser: o que faltar usa o padrão.
- Errou algo? O app avisa qual linha está errada e **não mexe no seu arquivo** até você corrigir.

Tudo o que dá para configurar (com os valores padrão):

```toml
[timer]
focus_minutes = 25
short_break_minutes = 5
long_break_minutes = 15
sessions_before_long_break = 4
auto_start_breaks = false        # começar a pausa sozinho
auto_start_focus = false         # começar o foco sozinho

[appearance]
layout = "cozy"                  # "cozy" (axolote) ou "classic" (anel)
character = "axolotl"            # pasta do personagem
theme = "Rosé Pine"              # nome de um tema, ou "custom" (usa [custom_theme])
ring_thickness = 10.0            # só no "classic"
ui_scale = 1.0                   # tamanho da interface
smooth_animation = true          # false = gasta menos CPU
glow = true                      # brilho no anel ("classic")
show_session_dots = true

[behavior]
always_on_top = false
notify_on_finish = true          # piscar na barra de tarefas
focus_on_finish = false          # trazer a janela pra frente
time_in_title = true             # tempo no título da janela
tray = true                      # ícone na bandeja
close_to_tray = true             # fechar = esconder na bandeja
system_notification = true
start_with_system = false
start_hidden = false             # abrir já escondido na bandeja

[window]
width = 360.0
height = 500.0
remember_size = true             # guarda o último tamanho ao fechar

[keys]                           # nomes de tecla: "Space", "R", ",", "1", "F5", "Escape", "ArrowLeft"…
toggle = "Space"
reset = "R"
skip = "S"
settings = ","
focus = "1"
short_break = "2"
long_break = "3"
back = "Escape"

[custom_theme]                   # usado quando theme = "custom"
background = "#191724"
surface = "#26233a"
text = "#e0def4"
muted = "#908caa"
focus = "#eb6f92"
short_break = "#9ccfd8"
long_break = "#c4a7e7"
```

### Criar seu próprio tema 🎨

Temas são arquivos `.toml`. Coloque o seu na pasta `themes`, ao lado do `config.toml`
(ex.: `%APPDATA%\dpomo\themes\dracula.toml`):

```toml
name = "Dracula"
background = "#282a36"   # fundo
surface = "#44475a"      # cartões, botões e trilho do anel
text = "#f8f8f2"         # texto
muted = "#a4a8c4"        # texto secundário
focus = "#ff79c6"        # cor do foco
short_break = "#50fa7b"  # cor da pausa curta
long_break = "#bd93f9"   # cor da pausa longa
```

Depois, nos ajustes, clique em **Recarregar temas**. Pronto!

Prefere escolher as cores clicando? Selecione **Personalizado**, ajuste as cores e clique em
**Salvar como arquivo**. Quer compartilhar? É só mandar o arquivo para alguém. 😉

### Criar seu próprio personagem 🦎

Vêm três personagens: o **axolote** 🦎, o **tomate** 🍅 e a **capivara** 🦫 (escolha em *Ajustes → Aparência*).
Cada um tem 4 poses: acena quando está parado, trabalha durante o foco, relaxa na pausa curta e dorme na
pausa longa. Quer outro bichinho? Crie uma pasta em `characters`, ao lado do `config.toml`
(ex.: `%APPDATA%\dpomo\characters\gato\`), com estes arquivos SVG:

| Arquivo | Quando aparece |
| ------- | -------------- |
| `idle.svg` | parado (**obrigatório**) |
| `focus.svg` | focando |
| `short_break.svg` | pausa curta |
| `long_break.svg` | pausa longa |

Faltou algum? Ele usa o `idle.svg` no lugar. Para dar um nome, crie `character.toml` com `name = "Gato"`.

Dica: no SVG, a cor `#e05a4f` vira a cor da fase (vermelho no foco, verde na pausa…) e `#7a6464`
vira a cor de texto secundário do tema. Assim seu personagem combina com qualquer tema.

## Instalar

Baixe o arquivo do seu sistema e abra:

| Sistema | Download | Como instalar |
| ------- | -------- | ------------- |
| **Windows** | [dpomo-setup.exe](https://github.com/diegorezm/dpomo/releases/latest/download/dpomo-setup.exe) | Abra e clique em *Avançar* até o fim. Não pede senha de administrador. |
| **Linux** (Ubuntu/Debian) | [dpomo-linux-amd64.deb](https://github.com/diegorezm/dpomo/releases/latest/download/dpomo-linux-amd64.deb) | Dois cliques no arquivo, ou `sudo apt install ./dpomo-linux-amd64.deb` |
| **macOS** | [dpomo-macos.dmg](https://github.com/diegorezm/dpomo/releases/latest/download/dpomo-macos.dmg) | Arraste o dpomo para *Aplicativos*. |

Outras opções (versão portátil para Windows, `.tar.gz` para outros Linux) estão na
[página de releases](https://github.com/diegorezm/dpomo/releases/latest).

> **Windows:** se aparecer "O Windows protegeu o computador", clique em *Mais informações* → *Executar assim mesmo*.
> Isso acontece porque o instalador não é assinado digitalmente (assinatura custa caro 😅).
>
> **macOS:** na primeira vez, clique com o **botão direito** no dpomo → *Abrir* → *Abrir*.
> O app não é notarizado pela Apple, então o macOS pede essa confirmação uma vez.
>
> **Linux:** para ver o ícone na bandeja no GNOME, instale a extensão *AppIndicator* (KDE já funciona direto).

Para desinstalar no Windows: *Configurações → Aplicativos → dpomo → Desinstalar*.

### Compilar você mesmo

Precisa do [Rust](https://rustup.rs):

```sh
cargo install --git https://github.com/diegorezm/dpomo
```

> **Linux:** instale antes `libxkbcommon-dev`, `libgl1-mesa-dev` e `libwayland-dev`.

## Licença

[MIT](LICENSE). Faça o que quiser. 🙂
