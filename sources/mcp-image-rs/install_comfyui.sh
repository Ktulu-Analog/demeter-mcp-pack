#!/usr/bin/env bash
# ==============================================================================
# install_comfyui.sh — Installation automatique de ComfyUI sur Linux
# ==============================================================================
# Auteur  : Pierre COUGET  ktulu.analog@gmail.com
# Licence : GNU Affero General Public License v3.0 (AGPL-3.0)
# Année   : 2026
# ==============================================================================
set -euo pipefail

# ── Couleurs ──────────────────────────────────────────────────────────────────
RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'
CYAN='\033[0;36m'; BOLD='\033[1m'; RESET='\033[0m'

info()    { echo -e "${CYAN}[INFO]${RESET}  $*"; }
success() { echo -e "${GREEN}[OK]${RESET}    $*"; }
warn()    { echo -e "${YELLOW}[WARN]${RESET}  $*"; }
error()   { echo -e "${RED}[ERROR]${RESET} $*" >&2; exit 1; }

# ── Configuration (modifiable) ────────────────────────────────────────────────
INSTALL_DIR="${HOME}/comfyui"
VENV_DIR="${INSTALL_DIR}/.venv"
PYTHON_MIN="3.10"
COMFYUI_REPO="https://github.com/comfyanonymous/ComfyUI.git"

# ── Bannière ──────────────────────────────────────────────────────────────────
echo -e "${BOLD}"
echo "  ╔══════════════════════════════════════════╗"
echo "  ║       Installation de ComfyUI            ║"
echo "  ╚══════════════════════════════════════════╝"
echo -e "${RESET}"
echo -e "  Dossier cible : ${CYAN}${INSTALL_DIR}${RESET}"
echo ""

# ── 1. Détection GPU ──────────────────────────────────────────────────────────
detect_gpu() {
  # On évite `nvidia-smi` sans arguments : il retourne un exit code non-nul
  # quand un display est attaché au GPU (cas typique d'une workstation/Quadro).
  # `--query-gpu` retourne toujours 0 si le driver est fonctionnel.
  if command -v nvidia-smi &>/dev/null \
     && nvidia-smi --query-gpu=name --format=csv,noheader &>/dev/null; then
    echo "nvidia"
  elif lspci 2>/dev/null | grep -qi "amd\|radeon"; then
    echo "amd"
  else
    echo "cpu"
  fi
}

# Détecte si le GPU NVIDIA est une architecture ancienne (Pascal/Maxwell/Volta = CC < 7.5)
# Ces GPUs nécessitent cu126 max — cu128+ abandonne le support sm_5x/sm_6x/sm_70
detect_nvidia_arch() {
  if ! command -v nvidia-smi &>/dev/null; then echo "unknown"; return; fi
  local cc
  cc=$(python3 -c "
import subprocess, re
out = subprocess.run(['nvidia-smi', '--query-gpu=compute_cap', '--format=csv,noheader'],
    capture_output=True, text=True).stdout.strip().split('\n')[0]
print(out)
" 2>/dev/null || echo "unknown")
  echo "$cc"
}

GPU_TYPE=$(detect_gpu)
case "$GPU_TYPE" in
  nvidia)
    CC=$(detect_nvidia_arch)
    GPU_NAME=$(nvidia-smi --query-gpu=name --format=csv,noheader 2>/dev/null | head -1 || echo "NVIDIA")
    info "GPU détecté : ${GPU_NAME} (Compute Capability ${CC})"
    # Avertissement architectures Pascal/Maxwell/Volta (CC < 7.5)
    python3 -c "
cc = '${CC}'
if cc not in ('unknown', '') and float(cc) < 7.5:
    print('\033[1;33m[WARN]  Architecture Pascal/Volta détectée (CC ${CC}) :\033[0m')
    print('        PyTorch cu128+ ne supporte plus ce GPU.')
    print('        Le script installe cu126, qui est le build recommandé.')
" 2>/dev/null || true
    ;;
  amd)    info "GPU détecté : AMD (ROCm)" ;;
  cpu)    warn "Aucun GPU détecté — mode CPU uniquement (lent)" ;;
esac

# ── 2. Dépendances système ────────────────────────────────────────────────────
info "Vérification des dépendances système..."

MISSING_PKGS=()
for cmd in git python3 pip3 curl; do
  command -v "$cmd" &>/dev/null || MISSING_PKGS+=("$cmd")
done

if [[ ${#MISSING_PKGS[@]} -gt 0 ]]; then
  warn "Paquets manquants : ${MISSING_PKGS[*]}"
  info "Installation via apt..."
  sudo apt-get update -qq
  sudo apt-get install -y git python3 python3-pip python3-venv curl
fi

# Vérifier la version de Python
PYTHON_BIN=$(command -v python3)
PYTHON_VERSION=$("$PYTHON_BIN" -c "import sys; print(f'{sys.version_info.major}.{sys.version_info.minor}')")
if ! python3 -c "import sys; sys.exit(0 if sys.version_info >= (3,10) else 1)"; then
  error "Python ${PYTHON_MIN}+ requis (installé : ${PYTHON_VERSION}). Mettez à jour Python."
fi
success "Python ${PYTHON_VERSION} ✓"

# ── 3. Clone du dépôt ─────────────────────────────────────────────────────────
if [[ -d "${INSTALL_DIR}/.git" ]]; then
  warn "ComfyUI déjà cloné dans ${INSTALL_DIR} — mise à jour (git pull)..."
  git -C "${INSTALL_DIR}" pull --ff-only
else
  info "Clonage de ComfyUI dans ${INSTALL_DIR}..."
  git clone --depth=1 "${COMFYUI_REPO}" "${INSTALL_DIR}"
fi
success "Dépôt ComfyUI prêt ✓"

# ── 4. Environnement virtuel Python ───────────────────────────────────────────
if [[ ! -d "${VENV_DIR}" ]]; then
  info "Création de l'environnement virtuel Python..."
  python3 -m venv "${VENV_DIR}"
fi
# shellcheck disable=SC1091
source "${VENV_DIR}/bin/activate"
pip install --upgrade pip --quiet
success "Virtualenv activé ✓"

# ── 5. Installation de PyTorch ────────────────────────────────────────────────
info "Installation de PyTorch (GPU: ${GPU_TYPE})..."

case "$GPU_TYPE" in
  nvidia)
    # cu126 = dernier build PyTorch supportant Pascal/Volta (CC 5.x/6.x/7.x)
    # cu128+ abandonne Pascal — ne PAS utiliser cu128 ou supérieur sur P6000/GTX10xx/V100
    pip install torch torchvision torchaudio --index-url https://download.pytorch.org/whl/cu126 --quiet
    ;;
  amd)
    pip install torch torchvision torchaudio --index-url https://download.pytorch.org/whl/rocm6.1 --quiet
    ;;
  cpu)
    pip install torch torchvision torchaudio --index-url https://download.pytorch.org/whl/cpu --quiet
    ;;
esac
success "PyTorch installé ✓"

# ── 6. Dépendances ComfyUI ────────────────────────────────────────────────────
info "Installation des dépendances ComfyUI..."
pip install -r "${INSTALL_DIR}/requirements.txt" --quiet
success "Dépendances installées ✓"

# ── 7. Création du script de lancement ───────────────────────────────────────
LAUNCH_SCRIPT="${INSTALL_DIR}/run.sh"
cat > "${LAUNCH_SCRIPT}" << 'LAUNCH'
#!/usr/bin/env bash
# Lance ComfyUI dans son virtualenv
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "${SCRIPT_DIR}/.venv/bin/activate"
exec python "${SCRIPT_DIR}/main.py" "$@"
LAUNCH
chmod +x "${LAUNCH_SCRIPT}"
success "Script de lancement créé : ${LAUNCH_SCRIPT} ✓"

# ── 8. Raccourci système (optionnel) ──────────────────────────────────────────
SYMLINK_PATH="${HOME}/.local/bin/comfyui"
if [[ -d "${HOME}/.local/bin" ]]; then
  ln -sf "${LAUNCH_SCRIPT}" "${SYMLINK_PATH}"
  success "Raccourci créé : comfyui ✓ (assurez-vous que ~/.local/bin est dans votre PATH)"
fi

# ── 9. Résumé ─────────────────────────────────────────────────────────────────
echo ""
echo -e "${BOLD}${GREEN}  ✔ Installation terminée !${RESET}"
echo ""
echo -e "  ${BOLD}Lancer ComfyUI :${RESET}"
echo -e "    ${CYAN}${LAUNCH_SCRIPT}${RESET}"
if [[ -L "${SYMLINK_PATH}" ]]; then
  echo -e "    ou simplement : ${CYAN}comfyui${RESET}"
fi
echo ""
echo -e "  ${BOLD}Options utiles :${RESET}"
echo -e "    --listen         Écouter sur 0.0.0.0 (accès réseau local)"
echo -e "    --port 8188      Changer le port (défaut : 8188)"
echo -e "    --lowvram        Mode VRAM réduite (GPU ≤ 6 Go)"
echo -e "    --cpu            Forcer le mode CPU"
echo ""
echo -e "  ${BOLD}Interface web :${RESET} ${CYAN}http://127.0.0.1:8188${RESET}"
echo -e "  ${BOLD}Modèles (checkpoints) :${RESET} ${CYAN}${INSTALL_DIR}/models/checkpoints/${RESET}"
echo ""
