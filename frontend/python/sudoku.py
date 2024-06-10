import json
import subprocess
import pickle
import json
import lzstring
from dataclasses import dataclass, field
from typing import *
from enum import Enum

class LineType(Enum):
    UNDIRECTED = 0
    DIRECTED = 1

@dataclass(init=False)
class Line:
    path: list[(int, int)]
    ty: LineType = LineType.UNDIRECTED

    def __init__(self, path, ty):
        assert len(path) > 0
        self.ty = ty
        if self.ty == LineType.UNDIRECTED:
            # pick a canonical orientation (for comparisons / hash)
            path = min(path, path[::-1])
        self.path = path


@dataclass
class Edge:
    # cell0 should compare less than cell1
    cell0: (int, int)
    cell1: (int, int)

    def __init__(self, cell0, cell1):
        (self.cell0, self.cell1) = (cell0, cell1) if cell0 < cell1 else (cell1, cell0)


@dataclass
class Kropki:
    class Color(Enum):
        WHITE = 0
        BLACK = 1
    color: Color
    edge: Edge


@dataclass
class XV:
    class Value(Enum):
        X = 10
        V = 5
    value: Value
    edge: Edge


@dataclass
class Digit:
    values: [int]

    @classmethod
    def blank(cls):
        return Digit([1, 2, 3, 4, 5, 6, 7, 8, 9])


@dataclass
class Constraints:
    thermometers: set[Line] = field(default_factory=set)
    palindromes: set[Line] = field(default_factory=set)
    renbans: set[Line] = field(default_factory=set)
    whispers: set[Line] = field(default_factory=set)
    kropkis: set[Kropki] = field(default_factory=set)
    xvs: set[XV] = field(default_factory=set)
    antiknight: bool = False
    antiking: bool = False
    antixv: bool = False

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
        if self._constraints.whispers:
            rules.append("Consecutive cells on a green line have a difference of at least 5.")
        if self._constraints.kropkis:
            rules.append("White dots indicate consecutive digits. Black dots indicate a 1:2 ratio.")
        if self._constraints.xvs or self._constraints.antixv:
            rules.append("Cells separated by a V sum to 5, cells separated by an X sum to 10.")
            if self._constraints.antixv:
                rules.append("All XV's are given.")
            else:
                rules.append("Not all XV's are necessarily given.")
        if self._constraints.antiknight:
            rules.append("Digits a knight's move away can not be the same.")
        if self._constraints.antiking:
            rules.append("Digits a king's move away can not be the same.")
        return " ".join(rules)

    # TODO renban, kropkis, XV
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
        if self.constraints.antiknight:
            js["antiknight"] = {}
        if self.constraints.antiking:
            js["antiking"] = {}
        for r, row in enumerate(self._board):
            for c, digit in enumerate(row):
                if len(digit.values) == 1:
                    js["grid"][r][c] = { "value": digit.values[0], "given": True }
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
                # backwards compatabilty fixes
                for name, field in Constraints.__dataclass_fields__.items():
                    try:
                        getattr(self._constraints, name)
                    except AttributeError:
                        setattr(self._constraints, name, field.default_factory())
                for r, row in enumerate(self._board):
                    for c, digit in enumerate(row):
                        if not isinstance(digit, Digit):
                            self._board[r][c] = Digit([digit]) if digit is not None else Digit.blank()

        except FileNotFoundError:
            self._board = [ [ Digit() for c in range(9) ] for r in range(9) ]
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
                domains[f"{r+1}:{c+1}"] = self._board[r][c].values

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

        for line in self._constraints.whispers:
            variables = [ f"{r+1}:{c+1}" for (r, c) in line.path ]
            constraints.append({
                "type": "Difference",
                "variables": variables,
                "description": "whisper",
                "threshold": 5,
            })

        for kropki in self._constraints.kropkis:
            variables = [ f"{r+1}:{c+1}" for (r, c) in [kropki.edge.cell0, kropki.edge.cell1] ]
            match kropki.color:
                case Kropki.Color.WHITE:
                    constraints.append({
                        "type": "ConsecutiveSet",
                        "variables": variables,
                        "description": "white kropki",
                    })
                case Kropki.Color.BLACK:
                    constraints.append({
                        "type": "Ratio",
                        "variables": variables,
                        "description": "black kropki",
                        "ratio": 2,
                    })
                case _:
                    assert(False)

        def edges():
            for r in range(9):
                for c in range(9):
                    if r < 8:
                        yield Edge((r, c), (r+1, c))
                    if c < 8:
                        yield Edge((r, c), (r, c+1))
        antixv_edges = list(edges())

        for xv in self._constraints.xvs:
            variables = [ f"{r+1}:{c+1}" for (r, c) in [xv.edge.cell0, xv.edge.cell1] ]
            constraints.append({
                "type": "DistinctSum",
                "variables": variables,
                "description": xv.value.name,
                "sum": xv.value.value
            })
            if self._constraints.antixv:
                antixv_edges.remove(xv.edge)

        if self._constraints.antixv:
            for edge in antixv_edges:
                variables = [ f"{r+1}:{c+1}" for (r, c) in [edge.cell0, edge.cell1] ]
                constraints.append({
                    "type": "DistinctAntisum",
                    "variables": variables,
                    "description": "antixv",
                    "antisums": [ 5, 10 ]
                })


        # TODO should avoid duplicate constraints

        moves = set()
        def try_move(r, c, x, y):
            if 0 <= r + x < 9 and 0 <= c + y < 9:
                moves.add(frozenset({(r, c), (r + x, c + y)}))

        def register_moves(description):
            for move in moves:
                variables = [ f"{r+1}:{c+1}" for (r, c) in move ]
                constraints.append({
                    "type": "NotEquals",
                    "variables": variables,
                    "description": description
                })
            moves.clear()

        if self._constraints.antiknight:
            for r in range(9):
                for c in range(9):
                    try_move(r, c, -2, -1)
                    try_move(r, c, +2, +1)
                    try_move(r, c, +2, -1)
                    try_move(r, c, -2, +1)
                    try_move(r, c, -1, -2)
                    try_move(r, c, +1, +2)
                    try_move(r, c, +1, -2)
                    try_move(r, c, -1, +2)
        register_moves("antiknight")

        if self._constraints.antiking:
            for r in range(9):
                for c in range(9):
                    try_move(r, c, -1, -1)
                    try_move(r, c,  0, -1)
                    try_move(r, c, +1, -1)
                    try_move(r, c, -1,  0)
                    try_move(r, c, +1,  0)
                    try_move(r, c, -1, +1)
                    try_move(r, c,  0, +1)
                    try_move(r, c, +1, +1)
        register_moves("antiking")

        solver_input = {
            "domains": domains,
            "constraints": constraints,
            "greedy": greedy,
            "breadcrumbs": trace,
            "max_depth": max_depth if branch else 0,
        }

        try:
            pipe = subprocess.run(
                ["../../backend/solver/target/release/solver"],
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
            [r, c] = variable.split(':')
            r, c = int(r) - 1, int(c) - 1
            board[r][c] = Digit(domain)
        return (f"{result.title()} ({duration_ms}ms)", board, pipe.stderr)

