import json
import subprocess
import pickle
import json
import lzstring
from dataclasses import dataclass, field
from typing import *

@dataclass
class Thermometer:
    path: list[(int, int)]

@dataclass
class Palindrome:
    path: list[(int, int)]

@dataclass
class Renban:
    path: list[(int, int)]

@dataclass
class Constraints:
    thermometers: list[Thermometer] = field(default_factory=list)
    palindromes: list[Palindrome] = field(default_factory=list)
    renbans: list[Renban] = field(default_factory=list)
    anti_knight: bool = False

class Sudoku(object):
    """
    Sudoku Puzzle representation
    """
    def __init__(self, sudoku_name):
        self._sudoku_name = sudoku_name
        self._sudoku_filename = f"sudokus/{sudoku_name}.sudoku"
        self.reset()

    def rules(self):
        rules = ["Normal Sudoku rules apply."]
        if self._constraints.thermometers:
            rules.append("Digits on a thermometer strictly increase from the bulb end.")
        if self._constraints.palindromes:
            rules.append("Digits on a yellow line form a palindrome.")
        if self._constraints.renbans:
            rules.append("Digits on a purple line form a consecutive set.")
        if self._constraints.anti_knight:
            rules.append("Digits a knights move away can not be the same.")
        return " ".join(rules)

    # TODO renban
    def to_json(self):
        js = {
            "title": self._sudoku_name,
            "author": "Dave",
            "ruleset": self.rules(),
            "size": 9,
            "grid": [ [ dict() for c in range(9) ] for r in range(9) ],
            "thermometer": [],
            "palindrome": [],
            "line": []
        }
        if self.constraints.anti_knight:
            js["antiknight"] = {}
        for r, row in enumerate(self._board):
            for c, digit in enumerate(row):
                if digit is not None:
                    js["grid"][r][c] = { "value": digit, "given": True }
        for line in self._constraints.thermometers:
            js["thermometer"].append({ "lines": [[ f"R{r+1}C{c+1}" for (r, c) in line.path ]]})
        for line in self._constraints.palindromes:
            path = [ f"R{r+1}C{c+1}" for (r, c) in line.path ]
            js["palindrome"].append({ "lines": [path]} )
            js["line"].append({
                "lines": [path],
                "baseC": "#FFDD00",
                "outlineC": "#FFDD00",
                "fontC": "#000000",
                "size": 0.5,
                "width": 0.5,
                "height": 0.5,
                "angle": 0,
             })
        return json.dumps(js)

    def to_url(self):
        return "https://app.crackingthecryptic.com/sudoku/?puzzleid=fpuzzles" + lzstring.LZString().compressToBase64(self.to_json())

    def save(self):
        with open(self._sudoku_filename, 'wb') as sudoku_file:
            data = (self._board, self._constraints)
            pickle.dump(data, sudoku_file)

    def reset(self):
        try:
            with open(self._sudoku_filename, 'rb') as sudoku_file:
                (self._board, self._constraints) = pickle.load(sudoku_file)
                for name, field in Constraints.__dataclass_fields__.items():
                    try:
                        getattr(self._constraints, name)
                    except AttributeError:
                        setattr(self._constraints, name, field.default_factory())
        except FileNotFoundError:
            self._board = [ [ None for c in range(9) ] for r in range(9) ]
            self._constraints = Constraints()

    def set(self, r, c, value):
        self._board[r][c] = value

    def get(self, r, c):
        return self._board[r][c]

    @property
    def constraints(self):
        return self._constraints

    def solve(self, branch, greedy, max_depth, trace):

        domains = {}
        for r in range(9):
            for c in range(9):
                if self._board[r][c] is None:
                    domain = [1, 2, 3, 4, 5, 6, 7, 8, 9]
                else:
                    domain = [ self._board[r][c] ]
                domains[f"{r+1}:{c+1}"] = domain

        constraints = []

        def permutation(coordinates, description):
            variables = [ f"{r+1}:{c+1}" for (r, c) in coordinates ]
            return {
                "type": "Permutation",
                "variables": variables,
                "domain": [1, 2, 3, 4, 5, 6, 7, 8, 9],
                "description": description
            }
        for r in range(9):
            constraints.append(permutation(
                [(r, c) for c in range(9)],
                f"sudoku row({r+1})"
            ))
        for c in range(9):
            constraints.append(permutation(
                [(r, c) for r in range(9)],
                f"sudoku col({c+1})"
            ))
        for r in range(3):
            for c in range(3):
                constraints.append(permutation(
                    [(r*3 + i, c*3 + j) for i in range(3) for j in range(3)],
                    f"sudoku box({r*3 + c + 1})"
                ))

        for line in self._constraints.thermometers:
            variables = [ f"{r+1}:{c+1}" for (r, c) in line.path ]
            constraints.append({
                "type": "Increasing",
                "variables": variables,
                "description": "thermometer"
            })

        for line in self._constraints.palindromes:
            n = len(line.path)//2
            for i in range(n):
                variables = [ f"{r+1}:{c+1}" for (r, c) in [line.path[i], line.path[-(i+1)]] ]
                constraints.append({
                    "type": "Equals",
                    "variables": variables,
                    "description": "palindrome"
                })

        for line in self._constraints.renbans:
            variables = [ f"{r+1}:{c+1}" for (r, c) in line.path ]
            constraints.append({
                "type": "ConsecutiveSet",
                "variables": variables,
                "description": "renban"
            })

        if self._constraints.anti_knight:
            knight_moves = set()
            def try_knight_move(r, c, x, y):
                if 0 <= r + x < 9 and 0 <= c + y < 9:
                    knight_moves.add(frozenset({(r, c), (r + x, c + y)}))
            for r in range(9):
                for c in range(9):
                    try_knight_move(r, c, -2, -1)
                    try_knight_move(r, c, +2, +1)
                    try_knight_move(r, c, +2, -1)
                    try_knight_move(r, c, -2, +1)
                    try_knight_move(r, c, -1, -2)
                    try_knight_move(r, c, +1, +2)
                    try_knight_move(r, c, +1, -2)
                    try_knight_move(r, c, -1, +2)
            for knight_move in knight_moves:
                variables = [ f"{r+1}:{c+1}" for (r, c) in knight_move ]
                constraints.append({
                    "type": "NotEquals",
                    "variables": variables,
                    "description": "antiknight"
                })

        solver_input = {
            "domains": domains,
            "constraints": constraints,
            "greedy": greedy,
            "breadcrumbs": trace,
            "max_depth": max_depth if branch else 0,
        }

        try:
            pipe = subprocess.run(
                ["./solver/target/release/solver"],
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                input=json.dumps(solver_input),
                encoding="ascii",
                timeout=30)
        except subprocess.TimeoutExpired:
            return ("Timed Out", None)

        solver_output = json.loads(pipe.stdout)
        result = solver_output["result"]
        duration_ms = solver_output["duration_ms"]
        board = [ [ self._board[r][c] for c in range(9) ] for r in range(9) ]
        for variable, domain in solver_output["domains"].items():
            if len(domain) == 1:
                [r, c] = variable.split(':')
                r, c = int(r) - 1, int(c) - 1
                board[r][c] = domain[0]
        return (f"{result.title()} ({duration_ms}ms)", board, pipe.stderr)

