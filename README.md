# 🍅 dpomo

Um timer Pomodoro **bonito**, **leve** e **do seu jeito**. Feito em Rust com [egui](https://github.com/emilk/egui).

![dpomo](assets/screenshot.png)

## O que é Pomodoro?

Você trabalha focado por **25 minutos**, descansa **5**. A cada **4 sessões**, faz uma pausa longa de **15**. Só isso. (Todos esses números você pode mudar.)

## Como usar

1. Abra o app.
2. Aperte o botão grande (ou **Espaço**) para começar.
3. Quando o tempo acabar, o app avisa piscando na barra de tarefas.

### Atalhos

| Tecla    | O que faz         |
| -------- | ----------------- |
| `Espaço` | Iniciar / pausar  |
| `R`      | Reiniciar a fase  |
| `S`      | Pular para a próxima fase |
| `,`      | Abrir ajustes     |
| `Esc`    | Voltar dos ajustes |

## Personalizar

Clique no ícone de ajustes (canto superior direito). Você pode mudar:

- ⏱️ **Tempos** de foco e pausas, e quantas sessões até a pausa longa
- 🎨 **Tema**: 8 prontos (Rosé Pine, Catppuccin, Tokyo Night, Nord, Gruvbox, Mono, Latte, Paper) ou **o seu** (veja abaixo)
- 💍 **Anel**: espessura e brilho
- 🔍 **Tamanho** da interface
- 📌 **Sempre no topo**, iniciar fases automaticamente e mais

Tudo é salvo num arquivo de texto simples que você também pode editar na mão:

| Sistema | Onde fica |
| ------- | --------- |
| Windows | `%APPDATA%\dpomo\config.toml` |
| Linux   | `~/.config/dpomo/config.toml` |
| macOS   | `~/Library/Application Support/dpomo/config.toml` |

Exemplo:

```toml
[timer]
focus_minutes = 50
short_break_minutes = 10

[appearance]
theme = "Nord"
```

Só escreva o que quiser mudar, o resto usa o padrão.

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

## Instalar

Você precisa do [Rust](https://rustup.rs) instalado.

```sh
git clone https://github.com/diegorezm/dpomo
cd dpomo
cargo run --release
```

Ou instale direto:

```sh
cargo install --git https://github.com/diegorezm/dpomo
```

> **Linux:** pode ser preciso instalar `libxkbcommon-dev`, `libgl1-mesa-dev` e `libwayland-dev` antes.

## Licença

[MIT](LICENSE). Faça o que quiser. 🙂
