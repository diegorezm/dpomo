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
- 🎨 **Tema**: 8 prontos (Rosé Pine, Catppuccin, Tokyo Night, Nord, Gruvbox, Mono, Latte, Paper) ou **o seu**, escolhendo cada cor
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
