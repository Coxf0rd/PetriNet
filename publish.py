"""
Публикация проекта в GitHub (Coxf0rd/PetriNet).

Что делает:
- Проверяет, что origin указывает на нужный репозиторий.
- (Опционально) подтягивает изменения из origin (rebase) или валидирует, что вы не отстаёте.
- Прогоняет проверки (fmt/check/test) и проверку "mojibake" (по правилам проекта).
- Увеличивает версию в Cargo.toml и Cargo.lock (patch: x.y.z -> x.y.(z+1)).
- (Windows) собирает portable exe через build_portable_exe.ps1 и обновляет PetriNet-<ver>.exe.
- Делает коммит с сообщением "Улучшение v<версия>" и пушит в origin/<branch>.

Важно:
- SSH ключ НЕ нужно "подставлять" в код. Git использует ключи из ~/.ssh и ssh-agent.
- Публичный ключ добавляется в GitHub Settings -> SSH and GPG keys.
"""

from __future__ import annotations

import argparse
import re
import subprocess
import sys
from pathlib import Path

REPO_URL = "https://github.com/Coxf0rd/PetriNet"
REPO_SSH_URL = "git@github.com:Coxf0rd/PetriNet.git"
DEFAULT_BRANCH = "main"


def run(cmd: list[str], *, cwd: Path | None = None, ok_codes: set[int] | None = None) -> str:
    """Run a command; return stdout; raise on non-OK exit codes."""

    print(f"$ {' '.join(cmd)}")
    proc = subprocess.run(
        cmd,
        cwd=str(cwd) if cwd is not None else None,
        text=True,
        capture_output=True,
    )
    if ok_codes is None:
        ok_codes = {0}
    if proc.returncode not in ok_codes:
        sys.stderr.write(proc.stdout)
        sys.stderr.write(proc.stderr)
        raise SystemExit(f"Команда завершилась с кодом {proc.returncode}: {' '.join(cmd)}")
    return proc.stdout


def git(args: list[str], *, cwd: Path) -> str:
    return run(["git", *args], cwd=cwd)


def bump_version(version: str) -> str:
    parts = version.split(".")
    if len(parts) != 3:
        raise ValueError("Ожидается семантическая версия x.y.z")
    major, minor, patch = map(int, parts)
    return f"{major}.{minor}.{patch + 1}"


def update_cargo_toml(path: Path, new_version: str) -> None:
    txt = path.read_text(encoding="utf-8")
    updated = re.sub(
        r"^version = \"(?P<v>[0-9]+\.[0-9]+\.[0-9]+)\"$",
        f"version = \"{new_version}\"",
        txt,
        count=1,
        flags=re.MULTILINE,
    )
    if txt == updated:
        raise RuntimeError("Не удалось обновить Cargo.toml")
    path.write_text(updated, encoding="utf-8")


def update_cargo_lock(path: Path, old_version: str, new_version: str) -> None:
    txt = path.read_text(encoding="utf-8")
    pattern = (
        r"(\[\[package\]\]\nname = \"petri_net_legacy_editor\"\n(?:.*\n)*?version = \")"
        + re.escape(old_version)
        + r"\""
    )
    updated, count = re.subn(pattern, f"\\1{new_version}\"", txt, count=1)
    if count == 0:
        raise RuntimeError("Не удалось обновить Cargo.lock")
    path.write_text(updated, encoding="utf-8")


def ensure_origin_remote(repo_root: Path, branch: str) -> None:
    """Ensure origin points to Coxf0rd/PetriNet, set it if missing, fetch branch."""

    try:
        origin = git(["remote", "get-url", "origin"], cwd=repo_root).strip()
    except SystemExit:
        origin = ""

    if not origin:
        print(f"origin не настроен: добавляю {REPO_SSH_URL}")
        git(["remote", "add", "origin", REPO_SSH_URL], cwd=repo_root)
        git(["fetch", "origin", branch], cwd=repo_root)
        return

    normalized = origin.removesuffix(".git")
    ok = normalized in {
        REPO_URL,
        REPO_URL.removesuffix(".git"),
        REPO_SSH_URL.removesuffix(".git"),
    }
    if not ok:
        raise SystemExit(
            f"origin указывает на другой репозиторий: {origin}\n"
            f"Ожидается: {REPO_URL} или {REPO_SSH_URL}\n"
            "Исправьте origin вручную."
        )

    git(["fetch", "origin", branch], cwd=repo_root)


def ensure_not_behind_origin(repo_root: Path, branch: str, *, pull_rebase: bool) -> None:
    git(["fetch", "origin", branch], cwd=repo_root)
    counts = git(["rev-list", "--left-right", "--count", f"origin/{branch}...HEAD"], cwd=repo_root).strip()
    behind_s, _ahead_s = counts.split()
    behind = int(behind_s)
    if behind > 0 and not pull_rebase:
        raise SystemExit(
            f"Локальная ветка отстаёт от origin/{branch} на {behind} коммит(ов).\n"
            f"Сначала выполните `git pull --rebase origin {branch}`, либо запустите скрипт с `--pull-rebase`."
        )
    if behind > 0 and pull_rebase:
        git(["pull", "--rebase", "origin", branch], cwd=repo_root)


def run_project_checks(repo_root: Path) -> None:
    # По правилам проекта (AGENTS.md)
    run(["cargo", "fmt"], cwd=repo_root)
    run(["cargo", "check"], cwd=repo_root)
    run(["cargo", "test"], cwd=repo_root)
    # rg возвращает 1, если совпадений нет (это OK)
    run(["rg", "-n", r"�|\?\?\?\?", "src", "README.md"], cwd=repo_root, ok_codes={0, 1})


def update_portable_exe(repo_root: Path, new_version: str) -> None:
    ps1 = repo_root / "build_portable_exe.ps1"
    if not ps1.exists():
        return

    run(["powershell", "-ExecutionPolicy", "Bypass", "-File", str(ps1)], cwd=repo_root)
    new_exe = repo_root / f"PetriNet-{new_version}.exe"
    if not new_exe.exists():
        print(f"Предупреждение: не найден {new_exe.name} после сборки, пропускаю обновление exe.")
        return

    tracked = git(["ls-files", "PetriNet-*.exe"], cwd=repo_root).splitlines()
    for old in tracked:
        if Path(old).name != new_exe.name:
            git(["rm", "-f", old], cwd=repo_root)
    git(["add", "-f", new_exe.name], cwd=repo_root)


def main() -> None:
    repo_root = Path(__file__).resolve().parents[1]
    cargo_toml = repo_root / "Cargo.toml"
    cargo_lock = repo_root / "Cargo.lock"

    parser = argparse.ArgumentParser(description=f"Publish to {REPO_URL}")
    parser.add_argument("--branch", default=DEFAULT_BRANCH)
    parser.add_argument("--pull-rebase", action="store_true", help="Автоматически сделать pull --rebase перед пушем")
    parser.add_argument("--skip-checks", action="store_true", help="Пропустить cargo fmt/check/test и rg-проверку")
    parser.add_argument("--skip-build", action="store_true", help="Пропустить build_portable_exe.ps1")
    args = parser.parse_args()

    ensure_origin_remote(repo_root, args.branch)
    ensure_not_behind_origin(repo_root, args.branch, pull_rebase=args.pull_rebase)

    data = cargo_toml.read_text(encoding="utf-8")
    match = re.search(r"^version = \"(?P<version>[0-9]+\.[0-9]+\.[0-9]+)\"$", data, flags=re.MULTILINE)
    if not match:
        raise RuntimeError("Не удалось найти версию в Cargo.toml")
    old_version = match.group("version")
    new_version = bump_version(old_version)

    if not args.skip_checks:
        run_project_checks(repo_root)

    update_cargo_toml(cargo_toml, new_version)
    update_cargo_lock(cargo_lock, old_version, new_version)

    # Стадим все изменения проекта.
    git(["add", "-A"], cwd=repo_root)

    if not args.skip_build:
        update_portable_exe(repo_root, new_version)
        git(["add", "-A"], cwd=repo_root)

    # Сообщение коммита: "улучшение + название версии"
    message = f"Улучшение v{new_version}"

    status = git(["status", "--porcelain"], cwd=repo_root).strip()
    if not status:
        print("Нет изменений для коммита.")
        return

    git(["commit", "-m", message], cwd=repo_root)
    git(["push", "origin", args.branch], cwd=repo_root)


if __name__ == "__main__":
    main()
