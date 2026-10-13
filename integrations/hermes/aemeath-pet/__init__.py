"""Hermes Agent lifecycle adapter for Aemeath Pet."""

from __future__ import annotations

import json
import os
import shutil
import subprocess
import sys
from pathlib import Path
from typing import Any


def _find_executable() -> str | None:
    override = os.environ.get("AEMEATH_PET_EXECUTABLE", "").strip()
    if override and Path(override).expanduser().is_file():
        return str(Path(override).expanduser())

    candidates: list[Path] = []
    if sys.platform == "darwin":
        candidates.extend(
            [
                Path.home()
                / "Applications/Aemeath Codex Pet.app/Contents/MacOS/aemeath-codex",
                Path("/Applications/Aemeath Codex Pet.app/Contents/MacOS/aemeath-codex"),
            ]
        )
    elif os.name == "nt":
        local_app_data = os.environ.get("LOCALAPPDATA", "")
        if local_app_data:
            candidates.append(
                Path(local_app_data) / "Aemeath Codex Pet/aemeath-codex.exe"
            )

    for candidate in candidates:
        if candidate.is_file():
            return str(candidate)
    return shutil.which("aemeath-codex")


def _emit(event: str, **kwargs: Any) -> None:
    executable = _find_executable()
    if not executable:
        return

    payload = {
        "tool_name": str(kwargs.get("tool_name") or ""),
        "status": str(kwargs.get("status") or ""),
        "session_id": str(kwargs.get("session_id") or ""),
    }
    creation_flags = 0
    if os.name == "nt":
        creation_flags = (
            getattr(subprocess, "DETACHED_PROCESS", 0)
            | getattr(subprocess, "CREATE_NEW_PROCESS_GROUP", 0)
        )

    try:
        subprocess.Popen(
            [
                executable,
                "--provider-hook",
                "hermes",
                event,
                json.dumps(payload, ensure_ascii=False),
            ],
            stdin=subprocess.DEVNULL,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
            start_new_session=os.name != "nt",
            creationflags=creation_flags,
        )
    except OSError:
        # Pet integration must never interrupt an agent run.
        return


def _on_session_start(**kwargs: Any) -> None:
    _emit("session-start", **kwargs)


def _on_session_end(**kwargs: Any) -> None:
    _emit("session-end", **kwargs)


def _on_pre_llm_call(**kwargs: Any) -> None:
    _emit("user-prompt-submit", **kwargs)


def _on_post_llm_call(**kwargs: Any) -> None:
    _emit("stop", **kwargs)


def _on_pre_tool_call(**kwargs: Any) -> None:
    _emit("pre-tool-use", **kwargs)


def _on_post_tool_call(**kwargs: Any) -> None:
    _emit("post-tool-use", **kwargs)


def _on_pre_approval_request(**kwargs: Any) -> None:
    _emit("permission-request", **kwargs)


def _on_subagent_start(**kwargs: Any) -> None:
    _emit("subagent-start", **kwargs)


def _on_subagent_stop(**kwargs: Any) -> None:
    _emit("subagent-stop", **kwargs)


def register(ctx: Any) -> None:
    ctx.register_hook("on_session_start", _on_session_start)
    ctx.register_hook("on_session_end", _on_session_end)
    ctx.register_hook("pre_llm_call", _on_pre_llm_call)
    ctx.register_hook("post_llm_call", _on_post_llm_call)
    ctx.register_hook("pre_tool_call", _on_pre_tool_call)
    ctx.register_hook("post_tool_call", _on_post_tool_call)
    ctx.register_hook("pre_approval_request", _on_pre_approval_request)
    ctx.register_hook("subagent_start", _on_subagent_start)
    ctx.register_hook("subagent_stop", _on_subagent_stop)
