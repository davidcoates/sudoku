import json
import pickle
import json
from http import HTTPStatus
import lzstring
import requests
from dataclasses import dataclass, field
from typing import *
from enum import Enum


@dataclass(init=False,frozen=True)
class Line:
    class Type(Enum):
        UNDIRECTED = 0
        DIRECTED = 1
    path: list[(int, int)]
    ty: Type = Type.UNDIRECTED

    def __init__(self, path, ty):
        assert len(path) > 0
        if ty == Line.Type.UNDIRECTED:
            # pick a canonical orientation (for comparisons / hash)
            path = min(path, path[::-1])
        super().__setattr__('ty', ty)
        super().__setattr__('path', path)

    def __hash__(self):
        return hash((tuple(self.path), self.ty))

    def encode(self, ty: str):
        return {
            "type": ty,
            "cells": [ f"{r+1}:{c+1}" for (r, c) in self.path ]
        }


@dataclass(frozen=True)
class Edge:
    # cell0 should compare less than cell1
    cell0: (int, int)
    cell1: (int, int)

    def __init__(self, cell0, cell1):
        (cell0, cell1) = (cell0, cell1) if cell0 < cell1 else (cell1, cell0)
        super().__setattr__('cell0', cell0)
        super().__setattr__('cell1', cell1)

    def encode(self):
        return [ f"{r+1}:{c+1}" for (r, c) in [self.cell0, self.cell1] ]


@dataclass(frozen=True)
class Kropki:
    class Color(Enum):
        WHITE = 0
        BLACK = 1
    color: Color
    edge: Edge

    def encode(self):
        match self.color:
            case Kropki.Color.WHITE:
                ty = "white_kropki"
            case Kropki.Color.BLACK:
                ty = "black_kropki"
            case _:
                assert False
        return {
            "type": ty,
            "cells": self.edge.encode()
        }


@dataclass(frozen=True)
class XV:
    class Value(Enum):
        X = 10
        V = 5
    value: Value
    edge: Edge

    def encode(self):
        match self.value:
            case XV.Value.V:
                ty = "v"
            case XV.Value.X:
                ty = "x"
            case _:
                assert False
        return {
            "type": ty,
            "cells": self.edge.encode()
        }


@dataclass(frozen=True)
class Digit:
    values: list[int]

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
    anti_knight: bool = False
    anti_king: bool = False

    def encode(self):

        constraints = []

        for line in self.thermometers:
            constraints.append(line.encode("thermometer"))

        for line in self.palindromes:
            constraints.append(line.encode("palindrome"))

        for line in self.renbans:
            constraints.append(line.encode("renban"))

        for line in self.whispers:
            constraints.append(line.encode("whisper"))

        for kropki in self.kropkis:
            constraints.append(kropki.encode())

        for xv in self.xvs:
            constraints.append(xv.encode())

        return {
            'locals': constraints,
            'globals': {
                "anti_king": self.anti_king,
                "anti_knight": self.anti_knight,
            }
        }


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
        if self._constraints.xvs:
            rules.append("Cells separated by a V sum to 5, cells separated by an X sum to 10.")
        if self._constraints.anti_knight:
            rules.append("Digits a knight's move away can not be the same.")
        if self._constraints.anti_king:
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
        if self.constraints.anti_knight:
            js["anti_knight"] = {}
        if self.constraints.anti_king:
            js["anti_king"] = {}
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
            self._board = [ [ Digit(list(range(1, 10))) for c in range(9) ] for r in range(9) ]
            self._constraints = Constraints()

    def set(self, r, c, value):
        self._board[r][c] = value

    def get(self, r, c):
        return self._board[r][c]

    @property
    def constraints(self):
        return self._constraints

    def solve(self, branch, greedy, trace):

        domains = {}
        for r in range(9):
            for c in range(9):
                domains[f"{r+1}:{c+1}"] = self._board[r][c].values

        solver_input = {
            "type": "sudoku",
            "domains": domains,
            "constraints": self._constraints.encode(),
            "config": {
                "greedy": greedy,
                "breadcrumbs": trace,
            }
        }

        resp = requests.post("http://localhost:3000/solve", json=solver_input)
        if resp.status_code != HTTPStatus.OK:
            return ("Error", None, None)

        solver_output = resp.json()
        result = solver_output["result"]
        duration_ms = solver_output["duration_ms"]
        board = [ [ self._board[r][c] for c in range(9) ] for r in range(9) ]
        for variable, domain in solver_output["domains"].items():
            [r, c] = variable.split(':')
            r, c = int(r) - 1, int(c) - 1
            board[r][c] = Digit(domain)

        breadcrumbs = [] # FIXME!
        return (f"{result.title()} ({duration_ms}ms)", board, breadcrumbs)

