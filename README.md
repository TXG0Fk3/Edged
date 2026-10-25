# Edged 📐

> Réplica fiel, moderna e de alta performance do **PowerToys Screen Ruler** nativa para **Wayland** (Hyprland, GNOME, Sway, River, KDE, etc.).

[![Rust](https://img.shields.io/badge/language-Rust-orange.svg)](https://www.rust-lang.org/)
[![Wayland](https://img.shields.io/badge/display-Wayland-blue.svg)](https://wayland.freedesktop.org/)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-green.svg)](LICENSE)

O **Edged** é uma régua de tela pixel-accurate projetada especialmente para o ecossistema Wayland moderno. Ele permite medir distâncias, caixas delimitadoras, espaçamentos, margens e padding entre elementos da interface com detecção de bordas em tempo real.

---

## ✨ Características

- ⚡ **Modelo de Ativação One-Shot**: O programa **não** roda em segundo plano consumindo memória. Inicia instantaneamente via comando de terminal e fecha assim que você copia a medida ou pressiona <kbd>Esc</kbd>.
- 🎯 **Detecção de Bordas em Tempo Real**: Caminha pelos pixels a partir do cursor nas 4 direções cardeais com base na diferença de cor euclidiana.
- 🔄 **Ajuste Dinâmico de Tolerância**: Gire a roda de scroll do mouse para aumentar ou diminuir a sensibilidade da detecção de bordas em tempo real, com feedback visual instantâneo (HUD).
- 📦 **Modo Bounds**: Clique e arraste para desenhar retângulos delimitadores exibindo `Largura × Altura`.
- 📌 **Fixação com Shift**: Mantenha <kbd>Shift</kbd> pressionado no modo Bounds para manter múltiplos retângulos fixados na tela.
- 📏 **Pés Delimitadores ("Draw Feet")**: Traços perpendiculares nas pontas das linhas para visualização precisa dos limites medidos.
- 📋 **Clique para Copiar**: Clique com o botão esquerdo para copiar a medida diretamente para a área de transferência (`wl-clipboard` / GTK) em formatos configuráveis (`compact`, `verbose` ou `css`).
- 🖥️ **Wayland Nativo e Multi-Monitor**: Suporte a `wlr-layer-shell` (em compositores wlroots como Hyprland e Sway) e fallback automático para janela de overlay fullscreen no GNOME Mutter. Suporta monitores múltiplos e fractional scaling.
- 📸 **Estratégia Híbrida de Captura de Tela**: Integração direta com `grim` (screencopy instantâneo) e `xdg-desktop-portal` (PipeWire / Screenshot portal sem janelas de diálogo).
- ⚙️ **Configuração Flexível**: Arquivo `~/.config/edged/config.toml` auto-gerado com valores padrão bem documentados.

---

## 🚀 Modos de Uso (CLI)

O Edged segue o padrão das ferramentas modernas do ecossistema Wayland (como `slurp`, `grim`, `swappy`):

```bash
edged                 # Inicia no modo padrão configurado (default: Spacing)
edged --bounds        # Modo Bounds: clique e arraste para medir áreas retangulares
edged --spacing       # Modo Spacing: mira em cruz medindo espaçamentos horizontais e verticais
edged --horizontal    # Medição contínua apenas horizontal
edged --vertical      # Medição contínua apenas vertical
```

### Argumentos Adicionais

```bash
edged --tolerance 25                # Sobrescreve a tolerância inicial (1 a 100%)
edged --config /caminho/config.toml # Usa um arquivo de configuração alternativo
edged --help                        # Exibe o manual de ajuda
edged --version                     # Exibe a versão instalada
```

---

## 🎮 Controles Durante a Medição

| Controle | Ação |
| :--- | :--- |
| **Mover Cursor** | Atualiza a detecção de bordas ou o dimensionamento da área em tempo real |
| **Scroll Up / Down** | Aumenta ou diminui a tolerância da detecção de borda dinamicamente |
| **Clique Esquerdo** | Copia a medição atual para a área de transferência e fecha o Edged |
| **Shift + Clique** *(Bounds)* | Fixa o retângulo desenhado na tela para permitir desenhar outros |
| **Esc / Tecla Q** | Fecha o Edged imediatamente sem copiar |
| **Clique Direito** | Cancela e fecha a ferramenta |

---

## ⌨️ Exemplos de Keybinds

### Hyprland (`~/.config/hypr/hyprland.conf`)

```ini
# Edged Screen Ruler Keybinds
bind = $mainMod, M, exec, edged --spacing
bind = $mainMod SHIFT, M, exec, edged --bounds
bind = $mainMod CTRL, H, exec, edged --horizontal
bind = $mainMod CTRL, V, exec, edged --vertical
```

### Sway (`~/.config/sway/config`)

```ini
# Edged Screen Ruler Keybinds
bindsym $mod+m exec edged --spacing
bindsym $mod+Shift+m exec edged --bounds
bindsym $mod+Control+h exec edged --horizontal
bindsym $mod+Control+v exec edged --vertical
```

### GNOME

Você pode adicionar os atalhos em **Configurações → Teclado → Atalhos Personalizados**:
1. **Nome**: `Edged Spacing Ruler` | **Comando**: `edged --spacing` | **Atalho**: <kbd>Super</kbd> + <kbd>M</kbd>
2. **Nome**: `Edged Bounds Ruler` | **Comando**: `edged --bounds` | **Atalho**: <kbd>Super</kbd> + <kbd>Shift</kbd> + <kbd>M</kbd>

Ou configure diretamente via terminal:

```bash
gsettings set org.gnome.settings-daemon.plugins.media-keys.custom-keybinding:/org/gnome/settings-daemon/plugins/media-keys/custom-keybindings/custom0/ name 'Edged Ruler'
gsettings set org.gnome.settings-daemon.plugins.media-keys.custom-keybinding:/org/gnome/settings-daemon/plugins/media-keys/custom-keybindings/custom0/ command 'edged --spacing'
gsettings set org.gnome.settings-daemon.plugins.media-keys.custom-keybinding:/org/gnome/settings-daemon/plugins/media-keys/custom-keybindings/custom0/ binding '<Super>m'
```

---

## ⚙️ Arquivo de Configuração

Na primeira execução, o Edged cria automaticamente o arquivo em:

`~/.config/edged/config.toml`

Exemplo completo e documentado:

```toml
[general]
# Modo padrão ao iniciar sem argumentos: "bounds", "spacing", "horizontal", "vertical"
default_mode = "spacing"

# Método de captura de tela: "auto", "grim", "portal", "command"
# - "auto": Tenta 'grim' primeiro (wlroots/Hyprland), com fallback para xdg-desktop-portal (GNOME/KDE)
# - "grim": Força uso do grim
# - "portal": Força uso do portal XDG Desktop
capture_method = "auto"

# Comando customizado caso capture_method = "command" (use %s para o caminho do arquivo temporário)
custom_capture_command = ""

# Se deve fechar o aplicativo imediatamente após copiar a medida com o clique esquerdo
close_on_copy = true

[ruler]
# Tolerância inicial para edge detection (em porcentagem: 1 a 100)
initial_tolerance = 15

# Variação da tolerância a cada "tick" de scroll da roda do mouse
tolerance_step = 2

# Desenhar "pés" (linhas delimitadoras nas extremidades das medições)
draw_feet = true

# Tamanho dos pés em pixels
feet_size = 10.0

# Espessura da linha de medição
line_thickness = 2.0

# Exibir mira central sutil na posição exata do cursor
draw_crosshair = true

[bounds]
# Manter Shift pressionado fixa os retângulos desenhados anteriormente na tela
shift_pins_bounds = true

# Espessura da borda do retângulo no modo Bounds
border_thickness = 2.0

[appearance]
# Opacidade do fundo escurecido da tela (0.0 = totalmente transparente, 0.05 = sutil)
overlay_opacity = 0.05

# Cores: formatos "#RRGGBB", "#RRGGBBAA" ou "rgba(r, g, b, a)"
line_color = "#3B82F6"
line_shadow_color = "rgba(0, 0, 0, 0.6)"

bounds_fill_color = "rgba(59, 130, 246, 0.15)"
bounds_border_color = "#3B82F6"

# Estilo das etiquetas numéricas (badges)
label_bg_color = "rgba(15, 23, 42, 0.90)"
label_text_color = "#F8FAFC"
label_border_color = "#475569"
label_font_family = "sans-serif"
label_font_size = 12.0
label_border_radius = 5.0
label_padding_x = 7.0
label_padding_y = 3.5

[units]
# Exibir o sufixo "px" nas etiquetas
show_pixel_unit = true

# Unidade secundária opcional: "none", "inch", "cm", "mm", "pt"
secondary_unit = "none"

# DPI para cálculo da unidade secundária
screen_dpi = 96.0

[clipboard]
# Formato do texto copiado ao clicar:
# - "compact": ex: "1920x1080", "120"
# - "verbose": ex: "1920 × 1080 px", "120 px"
# - "css":     ex: "width: 1920px; height: 1080px;", "width: 120px;"
format = "compact"
```

---

## 🛠️ Compilação e Instalação

### Pré-requisitos (Arch Linux)

```bash
sudo pacman -S rust cargo gtk4 gtk4-layer-shell cairo pango wl-clipboard
# Opcional (recomendado para Hyprland/Sway):
sudo pacman -S grim
```

### Pré-requisitos (Fedora)

```bash
sudo dnf install rust cargo gtk4-devel gtk4-layer-shell-devel cairo-devel pango-devel wl-clipboard
# Opcional para compositores wlroots:
sudo dnf install grim
```

### Pré-requisitos (Ubuntu / Debian 23.04+)

```bash
sudo apt install cargo libgtk-4-dev libgtk4-layer-shell-dev libcairo2-dev libpango1.0-dev wl-clipboard
# Opcional:
sudo apt install grim
```

### Compilando o Binário Release

```bash
git clone https://github.com/txg0fk3/Edged.git # ou seu repositório local
cd Edged
cargo build --release
```

O binário otimizado estará disponível em `target/release/edged`.

### Instalando no Sistema

```bash
# Copia para o PATH local do usuário
cp target/release/edged ~/.local/bin/

# Ou instala globalmente para todos os usuários
sudo cp target/release/edged /usr/local/bin/

# Opcional: Instala o arquivo .desktop para inicialização por launchers (wofi, rofi, etc.)
cp edged.desktop ~/.local/share/applications/
```

---

## 🧪 Testes

Para executar a suite de testes automatizados:

```bash
cargo test
```

A suite cobre:
- Detecção de caixas e bordas com limites de pixels
- Variação de tolerância em gradientes
- Cálculo de retângulos em Bounds
- Formatação de clipboard (`compact`, `verbose`, `css`)
- Parsing de cores CSS / Hex
- Criação e validação do `config.toml`

---

## 📄 Licença

Distribuído sob licença MIT ou Apache-2.0.
