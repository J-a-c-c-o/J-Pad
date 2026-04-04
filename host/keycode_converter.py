from __future__ import annotations

import re
from qmk_keycodes import CONSTANTS


_CALL_RE = re.compile(r"^([A-Za-z_][A-Za-z0-9_]*)\((.*)\)$")

_MOD_FUNCTIONS = {
	"LCTL": "QK_LCTL",
	"LSFT": "QK_LSFT",
	"LALT": "QK_LALT",
	"LGUI": "QK_LGUI",
	"RCTL": "QK_RCTL",
	"RSFT": "QK_RSFT",
	"RALT": "QK_RALT",
	"RGUI": "QK_RGUI",
	"C": "QK_LCTL",
	"S": "QK_LSFT",
	"A": "QK_LALT",
	"G": "QK_LGUI",
}

_MOD_COMBOS = {
	"LCS": ("QK_LCTL", "QK_LSFT"),
	"LCA": ("QK_LCTL", "QK_LALT"),
	"LCG": ("QK_LCTL", "QK_LGUI"),
	"LSA": ("QK_LSFT", "QK_LALT"),
	"LSG": ("QK_LSFT", "QK_LGUI"),
	"LAG": ("QK_LALT", "QK_LGUI"),
	"LCSG": ("QK_LCTL", "QK_LSFT", "QK_LGUI"),
	"LCAG": ("QK_LCTL", "QK_LALT", "QK_LGUI"),
	"LSAG": ("QK_LSFT", "QK_LALT", "QK_LGUI"),
	"RCS": ("QK_RCTL", "QK_RSFT"),
	"RCA": ("QK_RCTL", "QK_RALT"),
	"RCG": ("QK_RCTL", "QK_RGUI"),
	"RSA": ("QK_RSFT", "QK_RALT"),
	"RSG": ("QK_RSFT", "QK_RGUI"),
	"RAG": ("QK_RALT", "QK_RGUI"),
	"RCSG": ("QK_RCTL", "QK_RSFT", "QK_RGUI"),
	"RCAG": ("QK_RCTL", "QK_RALT", "QK_RGUI"),
	"RSAG": ("QK_RSFT", "QK_RALT", "QK_RGUI"),
	"HYPR": ("QK_LCTL", "QK_LSFT", "QK_LALT", "QK_LGUI"),
	"MEH": ("QK_LCTL", "QK_LSFT", "QK_LALT"),
}


def _require_constant(name: str) -> int:
	if name not in CONSTANTS:
		raise ValueError(f"Unknown keycode or constant: {name}")
	return CONSTANTS[name]


def keycode(expr: str) -> int:
	expr = expr.strip()

	if expr in CONSTANTS:
		return CONSTANTS[expr]

	if expr.lower().startswith("0x") or expr.isdigit():
		return int(expr, 0)

	call = _CALL_RE.match(expr)
	if not call:
		raise ValueError(f"Unsupported keycode expression: {expr}")

	name = call.group(1)
	raw_args = call.group(2).strip()
	args = [arg.strip() for arg in raw_args.split(",") if arg.strip()] if raw_args else []


	if name in _MOD_FUNCTIONS:
		if len(args) != 1:
			raise ValueError(f"{name}() expects exactly one argument")
		return _require_constant(_MOD_FUNCTIONS[name]) | (keycode(args[0]) & 0xFF)

	if name in _MOD_COMBOS:
		if len(args) != 1:
			raise ValueError(f"{name}() expects exactly one argument")
		mod_mask = 0
		for mod_name in _MOD_COMBOS[name]:
			mod_mask |= _require_constant(mod_name)
		return mod_mask | (keycode(args[0]) & 0xFF)

	raise ValueError(f"Unsupported function in keycode expression: {name}")


def step(*expressions: str) -> list[int]:
	return [keycode(expr) for expr in expressions]

