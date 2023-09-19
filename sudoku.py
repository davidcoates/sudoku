import json
import subprocess
import pickle
import json
import lzstring
from dataclasses import dataclass
from typing import *

class SudokuError(Exception):
    """
    An application specific error.
    """
    pass


@dataclass
class Thermo:
    path: list[(int, int)]


class Sudoku(object):
    """
    Sudoku Puzzle representation
    """
    def __init__(self, sudoku_filename):
        self._sudoku_filename = sudoku_filename
        self.reset()

    def to_json(self):
        js = {
            "size": 9,
            "grid": [ [ dict() for c in range(9) ] for r in range(9) ],
            "thermometer": []
        }
        for r, row in enumerate(self._board):
            for c, digit in enumerate(row):
                if digit is not None:
                    js["grid"][r][c] = { "value": digit, "given": True }
        for constraint in self.constraints:
            match constraint:
                case Thermo(path):
                    js["thermometer"].append({ "lines": [[ f"R{r+1}C{c+1}" for (r, c) in constraint.path ]]})
                case _:
                    assert(False)
        return json.dumps(js)

    def to_url(self):
        return "https://f-puzzles.com/?load=" + lzstring.LZString().compressToBase64(self.to_json())

    def save(self):
        with open(self._sudoku_filename, 'wb') as sudoku_file:
            data = (self._board, self.constraints)
            pickle.dump(data, sudoku_file)

    def reset(self):
        try:
            with open(self._sudoku_filename, 'rb') as sudoku_file:
                (self._board, self.constraints) = pickle.load(sudoku_file)
        except FileNotFoundError:
            self._board = [ [ None for c in range(9) ] for r in range(9) ]
            self.constraints = []

    def set(self, r, c, value):
        self._board[r][c] = value

    def get(self, r, c):
        return self._board[r][c]

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

        def thermo(thermo):
            variables = [ f"{r+1}:{c+1}" for (r, c) in thermo.path ]
            return {
                "type": "Increasing",
                "variables": variables,
                "description": "thermometer"
            }
        for constraint in self.constraints:
            match constraint:
                case Thermo():
                    constraints.append(thermo(constraint))
                case _:
                    assert(False)

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

