import json
import subprocess

class SudokuError(Exception):
    """
    An application specific error.
    """
    pass


class Sudoku(object):
    """
    Sudoku Puzzle representation
    """
    def __init__(self, sudoku_filename):
        self._sudoku_filename = sudoku_filename
        self.reset()

    def _create_board(self, board_json):
        board = []
        for line in board_json:
            line = line.strip()
            if len(line) != 9:
                raise SudokuError(
                    "Each line in the sudoku sudoku must be 9 chars long."
                )
            board.append([])

            for c in line:
                if c == '_':
                    board[-1].append(None)
                elif '1' <= c <= '9':
                    board[-1].append(int(c))
                else:
                    raise SudokuError(
                        "Valid characters for a sudoku sudoku must be in 1-9"
                    )

        if len(board) != 9:
            raise SudokuError("Each sudoku sudoku must be 9 lines long")
        return board

    def save(self):
        sudoku_dict = {
            "board": [ ''.join(map(lambda digit : str(digit) if digit else '_', row)) for row in self._board ]
        }
        with open(self._sudoku_filename, 'w') as sudoku_file:
            json.dump(sudoku_dict, sudoku_file)

    def reset(self):
        try:
            with open(self._sudoku_filename, 'r') as sudoku_file:
                sudoku = json.load(sudoku_file)
                self._board = self._create_board(sudoku["board"])
        except FileNotFoundError:
            self._board = [ [ None for c in range(9) ] for r in range(9) ]

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

