import os
import json
import numpy as np
import matplotlib.pyplot as plt
from collections import defaultdict
from ast import literal_eval

def parse_board_dict(board_dict):
    board = [['Empty' for _ in range(7)] for _ in range(7)]
    for key, value in board_dict['board'].items():
        pos = literal_eval(key)
        board[pos[0]][pos[1]] = value
    return board

def find_move(before, after):
    from_pos = to_pos = None
    moved_piece = None
    for r in range(7):
        for c in range(7):
            if before[r][c] != after[r][c]:
                if before[r][c] != 'Empty' and after[r][c] == 'Empty':
                    from_pos = (r, c)
                elif before[r][c] == 'Empty' and after[r][c] != 'Empty':
                    to_pos = (r, c)
                    moved_piece = after[r][c]
    return from_pos, to_pos, moved_piece

def count_pieces(board):
    """Helper to count the number of each piece type on the board."""
    counts = {'Attacker': 0, 'Defender': 0, 'King': 0}
    for row in board:
        for cell in row:
            if cell in counts:
                counts[cell] += 1
    return counts

def process_file(file_path, move_counter, att_capture_counter, def_capture_counter, king_capture_counter):
    with open(file_path, 'r') as f:
        lines = f.readlines()

    boards = []
    winner = None  # Variable to store the winner information

    # Process all lines except the last one (which contains the winner)
    for i, line in enumerate(lines[:-1]):
        line = line.strip()
        if not line:
            continue
        try:
            board_dict = json.loads(line)
            board = parse_board_dict(board_dict)
            boards.append(board)
        except json.JSONDecodeError as e:
            print(f"Skipping line {i + 1} in {file_path}: JSON decode error - {e}")

    # Process the last line to get the winner
    last_line = lines[-1].strip()
    if last_line.startswith("Winner:"):
        winner = last_line.split(":")[1].strip()
        if winner not in ["Attacker", "Defender", "Empty"]:
            print(f"Invalid winner data in last line: {last_line}")

    # Now, process moves and captures between boards
    for i in range(1, len(boards)):
        before = boards[i - 1]
        after = boards[i]

        before_counts = count_pieces(before)
        after_counts = count_pieces(after)

        from_pos, to_pos, piece = find_move(before, after)
        if from_pos and to_pos:
            move_counter[piece].append((from_pos, to_pos))

        # Check for genuine captures
        for r in range(7):
            for c in range(7):
                b_piece = before[r][c]
                a_piece = after[r][c]

                if b_piece in ['Attacker', 'Defender'] and a_piece == 'Empty':
                    # Check if this piece type's count decreased
                    if before_counts[b_piece] > after_counts[b_piece]:
                        if b_piece == 'Attacker':
                            att_capture_counter[r][c] += 1
                        else:
                            def_capture_counter[r][c] += 1

    # If the winner is Attacker, find the King's position in the last board state
    if winner == "Attacker" and boards:
        last_board = boards[-1]
        for r in range(7):
            for c in range(7):
                if last_board[r][c] == "King":
                    king_capture_counter[r][c] += 1
                    break


def analyze_folder(folder_path):
    move_counter = {
        'Attacker': [],
        'Defender': [],
        'King': []
    }

    att_capture_counter = np.zeros((7, 7), dtype=int)
    def_capture_counter = np.zeros((7, 7), dtype=int)
    king_capture_counter = np.zeros((7, 7), dtype=int)

    for filename in os.listdir(folder_path):
        if filename.endswith('.txt'):
            process_file(
                os.path.join(folder_path, filename),
                move_counter,
                att_capture_counter,
                def_capture_counter,
                king_capture_counter
            )

    return move_counter, att_capture_counter, def_capture_counter, king_capture_counter

def plot_moves_side_by_side(attacker_moves, defender_moves, king_moves):
    fig, axs = plt.subplots(1, 3, figsize=(12, 4))
    titles = ['Defender Moves', 'King Moves', 'Attacker Moves']
    move_sets = [defender_moves, king_moves, attacker_moves,]

    for ax, title, moves in zip(axs, titles, move_sets):
        ax.set_xlim(-0.5, 6.5)
        ax.set_ylim(-0.5, 6.5)
        ax.set_xticks(range(7))
        ax.set_yticks(range(7))
        ax.grid(True)
        ax.set_title(title)
        ax.invert_yaxis()

        freq = defaultdict(int)
        for move in moves:
            freq[move] += 1

        for (from_pos, to_pos), count in freq.items():
            fx, fy = from_pos[1], from_pos[0]  # col, row
            tx, ty = to_pos[1], to_pos[0]
            ax.plot([fx, tx], [fy, ty], linewidth=max(0.5, count / 10), color='blue', alpha=0.7)

    plt.tight_layout()
    plt.show()

def plot_heatmap(data, title, cmap='Reds'):
    fig, ax = plt.subplots(figsize=(6, 6))
    heatmap = ax.imshow(data, cmap=cmap)
    ax.set_xticks(range(7))
    ax.set_yticks(range(7))
    ax.set_xticklabels(range(7))
    ax.set_yticklabels(range(7))
    ax.set_title(title)
    ax.invert_yaxis()
    plt.colorbar(heatmap)
    plt.show()

# === USAGE ===
folder_path = "tournament_results/results_round_1"  # Replace this with your folder
move_data, att_capture_heatmap, def_capture_heatmap, king_capture_heatmap = analyze_folder(folder_path)

# Plotting
plot_moves_side_by_side(move_data['Attacker'], move_data['Defender'], move_data['King'])
plot_heatmap(att_capture_heatmap, "Defending Captures Heatmap")
plot_heatmap(def_capture_heatmap, "Attacking Captures Heatmap", cmap="Greens")
plot_heatmap(king_capture_heatmap, "King Capture Heatmap", cmap="Purples")