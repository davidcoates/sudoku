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
class Constraints:
    thermometers: list[Thermometer] = field(default_factory=list)


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
        return " ".join(rules)

    def to_json(self):
        js = {
            "title": self._sudoku_name,
            "author": "Dave",
            "ruleset": self.rules(),
            "size": 9,
            "grid": [ [ dict() for c in range(9) ] for r in range(9) ],
            "thermometer": []
        }
        for r, row in enumerate(self._board):
            for c, digit in enumerate(row):
                if digit is not None:
                    js["grid"][r][c] = { "value": digit, "given": True }
        for thermo in self._constraints.thermometers:
            js["thermometer"].append({ "lines": [[ f"R{r+1}C{c+1}" for (r, c) in thermo.path ]]})
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

    def solve(self):

        print(self.to_url())

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

        for thermo in self._constraints.thermometers:
            variables = [ f"{r+1}:{c+1}" for (r, c) in thermo.path ]
            constraints.append({
                "type": "Increasing",
                "variables": variables,
                "description": "thermometer"
            })

        solver_input = {
            "domains": domains,
            "constraints": constraints,
        }
        pipe = subprocess.run(
            ["./solver/target/debug/solver"],
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            input=json.dumps(solver_input),
            encoding="ascii")

        breadcrumbs = pipe.stderr
        solver_output = json.loads(pipe.stdout)
        result = solver_output["result"]
        duration_ms = solver_output["duration_ms"]
        board = [ [ self._board[r][c] for c in range(9) ] for r in range(9) ]
        for variable, domain in solver_output["domains"].items():
            if len(domain) == 1:
                [r, c] = variable.split(':')
                r, c = int(r) - 1, int(c) - 1
                board[r][c] = domain[0]
        return (f"{result.title()} ({duration_ms}ms)", board, breadcrumbs)

