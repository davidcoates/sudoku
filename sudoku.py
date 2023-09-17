import json
import subprocess
import pickle

class SudokuError(Exception):
    """
    An application specific error.
    """
    pass


class Thermo():

    def __init__(self, path):
        self._path = path

    @property
    def path(self):
        return self._path



class Sudoku(object):
    """
    Sudoku Puzzle representation
    """
    def __init__(self, sudoku_filename):
        self._sudoku_filename = sudoku_filename
        self.reset()

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
        domains = {}
        for r in range(9):
            for c in range(9):
                if self._board[r][c] is None:
                    domain = [1, 2, 3, 4, 5, 6, 7, 8, 9]
                else:
                    domain = [ self._board[r][c] ]
                domains[f"{r+1}:{c+1}"] = domain

        def distinct(coordinates):
            variables = [ f"{r+1}:{c+1}" for (r, c) in coordinates ]
            return {
                "type": "Permutation",
                "variables": variables,
                "domain": [1, 2, 3, 4, 5, 6, 7, 8, 9]
            }
        constraints = []
        for r in range(9):
            constraints.append(distinct([(r, c) for c in range(9)]))
        for c in range(9):
            constraints.append(distinct([(r, c) for r in range(9)]))
        for r in range(3):
            for c in range(3):
                constraints.append(distinct([(r*3 + i, c*3 + j) for i in range(3) for j in range(3)]))
        solver_input = {
            "domains": domains,
            "constraints": constraints,
        }
        solver_output = json.loads(subprocess.check_output(
            ["./solver/target/debug/solver"],
            input=json.dumps(solver_input),
            text=True,
        ))
        result = solver_output["result"]
        board = [ [ self._board[r][c] for c in range(9) ] for r in range(9) ]
        if result == "solved" or result == "stuck":
            for variable, domain in solver_output["domains"].items():
                if len(domain) == 1:
                    [r, c] = variable.split(':')
                    r, c = int(r) - 1, int(c) - 1
                    board[r][c] = domain[0]
        return (result, board)

